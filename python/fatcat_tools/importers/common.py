
import re
import sys
import csv
import json
import sqlite3
import datetime
import subprocess
from collections import Counter
from typing import Dict, Any, List, Optional
import lxml
import xml.etree.ElementTree as ET

import elasticsearch
from bs4 import BeautifulSoup
from confluent_kafka import Consumer, KafkaException

import fatcat_openapi_client
from fatcat_openapi_client import ReleaseEntity
from fatcat_openapi_client.rest import ApiException
from fuzzycat.matching import match_release_fuzzy
import fuzzycat.common
import fuzzycat.verify

# TODO: refactor so remove need for this (re-imports for backwards compatibility)
from fatcat_tools.normal import (clean_str as clean, is_cjk, b32_hex, LANG_MAP_MARC) # noqa: F401
from fatcat_tools.transforms import entity_to_dict


DATE_FMT: str = "%Y-%m-%d"
SANE_MAX_RELEASES: int = 200
SANE_MAX_URLS: int = 100

DOMAIN_REL_MAP: Dict[str, str] = {
    "archive.org": "archive",
    # LOCKSS, Portico, DuraSpace, etc would also be "archive"

    "arxiv.org": "repository",
    "babel.hathitrust.org": "repository",
    "cds.cern.ch": "repository",
    "deepblue.lib.umich.edu": "repository",
    "europepmc.org": "repository",
    "hal.inria.fr": "repository",
    "scielo.isciii.es": "repository",
    "www.dtic.mil": "repository",
    "www.jstage.jst.go.jp": "repository",
    "www.jstor.org": "repository",
    "www.ncbi.nlm.nih.gov": "repository",
    "ftp.ncbi.nlm.nih.gov": "repository",
    "www.scielo.br": "repository",
    "www.scielo.cl": "repository",
    "www.scielo.org.mx": "repository",
    "zenodo.org": "repository",
    "www.biorxiv.org": "repository",
    "www.medrxiv.org": "repository",

    "citeseerx.ist.psu.edu": "aggregator",
    "publisher-connector.core.ac.uk": "aggregator",
    "core.ac.uk": "aggregator",
    "static.aminer.org": "aggregator",
    "aminer.org": "aggregator",
    "pdfs.semanticscholar.org": "aggregator",
    "semanticscholar.org": "aggregator",
    "www.semanticscholar.org": "aggregator",

    "academic.oup.com": "publisher",
    "cdn.elifesciences.org": "publisher",
    "cell.com": "publisher",
    "dl.acm.org": "publisher",
    "downloads.hindawi.com": "publisher",
    "elifesciences.org": "publisher",
    "iopscience.iop.org": "publisher",
    "journals.plos.org": "publisher",
    "link.springer.com": "publisher",
    "onlinelibrary.wiley.com": "publisher",
    "works.bepress.com": "publisher",
    "www.biomedcentral.com": "publisher",
    "www.cell.com": "publisher",
    "www.nature.com": "publisher",
    "www.pnas.org": "publisher",
    "www.tandfonline.com": "publisher",
    "www.frontiersin.org": "publisher",
    "www.degruyter.com": "publisher",
    "www.mdpi.com": "publisher",
    "www.ahajournals.org": "publisher",
    "ehp.niehs.nih.gov": "publisher",
    "journals.tsu.ru": "publisher",
    "www.cogentoa.com": "publisher",

    "www.researchgate.net": "academicsocial",
    "academia.edu": "academicsocial",

    "wayback.archive-it.org": "webarchive",
    "web.archive.org": "webarchive",
    "archive.is": "webarchive",
}

def make_rel_url(raw_url: str, default_link_rel: str = "web"):
    # this is where we map specific domains to rel types, and also filter out
    # bad domains, invalid URLs, etc
    rel = default_link_rel
    for domain, domain_rel in DOMAIN_REL_MAP.items():
        if "//{}/".format(domain) in raw_url:
            rel = domain_rel
            break
    return (rel, raw_url)

def test_make_rel_url():
    assert make_rel_url("http://example.com/thing.pdf")[0] == "web"
    assert make_rel_url("http://example.com/thing.pdf", default_link_rel="jeans")[0] == "jeans"
    assert make_rel_url("https://web.archive.org/web/*/http://example.com/thing.pdf")[0] == "webarchive"
    assert make_rel_url("http://cell.com/thing.pdf")[0] == "publisher"

