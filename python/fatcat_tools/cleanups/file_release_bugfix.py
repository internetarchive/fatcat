import argparse
import os
import sys
from typing import Any, Dict

import fatcat_openapi_client
from fatcat_openapi_client import ApiClient, FileEntity

from fatcat_tools import authenticated_api, public_api, uuid2fcid
from fatcat_tools.importers.common import EntityImporter, JsonLinePusher
from fatcat_tools.normal import clean_doi


class FileReleaseBugfix(EntityImporter):
    """
    This is a one-off / one-time cleanup script for file entities which got
    imported with incorrect release ident mappings, due to a bug in the file
    ingest importer.

    While this calls itself a cleanup, it is based on the import code path. It
    is not integrated into the `fatcat_import` or `fatcat_cleanup` controller;
    instead it has a __main__ function and is invoked like:

        python -m fatcat_tools.cleans.file_release_bugfix - < blah.json
    """

    def __init__(self, api: ApiClient, **kwargs):

        eg_desc = (
            kwargs.pop("editgroup_description", None)
            or "Correct bad file/release import mappings"
        )
        eg_extra = kwargs.pop("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.FileReleaseBugfix")
        super().__init__(
            api,
            do_updates=True,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs,
        )
        self.testing_mode = False

    def want(self, row: Dict[str, Any]) -> bool:
        if not (
            row.get("edit_extra")
            and row["edit_extra"].get("link_source")
            and row["edit_extra"].get("link_source_id")
        ):
            self.counts["skip-partial"] += 1
            return False
        if row["edit_extra"]["link_source"] not in ["unpaywall", "doi"]:
            self.counts["skip-link-source"] += 1
            return False
        if row["edit_extra"].get("ingest_request_source") not in [
            "unpaywall",
            "fatcat-changelog",
        ]:
            self.counts["skip-ingest-request-source"] += 1
            return False
        if not row["edit_extra"]["link_source_id"].startswith("10."):
            self.counts["skip-source-id-not-doi"] += 1
            return False
        return True

    def parse_record(self, row: Dict[str, Any]) -> FileEntity:

        # bezerk mode doesn't make sense for this importer
        assert self.bezerk_mode is False

        file_ident = uuid2fcid(row["file_ident"])
        wrong_release_ident = uuid2fcid(row["wrong_release_ident"])
        edit_extra = row["edit_extra"]
        assert edit_extra["link_source"] in ["unpaywall", "doi"]
        file_edit_doi = clean_doi(edit_extra["link_source_id"])

        if not file_edit_doi:
            self.counts["skip-bad-doi"] += 1
            return False

        # check that the "wrong" release exists and doesn't have the DOI
        wrong_release = None
        try:
            wrong_release = self.api.get_release(wrong_release_ident)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        if not wrong_release:
            self.counts["skip-wrong-release-missing"] += 1
            return None

        if clean_doi(wrong_release.ext_ids.doi) == file_edit_doi:
            self.counts["skip-wrong-release-is-ok"] += 1
            return None

        # fetch the "correct" release, if any
        fixed_release_ids = []
        correct_release = None
        try:
            correct_release = self.api.lookup_release(doi=file_edit_doi)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        if correct_release:
            fixed_release_ids.append(correct_release.ident)

        fe = FileEntity(
            ident=file_ident,
            release_ids=fixed_release_ids,
            edit_extra=edit_extra,
        )
        fe._wrong_release_ident = wrong_release_ident
        return fe

    def try_update(self, fe: FileEntity) -> bool:

        wrong_release_ident = fe._wrong_release_ident
        assert len(wrong_release_ident) == 26

        # should always be existing... but in QA it might not be
        existing = None
        try:
            existing = self.api.get_file(fe.ident)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        if not existing:
            self.counts["skip-existing-not-found"] += 1
            return False

        if existing.state != "active":
            self.counts["skip-existing-entity-state"] += 1
            return False

        if wrong_release_ident not in existing.release_ids:
            self.counts["skip-existing-fixed"] += 1
            return False

        # fetch existing history to verify mismatch
        history = self.api.get_file_history(existing.ident)

        for entry in history:
            if entry.editgroup.editor.is_bot is not True:
                self.counts["skip-existing-edit-history-human"] += 1
                return False

        bad_edit = history[-1].edit
        if bad_edit.extra != fe.edit_extra:
            self.counts["skip-existing-edit-history-extra-mismatch"] += 1
            return False

        bad_editgroup = history[-1].editgroup
        if not bad_editgroup.extra:
            self.counts["skip-existing-editgroup-missing-extra"] += 1
            return False

        if (
            bad_editgroup.editor_id != "scmbogxw25evtcesfcab5qaboa"
            or bad_editgroup.extra.get("agent") != "fatcat_tools.IngestFileResultImporter"
            or not bad_editgroup.extra.get("git_rev", "").startswith("v0.3")
            or bad_editgroup.created.year != 2020
        ):
            self.counts["skip-existing-edit-history-mismatch"] += 1
            return False

        existing.release_ids = [ri for ri in existing.release_ids if ri != wrong_release_ident]

        if len(fe.release_ids) == 1:
            if fe.release_ids[0] not in existing.release_ids:
                existing.release_ids.append(fe.release_ids[0])

        existing.edit_extra = fe.edit_extra

        # not doing a check for "in current editgroup", because the source of
        # these corrections (entity dump) contains no dupes

        if not self.testing_mode:
            self.api.update_file(self.get_editgroup_id(), existing.ident, existing)
        self.counts["update"] += 1
        return False


def test_file_release_bugfix() -> None:
    api = public_api("http://localhost:9411/v0")
    frbc = FileReleaseBugfix(api=api)
    frbc.testing_mode = True

    assert frbc.want({"this": "asdf"}) is False

    example_line: Dict[str, Any] = {
        "file_ident": "00000000-0000-0000-3333-000000000002",
        "wrong_release_ident": "00000000-0000-0000-4444-000000000002",
        "edit_extra": {
            "link_source": "unpaywall",
            "link_source_id": "10.1371/journal.pmed.0020124",
            "ingest_request_source": "unpaywall",
        },
    }

    fe1 = frbc.parse_record(example_line)
    print(frbc.counts)
    frbc.try_update(fe1)

    # NOTE: this test is pretty incompleted


def main() -> None:
    parser = argparse.ArgumentParser(formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument(
        "--host-url", default="http://localhost:9411/v0", help="connect to this host/port"
    )
    parser.add_argument("--batch-size", help="size of batch to send", default=50, type=int)
    parser.set_defaults(
        auth_var="FATCAT_AUTH_WORKER_CLEANUP",
    )
    parser.add_argument(
        "json_file",
        help="File with jsonlines with cleanup context",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )

    args = parser.parse_args()
    api = authenticated_api(
        args.host_url,
        # token is an optional kwarg (can be empty string, None, etc)
        token=os.environ.get(args.auth_var),
    )

    frbc = FileReleaseBugfix(
        api,
        edit_batch_size=args.batch_size,
    )
    JsonLinePusher(frbc, args.json_file).run()


if __name__ == "__main__":
    main()
