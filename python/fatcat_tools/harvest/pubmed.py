"""
Pubmed harvest via FTP.

Assumptions:

* fixed hostname and directory structure
* XML files are gzip compressed
* accompanying HTML files contain correct dates
"""

import collections
import datetime
import ftplib
import gzip
import io
import os
import re
import shutil
import socket
import subprocess
import sys
import tempfile
import time
import xml.etree.ElementTree as ET
import zlib
from typing import Any, Dict, Generator, Optional
from urllib.parse import urlparse

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

    def __init__(
        self,
        kafka_hosts: str,
        produce_topic: str,
        state_topic: str,
        start_date: Optional[datetime.date] = None,
        end_date: Optional[datetime.date] = None,
    ):
        self.name = "Pubmed"
        self.host = "ftp.ncbi.nlm.nih.gov"
        self.produce_topic = produce_topic
        self.state_topic = state_topic
        self.kafka_config = {
            "bootstrap.servers": kafka_hosts,
            "message.max.bytes": 20000000,  # ~20 MBytes; broker is ~50 MBytes
        }
        self.loop_sleep = 60 * 60  # how long to wait, in seconds, between date checks
        self.state = HarvestState(start_date, end_date)
        self.state.initialize_from_kafka(self.state_topic, self.kafka_config)
        self.producer = self._kafka_producer()
        self.date_file_map: Optional[Dict[str, Any]] = None

    def _kafka_producer(self) -> Producer:
        def fail_fast(err: Any, _msg: None) -> None:
            if err is not None:
                print(f"Kafka producer delivery error: {err}", file=sys.stderr)
                print("Bailing out...", file=sys.stderr)
                # TODO: should it be sys.exit(-1)?
                raise KafkaException(err)

        self._kafka_fail_fast = fail_fast

        producer_conf = self.kafka_config.copy()
        producer_conf.update(
            {
                "delivery.report.only.error": True,
                "default.topic.config": {
                    "request.required.acks": -1,  # all brokers must confirm
                },
            }
        )
        return Producer(producer_conf)

    def fetch_date(self, date: datetime.date) -> bool:
        """
        Fetch file or files for a given date and feed Kafka one article per
        message. If the fetched XML does not contain a PMID an exception is
        raised.

        If no date file mapping is found, this will fail.
        """
        if self.date_file_map is None:
            raise ValueError("cannot fetch date without date file mapping")

        date_str = date.strftime("%Y-%m-%d")
        paths = self.date_file_map.get(date_str)
        if paths is None:
            print(
                "WARN: no pubmed update for this date: {} (UTC), available dates were: {}".format(
                    date_str, self.date_file_map
                ),
                file=sys.stderr,
            )
            return False

        count = 0
        for path in paths:
            # Fetch and decompress file.
            url = f"ftp://{self.host}{path}"
            filename = ftpretr(
                url, proxy_hostport="159.69.240.245:15201"
            )  # TODO: proxy obsolete, when networking issue is resolved
            with tempfile.NamedTemporaryFile(prefix="fatcat-ftp-tmp-", delete=False) as decomp:
                try:
                    gzf = gzip.open(filename)
                    shutil.copyfileobj(gzf, decomp)
                except zlib.error as exc:
                    print(
                        "[skip] retrieving {} failed with {} (maybe empty, missing or broken gzip)".format(
                            url, exc
                        ),
                        file=sys.stderr,
                    )
                    continue

            # Here, blob is the unparsed XML; we peek into it to use PMID as
            # message key. We need streaming, since some updates would consume
            # GBs otherwise.
            # WARNING: Parsing foreign XML exposes us at some
            # https://docs.python.org/3/library/xml.html#xml-vulnerabilities
            # here.
            for blob in xmlstream(decomp.name, "PubmedArticle", encoding="utf-8"):
                soup = BeautifulSoup(blob, "xml")
                pmid = soup.find("PMID")
                if pmid is None:
                    raise ValueError("no PMID found, please adjust identifier extraction")
                count += 1
                if count % 50 == 0:
                    print(f"... up to {count}", file=sys.stderr)
                self.producer.produce(
                    self.produce_topic, blob, key=pmid.text, on_delivery=self._kafka_fail_fast
                )

            self.producer.flush()
            os.remove(filename)
            os.remove(decomp.name)

        return True

    def run(self, continuous: bool = False) -> None:
        while True:
            self.date_file_map = generate_date_file_map(host=self.host)
            if len(self.date_file_map) == 0:
                raise ValueError(
                    "map from dates to files should not be empty, maybe the HTML changed?"
                )

            current = self.state.next_span(continuous)
            if current:
                print(f"Fetching citations updated on {current} (UTC)", file=sys.stderr)
                self.fetch_date(current)
                self.state.complete(
                    current, kafka_topic=self.state_topic, kafka_config=self.kafka_config
                )
                continue

            if continuous:
                print(f"Sleeping {self.loop_sleep} seconds...")
                time.sleep(self.loop_sleep)
            else:
                break
        print(f"{self.name} FTP ingest caught up")