class EntityImporter:
    """
    Base class for fatcat entity importers.

    The API exposed to record iterator is:

        push_record(raw_record)
        finish()

    The API that implementations are expected to fill in are:

        want(raw_record) -> boolean
        parse_record(raw_record) -> entity
        try_update(entity) -> boolean
        insert_batch([entity]) -> None

    This class exposes helpers for implementations:

        self.api
        self.get_editgroup_id()
        self.create_<entity>(entity) -> EntityEdit
            for related entity types
        self.push_entity(entity)
        self.counts['exists'] += 1
            if didn't update or insert because of existing)
        self.counts['update'] += 1
            if updated an entity

    Parameters:

        submit_mode: instead of accepting editgroups, only submits them.
            implementors must write insert_batch appropriately
    """

    def __init__(self, api, **kwargs):

        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['git_rev'] = eg_extra.get('git_rev',
            subprocess.check_output(["git", "describe", "--always"]).strip()).decode('utf-8')
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.EntityImporter')

        self.api = api
        self.do_updates = bool(kwargs.get('do_updates', True))
        self.do_fuzzy_match: bool = kwargs.get('do_fuzzy_match', True)
        self.bezerk_mode: bool = kwargs.get('bezerk_mode', False)
        self.submit_mode: bool = kwargs.get('submit_mode', False)
        self.edit_batch_size: int = kwargs.get('edit_batch_size', 100)
        self.editgroup_description: Optional[str] = kwargs.get('editgroup_description')
        self.editgroup_extra: Optional[Any] = eg_extra

        self.es_client = kwargs.get('es_client')
        if not self.es_client:
            self.es_client = elasticsearch.Elasticsearch("https://search.fatcat.wiki", timeout=120)

        self._issnl_id_map: Dict[str, Any] = dict()
        self._orcid_id_map: Dict[str, Any] = dict()
        self._orcid_regex = re.compile(r"^\d{4}-\d{4}-\d{4}-\d{3}[\dX]$")
        self._doi_id_map: Dict[str, Any] = dict()
        self._pmid_id_map: Dict[str, Any] = dict()

        self.reset()

    def reset(self) -> None:
        self.counts = Counter({'total': 0, 'skip': 0, 'insert': 0, 'update': 0, 'exists': 0})
        self._edit_count: int = 0
        self._editgroup_id: Optional[str] = None
        self._entity_queue: List[Any] = []
        self._edits_inflight: List[Any] = []

    def push_record(self, raw_record: Any) -> None:
        """
        Returns nothing.
        """
        self.counts['total'] += 1
        if (not raw_record) or (not self.want(raw_record)):
            self.counts['skip'] += 1
            return
        entity = self.parse_record(raw_record)
        if not entity:
            self.counts['skip'] += 1
            return
        if self.bezerk_mode:
            self.push_entity(entity)
            return
        if self.try_update(entity):
            self.push_entity(entity)
        return

    def parse_record(self, raw_record: Any) -> Optional[Any]:
        """
        Returns an entity class type, or None if we should skip this one.

        May have side-effects (eg, create related entities), but shouldn't
        update/mutate the actual entity.
        """
        # implementations should fill this in
        raise NotImplementedError

    def finish(self):
        """
        Gets called as cleanup at the end of imports, but can also be called at
        any time to "snip off" current editgroup progress. In other words, safe
        to call this but then continue pushing records.

        For example, in a persistent worker could call this if there have been
        no new entities fed in for more than some time period, to ensure that
        entities actually get created within a reasonable time frame.
        """
        if self._edit_count > 0:
            if self.submit_mode:
                self.api.submit_editgroup(self._editgroup_id)
            else:
                self.api.accept_editgroup(self._editgroup_id)
            self._editgroup_id = None
            self._edit_count = 0
            self._edits_inflight = []

        if self._entity_queue:
            self.insert_batch(self._entity_queue)
            self.counts['insert'] += len(self._entity_queue)
            self._entity_queue = []

        return self.counts

    def get_editgroup_id(self, edits=1):
        if self._edit_count >= self.edit_batch_size:
            if self.submit_mode:
                self.api.submit_editgroup(self._editgroup_id)
            else:
                self.api.accept_editgroup(self._editgroup_id)
            self._editgroup_id = None
            self._edit_count = 0
            self._edits_inflight = []

        if not self._editgroup_id:
            eg = self.api.create_editgroup(
                fatcat_openapi_client.Editgroup(
                    description=self.editgroup_description,
                    extra=self.editgroup_extra))
            self._editgroup_id = eg.editgroup_id

        self._edit_count += edits
        return self._editgroup_id

    def create_container(self, entity):
        eg_id = self.get_editgroup_id()
        self.counts['inserted.container'] += 1
        return self.api.create_container(eg_id, entity)

    def create_release(self, entity):
        eg_id = self.get_editgroup_id()
        self.counts['inserted.release'] += 1
        return self.api.create_release(eg_id, entity)

    def create_file(self, entity):
        eg_id = self.get_editgroup_id()
        self.counts['inserted.file'] += 1
        return self.api.create_file(eg_id, entity)

    def updated(self):
        """
        Implementations should call this from try_update() if the update was successful
        """
        self.counts['update'] += 1

    def push_entity(self, entity):
        self._entity_queue.append(entity)
        if len(self._entity_queue) >= self.edit_batch_size:
            self.insert_batch(self._entity_queue)
            self.counts['insert'] += len(self._entity_queue)
            self._entity_queue = []

    def want(self, raw_record: Any) -> bool:
        """
        Implementations can override for optional fast-path to drop a record.
        Must have no side-effects; returns bool.
        """
        return True

    def try_update(self, raw_record):
        """
        Passed the output of parse_record(). Should try to find an existing
        entity and update it (PUT), decide we should do nothing (based on the
        existing record), or create a new one.

        Implementations must update the exists/updated/skip counts
        appropriately in this method.

        Returns boolean: True if the entity should still be inserted, False otherwise
        """
        raise NotImplementedError

    def insert_batch(self, raw_records: List[Any]):
        raise NotImplementedError

    def is_orcid(self, orcid: str) -> bool:
        # TODO: replace with clean_orcid() from fatcat_tools.normal
        return self._orcid_regex.match(orcid) is not None

    def lookup_orcid(self, orcid: str):
        """Caches calls to the Orcid lookup API endpoint in a local dict"""
        if not self.is_orcid(orcid):
            return None
        if orcid in self._orcid_id_map:
            return self._orcid_id_map[orcid]
        creator_id = None
        try:
            rv = self.api.lookup_creator(orcid=orcid)
            creator_id = rv.ident
        except ApiException as ae:
            # If anything other than a 404 (not found), something is wrong
            if ae.status != 404:
                raise ae
        self._orcid_id_map[orcid] = creator_id # might be None
        return creator_id

    def is_doi(self, doi: str) -> bool:
        # TODO: replace with clean_doi() from fatcat_tools.normal
        return doi.startswith("10.") and doi.count("/") >= 1

    def lookup_doi(self, doi: str):
        """Caches calls to the doi lookup API endpoint in a local dict

        For identifier lookups only (not full object fetches)"""
        assert self.is_doi(doi)
        doi = doi.lower()
        if doi in self._doi_id_map:
            return self._doi_id_map[doi]
        release_id = None
        try:
            rv = self.api.lookup_release(doi=doi, hide="abstracts,refs,contribs")
            release_id = rv.ident
        except ApiException as ae:
            # If anything other than a 404 (not found), something is wrong
            if ae.status != 404:
                raise ae
        self._doi_id_map[doi] = release_id # might be None
        return release_id

    def lookup_pmid(self, pmid: str):
        """Caches calls to the pmid lookup API endpoint in a local dict

        For identifier lookups only (not full object fetches)"""
        if pmid in self._pmid_id_map:
            return self._pmid_id_map[pmid]
        release_id = None
        try:
            rv = self.api.lookup_release(pmid=pmid, hide="abstracts,refs,contribs")
            release_id = rv.ident
        except ApiException as ae:
            # If anything other than a 404 (not found), something is wrong
            if ae.status != 404:
                raise ae
        self._pmid_id_map[pmid] = release_id # might be None
        return release_id

    def is_issnl(self, issnl: str) -> bool:
        return len(issnl) == 9 and issnl[4] == '-'

    def lookup_issnl(self, issnl: str):
        """Caches calls to the ISSN-L lookup API endpoint in a local dict"""
        if issnl in self._issnl_id_map:
            return self._issnl_id_map[issnl]
        container_id = None
        try:
            rv = self.api.lookup_container(issnl=issnl)
            container_id = rv.ident
        except ApiException as ae:
            # If anything other than a 404 (not found), something is wrong
            if ae.status != 404:
                raise ae
        self._issnl_id_map[issnl] = container_id # might be None
        return container_id

    def read_issn_map_file(self, issn_map_file):
        print("Loading ISSN map file...", file=sys.stderr)
        self._issn_issnl_map = dict()
        for line in issn_map_file:
            if line.startswith("ISSN") or len(line) == 0:
                continue
            (issn, issnl) = line.split()[0:2]
            self._issn_issnl_map[issn] = issnl
            # double mapping makes lookups easy
            self._issn_issnl_map[issnl] = issnl
        print("Got {} ISSN-L mappings.".format(len(self._issn_issnl_map)), file=sys.stderr)

    def issn2issnl(self, issn: str) -> Optional[str]:
        if issn is None:
            return None
        return self._issn_issnl_map.get(issn)

    @staticmethod
    def generic_file_cleanups(existing):
        """
        Conservative cleanup of existing file entities.

        Intended to be used in most bulk cleanups and other file entity
        updates, to reduce edit volume for catalog size/churn efficiency.

        Note: the former check for 'None' as a wayback datetime has been
        completely cleaned up
        """

        # update old/deprecated 'rel' on URLs
        for i in range(len(existing.urls)):
            u = existing.urls[i]
            if u.rel == 'repository' and '://archive.org/download/' in u.url:
                existing.urls[i].rel = 'archive'
            if u.rel == 'social':
                u.rel = 'academicsocial'

        # remove URLs which are near-duplicates
        redundant_urls = []
        all_urls = [u.url for u in existing.urls]
        all_wayback_urls = [u.url for u in existing.urls if '://web.archive.org/web/' in u.url]
        for url in all_urls:
            # https/http redundancy
            if url.startswith('http://') and url.replace('http://', 'https://', 1) in all_urls:
                redundant_urls.append(url)
                continue
            # default HTTP port included and not included
            if ':80/' in url and url.replace(':80', '', 1) in all_urls:
                redundant_urls.append(url)
                continue
            # partial and complete wayback timestamps
            if '://web.archive.org/web/2017/' in url:
                original_url = "/".join(url.split("/")[5:])
                assert len(original_url) > 5
                for wb_url in all_wayback_urls:
                    alt_timestamp = wb_url.split("/")[4]
                    if len(alt_timestamp) >= 10 and original_url in wb_url:
                        redundant_urls.append(url)
                        break

        existing.urls = [u for u in existing.urls if u.url not in redundant_urls]
        return existing

    @staticmethod
    def generic_fileset_cleanups(existing):
        return existing

    def match_existing_release_fuzzy(self, release: ReleaseEntity) -> Optional[Tuple[str, str, ReleaseEntity]]:
        """
        This helper function uses fuzzycat (and elasticsearch) to look for
        existing release entities with similar metadata.

        Returns None if there was no match of any kind, or a single tuple
        (status: str, reason: str, existing: ReleaseEntity) if there was a match.

        Status string is one of the fuzzycat.common.Status, with "strongest
        match" in this sorted order:

        - EXACT
        - STRONG
        - WEAK
        - AMBIGUOUS

        Eg, if there is any EXACT match that is always returned; an AMBIGIOUS
        result is only returned if all the candidate matches were ambiguous.
        """

        # this map used to establish priority order of verified matches
        STATUS_SORT = {
            fuzzycat.common.Status.TODO: 0,
            fuzzycat.common.Status.EXACT: 10,
            fuzzycat.common.Status.STRONG: 20,
            fuzzycat.common.Status.WEAK: 30,
            fuzzycat.common.Status.AMBIGUOUS: 40,
            fuzzycat.common.Status.DIFFERENT: 60,
        }

        # TODO: the size here is a first guess; what should it really be?
        candidates = match_release_fuzzy(release, size=10, es=self.es_client)
        if not candidates:
            return None

        release_dict = entity_to_dict(release, api_client=self.api.api_client)
        verified = [(fuzzycat.verify.verify(release_dict, entity_to_dict(c, api_client=self.api.api_client)), c) for c in candidates]

        # chose the "closest" match
        closest = sorted(verified, key=lambda v: STATUS_SORT[v[0].status])[0]
        if closest[0].status == fuzzycat.common.Status.DIFFERENT:
            return None
        elif closest[0].status == fuzzycat.common.Status.TODO:
            raise NotImplementedError("fuzzycat verify hit a Status.TODO")
        else:
            return (closest[0].status.name, closest[0].reason.value, closest[1])


