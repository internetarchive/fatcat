import datetime
import sys
import time
from typing import Any, Optional

import sickle
from confluent_kafka import KafkaException, Producer
from bs4 import BeautifulSoup

from .harvest_common import HarvestState, requests_retry_session


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

    def __init__(
        self,
        kafka_hosts: str,
        produce_topic: str,
        state_topic: str,
        start_date: Optional[datetime.date] = None,
        end_date: Optional[datetime.date] = None,
    ):

        self.produce_topic = produce_topic
        self.state_topic = state_topic
        self.kafka_config = {
            "bootstrap.servers": kafka_hosts,
            "message.max.bytes": 20000000,  # ~20 MBytes; broker is ~50 MBytes
        }

        self.loop_sleep = 60 * 60  # how long to wait, in seconds, between date checks

        # optional; not all workers will need or use this HTTP session
        self.http_session = requests_retry_session()

        self.endpoint_url = None  # needs override
        self.metadata_prefix = None  # needs override
        self.name = "unnamed"
        self.state = HarvestState(start_date, end_date)
        self.state.initialize_from_kafka(self.state_topic, self.kafka_config)
        print(self.state, file=sys.stderr)

    def fetch_date(self, date: datetime.date) -> None:
        def fail_fast(err: Any, _msg: Any) -> None:
            if err is not None:
                print("Kafka producer delivery error: {}".format(err), file=sys.stderr)
                print("Bailing out...", file=sys.stderr)
                # TODO: should it be sys.exit(-1)?
                raise KafkaException(err)

        producer_conf = self.kafka_config.copy()
        producer_conf.update(
            {
                "delivery.report.only.error": True,
                "default.topic.config": {
                    "request.required.acks": -1,  # all brokers must confirm
                },
            }
        )
        producer = Producer(producer_conf)

        api = sickle.Sickle(self.endpoint_url, max_retries=5, retry_status_codes=[503])
        date_str = date.isoformat()
        # this dict kwargs hack is to work around 'from' as a reserved python keyword
        # recommended by sickle docs
        try:
            records = api.ListRecords(
                **{
                    "metadataPrefix": self.metadata_prefix,
                    "from": date_str,
                    "until": date_str,
                }
            )
        except sickle.oaiexceptions.NoRecordsMatch:
            print(
                "WARN: no OAI-PMH records for this date: {} (UTC)".format(date_str),
                file=sys.stderr,
            )
            return

        count = 0
        for item in records:
            count += 1
            if count % 50 == 0:
                print("... up to {}".format(count), file=sys.stderr)
            self.produce_record(item, producer)
        producer.flush()

    def produce_record(self, item: sickle.models.Record, producer: Producer) -> None:
        """
        The intent of this function is to allow overloading the record type
        being passed along to the Kafka topic
        """
        producer.produce(
            self.produce_topic,
            item.raw.encode("utf-8"),
            key=item.header.identifier.encode("utf-8"),
            on_delivery=fail_fast,
        )

    def run(self, continuous: bool = False) -> None:

        while True:
            current = self.state.next_span(continuous)
            if current:
                print("Fetching DOIs updated on {} (UTC)".format(current), file=sys.stderr)
                self.fetch_date(current)
                self.state.complete(
                    current, kafka_topic=self.state_topic, kafka_config=self.kafka_config
                )
                continue

            if continuous:
                print("Sleeping {} seconds...".format(self.loop_sleep), file=sys.stderr)
                time.sleep(self.loop_sleep)
            else:
                break
        print("{} OAI-PMH ingest caught up".format(self.name), file=sys.stderr)


class HarvestArxivWorker(HarvestOaiPmhWorker):
    """
    Arxiv refs:
    - http://export.arxiv.org/oai2?verb=GetRecord&identifier=oai:arXiv.org:0804.2273&metadataPrefix=arXiv
    - http://export.arxiv.org/oai2?verb=GetRecord&identifier=oai:arXiv.org:0804.2273&metadataPrefix=arXivRaw

    All records are work-level. Some metadata formats have internal info about
    specific versions. The 'arXivRaw' format does, so i'm using that.
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.endpoint_url = "https://export.arxiv.org/oai2"
        self.metadata_prefix = "arXivRaw"
        self.name = "arxiv"


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

    def parse_doaj_article_id(self, raw_xml: bytes) -> str:
        # XXX: don't parse XML; instead just handle item.oai_identifier
        soup = BeautifulSoup(raw_xml, "xml")
        elem = soup.find("record header identifier")
        oai_id = elem.text.strip()
        assert oai_id.startswith("oai:doaj.org/article:")
        article_id = oai_id.replace("oai:doaj.org/article:", "")
        assert len(article_id) == 32 and article_id == article_id.lower()
        return article_id

    def produce_record(self, item: sickle.models.Record, producer: Producer) -> None:
        """
        For each OAI-PMH record, do an API call to get the JSON format
        response, and publish that to Kafka instead of the OAI-PMH XML
        """

        article_id = self.parse_doaj_article_id(item.raw)
        resp = self.http_session.get(f"https://doaj.org/api/articles/{article_id}")
        resp.raise_for_status()
        assert resp.json()['id'] == article_id
        producer.produce(
            self.produce_topic,
            resp.content.encode("utf-8"),
            key=article_id.encode("utf-8"),
            on_delivery=fail_fast,
        )
