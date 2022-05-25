from typing import Any, Optional

from confluent_kafka import KafkaException, Producer


def kafka_fail_fast(err: Optional[Any], _msg: Any) -> None:
    if err is not None:
        print(f"Kafka producer delivery error: {err}")
        print("Bailing out...")
        # TODO: should it be sys.exit(-1)?
        raise KafkaException(err)


def simple_kafka_producer(kafka_hosts: str) -> Producer:
    """
    kafka_hosts should be a string with hostnames separated by ',', not a list
    of hostnames
    """

    kafka_config = {
        "bootstrap.servers": kafka_hosts,
        "message.max.bytes": 20000000,  # ~20 MBytes; broker-side max is ~50 MBytes
        "delivery.report.only.error": True,
        "default.topic.config": {
            "request.required.acks": -1,
        },
    }
    return Producer(kafka_config)