class RecordPusher:
    """
    Base class for different importer sources. Pretty trivial interface, just
    wraps an importer and pushes records in to it.
    """

    def __init__(self, importer, **kwargs):
        self.importer = importer

    def run(self):
        """
        This will look something like:

            for line in sys.stdin:
                record = json.loads(line)
                self.importer.push_record(record)
            print(self.importer.finish())
        """
        raise NotImplementedError


class JsonLinePusher(RecordPusher):

    def __init__(self, importer, json_file, **kwargs):
        self.importer = importer
        self.json_file = json_file

    def run(self):
        for line in self.json_file:
            if not line:
                continue
            record = json.loads(line)
            self.importer.push_record(record)
        counts = self.importer.finish()
        print(counts, file=sys.stderr)
        return counts


class CsvPusher(RecordPusher):

    def __init__(self, importer, csv_file, **kwargs):
        self.importer = importer
        self.reader = csv.DictReader(csv_file, delimiter=kwargs.get('delimiter', ','))

    def run(self):
        for line in self.reader:
            if not line:
                continue
            self.importer.push_record(line)
        counts = self.importer.finish()
        print(counts, file=sys.stderr)
        return counts


class LinePusher(RecordPusher):

    def __init__(self, importer, text_file, **kwargs):
        self.importer = importer
        self.text_file = text_file

    def run(self):
        for line in self.text_file:
            if not line:
                continue
            self.importer.push_record(line)
        counts = self.importer.finish()
        print(counts, file=sys.stderr)
        return counts


