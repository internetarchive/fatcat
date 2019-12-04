
import re
import sys
import csv
import json
import time
import itertools
import datetime
import requests
from confluent_kafka import Producer, KafkaException

from fatcat_tools.workers import most_recent_message
from .harvest_common import HarvestState, requests_retry_session


class HarvestCrossrefWorker:
    """
    Notes on crossref API:

    - from-index-date is the updated time

    https://api.crossref.org/works?filter=from-index-date:2018-11-14&rows=2

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

    def params(self, date_str):
        filter_param = 'from-index-date:{},until-index-date:{}'.format(
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

        def fail_fast(err, msg):
            if err is not None:
                print("Kafka producer delivery error: {}".format(err))
                print("Bailing out...")
                # TODO: should it be sys.exit(-1)?
                raise KafkaException(err)

        producer_conf = self.kafka_config.copy()
        producer_conf.update({
            'delivery.report.only.error': True,
            'default.topic.config': {
                'request.required.acks': -1, # all brokers must confirm
            },
        })
        producer = Producer(producer_conf)

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
                print("got HTTP {}, pausing for 30 seconds".format(http_resp.status_code))
                # keep kafka producer connection alive
                producer.poll(0)
                time.sleep(30.0)
                continue
            http_resp.raise_for_status()
            resp = http_resp.json()
            items = self.extract_items(resp)
            count += len(items)
            print("... got {} ({} of {}), HTTP fetch took {}".format(len(items), count,
                self.extract_total(resp), http_resp.elapsed))
            #print(json.dumps(resp))
            for work in items:
                producer.produce(
                    self.produce_topic,
                    json.dumps(work).encode('utf-8'),
                    key=self.extract_key(work),
                    on_delivery=fail_fast)
            producer.poll(0)
            if len(items) < self.api_batch_size:
                break
            params = self.update_params(params, resp)
        producer.flush()

    def extract_items(self, resp):
        return resp['message']['items']

    def extract_total(self, resp):
        return resp['message']['total-results']

    def run(self, continuous=False):

        while True:
            current = self.state.next(continuous)
            if current:
                print("Fetching DOIs updated on {} (UTC)".format(current))
                self.fetch_date(current)
                self.state.complete(current,
                    kafka_topic=self.state_topic,
                    kafka_config=self.kafka_config)
                continue

            if continuous:
                print("Sleeping {} seconds...".format(self.loop_sleep))
                time.sleep(self.loop_sleep)
            else:
                break
        print("{} DOI ingest caught up".format(self.name))


class HarvestDataciteWorker(HarvestCrossrefWorker):
    """
    datacite has a REST API as well as OAI-PMH endpoint.

    have about 8 million

    bulk export notes: https://github.com/datacite/datacite/issues/188

    fundamentally, very similar to crossref. don't have a scrape... maybe
    could/should use this script for that, and dump to JSON?
    """

    def __init__(self, kafka_hosts, produce_topic, state_topic, contact_email,
            api_host_url="https://api.datacite.org/works",
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
        return {
            'from-update-date': date_str,
            'until-update-date': date_str,
            'page[size]': self.api_batch_size,
            'page[number]': 1,
        }

    def extract_items(self, resp):
        return resp['data']

    def extract_total(self, resp):
        return resp['meta']['total']

    def extract_key(self, obj):
        return obj['attributes']['doi'].encode('utf-8')

    def update_params(self, params, resp):
        params['page[number]'] = resp['meta']['page'] + 1
        return params
