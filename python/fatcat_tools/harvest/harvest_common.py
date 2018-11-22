
import sys
import json
import time
import datetime

# Used for parsing ISO date format (YYYY-MM-DD)
DATE_FMT = "%Y-%m-%d"

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

    def __init__(self, start_date=None, end_date=None, catchup_days=7):
        self.to_process = set()
        self.completed = set()

        if catchup_days or start_date or end_date:
            self.enqueue_period(start_date, end_date, catchup_days)

    def enqueue_period(self, start_date=None, end_date=None, catchup_days=7):
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
            if not current in self.completed:
                self.to_process.add(current)
            current += datetime.timedelta(days=1)

    def next(self, continuous=False):
        """
        Gets next timespan (date) to be processed, or returns None if completed.

        If 'continuous' arg is True, will try to enqueue recent possibly valid
        timespans; the idea is to call next() repeatedly, and it will return a
        new timespan when it becomes "available".
        """
        if continuous:
            # enqueue yesterday
            self.enqueue_period(start_date=datetime.datetime.utcnow().date() - datetime.timedelta(days=1))
        if not self.to_process:
            return None
        return sorted(list(self.to_process))[0]

    def update(self, state_json):
        """
        Merges a state JSON object into the current state.

        This is expected to be used to "catch-up" on previously serialized
        state stored on disk or in Kafka.
        """
        state = json.loads(state_json)
        if 'completed-date' in state:
            date = datetime.datetime.strptime(state['completed-date'], DATE_FMT).date()
            self.complete(date)

    def complete(self, date, kafka_topic=None):
        """
        Records that a date has been processed successfully.

        Updates internal state and returns a JSON representation to be
        serialized. Will publish to a kafka topic if passed as an argument.

        kafka_topic should have type pykafka.Topic (not str)
        """
        try:
            self.to_process.remove(date)
        except KeyError:
            pass
        self.completed.add(date)
        state_json = json.dumps({
            'in-progress-dates': [str(d) for d in self.to_process],
            'completed-date': str(date),
        }).encode('utf-8')
        if kafka_topic:
            with kafka_topic.get_sync_producer() as producer:
                producer.produce(state_json)
        return state_json

    def initialize_from_kafka(self, kafka_topic):
        """
        kafka_topic should have type pykafka.Topic (not str)
        """
        if not kafka_topic:
            return

        print("Fetching state from kafka topic: {}".format(kafka_topic.name))
        consumer = kafka_topic.get_simple_consumer(consumer_timeout_ms=1000)
        c = 0
        while True:
            msg = consumer.consume(block=True)
            if not msg:
                break
            #sys.stdout.write('.')
            self.update(msg.value.decode('utf-8'))
            c += 1
        print("... got {} state update messages, done".format(c))

