
import re
import sys
import csv
import json
import time
import requests
import itertools
import datetime
from pykafka import KafkaClient
import sickle

from fatcat_tools.workers import most_recent_message
from .harvest_common import HarvestState


class HarvestOaiPmhWorker:
    """
    Base class for OAI-PMH harvesters. Uses the 'sickle' protocol library.

    Typically run as a single process; harvests records and publishes in raw
    (XML) format to a Kafka topic, one-message-per-document.

    Based on Crossref importer, with the HarvestState internal class managing
    progress with day-level granularity. Note that this depends on the OAI-PMH
    endpoint being correct! In that it must be possible to poll for only
    records updated on a particular date (typically "yesterday").

    Was very tempted to re-use <https://github.com/miku/metha> for this OAI-PMH
    stuff to save on dev time, but i'd already built the Crossref harvester and
    would want something similar operationally. Oh well!
    """


    def __init__(self, kafka_hosts, produce_topic, state_topic,
            start_date=None, end_date=None):

        self.produce_topic = produce_topic
        self.state_topic = state_topic
        self.kafka = KafkaClient(hosts=kafka_hosts, broker_version="1.0.0")

        self.loop_sleep = 60*60 # how long to wait, in seconds, between date checks

        self.endpoint_url = None # needs override
        self.metadata_prefix = None  # needs override
        self.name = "unnamed"
        self.state = HarvestState(start_date, end_date)
        self.state.initialize_from_kafka(self.kafka.topics[self.state_topic])


    def fetch_date(self, date):

        api = sickle.Sickle(self.endpoint_url)
        date_str = date.isoformat()
        produce_topic = self.kafka.topics[self.produce_topic]
        # this dict kwargs hack is to work around 'from' as a reserved python keyword
        # recommended by sickle docs
        try:
            records = api.ListRecords(**{
                'metadataPrefix': self.metadata_prefix,
                'from': date_str,
                'until': date_str,
            })
        except sickle.oaiexceptions.NoRecordsMatch:
            print("WARN: no OAI-PMH records for this date: {} (UTC)".format(date_str))
            return

        count = 0
        with produce_topic.get_producer() as producer:
            for item in records:
                count += 1
                if count % 50 == 0:
                    print("... up to {}".format(count))
                producer.produce(item.raw.encode('utf-8'), partition_key=item.header.identifier.encode('utf-8'))

    def run(self, continuous=False):

        while True:
            current = self.state.next(continuous)
            if current:
                print("Fetching DOIs updated on {} (UTC)".format(current))
                self.fetch_date(current)
                self.state.complete(current, kafka_topic=self.kafka.topics[self.state_topic])
                continue

            if continuous:
                print("Sleeping {} seconds...".format(self.loop_sleep))
                time.sleep(self.loop_sleep)
            else:
                break
        print("{} OAI-PMH ingest caught up".format(self.name))


class HarvestArxivWorker(HarvestOaiPmhWorker):
    """
    Arxiv refs:
    - http://export.arxiv.org/oai2?verb=GetRecord&identifier=oai:arXiv.org:0804.2273&metadataPrefix=arXiv
    - http://export.arxiv.org/oai2?verb=GetRecord&identifier=oai:arXiv.org:0804.2273&metadataPrefix=arXivRaw

    All records are work-level. Some metadata formats have internal info about
    specific versions. The 'arXiv' format does, so i'm using that.
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs) 
        self.endpoint_url = "https://export.arxiv.org/oai2"
        self.metadata_prefix = "arXiv"
        self.name = "arxiv"


class HarvestPubmedWorker(HarvestOaiPmhWorker):
    """
    Pubmed refs:
    - https://www.ncbi.nlm.nih.gov/pmc/tools/oai/
    - https://www.ncbi.nlm.nih.gov/pmc/oai/oai.cgi?verb=GetRecord&identifier=oai:pubmedcentral.nih.gov:152494&metadataPrefix=pmc_fm
    - https://github.com/titipata/pubmed_parser
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs) 
        self.endpoint_url = "https://www.ncbi.nlm.nih.gov/pmc/oai/oai.cgi"
        self.metadata_prefix = "pmc_fm"
        self.name = "pubmed"


class HarvestDoajJournalWorker(HarvestOaiPmhWorker):
    """
    WARNING: DOAJ OAI-PMH doesn't seem to respect 'from' and 'until' params

    As an alternative, could use:
    - https://github.com/miku/doajfetch
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs) 
        self.endpoint_url = "https://www.doaj.org/oai"
        self.metadata_prefix = "oai_dc"
        self.name = "doaj-journal"


class HarvestDoajArticleWorker(HarvestOaiPmhWorker):
    """
    WARNING: DOAJ OAI-PMH doesn't seem to respect 'from' and 'until' params
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs) 
        self.endpoint_url = "https://www.doaj.org/oai.article"
        self.metadata_prefix = "oai_doaj"
        self.name = "doaj-article"

