
import sys
import json
import time
from confluent_kafka import Producer, KafkaException
from urllib.parse import urlparse, parse_qs

from .harvest_common import HarvestState, requests_retry_session


class HarvestCrossrefWorker:
    """
    Crossref API date fields (and our interpretation)::

    - https://github.com/CrossRef/rest-api-doc#filter-names
    - *-index-date: "metadata indexed" is the API/index record update time
    - *-deposit-date: "metadata last (re)deposited" is the catalog record update time
    - *-update-date: "Metadata updated (Currently the same as *-deposit-date)"
    - *-created-date: "metadata first deposited"
    - *-pub-date (etc): publisher-supplied, not "meta-meta-data"

    https://api.crossref.org/works?filter=from-index-date:2018-11-14&rows=2

    Also from the REST API:

        Notes on incremental metadata updates

        When using time filters to retrieve periodic, incremental metadata
        updates, the from-index-date filter should be used over
        from-update-date, from-deposit-date, from-created-date and
        from-pub-date. The timestamp that from-index-date filters on is
        guaranteed to be updated every time there is a change to metadata
        requiring a reindex.

    However, when Crossref re-indexes tens of millions of rows, using
    from-index-date can be very slow, taking several days to process a single
    day of updates.

    I think the design is going to have to be a cronjob or long-running job
    (with long sleeps) which publishes "success through" to a separate state
    queue, as simple YYYY-MM-DD strings.

    Within a day, will need to use a resumption token. Maybe should use a
    crossref library... meh.

    will want to have some mechanism in kafka consumer (pushing to fatcat) to group
    in batches as well. maybe even pass through as batches? or just use timeouts on
    iteration.

    logic of this worker:
    - on start, fetch latest date from state feed
    - in a function (unit-testable), decide which dates to ingest
    - for each date needing update:
        - start a loop for just that date, using resumption token for this query
        - when done, publish to state feed, with immediate sync

    TODO: what sort of parallelism? I guess multi-processing on dates, but need
    to be careful how state is serialized back into kafka.
    """

    def __init__(self, kafka_hosts, produce_topic, state_topic, contact_email,
            api_host_url="https://api.crossref.org/works", start_date=None,
            end_date=None):

        self.api_host_url = api_host_url
        self.produce_topic = produce_topic
        self.state_topic = state_topic
        self.contact_email = contact_email
        self.kafka_config = {
            'bootstrap.servers': kafka_hosts,
            'message.max.bytes': 20000000, # ~20 MBytes; broker is ~50 MBytes
        }

        self.state = HarvestState(start_date, end_date)
        self.state.initialize_from_kafka(self.state_topic, self.kafka_config)

        self.loop_sleep = 60*60 # how long to wait, in seconds, between date checks
        self.api_batch_size = 50
        self.name = "Crossref"
        self.producer = self._kafka_producer()

    def _kafka_producer(self):

        def fail_fast(err, msg):
            if err is not None:
                print("Kafka producer delivery error: {}".format(err), file=sys.stderr)
                print("Bailing out...", file=sys.stderr)
                # TODO: should it be sys.exit(-1)?
                raise KafkaException(err)

        self._kafka_fail_fast = fail_fast

        producer_conf = self.kafka_config.copy()
        producer_conf.update({
            'delivery.report.only.error': True,
            'default.topic.config': {
                'request.required.acks': -1, # all brokers must confirm
            },
        })
        return Producer(producer_conf)

    def params(self, date_str):
        filter_param = 'from-update-date:{},until-update-date:{}'.format(
            date_str, date_str)
        return {
            'filter': filter_param,
            'rows': self.api_batch_size,
            'cursor': '*',
        }

    def update_params(self, params, resp):
        params['cursor'] = resp['message']['next-cursor']
        return params

    def extract_key(self, obj):
        return obj['DOI'].encode('utf-8')

    def fetch_date(self, date):

        date_str = date.isoformat()
        params = self.params(date_str)
        http_session = requests_retry_session()
        http_session.headers.update({
            'User-Agent': 'fatcat_tools/0.1.0 (https://fatcat.wiki; mailto:{}) python-requests'.format(
                self.contact_email),
        })
        count = 0
        while True:
            http_resp = http_session.get(self.api_host_url, params=params)
            if http_resp.status_code == 503:
                # crude backoff; now redundant with session exponential
                # backoff, but allows for longer backoff/downtime on remote end
                print("got HTTP {}, pausing for 30 seconds".format(http_resp.status_code), file=sys.stderr)
                # keep kafka producer connection alive
                self.producer.poll(0)
                time.sleep(30.0)
                continue
            http_resp.raise_for_status()
            try:
                resp_body = http_resp.text
                resp = json.loads(resp_body)
            except json.JSONDecodeError as exc:
                # Datacite API returned HTTP 200, but JSON seemed unparseable.
                # It might be a glitch, so we retry.
                print("failed to decode body from {}: {}".format(http_resp.url, resp_body), file=sys.stderr)
                raise
            items = self.extract_items(resp)
            count += len(items)
            print("... got {} ({} of {}), HTTP fetch took {}".format(len(items), count,
                self.extract_total(resp), http_resp.elapsed), file=sys.stderr)
            #print(json.dumps(resp))
            for work in items:
                self.producer.produce(
                    self.produce_topic,
                    json.dumps(work).encode('utf-8'),
                    key=self.extract_key(work),
                    on_delivery=self._kafka_fail_fast)
            self.producer.poll(0)
            if len(items) < self.api_batch_size:
                break
            params = self.update_params(params, resp)
        self.producer.flush()

    def extract_items(self, resp):
        return resp['message']['items']

    def extract_total(self, resp):
        return resp['message']['total-results']

    def run(self, continuous=False):

        while True:
            current = self.state.next_span(continuous)
            if current:
                print("Fetching DOIs updated on {} (UTC)".format(current), file=sys.stderr)
                self.fetch_date(current)
                self.state.complete(current,
                    kafka_topic=self.state_topic,
                    kafka_config=self.kafka_config)
                continue

            if continuous:
                print("Sleeping {} seconds...".format(self.loop_sleep), file=sys.stderr)
                time.sleep(self.loop_sleep)
            else:
                break
        print("{} DOI ingest caught up".format(self.name), file=sys.stderr)


