"""
Pubmed harvest via FTP.
"""

import collections
import io
import re
import sys
import tempfile
import xml.etree.ElementTree as ET
from ftplib import FTP
from urllib.parse import urljoin, urlparse

import dateparser
from bs4 import BeautifulSoup
from confluent_kafka import KafkaException, Producer

from .harvest_common import HarvestState


class PubmedFTPWorker:
    """
    Access Pubmed FTP host for daily updates.

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

    The HTML contains the date.

        <html>
        <head><title></title></head>
        <body>
        <h4>Filename: pubmed20n1019.xml -- Created: Wed Dec 18 14:31:09 EST 2019</h4>
        <table cellspacing="0" cellpadding="0" border="0" width="300">
        <tr>

    When this workers starts, it will figure out a mapping from date to XML
    files by looking at all the HTML files.
    """
    def __init__(self, kafka_hosts, produce_topic, state_topic, start_date=None, end_data=None):
        self.host = 'ftp.ncbi.nlm.nih.gov'
        self.produce_topic = produce_topic
        self.state_topic = state_topic
        self.kafka_config = {
            'bootstrap.servers': kafka_hosts,
            'message.max.bytes': 20000000, # ~20 MBytes; broker is ~50 MBytes
        }
        self.loop_sleep = 60*60 # how long to wait, in seconds, between date checks
        self.state = HarvestState(start_date, end_date)
        self.state.initialize_from_kafka(self.state_topic, self.kafka_config)
        self.date_file_map = self.generate_date_file_map()
        if len(self.date_file_map) == 0:
            raise ValueError('mapping from dates to files should not be empty')

    def generate_date_file_map(self):
        """
        Generate a dictionary mapping date (strings) to filepaths. The date is
        parsed from pubmed20n1016_stats.html, mapping to absolute path on FTP,
        e.g. "2020-01-02": "/pubmed/updatefiles/pubmed20n1016.xml.gz".
        """
	mapping = collections.defaultdict(set)
        pattern = re.compile(r'Filename: ([^ ]*.xml) -- Created: ([^<]*)')
        ftp = FTP(self.host)
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
                print('pattern miss on: {}, may need to adjust pattern: {}'.format(contents, pattern),
                      file=sys.stderr)
		continue
	    filename, filedate = match.groups()  # ('pubmed20n1017.xml', 'Tue Dec 17 15:23:32 EST 2019')
	    date = dateparser.parse(filedate)
	    fullpath = '/pubmed/updatefiles/{}.gz'.format(filename)
	    mapping[date.format('%Y-%m-%d')].add(fullpath)

        self.date_file_map = mapping
        print('generated date-file mapping for {} dates'.format(len(mapping)), file=sys.stderr)


    def fetch_date(self, date):
        """
        Fetch file for a given date and feed Kafka one article per message.
        """
        def fail_fast(err, msg):
            if err is not None:
                print("Kafka producer delivery error: {}".format(err), file=sys.stderr)
                print("Bailing out...", file=sys.stderr)
                raise KafkaException(err)

        producer_conf = self.kafka_config.copy()
        producer_conf.update({
            'delivery.report.only.error': True,
            'default.topic.config': {
                'request.required.acks': -1, # all brokers must confirm
            },
        })
        producer = Producer(producer_conf)

        date_str = date.format('%Y-%m-%d')
        paths = self.date_file_map.get(date_str)
        if paths is None:
            print("WARN: no pubmed update for this date: {} (UTC), available dates were: {}".format(date_str, self.date_file_map), file=sys.stderr)
            return

        count = 0
        for path in paths:
            filename = ftpretr(urljoin(self.host, path))
            for blob in xmlstream(filename, 'PubmedArticle', encoding='utf-8'):
                soup = BeautifulSoup(blob)
                pmid = soup.find('PMID')
                if pmid is None:
                    raise ValueError('no PMID found, adjust identifier extraction')
                count += 1
                if count % 50 == 0:
                    print("... up to {} from {}".format(count, filename))
                producer.produce(
                    self.produce_topic,
                    blob,
                    key=pmid.text,
                    on_delivery=fail_fast)
            producer.flush()

    def run(self, continuous=False):
        while True:
            current = self.state.next(continuous)
            if current:
                print("Fetching DOIs updated on {} (UTC)".format(current))
                self.fetch_date(current)
                self.state.complete(current,
                    kafka_topic=self.state_topic,
                    kafka_config=self.kafka_config)
                continue

            if continuous:
                print("Sleeping {} seconds...".format(self.loop_sleep))
                time.sleep(self.loop_sleep)
            else:
                break
        print("{} DOI ingest caught up".format(self.name))


class ftpretr(uri):
    """
    Fetch (RETR) a remote file to a local temporary file.
    """
    parsed = urlparse(uri)
    server, path = parsed.netloc, parsed.path
    ftp = FTP(self.server)
    ftp.login()
    with tempfile.NamedTemporaryFile(prefix='fatcat-ftp-tmp-', delete=False) as f:
        ftp.retrbinary('RETR %s' % path, f.write)
    ftp.close()
    return f.name


def xmlstream(filename, tag, encoding='utf-8'):
    """
    Given a path to an XML file and a tag name (without namespace), stream
    through the XML, and emit the element denoted by tag for processing as string.

    for snippet in xmlstream("sample.xml", "sometag"):
	print(len(snippet))
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
    _, root = next(context)

    for event, elem in context:
        if not strip_ns(elem.tag) == tag or event == 'start':
            continue

        yield ET.tostring(elem, encoding=encoding)
        root.clear()
