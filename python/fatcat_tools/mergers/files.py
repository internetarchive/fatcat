import argparse
import os
import sys
from typing import Any, Dict, List, Optional

import fatcat_openapi_client
from fatcat_openapi_client.models import FileEntity

from fatcat_tools import authenticated_api
from fatcat_tools.importers import JsonLinePusher

from .common import EntityMerger


class FileMerger(EntityMerger):
    """
    Combines file entities into a single primary. Merges any existing partial
    metadata (such as release_ids and URLs). Can choose a primary if necessary.

    The primary is only updated if needed.

    TODO: relies on API server to detect "redirect of redirect" situation
    """

    def __init__(self, api: fatcat_openapi_client.ApiClient, **kwargs) -> None:

        eg_desc = kwargs.get("editgroup_description", "Automated merge of file entities")
        eg_extra = kwargs.get("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.FileMerger")
        super().__init__(api, editgroup_description=eg_desc, editgroup_extra=eg_extra, **kwargs)
        self.entity_type_name = "file"

    def choose_primary_file(self, entities: List[FileEntity]) -> str:
        """
        TODO: could incorporate number of redirected entities already pointing at an entity
        """
        assert entities and len(entities) >= 2

        # want to sort in descending order, so reverse=True
        entities = sorted(
            entities,
            key=lambda a: (
                # has complete metadata?
                bool(a.sha256 and a.md5 and a.sha1 and (a.size is not None)),
                # has releases associated?
                bool(a.release_ids),
                # has URLs?
                bool(a.urls),
                # has extra metadata?
                bool(a.extra),
                # number of release_ids
                len(a.release_ids or []),
            ),
            reverse=True,
        )
        return entities[0].ident

    def merge_file_metadata_from(self, primary: FileEntity, other: FileEntity) -> bool:
        """
        Compares a primary to an other. If there are helpful metadata fields in
        the other, copy them to primary, in-place.

        This is intended to extract any useful metadata from "other" before it
        gets redirected to "primary".

        Returns True if the primary was updated, False otherwise.
        """
        updated = False
        # NOTE: intentionally not including sha1 here
        for k in ["size", "mimetype", "sha256", "md5"]:
            if not getattr(primary, k) and getattr(other, k):
                setattr(primary, k, getattr(other, k))
                updated = True

        if not primary.urls:
            primary.urls = []
        if not primary.release_ids:
            primary.release_ids = []

        if other.extra:
            if not primary.extra:
                primary.extra = other.extra
                updated = True
            else:
                for k in other.extra.keys():
                    if k not in primary.extra:
                        primary.extra[k] = other.extra[k]
                        updated = True

        for u in other.urls or []:
            if u not in primary.urls:
                primary.urls.append(u)
                updated = True

        for i in other.release_ids or []:
            if i not in primary.release_ids:
                primary.release_ids.append(i)
                updated = True

        return updated

    def try_merge(
        self,
        dupe_ids: List[str],
        primary_id: Optional[str] = None,
        evidence: Optional[Dict[str, Any]] = None,
    ) -> int:

        # currently requires for extid validation
        if not evidence or not (evidence.get("extid_type") and evidence.get("extid")):
            self.counts["skip-missing-evidence"] += 1
            return 0

        updated_entities = 0
        entities: Dict[str, FileEntity] = dict()
        eg_id = self.get_editgroup_id()

        all_ids = dupe_ids.copy()
        if primary_id:
            all_ids.append(primary_id)
        for ident in all_ids:
            try:
                entities[ident] = self.api.get_file(ident)
            except fatcat_openapi_client.ApiException as ae:
                if ae.status == 404:
                    self.counts["skip-entity-not-found"] += 1
                    return 0
                else:
                    raise
            if entities[ident].state != "active":
                self.counts["skip-not-active-entity"] += 1
                return 0
            if getattr(entities[ident].ext_ids, evidence["extid_type"]) != evidence["extid"]:
                self.counts["skip-extid-mismatch"] += 1
                return 0

        if not primary_id:
            primary_id = self.choose_primary_file(list(entities.values()))
            dupe_ids = [d for d in dupe_ids if d != primary_id]

        # ensure primary is not in dupes
        assert primary_id not in dupe_ids

        primary = entities[primary_id]
        primary_updated = False
        for other_id in dupe_ids:
            other = entities[other_id]
            primary_updated = self.merge_file_metadata_from(primary, other) or primary_updated
            self.api.update_file(
                eg_id,
                other.ident,
                FileEntity(
                    redirect=primary.ident,
                    edit_extra=evidence,
                ),
            )
            updated_entities += 1

        if primary_updated:
            self.api.update_file(eg_id, primary.ident, primary)
            updated_entities += 1

        return updated_entities


def run_merge_files(args: argparse.Namespace) -> None:
    em = FileMerger(args.api, edit_batch_size=args.batch_size, dry_run_mode=args.dry_run)
    JsonLinePusher(em, args.json_file).run()


def main() -> None:
    """
    Invoke like:

        python3 -m fatcat_tools.mergers.files [options]
    """
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--host-url", default="http://localhost:9411/v0", help="connect to this host/port"
    )
    parser.add_argument("--batch-size", help="size of batch to send", default=50, type=int)
    parser.add_argument(
        "--editgroup-description-override",
        help="editgroup description override",
        default=None,
        type=str,
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="don't actually commit merges, just count what would have been",
    )
    parser.set_defaults(
        auth_var="FATCAT_AUTH_API_TOKEN",
    )
    subparsers = parser.add_subparsers()

    sub_merge_files = subparsers.add_parser("merge-files")
    sub_merge_files.set_defaults(func=run_merge_files)
    sub_merge_files.add_argument(
        "json_file",
        help="source of merge lines to process (or stdin)",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)

    # allow editgroup description override via env variable (but CLI arg takes
    # precedence)
    if not args.editgroup_description_override and os.environ.get(
        "FATCAT_EDITGROUP_DESCRIPTION"
    ):
        args.editgroup_description_override = os.environ.get("FATCAT_EDITGROUP_DESCRIPTION")

    args.api = authenticated_api(
        args.host_url,
        # token is an optional kwarg (can be empty string, None, etc)
        token=os.environ.get(args.auth_var),
    )
    args.func(args)


if __name__ == "__main__":
    main()
