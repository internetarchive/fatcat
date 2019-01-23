
import re
import sys
import csv
import json
import itertools
import subprocess
from collections import Counter
import pykafka

import fatcat_client
from fatcat_client.rest import ApiException


class EntityImporter:
    """
    Base class for fatcat entity importers.
    
    The API exposed to record iterator is:

        push_record(raw_record)
        finish()

    The API that implementations are expected to fill in are:

        want(raw_record) -> boolean
        parse(raw_record) -> entity
        try_update(entity) -> boolean
        insert_batch([entity]) -> None

    This class exposes helpers for implementations:

        self.api
        self.create_related_*(entity) for all entity types
        self.push_entity(entity)
        self.counts['exits'] += 1 (if didn't update or insert because of existing)
        self.counts['update'] += 1 (if updated an entity)
    """

    def __init__(self, api, **kwargs):

        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['git_rev'] = eg_extra.get('git_rev',
            subprocess.check_output(["git", "describe", "--always"]).strip()).decode('utf-8')
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.EntityImporter')
        
        self.api = api
        self.bezerk_mode = kwargs.get('bezerk_mode', False)
        self._editgroup_description = kwargs.get('editgroup_description')
        self._editgroup_extra = kwargs.get('editgroup_extra')
        self.counts = Counter({'skip': 0, 'insert': 0, 'update': 0, 'exists': 0, 'sub.container': 0})
        self._edit_count = 0
        self._editgroup_id = None
        self._entity_queue = []

    def push_record(self, raw_record):
        """
        Returns nothing.
        """
        if (not raw_record) or (not self.want(raw_record):
            self.counts['skip'] += 1
            return
        entity = self.parse_record(raw_record)
        if self.bezerk_mode:
            self.push_entity(entity)
            return
        if self.try_update(entity):
            self.push_entity(entity)
        return

    def finish(self, raw_record):
        if self._edit_count > 0:
            self.api.accept_editgroup(self._editgroup_id)
            self._editgroup_id = None
            self._edit_count = 0

        if self._entity_queue:
            self.insert_batch(self._entity_queue)
            self.counts['insert'] += len(_entity_queue)
            self._entity_queue = 0

        self.counts['total'] = counts['skip'] + counts['insert'] + \
            counts['update'] + counts['exists']
        return self.counts

    def _get_editgroup(self, edits=1):
        if self._edit_count >= self.edit_batch_size:
            self.api.accept_editgroup(self._editgroup_id)
            self._editgroup_id = None
            self._edit_count = 0

        if not self._editgroup_id:
            eg = self.api.create_editgroup(
                description=self._editgroup_description,
                extra=self._editgroup_extra)
            self._editgroup_id = eg.editgroup_id

        self._edit_count += edits
        return self._editgroup_id

    def create_container(self, entity):
        eg = self._get_editgroup()
        self.api.create_container(entity, editgroup_id=eg.editgroup_id)
        self.counts['sub.container'] += 1

    def updated(self):
        """
        Implementations should call this from try_update() if the update was successful
        """
        self.counts['update'] += 1

    def push_entity(self, entity):
        self._entity_queue.append(entity)
        if len(self._entity_queue) >= self.edit_batch_size:
            self.insert_batch(self._entity_queue)
            self.counts['insert'] += len(_entity_queue)
            self._entity_queue = 0

    def want(self, raw_record):
        """
        Implementations can override for optional fast-path to drop a record.
        Must have no side-effects; returns bool.
        """
        return True

    def parse(self, raw_record):
        """
        Returns an entity class type. expected to always succeed, or raise an
        exception (`want()` should be used to filter out bad records). May have
        side-effects (eg, create related entities), but shouldn't update/mutate
        the actual entity.
        """
        raise NotImplementedError

    def try_update(self, raw_record):
        """
        Passed the output of parse(). Should try to find an existing entity and
        update it (PUT), decide we should do nothing (based on the existing
        record), or create a new one.

        Implementations should update exists/updated counts appropriately.

        Returns boolean: True if 
        """
        raise NotImplementedError

    def insert_batch(self, raw_record):
        raise NotImplementedError


class RecordPusher:
    """
    Base class for different importer sources. Pretty trivial interface, just
    wraps an importer and pushes records in to it.
    """

    def __init__(self, importer, **kwargs):

        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['git_rev'] = eg_extra.get('git_rev',
            subprocess.check_output(["git", "describe", "--always"]).strip()).decode('utf-8')
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.EntityImporter')
        
        self.api = api
        self.bezerk_mode = kwargs.get('bezerk_mode', False)
        self._editgroup_description = kwargs.get('editgroup_description')

    def run(self):
        """
        This will look something like:

            for line in sys.stdin:
                record = json.loads(line)
                self.importer.push_record(record)
            print(self.importer.finish())
        """
        raise NotImplementedError


# from: https://docs.python.org/3/library/itertools.html
def grouper(iterable, n, fillvalue=None):
    "Collect data into fixed-length chunks or blocks"
    args = [iter(iterable)] * n
    return itertools.zip_longest(*args, fillvalue=fillvalue)

def make_kafka_consumer(hosts, env, topic_suffix, group):
    topic_name = "fatcat-{}.{}".format(env, topic_suffix).encode('utf-8')
    client = pykafka.KafkaClient(hosts=hosts, broker_version="1.0.0")
    consume_topic = client.topics[topic_name]
    print("Consuming from kafka topic {}, group {}".format(topic_name, group))

    consumer = consume_topic.get_balanced_consumer(
        consumer_group=group.encode('utf-8'),
        managed=True,
        auto_commit_enable=True,
        auto_commit_interval_ms=30000, # 30 seconds
        compacted_topic=True,
    )
    return consumer

class FatcatImporter:
    """
    Base class for fatcat importers
    """

    def __init__(self, api, **kwargs):

        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['git_rev'] = eg_extra.get('git_rev',
            subprocess.check_output(["git", "describe", "--always"]).strip()).decode('utf-8')
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.FatcatImporter')
        
        self.api = api
        self._editgroup_description = kwargs.get('editgroup_description')
        self._editgroup_extra = kwargs.get('editgroup_extra')
        issn_map_file = kwargs.get('issn_map_file')

        self._issnl_id_map = dict()
        self._orcid_id_map = dict()
        self._doi_id_map = dict()
        if issn_map_file:
            self.read_issn_map_file(issn_map_file)
        self._orcid_regex = re.compile("^\\d{4}-\\d{4}-\\d{4}-\\d{3}[\\dX]$")
        self.counts = Counter({'insert': 0, 'update': 0, 'processed_lines': 0})

    def _editgroup(self):
        eg = fatcat_client.Editgroup(
            description=self._editgroup_description,
            extra=self._editgroup_extra,
        )
        return self.api.create_editgroup(eg)

    def describe_run(self):
        print("Processed {} lines, inserted {}, updated {}.".format(
            self.counts['processed_lines'], self.counts['insert'], self.counts['update']))

    def create_row(self, row, editgroup_id=None):
        # sub-classes expected to implement this
        raise NotImplementedError

    def create_batch(self, rows, editgroup_id=None):
        # sub-classes expected to implement this
        raise NotImplementedError

    def process_source(self, source, group_size=100):
        """Creates and auto-accepts editgroup every group_size rows"""
        eg = self._editgroup()
        i = 0
        for i, row in enumerate(source):
            self.create_row(row, editgroup_id=eg.editgroup_id)
            if i > 0 and (i % group_size) == 0:
                self.api.accept_editgroup(eg.editgroup_id)
                eg = self._editgroup()
            self.counts['processed_lines'] += 1
        if i == 0 or (i % group_size) != 0:
            self.api.accept_editgroup(eg.editgroup_id)

    def process_batch(self, source, size=50, decode_kafka=False):
        """Reads and processes in batches (not API-call-per-)"""
        for rows in grouper(source, size):
            if decode_kafka:
                rows = [msg.value.decode('utf-8') for msg in rows]
            self.counts['processed_lines'] += len(rows)
            #eg = self._editgroup()
            #self.create_batch(rows, editgroup_id=eg.editgroup_id)
            self.create_batch(rows)

    def process_csv_source(self, source, group_size=100, delimiter=','):
        reader = csv.DictReader(source, delimiter=delimiter)
        self.process_source(reader, group_size)

    def process_csv_batch(self, source, size=50, delimiter=','):
        reader = csv.DictReader(source, delimiter=delimiter)
        self.process_batch(reader, size)

    def is_issnl(self, issnl):
        return len(issnl) == 9 and issnl[4] == '-'

    def lookup_issnl(self, issnl):
        """Caches calls to the ISSN-L lookup API endpoint in a local dict"""
        if issnl in self._issnl_id_map:
            return self._issnl_id_map[issnl]
        container_id = None
        try:
            rv = self.api.lookup_container(issnl=issnl)
            container_id = rv.ident
        except ApiException as ae:
            # If anything other than a 404 (not found), something is wrong
            assert ae.status == 404
        self._issnl_id_map[issnl] = container_id # might be None
        return container_id

    def is_orcid(self, orcid):
        return self._orcid_regex.match(orcid) is not None

    def lookup_orcid(self, orcid):
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
            assert ae.status == 404
        self._orcid_id_map[orcid] = creator_id # might be None
        return creator_id

    def is_doi(self, doi):
        return doi.startswith("10.") and doi.count("/") >= 1

    def lookup_doi(self, doi):
        """Caches calls to the doi lookup API endpoint in a local dict"""
        assert self.is_doi(doi)
        doi = doi.lower()
        if doi in self._doi_id_map:
            return self._doi_id_map[doi]
        release_id = None
        try:
            rv = self.api.lookup_release(doi=doi)
            release_id = rv.ident
        except ApiException as ae:
            # If anything other than a 404 (not found), something is wrong
            assert ae.status == 404
        self._doi_id_map[doi] = release_id # might be None
        return release_id

    def read_issn_map_file(self, issn_map_file):
        print("Loading ISSN map file...")
        self._issn_issnl_map = dict()
        for line in issn_map_file:
            if line.startswith("ISSN") or len(line) == 0:
                continue
            (issn, issnl) = line.split()[0:2]
            self._issn_issnl_map[issn] = issnl
            # double mapping makes lookups easy
            self._issn_issnl_map[issnl] = issnl
        print("Got {} ISSN-L mappings.".format(len(self._issn_issnl_map)))

    def issn2issnl(self, issn):
        if issn is None:
            return None
        return self._issn_issnl_map.get(issn)