class HarvestDataciteWorker(HarvestCrossrefWorker):
    """
    datacite has a REST API as well as OAI-PMH endpoint.

    have about 8 million

    bulk export notes: https://github.com/datacite/datacite/issues/188

    fundamentally, very similar to crossref. don't have a scrape... maybe
    could/should use this script for that, and dump to JSON?
    """

    def __init__(self, kafka_hosts, produce_topic, state_topic, contact_email,
            api_host_url="https://api.datacite.org/dois",
            start_date=None, end_date=None):
        super().__init__(kafka_hosts=kafka_hosts,
                         produce_topic=produce_topic,
                         state_topic=state_topic,
                         api_host_url=api_host_url,
                         contact_email=contact_email,
                         start_date=start_date,
                         end_date=end_date)

        # for datecite, it's "from-update-date"
        self.name = "Datacite"

    def params(self, date_str):
        """
        Dates have to be supplied in 2018-10-27T22:36:30.000Z format.
        """
        return {
            'query': 'updated:[{}T00:00:00.000Z TO {}T23:59:59.999Z]'.format(date_str, date_str),
            'page[size]': self.api_batch_size,
            'page[cursor]': 1,
        }

    def extract_items(self, resp):
        return resp['data']

    def extract_total(self, resp):
        return resp['meta']['total']

    def extract_key(self, obj):
        return obj['attributes']['doi'].encode('utf-8')

    def update_params(self, params, resp):
        """
        Using cursor mechanism (https://support.datacite.org/docs/pagination#section-cursor).

        $ curl -sL https://is.gd/cLbE5h | jq -r .links.next

        Example: https://is.gd/cLbE5h

        Further API errors reported:
            https://github.com/datacite/datacite/issues/897 (HTTP 400)
            https://github.com/datacite/datacite/issues/898 (HTTP 500)
        """
        parsed = urlparse(resp['links']['next'])
        page_cursor = parse_qs(parsed.query).get('page[cursor]')
        if not page_cursor:
            raise ValueError('no page[cursor] in .links.next')
        params['page[cursor]'] = page_cursor[0]
        return params
