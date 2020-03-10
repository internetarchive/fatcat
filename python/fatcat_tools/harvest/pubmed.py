"""
Pubmed harvest via FTP.

Assumptions:

* fixed hostname and directory structure
* XML files are gzip compressed
* accompanying HTML files contain correct dates
"""

import collections
import gzip
import io
import os
import re
import shutil
import sys
import tempfile
import time
import xml.etree.ElementTree as ET
from ftplib import FTP
from urllib.parse import urljoin, urlparse

import dateparser
from bs4 import BeautifulSoup
from confluent_kafka import KafkaException, Producer

from .harvest_common import HarvestState


class PubmedFTPWorker:
    """
    Access Pubmed FTP for daily updates.

    * Server directory: ftp://ftp.ncbi.nlm.nih.gov/pubmed/updatefiles
    * Docs: ftp://ftp.ncbi.nlm.nih.gov/pubmed/updatefiles/README.txt

    Daily Update Files (02/2020)
    ----------------------------
    Each day, NLM produces update files that include new, revised and deleted
    citations. The first Update file to be loaded after loading the complete
    set of 2019 MEDLINE/PubMed Baseline files is pubmed20n1016.xml.

    Usually, three files per update, e.g.:

    * ftp://ftp.ncbi.nlm.nih.gov/pubmed/updatefiles/pubmed20n1016_stats.html
    * ftp://ftp.ncbi.nlm.nih.gov/pubmed/updatefiles/pubmed20n1016.xml.gz
    * ftp://ftp.ncbi.nlm.nih.gov/pubmed/updatefiles/pubmed20n1016.xml.gz.md5

    Currently (02/2020) the HTML contains the date.

        <html>
        <head><title></title></head>
        <body>
        <h4>Filename: pubmed20n1019.xml -- Created: Wed Dec 18 14:31:09 EST 2019</h4>
        <table cellspacing="0" cellpadding="0" border="0" width="300">
        <tr>

    """
    def __init__(self, kafka_hosts, produce_topic, state_topic, start_date=None, end_date=None):
        self.name = 'Pubmed'
        self.host = 'ftp.ncbi.nlm.nih.gov'
        self.produce_topic = produce_topic
        self.state_topic = state_topic
        self.kafka_config = {
            'bootstrap.servers': kafka_hosts,
            'message.max.bytes': 20000000,  # ~20 MBytes; broker is ~50 MBytes
        }
        self.loop_sleep = 60 * 60  # how long to wait, in seconds, between date checks
        self.state = HarvestState(start_date, end_date)
        self.state.initialize_from_kafka(self.state_topic, self.kafka_config)
        self.producer = self._kafka_producer()
        self.date_file_map = None

    def _kafka_producer(self):
        def fail_fast(err, msg):
            if err is not None:
                print("Kafka producer delivery error: {}".format(err), file=sys.stderr)
                print("Bailing out...", file=sys.stderr)
                # TODO: should it be sys.exit(-1)?
                raise KafkaException(err)

        self._kafka_fail_fast = fail_fast

        producer_conf = self.kafka_config.copy()
        producer_conf.update({
            'delivery.report.only.error': True,
            'default.topic.config': {
                'request.required.acks': -1,  # all brokers must confirm
            },
        })
        return Producer(producer_conf)

    def fetch_date(self, date):
        """
        Fetch file for a given date and feed Kafka one article per message. If
        the fetched XML does not contain a PMID, this method will fail.

        If no date file mapping is found, this will fail.
        """
        if self.date_file_map is None:
            raise ValueError("cannot fetch date without date file mapping")

        date_str = date.strftime('%Y-%m-%d')
        paths = self.date_file_map.get(date_str)
        if paths is None:
            print("WARN: no pubmed update for this date: {} (UTC), available dates were: {}".format(date_str, self.date_file_map), file=sys.stderr)
            return False

        count = 0
        for path in paths:
            # Fetch and decompress file.
            url = "ftp://{}{}".format(self.host, path)
            filename = ftpretr(url)
            with tempfile.NamedTemporaryFile(prefix='fatcat-ftp-tmp-', delete=False) as decomp:
                gzf = gzip.open(filename)
                shutil.copyfileobj(gzf, decomp)

            # Here, blob is the unparsed XML; we peek into it to use PMID as
            # message key. We need streaming, since some updates would consume
            # GBs otherwise.
            # WARNING: Parsing foreign XML exposes us at some
            # https://docs.python.org/3/library/xml.html#xml-vulnerabilities
            # here.
            for blob in xmlstream(decomp.name, 'PubmedArticle', encoding='utf-8'):
                soup = BeautifulSoup(blob, 'xml')
                pmid = soup.find('PMID')
                if pmid is None:
                    raise ValueError("no PMID found, please adjust identifier extraction")
                count += 1
                if count % 50 == 0:
                    print("... up to {}".format(count), file=sys.stderr)
                self.producer.produce(self.produce_topic, blob, key=pmid.text, on_delivery=self._kafka_fail_fast)

            self.producer.flush()
            os.remove(filename)
            os.remove(decomp.name)

        return True

    def run(self, continuous=False):
        while True:
            self.date_file_map = generate_date_file_map(host=self.host)
            if len(self.date_file_map) == 0:
                raise ValueError("map from dates to files should not be empty, maybe the HTML changed?")

            current = self.state.next(continuous)
            if current:
                print("Fetching citations updated on {} (UTC)".format(current), file=sys.stderr)
                self.fetch_date(current)
                self.state.complete(current, kafka_topic=self.state_topic, kafka_config=self.kafka_config)
                continue

            if continuous:
                print("Sleeping {} seconds...".format(self.loop_sleep))
                time.sleep(self.loop_sleep)
            else:
                break
        print("{} FTP ingest caught up".format(self.name))