class SqlitePusher(RecordPusher):

    def __init__(self, importer, db_file, table_name, where_clause="", **kwargs):
        self.importer = importer
        self.db = sqlite3.connect(db_file, isolation_level='EXCLUSIVE')
        self.db.row_factory = sqlite3.Row
        self.table_name = table_name
        self.where_clause = where_clause

    def run(self):
        cur = self.db.execute("SELECT * FROM {} {};".format(
            self.table_name, self.where_clause))
        for row in cur:
            self.importer.push_record(row)
        counts = self.importer.finish()
        print(counts, file=sys.stderr)
        return counts


class Bs4XmlLinesPusher(RecordPusher):

    def __init__(self, importer, xml_file, prefix_filter=None, **kwargs):
        self.importer = importer
        self.xml_file = xml_file
        self.prefix_filter = prefix_filter

    def run(self):
        for line in self.xml_file:
            if not line:
                continue
            if self.prefix_filter and not line.startswith(self.prefix_filter):
                continue
            soup = BeautifulSoup(line, "xml")
            self.importer.push_record(soup)
            soup.decompose()
        counts = self.importer.finish()
        print(counts, file=sys.stderr)
        return counts


class Bs4XmlFilePusher(RecordPusher):

    def __init__(self, importer, xml_file, record_tag, **kwargs):
        self.importer = importer
        self.xml_file = xml_file
        self.record_tag = record_tag

    def run(self):
        soup = BeautifulSoup(self.xml_file, "xml")
        for record in soup.find_all(self.record_tag):
            self.importer.push_record(record)
            record.decompose()
        counts = self.importer.finish()
        soup.decompose()
        print(counts, file=sys.stderr)
        return counts


