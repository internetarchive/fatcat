import requests

from fatcat_web import Config


def kafka_pixy_produce(topic, msg, key=None, sync=True, timeout=25):
    """
    Simple helper to public a message to the given Kafka topic, via the
    configured kafka-pixy HTTP gateway

    topic: string
    msg: string
    key: optional, bytes
    timeout: seconds
    """

    if not Config.KAFKA_PIXY_ENDPOINT:
        raise Exception("Kafka produce error: kafka-pixy endpoint not configured")

    params = dict()
    if key:
        params["key"] = key
    if sync:
        params["sync"] = True
    resp = requests.post(
        "{}/topics/{}/messages".format(Config.KAFKA_PIXY_ENDPOINT, topic),
        params=params,
        data=msg,
        headers={"Content-Type": "text/plain"},
        timeout=timeout,
    )
    resp.raise_for_status()
    # print(resp.json())
