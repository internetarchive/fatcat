import argparse
import os
import sys
from typing import Any, Dict, List, Optional

import fatcat_openapi_client
from fatcat_openapi_client.models import ContainerEntity

from fatcat_tools import authenticated_api
from fatcat_tools.harvest.harvest_common import requests_retry_session
from fatcat_tools.importers import JsonLinePusher

from .common import EntityMerger


class ContainerMerger(EntityMerger):
    """
    Combines container entities into a single primary. Does not merge partial
    metadata (identifiers, etc). Can chose "primary" container to redirect to,
    if necessary.

    The `max_container_releases` argument (int or None) can be used to
    prevent redirecting containers which already have releases pointed at them
    (based on release ES index stats). If set to 0, no releases are allowed. If
    set to None (or a negative number), the parameter is ignored.

    The `clobber_human_edited` flag (boolean) can be used to allow updating
    entities even if they have had human edits in the past.

    This merger makes external HTTP requests to fatcat.wiki, for the purpose of
    fetching release stats.
    """

    def __init__(self, api: fatcat_openapi_client.ApiClient, **kwargs) -> None:

        eg_desc = (
            kwargs.pop("editgroup_description", None) or "Automated merge of container entities"
        )
        eg_extra = kwargs.pop("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.ContainerMerger")
        self.dry_run_mode: bool = eg_extra.get("dry_run_mode", False)
        self.clobber_human_edited: bool = eg_extra.get("clobber_human_edited", False)
        self.max_container_releases: Optional[int] = eg_extra.get("max_container_releases", 0)
        if self.max_container_releases and self.max_container_releases < 0:
            self.max_container_releases = None
        super().__init__(api, editgroup_description=eg_desc, editgroup_extra=eg_extra, **kwargs)
        self.entity_type_name = "container"
        self.http_session = requests_retry_session()

    def choose_primary_container(
        self,
        entities: List[ContainerEntity],
        redirects: Dict[str, List[str]],
        release_counts: Dict[str, int],
    ) -> str:
        assert entities and len(entities) >= 2

        # want to sort in descending order, so reverse=True
        entities = sorted(
            entities,
            key=lambda a: (
                # linked release counts
                release_counts[a.ident],
                # number of redirected entities
                len(redirects[a.ident]),
                # not a stub
                bool(a.container_type != "stub"),
                # has a strong identifier?
                bool(a.issnl or (a.extra and a.extra.get("dblp"))),
                # has IA sim metadata?
                bool(a.extra and a.extra.get("ia")),
                # has additional metadata?
                bool(a.publication_status or a.container_type),
                bool(a.extra),
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

        # currently required for extid validation
        if not evidence or not (evidence.get("extid_type") and evidence.get("extid")):
            self.counts["skip-missing-evidence"] += 1
            return 0

        updated_entities = 0
        entities: Dict[str, ContainerEntity] = dict()
        redirects: Dict[str, List[str]] = dict()
        release_counts: Dict[str, int] = dict()
        eg_id = self.get_editgroup_id()

        all_ids = dupe_ids.copy()
        if primary_id:
            all_ids.append(primary_id)
        for ident in all_ids:
            try:
                entities[ident] = self.api.get_container(ident)
                redirects[ident] = self.api.get_container_redirects(ident)
            except fatcat_openapi_client.ApiException as ae:
                if ae.status == 404:
                    self.counts["skip-entity-not-found"] += 1
                    return 0
                else:
                    raise
            if entities[ident].state != "active":
                self.counts["skip-not-active-entity"] += 1
                return 0
            if getattr(entities[ident], evidence["extid_type"]) != evidence["extid"]:
                self.counts["skip-extid-mismatch"] += 1
                return 0
            if not self.clobber_human_edited:
                edit_history = self.api.get_container_history(ident)
                for edit in edit_history:
                    if edit.editor.is_bot is not True:
                        self.counts["skip-human-edited"] += 1
                        return 0
            resp = self.http_session.get("https://fatcat.wiki/container/{ident}/stats.json")
            resp.raise_for_status()
            stats = resp.json()
            release_counts[ident] = stats["total"]
            if self.max_container_releases is not None:
                if release_counts[ident] > self.max_container_releases:
                    self.counts["skip-container-release-count"] += 1
                    continue

        if not primary_id:
            primary_id = self.choose_primary_container(
                list(entities.values()), redirects, release_counts
            )
            dupe_ids = [d for d in dupe_ids if d != primary_id]

        assert primary_id not in dupe_ids

        primary = entities[primary_id]
        for other_id in dupe_ids:
            other = entities[other_id]
            if not self.dry_run_mode:
                self.api.update_container(
                    eg_id,
                    other.ident,
                    ContainerEntity(
                        redirect=primary.ident,
                        edit_extra=evidence,
                    ),
                )
            updated_entities += 1

        return updated_entities


def run_merge_containers(args: argparse.Namespace) -> None:
    em = ContainerMerger(
        args.api,
        edit_batch_size=args.batch_size,
        dry_run_mode=args.dry_run,
        max_container_releases=args.max_container_releases,
        clobber_human_edited=args.clobber_human_edited,
        editgroup_description=args.editgroup_description_override,
    )
    JsonLinePusher(em, args.json_file).run()


def main() -> None:
    """
    Invoke like:

        python3 -m fatcat_tools.mergers.containers [options]
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

    sub_merge_containers = subparsers.add_parser("merge-containers")
    sub_merge_containers.set_defaults(func=run_merge_containers)
    sub_merge_containers.add_argument(
        "json_file",
        help="source of merge lines to process (or stdin)",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    parser.add_argument(
        "--clobber-human-edited",
        action="store_true",
        help="if set, entities which have non-bot (human) edits can be updated/redirected",
    )
    parser.add_argument(
        "--max-container-releases",
        default=0,
        type=int,
        help="if container has more than this many releases linked, don't update (set to -1 to disable limit)",
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