class Bs4XmlLargeFilePusher(RecordPusher):
    """
    This is a variant of Bs4XmlFilePusher which parses large files
    incrementally, instead of loading the whole thing in RAM first.

    The dominant source of RAM utilization at start-up is the large ISSN/ISSN-L
    map. This can be confirmed in local development by using the small map in
    ./tests/files/.

    Current implementation is weird/inefficient in that it re-parses with
    BeautifulSoup (lxml) every article, but I didn't want to mangle or re-write
    with a different BS back-end.

    Did at least casual testing and all of: record.decompose(),
    soup.decompose(), element.clear(), root.clear() helped with memory usage.
    With all of these, memory growth is very slow and can probably be explained
    by inner container/release API lookup caches.
    """

    def __init__(self, importer, xml_file, record_tags, use_lxml=False, **kwargs):
        self.importer = importer
        self.xml_file = xml_file
        self.record_tags = record_tags
        self.use_lxml = use_lxml

    def run(self):
        if self.use_lxml:
            elem_iter = lxml.etree.iterparse(self.xml_file, ["start", "end"], load_dtd=True)
        else:
            elem_iter = ET.iterparse(self.xml_file, ["start", "end"])
        root = None
        for (event, element) in elem_iter:
            if (root is not None) and event == "start":
                root = element
                continue
            if not (element.tag in self.record_tags and event == "end"):
                continue
            if self.use_lxml:
                soup = BeautifulSoup(lxml.etree.tostring(element), "xml")
            else:
                soup = BeautifulSoup(ET.tostring(element), "xml")
            for record in soup.find_all():
                if record.name not in self.record_tags:
                    continue
                self.importer.push_record(record)
                record.decompose()
            soup.decompose()
            element.clear()
            if root is not None:
                root.clear()
        counts = self.importer.finish()
        print(counts, file=sys.stderr)
        return counts


