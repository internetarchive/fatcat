import argparse
import os
import sys
from typing import Any, Dict, List, Optional

import fatcat_openapi_client
from fatcat_openapi_client.models import ReleaseEntity, WorkEntity

from fatcat_tools import authenticated_api
from fatcat_tools.importers import JsonLinePusher

from .common import EntityMerger


class ReleaseMerger(EntityMerger):
    """
    Hard merges a set of release entities, redirecting all entities to a single
    primary release. This is different from "grouping" multiple releases under
    a single work.

    A "primary" (which the other releases will redirect to) can be provided, or
    one will be chosen from the set of duplicate releases based on the
    completeness of metadata and other heuristic factors.

    Releases are some of the most complex entities to merge, because of
    the complexity of bibliographic metadata and the number of related entities
    which also need to be updated.

    File, Fileset, and Webcapture entities which currently point to a release
    which gets redirected will be updated to point at the "primary" release.

    Any Work entities which will end up with no releases pointing at them after
    the merging will get redirected to the work corresponding to the "primary"
    release.

    NOTE: the "primary" release will currently (as implemented) *not* get
    updated with metadata from all the redirected releases
    """

    def __init__(self, api: fatcat_openapi_client.ApiClient, **kwargs) -> None:

        eg_desc = (
            kwargs.pop("editgroup_description", None) or "Automated merge of release entities"
        )
        eg_extra = kwargs.get("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.ReleaseMerger")
        self.dry_run_mode: bool = eg_extra.get("dry_run_mode", False)
        super().__init__(api, editgroup_description=eg_desc, editgroup_extra=eg_extra, **kwargs)
        self.entity_type_name = "release"

    def choose_primary_release(
        self, entities: List[ReleaseEntity], existing_redirects: Dict[str, List[str]]
    ) -> str:
        assert entities and len(entities) >= 2

        # want to sort in descending order, so reverse=True
        entities = sorted(
            entities,
            key=lambda a: (
                # number of entities already redirected to this one
                len(existing_redirects[a.ident]),
                # number of file/fileset/webcapture entities (would need to update)
                int(len(a.files or []) + len(a.filesets or []) + len(a.webcaptures or [])),
                # has a strong identifier?
                bool(a.ext_id.doi or a.ext_id.pmid or a.ext_id.arxiv_id),
                # has any identifier?
                bool(a.ext_id),
                # has basic metadata?
                bool(a.release_type),
                bool(a.release_status),
                bool(a.release_year),
                bool(a.container_id),
                # has refs, abstracts, extra stuff?
                bool(a.refs),
                bool(a.abstracts),
            ),
            reverse=True,
        )
        return entities[0].ident

    def try_merge(
        self,
        dupe_ids: List[str],
        primary_id: Optional[str] = None,
        evidence: Optional[Dict[str, Any]] = None,
    ) -> int:

        # TODO: this code is pretty old and has only been partially refactored.
        # Needs more testing and review.
        raise NotImplementedError

        updated_entities = 0
        releases = dict()
        existing_redirects: Dict[str, List[str]] = dict()

        all_ids = dupe_ids.copy()
        if primary_id:
            all_ids.append(primary_id)
        for ident in all_ids:
            releases[ident] = self.api.get_release(ident, expand="files,filesets,webcaptures")
            existing_redirects[ident] = self.api.get_release_redirects(ident)

        if not primary_id:
            primary_id = self.choose_primary_release(
                list(releases.values()), existing_redirects
            )
            dupe_ids = [d for d in dupe_ids if d != primary_id]

        assert primary_id not in dupe_ids

        primary_work_id = releases[primary_id].work_id
        updated_work_ids = []
        redirected_release_ids = []

        if self.dry_run_mode:
            eg_id = "dummy-editgroup-id"
        else:
            eg_id = self.get_editgroup_id()

        # execute all the release redirects
        for release in releases.values():
            if release.ident == primary_id:
                continue

            # file redirects
            for e in release.files:
                assert release.ident in e.release_ids
                e.release_ids.remove(release.ident)
                if primary_id not in e.release_ids:
                    e.release_ids.append(primary_id)
                if not self.dry_run_mode:
                    self.api.update_file(eg_id, e.ident, e)
                updated_entities += 1
                self.counts["updated-files"] += 1

            # fileset redirects
            for e in release.filesets:
                assert release.ident in e.release_ids
                e.release_ids.remove(release.ident)
                if primary_id not in e.release_ids:
                    e.release_ids.append(primary_id)
                if not self.dry_run_mode:
                    self.api.update_fileset(eg_id, e.ident, e)
                updated_entities += 1
                self.counts["updated-filesets"] += 1

            # webcapture redirects
            for e in release.webcaptures:
                assert release.ident in e.release_ids
                e.release_ids.remove(release.ident)
                if primary_id not in e.release_ids:
                    e.release_ids.append(primary_id)
                if not self.dry_run_mode:
                    self.api.update_webcapture(eg_id, e.ident, e)
                updated_entities += 1
                self.counts["updated-webcaptures"] += 1

            # the release redirect itself
            updated_work_ids.append(release.work_id)
            redirected_release_ids.append(release.ident)
            if not self.dry_run_mode:
                self.api.update_release(
                    eg_id,
                    release.ident,
                    ReleaseEntity(redirect=primary_id, edit_extra=evidence),
                )
            updated_entities += 1
            self.counts["updated-releases"] += 1

        # lastly, clean up any merged work entities
        redirected_release_ids = list(set(redirected_release_ids))
        updated_work_ids = list(set(updated_work_ids))
        assert primary_work_id not in updated_work_ids
        for work_id in updated_work_ids:
            work_releases = self.api.get_work_releases(work_id, hide="abstracts,refs")
            rids = {r.ident for r in work_releases}
            if rids.issubset(redirected_release_ids):
                # all the releases for this work were updated/merged; we should
                # redirect to primary work_id
                # also check we don't have any in-flight edit conflicts
                assert work_id not in self._idents_inflight
                self._idents_inflight.append(work_id)
                if not self.dry_run_mode:
                    self.api.update_work(eg_id, work_id, WorkEntity(redirect=primary_work_id))
                updated_entities += 1
                self.counts["updated-works"] += 1

        return updated_entities


def run_merge_releases(args: argparse.Namespace) -> None:
    em = ReleaseMerger(args.api, edit_batch_size=args.batch_size, dry_run_mode=args.dry_run)
    JsonLinePusher(em, args.json_file).run()


def main() -> None:
    """
    Invoke like:

        python3 -m fatcat_tools.mergers.releases [options]
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
        auth_var="FATCAT_API_AUTH_TOKEN",
    )
    subparsers = parser.add_subparsers()

    sub_merge_releases = subparsers.add_parser("merge-releases")
    sub_merge_releases.set_defaults(func=run_merge_releases)
    sub_merge_releases.add_argument(
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
