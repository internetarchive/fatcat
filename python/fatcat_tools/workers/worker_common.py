
import re
import sys
import csv
import json
import itertools
from itertools import islice
from pykafka import KafkaClient
from pykafka.common import OffsetType

import fatcat_client
from fatcat_client.rest import ApiException


def most_recent_message(topic):
    """
    Tries to fetch the most recent message from a given topic.
    This only makes sense for single partition topics, though could be
    extended with "last N" behavior.

    Following "Consuming the last N messages from a topic"
    from https://pykafka.readthedocs.io/en/latest/usage.html#consumer-patterns
    """
    consumer = topic.get_simple_consumer(
        auto_offset_reset=OffsetType.LATEST,
        reset_offset_on_start=True)
    offsets = [(p, op.last_offset_consumed - 1)
                for p, op in consumer._partitions.items()]
    offsets = [(p, (o if o > -1 else -2)) for p, o in offsets]
    if -2 in [o for p, o in offsets]:
        consumer.stop()
        return None
    else:
        consumer.reset_offsets(offsets)
        msg = islice(consumer, 1)
        if msg:
            val = list(msg)[0].value
            consumer.stop()
            return val
        else:
            consumer.stop()
            return None

class FatcatWorker:
    """
    Common code for for Kafka producers and consumers.
    """

    def __init__(self, kafka_hosts, produce_topic=None, consume_topic=None, api=None):
        if api:
            self.api = api
        self.kafka = KafkaClient(hosts=kafka_hosts, broker_version="1.0.0")
        self.produce_topic = produce_topic
        self.consume_topic = consume_topic