def generate_date_file_map(host='ftp.ncbi.nlm.nih.gov'):
    """
    Generate a DefaultDict[string, set] mapping dates to absolute filepaths on
    the server (mostly we have one file, but sometimes more).

    Example: {"2020-01-02": set(["/pubmed/updatefiles/pubmed20n1016.xml.gz"]), ...}
    """
    mapping = collections.defaultdict(set)
    pattern = re.compile(r'Filename: ([^ ]*.xml) -- Created: ([^<]*)')
    ftp = FTP(host)
    ftp.login()
    filenames = ftp.nlst('/pubmed/updatefiles')

    for name in filenames:
        if not name.endswith('.html'):
            continue
        sio = io.StringIO()
        ftp.retrlines('RETR {}'.format(name), sio.write)
        contents = sio.getvalue()
        match = pattern.search(contents)
        if match is None:
            print('pattern miss in {} on: {}, may need to adjust pattern: {}'.format(name, contents, pattern), file=sys.stderr)
            continue
        filename, filedate = match.groups()  # ('pubmed20n1017.xml', 'Tue Dec 17 15:23:32 EST 2019')
        date = dateparser.parse(filedate)
        fullpath = '/pubmed/updatefiles/{}.gz'.format(filename)
        date_str = date.strftime('%Y-%m-%d')
        mapping[date_str].add(fullpath)
        print('added entry for {}: {}'.format(date_str, fullpath))

    print('generated date-file mapping for {} dates'.format(len(mapping)), file=sys.stderr)
    return mapping


def ftpretr(url):
    """
    Note: This might move into a generic place in the future.

    Fetch (RETR) a remote file given by its URL (e.g.
    "ftp://ftp.ncbi.nlm.nih.gov/pubmed/updatefiles/pubmed20n1016.xml.gz") to a
    local temporary file. Returns the name of the local, closed temporary file.

    It is the reponsibility of the caller to cleanup the temporary file.
    """
    parsed = urlparse(url)
    server, path = parsed.netloc, parsed.path
    ftp = FTP(server)
    ftp.login()
    with tempfile.NamedTemporaryFile(prefix='fatcat-ftp-tmp-', delete=False) as f:
        print('retrieving {} from {} to {} ...'.format(path, server, f.name), file=sys.stderr)
        ftp.retrbinary('RETR %s' % path, f.write)
    ftp.close()
    return f.name


def xmlstream(filename, tag, encoding='utf-8'):
    """
    Note: This might move into a generic place in the future.

    Given a path to an XML file and a tag name (without namespace), stream
    through the XML and yield elements denoted by tag as string.

    for snippet in xmlstream("sample.xml", "sometag"):
        print(len(snippet))

    Known vulnerabilities: https://docs.python.org/3/library/xml.html#xml-vulnerabilities
    """
    def strip_ns(tag):
        if not '}' in tag:
            return tag
        return tag.split('}')[1]

    # https://stackoverflow.com/a/13261805, http://effbot.org/elementtree/iterparse.htm
    context = iter(ET.iterparse(filename, events=(
        'start',
        'end',
    )))
    try:
        _, root = next(context)
    except StopIteration:
        return

    for event, elem in context:
        if not strip_ns(elem.tag) == tag or event == 'start':
            continue

        yield ET.tostring(elem, encoding=encoding)
        root.clear()
