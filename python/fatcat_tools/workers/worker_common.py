
import re
import sys
import csv
import json
import itertools
from itertools import islice
from confluent_kafka import Consumer, KafkaException, TopicPartition

import fatcat_client
from fatcat_client.rest import ApiException


def most_recent_message(topic, kafka_config):
    """
    Tries to fetch the most recent message from a given topic.

    This only makes sense for single partition topics (it works with only the
    first partition), though could be extended with "last N" behavior.
    """

    print("Fetching most Kafka message from {}".format(topic))

    conf = kafka_config.copy()
    conf.update({
        'group.id': 'worker-init-last-msg', # should never commit
        'delivery.report.only.error': True,
        'enable.auto.commit': False,
        'default.topic.config': {
            'request.required.acks': -1,
            'auto.offset.reset': 'latest',
        },
    })

    consumer = Consumer(conf)

    hwm = consumer.get_watermark_offsets(
        TopicPartition(topic, 0),
        timeout=5.0,
        cached=False)
    if not hwm:
        raise Exception("Kafka consumer timeout, or topic {} doesn't exist".format(topic))
    print("High watermarks: {}".format(hwm))

    if hwm[1] == 0:
        print("topic is new; not 'most recent message'")
        return None

    consumer.assign([TopicPartition(topic, 0, hwm[1]-1)])
    msg = consumer.poll(2.0)
    consumer.close()
    if not msg:
        raise Exception("Failed to fetch most recent kafka message")
    if msg.error():
        raise KafkaException(msg.error())
    return msg.value()


class FatcatWorker:
    """
    Common code for for Kafka producers and consumers.
    """

    def __init__(self, kafka_hosts, produce_topic=None, consume_topic=None, api=None):
        if api:
            self.api = api
        self.kafka_config = {
            'bootstrap.servers': kafka_hosts,
            'delivery.report.only.error': True,
            'message.max.bytes': 20000000, # ~20 MBytes; broker is ~50 MBytes
            'default.topic.config': {
                'request.required.acks': 'all',
            },
        }
        self.produce_topic = produce_topic
        self.consume_topic = consume_topic
