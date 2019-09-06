
"""
Tools for merging entities in various ways.

    group-releases: pull all release entities under a single work
        => merges work entities
    merge-releases: merge release entities together
        => groups files/filesets/webcaptures
    merge-containers: merge container entities
    merge-files: merge file entities

Input format is JSON lines with keys:

    idents (required): array of string identifiers
    primary (optional): single string identifier

"""

import subprocess
from collections import Counter

import fatcat_api_client
from fatcat_api_client.rest import ApiException


class EntityMerger:
    """
    API for individual jobs:

        # record iterators sees
        push_record(raw_record)
        finish()

        # provided helpers
        self.api
        self.get_editgroup_id()
        counts({'lines', 'skip', 'merged', 'updated'})

        # implemented per-task
        try_merge(idents, primary=None) -> int (entities updated)

    This class is pretty similar to EntityImporter, but isn't subclassed.
    """

    def __init__(self, api, **kwargs):

        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['git_rev'] = eg_extra.get('git_rev',
            subprocess.check_output(["git", "describe", "--always"]).strip()).decode('utf-8')
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.EntityMerger')

        self.api = api
        self.dry_run_mode = kwargs.get('dry_run_mode', True)
        self.edit_batch_size = kwargs.get('edit_batch_size', 50)
        self.editgroup_description = kwargs.get('editgroup_description')
        self.editgroup_extra = eg_extra
        self.reset()

        if self.dry_run_mode:
            print("Running in dry-run mode!")

    def reset(self):
        self.counts = Counter({'lines': 0, 'skip': 0, 'merged': 0, 'updated-total': 0})
        self._edit_count = 0
        self._editgroup_id = None
        self._entity_queue = []
        self._idents_inflight = []

    def push_record(self, line):
        """
        Intended to be called by "pusher" class (which could be pulling from
        JSON file, Kafka, whatever).

        Input is expected to be a dict-like object with key "idents", and
        optionally "primary".

        Returns nothing.
        """
        self.counts['lines'] += 1
        if (not raw_record):
            self.counts['skip'] += 1
            return
        primary = line.get('primary')
        idents = list(set(line.get('idents')))
        if primary and primary not in idents:
            idents.append(primary)
        if not idents or len(idents) <= 1:
            self.counts['skip'] += 1
            return
        for i in idents:
            if i in self._idents_inflight:
                raise ValueError("Entity already part of in-process merge operation: {}".format(i))
            self._idents.inflight.append(i)
        count = self.try_merge(idents, primary=primary)
        if count:
            self.counts['merged'] += 1
            self.counts['updated-total'] += count
            self._edit_count += count
        else:
            self.counts['skip'] += 1
        if self._edit_count >= self.edit_batch_size:
            self.api.accept_editgroup(self._editgroup_id)
            self._editgroup_id = None
            self._edit_count = 0
            self._idents_inflight = []
        return

    def try_merge(self, idents, primary=None):
        # implementations should fill this in
        raise NotImplementedError

    def finish(self):
        if self._edit_count > 0:
            self.api.accept_editgroup(self._editgroup_id)
            self._editgroup_id = None
            self._edit_count = 0
            self._idents_inflight = []

        return self.counts

    def get_editgroup_id(self):

        if not self._editgroup_id:
            eg = self.api.create_editgroup(
                fatcat_api_client.Editgroup(
                    description=self.editgroup_description,
                    extra=self.editgroup_extra))
            self._editgroup_id = eg.editgroup_id

        return self._editgroup_id
