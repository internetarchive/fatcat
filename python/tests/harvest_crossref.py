import datetime
import json

import responses

from fatcat_tools.harvest import *


@responses.activate
def test_crossref_harvest_date(mocker):

    # mock out the harvest state object so it doesn't try to actually connect
    # to Kafka
    mocker.patch("fatcat_tools.harvest.harvest_common.HarvestState.initialize_from_kafka")

    # mock day request to crossref API
    with open("tests/files/crossref_api_works.json") as f:
        crossref_resp = json.loads(f.readline())
    responses.add(
        responses.GET, "https://api.crossref.org/works", json=crossref_resp, status=200
    )

    harvester = HarvestCrossrefWorker(
        kafka_hosts="dummy",
        produce_topic="dummy-produce-topic",
        state_topic="dummy-state-topic",
        contact_email="test@fatcat.wiki",
    )

    harvester.producer = mocker.Mock()

    harvester.fetch_date(datetime.date(2019, 2, 3))

    assert len(responses.calls) == 1

    # ensure email was included in User-Agent
    assert "mailto:test@fatcat.wiki" in responses.calls[0].request.headers["User-Agent"]

    # check that correct date param was passed as expected
    assert "filter=from-update-date%3A2019-02-03" in responses.calls[0].request.url

    # check that we published the expected number of DOI objects were published
    # to the (mock) kafka topic
    assert harvester.producer.produce.call_count == 3
    assert harvester.producer.flush.call_count == 1
    assert harvester.producer.poll.called_once_with(0)
