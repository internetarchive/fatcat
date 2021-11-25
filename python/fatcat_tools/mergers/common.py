"""
Tools for merging entities in various ways.

    merge-releases: merge release entities together
        => groups files/filesets/webcaptures
        => merges work entities if needed
    merge-works: pull all release entities under a single work
        => merges work entities
    merge-containers: merge container entities
    merge-files: merge file entities
"""

import subprocess
from collections import Counter
from typing import Any, Dict, List, Optional

import fatcat_openapi_client

from fatcat_tools.importers import EntityImporter


class EntityMerger(EntityImporter):
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
        try_merge(dupe_ids: List[str], primary_id: Optional[str] = None, evidence: Optional[Dict[str, Any]] = None) -> None
    """

    def __init__(self, api: fatcat_openapi_client.ApiClient, **kwargs) -> None:

        eg_extra = kwargs.get("editgroup_extra", dict())
        eg_extra["git_rev"] = eg_extra.get(
            "git_rev", subprocess.check_output(["git", "describe", "--always"]).strip()
        ).decode("utf-8")
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.EntityMerger")

        self.api = api
        self.dry_run_mode = kwargs.get("dry_run_mode", True)
        self.edit_batch_size = kwargs.get("edit_batch_size", 50)
        self.editgroup_description = kwargs.get("editgroup_description")
        self.editgroup_extra = eg_extra
        self.reset()
        self.entity_type_name = "common"

        if self.dry_run_mode:
            print("Running in dry-run mode!")

    def reset(self) -> None:
        self.counts = Counter({"lines": 0, "skip": 0, "merged": 0, "updated-total": 0})
        self._edit_count = 0
        self._editgroup_id: Optional[str] = None
        self._idents_inflight: List[str] = []

    def push_record(self, record: Dict[str, Any]) -> None:
        """
        Intended to be called by "pusher" class (which could be pulling from
        JSON file, Kafka, whatever).

        Input is expected to be a dict-like object with keys:

            entity_type: str
            primary_id: Optional[str]
            duplicate_ids: [str] (order not preserved)
            evidence: Optional[dict]
                # can be anything, entity- or merger-specific
                # some variables might be...
                extid: str
                extid_type: str

        Returns nothing.
        """
        self.counts["lines"] += 1
        if not record:
            self.counts["skip-blank-line"] += 1
            return
        if record.get("entity_type") != self.entity_type_name:
            self.counts["skip-entity-type"] += 1
            return
        primary_id: Optional[str] = record.get("primary_id")
        duplicate_ids: List[str] = list(set(record["duplicate_ids"]))
        duplicate_ids = [di for di in duplicate_ids if di != primary_id]
        if not duplicate_ids or (len(duplicate_ids) <= 1 and not primary_id):
            self.counts["skip-no-dupes"] += 1
            return
        all_ids = duplicate_ids
        if primary_id:
            all_ids.append(primary_id)
        for i in all_ids:
            if i in self._idents_inflight:
                raise ValueError(
                    "Entity already part of in-process merge operation: {}".format(i)
                )
            self._idents_inflight.append(i)
        count = self.try_merge(
            duplicate_ids, primary_id=primary_id, evidence=record.get("evidence")
        )
        if count:
            self.counts["merged"] += 1
            self.counts["updated-entities"] += count
            self._edit_count += count
        else:
            self.counts["skip"] += 1
        if self._edit_count >= self.edit_batch_size:
            if not self.dry_run_mode:
                self.api.accept_editgroup(self._editgroup_id)
            self._editgroup_id = None
            self._edit_count = 0
            self._idents_inflight = []
        return

    def try_merge(
        self,
        dupe_ids: List[str],
        primary_id: Optional[str] = None,
        evidence: Optional[Dict[str, Any]] = None,
    ) -> int:
        # implementations should fill this in
        raise NotImplementedError

    def finish(self) -> Counter:
        if self._edit_count > 0:
            if not self.dry_run_mode:
                self.api.accept_editgroup(self._editgroup_id)
            self._editgroup_id = None
            self._edit_count = 0
            self._idents_inflight = []

        return self.counts

    def get_editgroup_id(self, _edits: int = 1) -> str:
        """
        This version of get_editgroup_id() is similar to the EntityImporter
        version, but does not update self._edit_count or submit editgroups. The
        edits parameter is ignored.
        """

        if not self._editgroup_id:
            eg = self.api.create_editgroup(
                fatcat_openapi_client.Editgroup(
                    description=self.editgroup_description, extra=self.editgroup_extra
                )
            )
            self._editgroup_id = eg.editgroup_id
        assert self._editgroup_id
        return self._editgroup_id
