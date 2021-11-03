from confluent_kafka import KafkaException, Producer


def kafka_fail_fast(err, msg):
    if err is not None:
        print("Kafka producer delivery error: {}".format(err))
        print("Bailing out...")
        # TODO: should it be sys.exit(-1)?
        raise KafkaException(err)


def simple_kafka_producer(kafka_hosts):

    kafka_config = {
        "bootstrap.servers": kafka_hosts,
        "message.max.bytes": 20000000,  # ~20 MBytes; broker-side max is ~50 MBytes
        "delivery.report.only.error": True,
        "default.topic.config": {
            "request.required.acks": -1,
        },
    }
    return Producer(kafka_config)