def generate_date_file_map(host: str = "ftp.ncbi.nlm.nih.gov") -> Dict[str, Any]:
    """
    Generate a DefaultDict[string, set] mapping dates to absolute filepaths on
    the server (mostly we have one file, but sometimes more).

    Example: {"2020-01-02": set(["/pubmed/updatefiles/pubmed20n1016.xml.gz"]), ...}
    """
    mapping = collections.defaultdict(set)
    pattern = re.compile(r"Filename: ([^ ]*.xml) -- Created: ([^<]*)")
    ftp = ftplib.FTP(host)
    ftp.login()
    filenames = ftp.nlst("/pubmed/updatefiles")
    retries, retry_delay = 10, 60

    for name in filenames:
        if not name.endswith(".html"):
            continue
        sio = io.StringIO()
        for i in range(retries):
            try:
                # Previously, from 2020-12-14 to 2021-06-30 everything worked
                # fine, then a request for
                # /pubmed/updatefiles/pubmed21n1328_stats.html would always
                # fail with an EOFError, or when retried with a 32
                # BrokenPipeError. Suspecting the server for some unknown
                # reason dropped the connection.
                #
                # Using a fresh client, the exact same file would work just
                # fine. So when we retry, we setup a new client here as well.
                if i > 0:
                    ftp = ftplib.FTP(host)
                    ftp.login()
                    sio.truncate(0)
                ftp.retrlines(f"RETR {name}", sio.write)
            except (EOFError, ftplib.error_temp, socket.gaierror, BrokenPipeError) as exc:
                print(
                    "ftp retr on {} failed with {} ({}) ({} retries left)".format(
                        name, exc, type(exc), retries - (i + 1)
                    ),
                    file=sys.stderr,
                )
                if i + 1 == retries:
                    raise
                else:
                    time.sleep(retry_delay)
            else:
                break
        contents = sio.getvalue()
        match = pattern.search(contents)
        if match is None:
            print(
                "pattern miss in {} on: {}, may need to adjust pattern: {}".format(
                    name, contents, pattern
                ),
                file=sys.stderr,
            )
            continue
        (
            filename,
            filedate,
        ) = match.groups()  # ('pubmed20n1017.xml', 'Tue Dec 17 15:23:32 EST 2019')
        date = dateparser.parse(filedate)
        assert date is not None
        fullpath = f"/pubmed/updatefiles/{filename}.gz"
        date_str = date.strftime("%Y-%m-%d")
        mapping[date_str].add(fullpath)
        print(f"added entry for {date_str}: {fullpath}", file=sys.stderr)

    print(f"generated date-file mapping for {len(mapping)} dates", file=sys.stderr)
    return mapping