class Bs4XmlFileListPusher(RecordPusher):

    def __init__(self, importer, list_file, record_tag, **kwargs):
        self.importer = importer
        self.list_file = list_file
        self.record_tag = record_tag

    def run(self):
        for xml_path in self.list_file:
            xml_path = xml_path.strip()
            if not xml_path or xml_path.startswith("#"):
                continue
            with open(xml_path, 'r') as xml_file:
                soup = BeautifulSoup(xml_file, "xml")
                for record in soup.find_all(self.record_tag):
                    self.importer.push_record(record)
                    record.decompose()
                soup.decompose()
        counts = self.importer.finish()
        print(counts)
        return counts

class KafkaBs4XmlPusher(RecordPusher):
    """
    Fetch XML for an article from Kafka, parse via Bs4.
    """
    def __init__(self, importer, kafka_hosts, kafka_env, topic_suffix, group, **kwargs):
        self.importer = importer
        self.consumer = make_kafka_consumer(
            kafka_hosts,
            kafka_env,
            topic_suffix,
            group,
            kafka_namespace=kwargs.get('kafka_namespace', 'fatcat')
        )
        self.poll_interval = kwargs.get('poll_interval', 5.0)
        self.consume_batch_size = kwargs.get('consume_batch_size', 25)

    def run(self):
        count = 0
        last_push = datetime.datetime.now()
        while True:
            # Note: this is batch-oriented, because underlying importer is
            # often batch-oriented, but this doesn't confirm that entire batch
            # has been pushed to fatcat before commiting offset. Eg, consider
            # case where there there is one update and thousands of creates;
            # update would be lingering in importer, and if importer crashed
            # never created.
            # This is partially mitigated for the worker case by flushing any
            # outstanding editgroups every 5 minutes, but there is still that
            # window when editgroups might be hanging (unsubmitted).
            batch = self.consumer.consume(
                num_messages=self.consume_batch_size,
                timeout=self.poll_interval)
            print("... got {} kafka messages ({}sec poll interval) {}".format(
                len(batch), self.poll_interval, self.importer.counts))
            if not batch:
                if datetime.datetime.now() - last_push > datetime.timedelta(minutes=5):
                    # it has been some time, so flush any current editgroup
                    self.importer.finish()
                    last_push = datetime.datetime.now()
                    #print("Flushed any partial import batch: {}".format(self.importer.counts))
                continue
            # first check errors on entire batch...
            for msg in batch:
                if msg.error():
                    raise KafkaException(msg.error())
            # ... then process
            for msg in batch:
                soup = BeautifulSoup(msg.value().decode('utf-8'), "xml")
                self.importer.push_record(soup)
                soup.decompose()
                count += 1
                if count % 500 == 0:
                    print("Import counts: {}".format(self.importer.counts))
            last_push = datetime.datetime.now()
            for msg in batch:
                # locally store offsets of processed messages; will be
                # auto-commited by librdkafka from this "stored" value
                self.consumer.store_offsets(message=msg)

        # TODO: should catch UNIX signals (HUP?) to shutdown cleanly, and/or
        # commit the current batch if it has been lingering
        counts = self.importer.finish()
        print(counts)
        self.consumer.close()
        return counts

