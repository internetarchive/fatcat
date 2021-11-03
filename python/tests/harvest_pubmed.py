"""
Test pubmed FTP harvest.
"""

import datetime
import os

import pytest

from fatcat_tools.harvest import *


def test_pubmed_harvest_date(mocker):

    # mock out the harvest state object so it doesn't try to actually connect
    # to Kafka
    mocker.patch('fatcat_tools.harvest.harvest_common.HarvestState.initialize_from_kafka')

    # Mocking a file fetched from FTP, should contain some 'PubmedArticle' elements.
    # $ zcat tests/files/pubmedsample_2019.xml.gz | grep -c '<PubmedArticle>'
    # 176
    file_to_retrieve = os.path.join(os.path.dirname(__file__), 'files/pubmedsample_2019.xml.gz')
    ftpretr = mocker.patch('fatcat_tools.harvest.pubmed.ftpretr')
    ftpretr.return_value = file_to_retrieve

    test_date = '2020-02-20'

    # We'll need one entry in the date_file_map.
    generate_date_file_map = mocker.patch('fatcat_tools.harvest.pubmed.generate_date_file_map')
    generate_date_file_map.return_value = {test_date: set(['dummy'])}

    # For cleanup.
    os.remove = mocker.Mock()

    harvester = PubmedFTPWorker(
        kafka_hosts="dummy",
        produce_topic="dummy-produce-topic",
        state_topic="dummy-state-topic",
    )

    harvester.producer = mocker.Mock()
    harvester.date_file_map = generate_date_file_map()
    # Since we mock out the FTP fetch, the concrete date does not matter here.
    harvester.fetch_date(datetime.datetime.strptime(test_date, '%Y-%m-%d'))

    # check that we published the expected number of DOI objects were published
    # to the (mock) kafka topic
    assert harvester.producer.produce.call_count == 176
    assert harvester.producer.flush.call_count == 1
    assert os.remove.call_count == 2

def test_pubmed_harvest_date_no_pmid(mocker):
    # mock out the harvest state object so it doesn't try to actually connect
    # to Kafka
    mocker.patch('fatcat_tools.harvest.harvest_common.HarvestState.initialize_from_kafka')

    file_to_retrieve = os.path.join(os.path.dirname(__file__), 'files/pubmedsample_no_pmid_2019.xml.gz')
    ftpretr = mocker.patch('fatcat_tools.harvest.pubmed.ftpretr')
    ftpretr.return_value = file_to_retrieve

    test_date = '2020-02-20'

    # We'll need one entry in the date_file_map.
    generate_date_file_map = mocker.patch('fatcat_tools.harvest.pubmed.generate_date_file_map')
    generate_date_file_map.return_value = {test_date: set(['dummy'])}

    harvester = PubmedFTPWorker(
        kafka_hosts="dummy",
        produce_topic="dummy-produce-topic",
        state_topic="dummy-state-topic",
    )

    harvester.producer = mocker.Mock()

    # The file has not PMID, not importable.
    with pytest.raises(ValueError):
        harvester.fetch_date(datetime.datetime.strptime(test_date, '%Y-%m-%d'))