def ftpretr(
    url: str, max_retries: int = 10, retry_delay: int = 1, proxy_hostport: Optional[str] = None
) -> str:
    """
    Note: This might move into a generic place in the future.

    Fetch (RETR) a remote file given by its URL (e.g.
    "ftp://ftp.ncbi.nlm.nih.gov/pubmed/updatefiles/pubmed20n1016.xml.gz") to a
    local temporary file. Returns the name of the local, closed temporary file.

    It is the responsibility of the caller to cleanup the temporary file.

    Implements a basic retry mechanism, e.g. that became an issue in 08/2021,
    when we encountered EOFError while talking to the FTP server. Retry delay in seconds.
    """
    if proxy_hostport is not None:
        return ftpretr_via_http_proxy(
            url, proxy_hostport, max_retries=max_retries, retry_delay=retry_delay
        )
    parsed = urlparse(url)
    server, path = parsed.netloc, parsed.path
    for i in range(max_retries):
        try:
            ftp = ftplib.FTP(server)
            ftp.login()
            with tempfile.NamedTemporaryFile(prefix="fatcat-ftp-tmp-", delete=False) as f:
                print(
                    f"retrieving {path} from {server} to {f.name} ...",
                    file=sys.stderr,
                )
                ftp.retrbinary("RETR %s" % path, f.write)
            ftp.close()
        except EOFError as exc:
            print(
                "ftp retrbinary on {} failed with {} ({}) ({} retries left)".format(
                    path, exc, type(exc), max_retries - (i + 1)
                ),
                file=sys.stderr,
            )
            if i + 1 == max_retries:
                raise
            else:
                time.sleep(retry_delay)
        else:
            return f.name
    assert False, "Unreachable code branch"


def ftpretr_via_http_proxy(
    url: str,
    proxy_hostport: str = "ftp.ncbi.nlm.nih.gov",
    max_retries: int = 10,
    retry_delay: int = 1,
) -> str:
    """
    Fetch file from FTP via external HTTP proxy, e.g. ftp.host.com:/a/b/c would
    be retrievable via proxy.com/a/b/c; (in 09/2021 we used
    "159.69.240.245:15201" as proxy_hostport but that started to fail
    2021-10-15; just switch to NIH's http version).
    """
    parsed = urlparse(url)
    _, path = parsed.netloc, parsed.path
    for i in range(max_retries):
        try:
            url = f"http://{proxy_hostport}{path}"
            print(f"retrieving file via proxy (ftpup) from {url}", file=sys.stderr)
            with tempfile.NamedTemporaryFile(prefix="fatcat-ftp-tmp-", delete=False) as f:
                cmd = ["wget", "-c", url, "-O", f.name]
                subprocess.run(cmd)
                return f.name
        except (subprocess.CalledProcessError, OSError, ValueError) as exc:
            print(
                "ftp fetch {} failed with {} ({}) ({} retries left)".format(
                    url, exc, type(exc), max_retries - (i + 1)
                ),
                file=sys.stderr,
            )
            if i + 1 == max_retries:
                raise
            time.sleep(retry_delay)
    assert False, "Unreachable code branch"


def xmlstream(filename: str, tag: str, encoding: str = "utf-8") -> Generator[Any, Any, Any]:
    """
    Note: This might move into a generic place in the future.

    Given a path to an XML file and a tag name (without namespace), stream
    through the XML and yield elements denoted by tag as string.

    for snippet in xmlstream("sample.xml", "sometag"):
        print(len(snippet))

    Known vulnerabilities: https://docs.python.org/3/library/xml.html#xml-vulnerabilities
    """

    def strip_ns(tag: str) -> str:
        if "}" not in tag:
            return tag
        return tag.split("}")[1]

    # https://stackoverflow.com/a/13261805, http://effbot.org/elementtree/iterparse.htm
    context = iter(
        ET.iterparse(
            filename,
            events=(
                "start",
                "end",
            ),
        )
    )
    try:
        _, root = next(context)
    except StopIteration:
        return

    for event, elem in context:
        if not strip_ns(elem.tag) == tag or event == "start":
            continue

        yield ET.tostring(elem, encoding=encoding)
        root.clear()