class KafkaJsonPusher(RecordPusher):

    def __init__(self, importer, kafka_hosts, kafka_env, topic_suffix, group, **kwargs):
        self.importer = importer
        self.consumer = make_kafka_consumer(
            kafka_hosts,
            kafka_env,
            topic_suffix,
            group,
            kafka_namespace=kwargs.get('kafka_namespace', 'fatcat')
        )
        self.poll_interval = kwargs.get('poll_interval', 5.0)
        self.consume_batch_size = kwargs.get('consume_batch_size', 100)
        self.force_flush = kwargs.get('force_flush', False)

    def run(self):
        count = 0
        last_push = datetime.datetime.now()
        last_force_flush = datetime.datetime.now()
        while True:
            # Note: this is batch-oriented, because underlying importer is
            # often batch-oriented, but this doesn't confirm that entire batch
            # has been pushed to fatcat before committing offset. Eg, consider
            # case where there there is one update and thousands of creates;
            # update would be lingering in importer, and if importer crashed
            # never created.
            # This is partially mitigated for the worker case by flushing any
            # outstanding editgroups every 5 minutes, but there is still that
            # window when editgroups might be hanging (unsubmitted).
            batch = self.consumer.consume(
                num_messages=self.consume_batch_size,
                timeout=self.poll_interval)
            print("... got {} kafka messages ({}sec poll interval) {}".format(
                len(batch), self.poll_interval, self.importer.counts))
            if self.force_flush:
                # this flushing happens even if there have been 'push' events
                # more recently. it is intended for, eg, importers off the
                # ingest file stream *other than* the usual file importer. the
                # web/HTML and savepapernow importers get frequent messages,
                # but rare 'want() == True', so need an extra flush
                if datetime.datetime.now() - last_force_flush > datetime.timedelta(minutes=5):
                    self.importer.finish()
                    last_push = datetime.datetime.now()
                    last_force_flush = datetime.datetime.now()
            if not batch:
                if datetime.datetime.now() - last_push > datetime.timedelta(minutes=5):
                    # it has been some time, so flush any current editgroup
                    self.importer.finish()
                    last_push = datetime.datetime.now()
                    last_force_flush = datetime.datetime.now()
                    #print("Flushed any partial import batch: {}".format(self.importer.counts))
                continue
            # first check errors on entire batch...
            for msg in batch:
                if msg.error():
                    raise KafkaException(msg.error())
            # ... then process
            for msg in batch:
                record = json.loads(msg.value().decode('utf-8'))
                self.importer.push_record(record)
                count += 1
                if count % 500 == 0:
                    print("Import counts: {}".format(self.importer.counts))
            last_push = datetime.datetime.now()
            for msg in batch:
                # locally store offsets of processed messages; will be
                # auto-commited by librdkafka from this "stored" value
                self.consumer.store_offsets(message=msg)

        # TODO: should catch UNIX signals (HUP?) to shutdown cleanly, and/or
        # commit the current batch if it has been lingering
        counts = self.importer.finish()
        print(counts)
        self.consumer.close()
        return counts


def make_kafka_consumer(hosts, env, topic_suffix, group, kafka_namespace="fatcat"):
    topic_name = "{}-{}.{}".format(kafka_namespace, env, topic_suffix)

    def fail_fast(err, partitions):
        if err is not None:
            print("Kafka consumer commit error: {}".format(err))
            print("Bailing out...")
            # TODO: should it be sys.exit(-1)?
            raise KafkaException(err)
        for p in partitions:
            # check for partition-specific commit errors
            if p.error:
                print("Kafka consumer commit error: {}".format(p.error))
                print("Bailing out...")
                # TODO: should it be sys.exit(-1)?
                raise KafkaException(p.error)
        #print("Kafka consumer commit successful")
        pass

    # previously, using pykafka
    #auto_commit_enable=True,
    #auto_commit_interval_ms=30000, # 30 seconds
    conf = {
        'bootstrap.servers': hosts,
        'group.id': group,
        'on_commit': fail_fast,
        # messages don't have offset marked as stored until pushed to
        # elastic, but we do auto-commit stored offsets to broker
        'enable.auto.offset.store': False,
        'enable.auto.commit': True,
        # user code timeout; if no poll after this long, assume user code
        # hung and rebalance (default: 5min)
        'max.poll.interval.ms': 120000,
        'default.topic.config': {
            'auto.offset.reset': 'latest',
        },
    }

    def on_rebalance(consumer, partitions):
        for p in partitions:
            if p.error:
                raise KafkaException(p.error)
        print("Kafka partitions rebalanced: {} / {}".format(
            consumer, partitions))

    consumer = Consumer(conf)
    # NOTE: it's actually important that topic_name *not* be bytes (UTF-8
    # encoded)
    consumer.subscribe([topic_name],
        on_assign=on_rebalance,
        on_revoke=on_rebalance,
    )
    print("Consuming from kafka topic {}, group {}".format(topic_name, group))
    return consumer
