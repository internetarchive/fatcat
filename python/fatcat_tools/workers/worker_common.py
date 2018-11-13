
import re
import sys
import csv
import json
import itertools
import fatcat_client
from pykafka import KafkaClient
from fatcat_client.rest import ApiException


class FatcatWorker:
    """
    Common code for for Kafka producers and consumers.
    """

    def __init__(self, kafka_hosts, produce_topic=None, consume_topic=None, api_host_url=None):
        if api_host_url:
            conf = fatcat_client.Configuration()
            conf.host = api_host_url
            self.api = fatcat_client.DefaultApi(fatcat_client.ApiClient(conf))
        self.kafka = KafkaClient(hosts=kafka_hosts, broker_version="1.0.0")
        self.produce_topic = produce_topic
        self.consume_topic = consume_topic

