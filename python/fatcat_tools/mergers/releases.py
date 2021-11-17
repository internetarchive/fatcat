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
    primary release.

    Will also redirect works (if appropriate), and re-point {files, filesets,
    webcaptures} to the new merged release.
    """

    def __init__(self, api: fatcat_openapi_client.ApiClient, **kwargs) -> None:

        eg_desc = kwargs.get("editgroup_description", "Automated merge of release entities")
        eg_extra = kwargs.get("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.ReleaseMerger")
        super().__init__(api, editgroup_description=eg_desc, editgroup_extra=eg_extra, **kwargs)
        self.entity_type_name = "release"

    def try_merge(
        self,
        dupe_ids: List[str],
        primary_id: Optional[str] = None,
        evidence: Optional[Dict[str, Any]] = None,
    ) -> int:
        """
        XXX: review/refactor; this code is very old
        """

        updated_entities = 0
        releases = dict()
        eg_id = self.get_editgroup_id()

        all_ids = dupe_ids.copy()
        if primary_id:
            all_ids.append(primary_id)
        for ident in all_ids:
            releases[ident] = self.api.get_release(ident, expand="files,filesets,webcaptures")

        if not primary_id:
            # XXX:
            primary_id = dupe_ids[0]
            dupe_ids = [d for d in dupe_ids if d != primary_id]

        primary_work_id = releases[primary_id].work_id
        updated_work_ids = []
        redirected_release_ids = []

        # execute all the release redirects
        for release in releases.values():
            if release.ident == primary_id:
                continue

            # file redirects
            for e in release.files:
                e.release_ids.remove(release.ident)
                if primary_id not in e.release_ids:
                    e.release_ids.append(primary_id)
                if not self.dry_run_mode:
                    self.api.update_file(eg_id, e.ident, e)
                updated_entities += 1
                self.counts["updated-files"] += 1

            # fileset redirects
            for e in release.filesets:
                e.release_ids.remove(release.ident)
                if primary_id not in e.release_ids:
                    e.release_ids.append(primary_id)
                if not self.dry_run_mode:
                    self.api.update_fileset(eg_id, e.ident, e)
                updated_entities += 1
                self.counts["updated-filesets"] += 1

            # webcapture redirects
            for e in release.webcaptures:
                e.release_ids.remove(release.ident)
                if primary_id not in e.release_ids:
                    e.release_ids.append(primary_id)
                if not self.dry_run_mode:
                    self.api.update_webcapture(eg_id, e.ident, e)
                updated_entities += 1
                self.counts["updated-webcaptures"] += 1

            # release redirect itself
            updated_work_ids.append(release.work_id)
            redirected_release_ids.append(release.ident)
            if not self.dry_run_mode:
                self.api.update_release(
                    eg_id, release.ident, ReleaseEntity(redirect=primary_id)
                )
            updated_entities += 1
            self.counts["updated-releases"] += 1

        # lastly, clean up any merged work entities
        redirected_release_ids = list(set(redirected_release_ids))
        updated_work_ids = list(set(updated_work_ids))
        assert primary_work_id not in updated_work_ids
        for work_id in updated_work_ids:
            work_releases = self.api.get_work_releases(work_id)
            rids = set([r.ident for r in work_releases])
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
        auth_var="FATCAT_AUTH_API_TOKEN",
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
