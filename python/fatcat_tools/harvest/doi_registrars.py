
import re
import sys
import csv
import json
import time
import requests
import itertools
import datetime
from pykafka import KafkaClient

from fatcat_tools.workers.worker_common import most_recent_message

# Skip pylint due to:
#   AttributeError: 'NoneType' object has no attribute 'scope'
# in 'astroid/node_classes.py'
# pylint: skip-file

DATE_FMT = "%Y-%m-%d"


class HarvestCrossrefWorker:
    """
    Notes on crossref API:

    - from-index-date is the updated time
    - is-update can be false, to catch only new or only old works

    https://api.crossref.org/works?filter=from-index-date:2018-11-14,is-update:false&rows=2

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
            end_date=None, is_update_filter=None):

        self.api_host_url = api_host_url
        self.produce_topic = produce_topic
        self.state_topic = state_topic
        self.contact_email = contact_email
        self.kafka = KafkaClient(hosts=kafka_hosts, broker_version="1.0.0")
        self.is_update_filter = is_update_filter

        # these are both optional, and should be datetime.date
        self.start_date = start_date
        self.end_date = end_date

        self.loop_sleep = 60*60 # how long to wait, in seconds, between date checks
        self.api_batch_size = 50
        # for crossref, it's "from-index-date"
        self.name = "Crossref"

    def get_latest_date(self):

        state_topic = self.kafka.topics[self.state_topic]
        latest = most_recent_message(state_topic)
        if latest:
            latest = datetime.datetime.strptime(latest.decode('utf-8'), DATE_FMT).date()
        print("Latest date found: {}".format(latest))
        return latest

    def params(self, date_str):
        filter_param = 'from-index-date:{},until-index-date:{}'.format(
            date_str, date_str)
        if self.is_update_filter is not None:
            filter_param += ',is_update:{}'.format(bool(self.is_update_filter))
        return {
            'filter': filter_param,
            'rows': self.api_batch_size,
            'cursor': '*',
        }

    def update_params(self, params, resp):
        params['cursor'] = resp['message']['next-cursor']
        return params

    def fetch_date(self, date):

        state_topic = self.kafka.topics[self.state_topic]
        produce_topic = self.kafka.topics[self.produce_topic]

        date_str = date.strftime(DATE_FMT)
        params = self.params(date_str)
        headers = {
            'User-Agent': 'fatcat_tools/0.1.0 (https://fatcat.wiki; mailto:{}) python-requests'.format(self.contact_email),
        }
        count = 0
        with produce_topic.get_producer() as producer:
            while True:
                http_resp = requests.get(self.api_host_url, params, headers=headers)
                if http_resp.status_code == 503:
                    # crud backoff
                    print("got HTTP {}, pausing for 30 seconds".format(http_resp.status_code))
                    time.sleep(30.0)
                    continue
                assert http_resp.status_code == 200
                resp = http_resp.json()
                items = self.extract_items(resp)
                count += len(items)
                print("... got {} ({} of {}) in {}".format(len(items), count,
                    self.extract_total(resp), http_resp.elapsed))
                #print(json.dumps(resp))
                for work in items:
                    producer.produce(json.dumps(work).encode('utf-8'))
                if len(items) < self.api_batch_size:
                    break
                params = self.update_params(params, resp)

        # record our completion state
        with state_topic.get_sync_producer() as producer:
            producer.produce(date.strftime(DATE_FMT).encode('utf-8'))

    def extract_items(self, resp):
        return resp['message']['items']

    def extract_total(self, resp):
        return resp['message']['total-results']

    def run_once(self):
        today_utc = datetime.datetime.utcnow().date()
        if self.start_date is None:
            self.start_date = self.get_latest_date()
            if self.start_date:
                # if we are continuing, start day after last success
                self.start_date = self.start_date + datetime.timedelta(days=1)
        if self.start_date is None:
            # bootstrap to yesterday (don't want to start on today until it's over)
            self.start_date = datetime.datetime.utcnow().date()
        if self.end_date is None:
            # bootstrap to yesterday (don't want to start on today until it's over)
            self.end_date = today_utc - datetime.timedelta(days=1)
        print("Harvesting from {} through {}".format(self.start_date, self.end_date))
        current = self.start_date
        while current <= self.end_date:
            print("Fetching DOIs updated on {} (UTC)".format(current))
            self.fetch_date(current)
            current += datetime.timedelta(days=1)
        print("{} DOI ingest caught up through {}".format(self.name, self.end_date))
        return self.end_date

    def run_loop(self):
        while True:
            last = self.run_once()
            self.start_date = last
            self.end_date = None
            print("Sleeping {} seconds...".format(self.loop_sleep))
            time.sleep(self.loop_sleep())



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

    def update_params(self, params, resp):
        params['page[number]'] = resp['meta']['page'] + 1
        return params
