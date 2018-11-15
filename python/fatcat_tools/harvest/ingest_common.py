
"""
logic:
- on start, fetch latest date from state feed
- in a function (unit-testable), decide which dates to ingest
- for each date needing update:
    - start a loop for just that date, using resumption token for this query
    - when done, publish to state feed, with immediate sync
"""

import re
import sys
import csv
import json
import requests
import itertools
import datetime
from pykafka import KafkaClient

from fatcat_tools.workers.worker_common import most_recent_message

DATE_FMT = "%Y-%m-%d"

class DoiApiHarvest:
    """
    This class supports core features for both the Crossref and Datacite REST
    APIs for fetching updated metadata (the Datacite API seems to be moduled on
    the Crossref API).

    Implementations must provide the push results function.
    """

    def __init__(self, kafka_hosts, produce_topic, state_topic, api_host_url,
            contact_email, start_date=None, end_date=None):
        self.loop_sleep = 60*60 # how long to wait, in seconds, between date checks
        self.api_batch_size = 50
        self.api_host_url = api_host_url
        self.produce_topic = produce_topic
        self.state_topic = state_topic
        self.contact_email = contact_email
        self.kafka = KafkaClient(hosts=kafka_hosts, broker_version="1.0.0")
        self.is_update_filter = None
        self.update_filter_name = "index"

        # these are both optional, and should be datetime.date
        self.start_date = start_date
        self.end_date = end_date

    def get_latest_date(self):

        state_topic = self.kafka.topics[self.state_topic]
        latest = most_recent_message(state_topic)
        if latest:
            latest = datetime.datetime.strptime(latest.decode('utf-8'), DATE_FMT).date()
        print("Latest date found: {}".format(latest))
        return latest

    def fetch_date(self, date):

        state_topic = self.kafka.topics[self.state_topic]
        produce_topic = self.kafka.topics[self.produce_topic]
 
        date_str = date.strftime(DATE_FMT)
        filter_param = 'from-{index}-date:{},until-{index}-date:{}'.format(
            date_str, date_str, index=self.update_filter_name)
        if self.is_update_filter is not None:
            filter_param += ',is_update:{}'.format(bool(is_update))
        params = {
            'filter': filter_param,
            'rows': self.api_batch_size,
            'cursor': '*',
        }
        headers = {
            'User-Agent': 'fatcat_tools/0.1.0 (https://fatcat.wiki; mailto:{}) python-requests'.format(self.contact_email),
        }
        count = 0
        with produce_topic.get_producer() as producer:
            while True:
                http_resp = requests.get(self.api_host_url, params, headers=headers)
                assert http_resp.status_code is 200
                resp = http_resp.json()
                items = resp['message']['items']
                count += len(items)
                print("... got {} ({} of {}) in {}".format(len(items), count,
                    resp['message']['total-results']), http_resp.elapsed)
                #print(json.dumps(resp))
                for work in items:
                    producer.produce(json.dumps(work).encode('utf-8'))
                if len(items) < params['rows']:
                    break
                params['cursor'] = resp['message']['next-cursor']

        # record our completion state
        with state_topic.get_sync_producer() as producer:
            producer.produce(date.strftime(DATE_FMT).encode('utf-8'))
        

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
        print("Crossref DOI ingest caught up through {}".format(self.end_date))
        return self.end_date

    def run_loop(self):
        while True:
            last = self.run_once()
            self.start_date = last
            self.end_date = None
            print("Sleeping {} seconds...".format(self.loop_sleep))
            time.sleep(self.loop_sleep())

