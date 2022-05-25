import copy
import json
import subprocess
import sys
from collections import Counter
from typing import Any, Dict, List, Sequence

from fatcat_openapi_client import ApiClient, Editgroup

from fatcat_tools.transforms import entity_from_dict, entity_to_dict


class EntityCleaner:
    """
    API for individual jobs:

        # record iterators sees
        push_record(record)
        finish()

        # provided helpers
        self.api
        self.get_editgroup_id()
        counts({'lines', 'skip', 'merged', 'updated'})

        # implemented per-task
        try_merge(idents, primary=None) -> int (entities updated)

    This class is pretty similar to EntityImporter, but isn't subclassed.
    """

    def __init__(self, api: ApiClient, entity_type: Any, **kwargs) -> None:

        eg_extra = kwargs.get("editgroup_extra", dict())
        eg_extra["git_rev"] = eg_extra.get(
            "git_rev", subprocess.check_output(["git", "describe", "--always"]).strip()
        ).decode("utf-8")
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.EntityCleaner")

        self.api = api
        self.entity_type = entity_type
        self.dry_run_mode = kwargs.get("dry_run_mode", True)
        self.edit_batch_size = kwargs.get("edit_batch_size", 50)
        self.editgroup_description = kwargs.get(
            "editgroup_description", "Generic Entity Cleaner Bot"
        )
        self.editgroup_extra = eg_extra
        self.reset()
        self.ac = ApiClient()

        if self.dry_run_mode:
            print("Running in dry-run mode!")

    def reset(self) -> None:
        self.counts = Counter({"lines": 0, "cleaned": 0, "updated": 0})
        self._edit_count = 0
        self._editgroup_id = None
        self._entity_queue: List[Any] = []
        self._idents_inflight: List[str] = []

    def push_record(self, record: Dict[str, Any]) -> None:
        """
        Intended to be called by "pusher" class (which could be pulling from
        JSON file, Kafka, whatever).

        Input is expected to be an entity in JSON-like dict form.

        Returns nothing.
        """
        self.counts["lines"] += 1
        if not record:
            self.counts["skip-null"] += 1
            return

        entity = entity_from_dict(record, self.entity_type, api_client=self.ac)

        if entity.state != "active":
            self.counts["skip-inactive"] += 1
            return

        cleaned = self.clean_entity(copy.deepcopy(entity))
        if entity == cleaned:
            self.counts["skip-clean"] += 1
            return
        else:
            self.counts["cleaned"] += 1

        if self.dry_run_mode:
            entity_dict = entity_to_dict(entity, api_client=self.ac)
            print(json.dumps(entity_dict))
            return

        if entity.ident in self._idents_inflight:
            raise ValueError(
                f"Entity already part of in-process update: {entity.ident}"
            )

        updated = self.try_update(cleaned)
        if updated:
            self.counts["updated"] += updated
            self._edit_count += updated
            self._idents_inflight.append(entity.ident)

        if self._edit_count >= self.edit_batch_size:
            self.api.accept_editgroup(self._editgroup_id)
            self._editgroup_id = None
            self._edit_count = 0
            self._idents_inflight = []
        return

    def clean_entity(self, entity: Any) -> Any:
        """
        Mutates entity in-place and returns it
        """
        # implementations should fill this in
        raise NotImplementedError

    def try_update(self, entity: Any) -> int:
        """
        Returns edit count (number of entities updated).

        If >= 1, does not need to update self.counts. If no entities updated,
        do need to update counts internally.
        """
        # implementations should fill this in
        raise NotImplementedError

    def finish(self) -> Counter:
        if self._edit_count > 0:
            self.api.accept_editgroup(self._editgroup_id)
            self._editgroup_id = None
            self._edit_count = 0
            self._idents_inflight = []

        return self.counts

    def get_editgroup_id(self) -> str:

        if not self._editgroup_id:
            eg = self.api.create_editgroup(
                Editgroup(description=self.editgroup_description, extra=self.editgroup_extra)
            )
            self._editgroup_id = eg.editgroup_id

        assert self._editgroup_id
        return self._editgroup_id


class JsonLinePusher:
    def __init__(self, cleaner: EntityCleaner, json_file: Sequence, **kwargs) -> None:
        self.cleaner = cleaner
        self.json_file = json_file

    def run(self) -> Counter:
        for line in self.json_file:
            if not line:
                continue
            record = json.loads(line)
            self.cleaner.push_record(record)
        counts = self.cleaner.finish()
        print(counts, file=sys.stderr)
        return counts
