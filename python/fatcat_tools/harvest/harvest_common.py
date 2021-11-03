import datetime
import json
import sys
from typing import Any, Dict, Optional, Sequence, Set

import requests
from confluent_kafka import Consumer, KafkaException, Producer, TopicPartition
from requests.adapters import HTTPAdapter

# unclear why pylint chokes on this import. Recent 'requests' and 'urllib3' are
# in Pipenv.lock, and there are no errors in QA
from requests.packages.urllib3.util.retry import Retry  # pylint: disable=import-error

# Used for parsing ISO date format (YYYY-MM-DD)
DATE_FMT = "%Y-%m-%d"


def requests_retry_session(
    retries: int = 10,
    backoff_factor: int = 3,
    status_forcelist: Sequence[int] = (500, 502, 504),
    session: requests.Session = None,
) -> requests.Session:
    """
    From: https://www.peterbe.com/plog/best-practice-with-retries-with-requests
    """
    session = session or requests.Session()
    retry = Retry(
        total=retries,
        read=retries,
        connect=retries,
        backoff_factor=backoff_factor,
        status_forcelist=status_forcelist,
    )
    adapter = HTTPAdapter(max_retries=retry)
    session.mount("http://", adapter)
    session.mount("https://", adapter)
    return session


class HarvestState:
    """
    First version of this works with full days (dates)

    General concept is to have harvesters serialize state when they make
    progress and push to kafka. On startup, harvesters are given a task (extend
    of work), and consume the full history to see what work remains to be done.

    The simplest flow is:
    - harvester is told to collect last N days of updates
    - creates an to_process set
    - for each update, pops date from in_progress (if exits)

    NOTE: this thing is sorta over-engineered... but might grow in the future
    NOTE: should this class manage the state topic as well? Hrm.
    """

    def __init__(
        self,
        start_date: Optional[datetime.date] = None,
        end_date: Optional[datetime.date] = None,
        catchup_days: int = 14,
    ):
        self.to_process: Set[datetime.date] = set()
        self.completed: Set[datetime.date] = set()

        if catchup_days or start_date or end_date:
            self.enqueue_period(start_date, end_date, catchup_days)

    def __str__(self) -> str:
        return "<HarvestState to_process={}, completed={}>".format(
            len(self.to_process), len(self.completed)
        )

    def enqueue_period(
        self,
        start_date: Optional[datetime.date] = None,
        end_date: Optional[datetime.date] = None,
        catchup_days: int = 14,
    ) -> None:
        """
        This function adds a time period to the "TODO" list, unless the dates
        have already been processed.

        By default the period is "<catchup_days> ago until yesterday"
        """

        today_utc = datetime.datetime.utcnow().date()
        if start_date is None:
            # bootstrap to N days ago
            start_date = today_utc - datetime.timedelta(days=catchup_days)
        if end_date is None:
            # bootstrap to yesterday (don't want to start on today until it's over)
            end_date = today_utc - datetime.timedelta(days=1)

        current = start_date
        while current <= end_date:
            if current not in self.completed:
                self.to_process.add(current)
            current += datetime.timedelta(days=1)

    def next_span(self, continuous: bool = False) -> Optional[datetime.date]:
        """
        Gets next timespan (date) to be processed, or returns None if completed.

        If 'continuous' arg is True, will try to enqueue recent possibly valid
        timespans; the idea is to call next_span() repeatedly, and it will return a
        new timespan when it becomes "available".
        """
        if continuous:
            # enqueue yesterday
            self.enqueue_period(
                start_date=datetime.datetime.utcnow().date() - datetime.timedelta(days=1)
            )
        if not self.to_process:
            return None
        return sorted(list(self.to_process))[0]

    def update(self, state_json: str) -> None:
        """
        Merges a state JSON object into the current state.

        This is expected to be used to "catch-up" on previously serialized
        state stored on disk or in Kafka.
        """
        state = json.loads(state_json)
        if "completed-date" in state:
            date = datetime.datetime.strptime(state["completed-date"], DATE_FMT).date()
            self.complete(date)

    def complete(
        self,
        date: datetime.date,
        kafka_topic: Optional[str] = None,
        kafka_config: Optional[Dict] = None,
    ) -> bytes:
        """
        Records that a date has been processed successfully.

        Updates internal state and returns a JSON representation to be
        serialized. Will publish to a kafka topic if passed as an argument.

        kafka_topic should be a string. A producer will be created and destroyed.
        """
        try:
            self.to_process.remove(date)
        except KeyError:
            pass
        self.completed.add(date)
        state_json = json.dumps(
            {
                "in-progress-dates": [str(d) for d in self.to_process],
                "completed-date": str(date),
            }
        ).encode("utf-8")
        if kafka_topic:
            assert kafka_config

            def fail_fast(err: Any, _msg: Any) -> None:
                if err:
                    raise KafkaException(err)

            print("Committing status to Kafka: {}".format(kafka_topic), file=sys.stderr)
            producer_conf = kafka_config.copy()
            producer_conf.update(
                {
                    "delivery.report.only.error": True,
                    "default.topic.config": {
                        "request.required.acks": -1,  # all brokers must confirm
                    },
                }
            )
            producer = Producer(producer_conf)
            producer.produce(kafka_topic, state_json, on_delivery=fail_fast)
            producer.flush()
        return state_json

    def initialize_from_kafka(self, kafka_topic: str, kafka_config: Dict[str, Any]) -> None:
        """
        kafka_topic should have type str

        TODO: this method does not fail if client can't connect to host.
        """
        if not kafka_topic:
            return

        print("Fetching state from kafka topic: {}".format(kafka_topic), file=sys.stderr)

        def fail_fast(err: Any, _msg: Any) -> None:
            if err:
                raise KafkaException(err)

        conf = kafka_config.copy()
        conf.update(
            {
                "group.id": "dummy_init_group",  # should never be committed
                "enable.auto.commit": False,
                "auto.offset.reset": "earliest",
                "session.timeout.ms": 10000,
            }
        )
        consumer = Consumer(conf)

        # this watermark fetch is mostly to ensure we are connected to broker and
        # fail fast if not, but we also confirm that we read to end below.
        hwm = consumer.get_watermark_offsets(
            TopicPartition(kafka_topic, 0), timeout=5.0, cached=False
        )
        if not hwm:
            raise Exception(
                "Kafka consumer timeout, or topic {} doesn't exist".format(kafka_topic)
            )

        consumer.assign([TopicPartition(kafka_topic, 0, 0)])
        c = 0
        while True:
            msg = consumer.poll(timeout=2.0)
            if not msg:
                break
            if msg.error():
                raise KafkaException(msg.error())
            # sys.stdout.write('.')
            self.update(msg.value().decode("utf-8"))
            c += 1
        consumer.close()

        # verify that we got at least to HWM
        assert c >= hwm[1]
        print("... got {} state update messages, done".format(c), file=sys.stderr)
