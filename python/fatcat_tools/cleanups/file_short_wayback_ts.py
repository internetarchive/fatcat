import argparse
import copy
import os
import sys
from typing import Any, Dict

import fatcat_openapi_client
from fatcat_openapi_client import ApiClient, FileEntity

from fatcat_tools import authenticated_api, entity_from_dict, public_api
from fatcat_tools.importers.common import EntityImporter, JsonLinePusher


class FileShortWaybackTimestampCleanup(EntityImporter):
    """
    This is a one-off / one-time cleanup script for file entities, fix short
    timestamps in wayback URLs. These timestamps are supposed to have 14 digits
    (datetime with year, hour, seconds, etc). Some legacy file imports ended up
    with only 4 or 12 digits.

    While this calls itself a cleanup, it is based on the import code path. It
    is not integrated into the `fatcat_import` or `fatcat_cleanup` controller;
    instead it has a __main__ function and is invoked like:

        python -m fatcat_tools.cleans.file_short_wayback-ts < blah.json
    """

    def __init__(self, api: ApiClient, **kwargs):

        eg_desc = kwargs.pop("editgroup_description", None) or "Expand short wayback timestamps"
        eg_extra = kwargs.pop("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get(
            "agent", "fatcat_tools.FileShortWaybackTimestampCleanup"
        )
        super().__init__(
            api,
            do_updates=True,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs
        )
        self.testing_mode = False

    def want(self, row: Dict[str, Any]) -> bool:
        if row["status"].startswith("success"):
            return True
        else:
            self.counts["skip-status"] += 1
            return False

    def parse_record(self, row: Dict[str, Any]) -> FileEntity:

        # bezerk mode doesn't make sense for this importer
        assert self.bezerk_mode is False

        fe: FileEntity = entity_from_dict(row["file_entity"], FileEntity)
        status: str = row["status"]
        assert status.startswith("success")
        url_expansions: Dict[str, str] = row["full_urls"]
        assert len(url_expansions) >= 1

        # actual cleanup happens here
        any_fixed = False
        for fe_url in fe.urls:
            if "://web.archive.org/web/" not in fe_url.url:
                continue
            seq = fe_url.url.split("/")
            partial_ts = seq[4]
            original_url = "/".join(seq[5:])
            if seq[2] != "web.archive.org":
                continue
            if len(partial_ts) not in [4, 12]:
                continue
            if fe_url.url in url_expansions:
                fix_url = url_expansions[fe_url.url]
                # defensive checks
                assert partial_ts in fix_url
                assert "://" in fix_url
                assert fe_url.url.endswith(original_url)
                assert fix_url.endswith(original_url)
                fe_url.url = fix_url
                any_fixed = True

        if not any_fixed:
            self.counts["skip-no-fixes"] += 1
            return None

        # do any other generic file entity cleanups
        # this includes removing duplicates
        fe = self.generic_file_cleanups(fe)

        # verify that there are no exact duplicates
        final_urls = [u.url for u in fe.urls]
        assert len(final_urls) == len(list(set(final_urls)))

        return fe

    def try_update(self, fe: FileEntity) -> bool:

        # should always be existing
        try:
            existing = self.api.get_file(fe.ident)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        if not existing:
            self.counts["skip-existing-not-found"] += 1
            return False

        if existing.sha1 != fe.sha1:
            self.counts["skip-existing-mismatch"] += 1
            return False

        assert fe.revision and existing.revision
        if existing.revision != fe.revision:
            self.counts["skip-revision-changed"] += 1
            return False

        # verify that at least one URL remains
        if not fe.urls or len(fe.urls) < 1:
            self.counts["skip-no-urls"] += 1
            return False

        # verify that all wayback urls have 14-digit timestamps, and are generally well-formed
        for u in fe.urls:
            if "://web.archive.org/web/" not in u.url:
                continue
            if u.rel != "webarchive":
                self.counts["skip-bad-wayback-rel"] += 1
                return False
            seg = u.url.split("/")
            if (
                len(seg) < 6
                or seg[0] != "https:"
                or seg[2] != "web.archive.org"
                or seg[3] != "web"
            ):
                self.counts["skip-bad-wayback"] += 1
                return False
            if len(seg[4]) != 14 or not seg[4].isdigit():
                self.counts["skip-bad-wayback-timestamp"] += 1
                return False

        if existing == fe or existing.urls == fe.urls:
            self.counts["skip-no-change"] += 1
            return False

        # not doing a check for "in current editgroup", because the source of
        # these corrections (entity dump) contains no dupes

        if not self.testing_mode:
            self.api.update_file(self.get_editgroup_id(), existing.ident, existing)
        self.counts["update"] += 1
        return False


def test_short_wayback_ts() -> None:
    api = public_api("http://localhost:9411/v0")
    fswtc = FileShortWaybackTimestampCleanup(api=api)
    fswtc.testing_mode = True

    assert fswtc.want({"status": "fail"}) is False
    assert fswtc.want({"status": "success-self"}) is True

    example_line: Dict[str, Any] = {
        "status": "success-db",
        "file_entity": {
            # note: doesn't match actual entity
            "release_ids": ["waldfsctnbcpdbmasgduhaaaaa"],
            "mimetype": "application/pdf",
            "urls": [
                {
                    "url": "https://papiro.unizar.es/ojs/index.php/ais/article/download/2187/1971",
                    "rel": "web",
                },
                {
                    "url": "https://web.archive.org/web/201904301022/https://papiro.unizar.es/ojs/index.php/ais/article/download/2187/1971",
                    "rel": "webarchive",
                },
            ],
            "sha256": "0b9e09480ed2e1f08f3c6c72f57ce12b52ea265f580f8810e606b49d64234b29",
            "sha1": "be714299b9be21b5afdaa7affd7d710c58269433",
            "md5": "9edb542be5b3446a1905e61a8a3abebd",
            "size": 666242,
            "revision": "fe949be8-7bf9-4c17-be28-8e3e90fb85bd",
            "ident": "4ghpvs2t2rdtrdum2mkreh62me",
            "state": "active",
        },
        "full_urls": {
            "https://web.archive.org/web/201904301022/https://papiro.unizar.es/ojs/index.php/ais/article/download/2187/1971": "https://web.archive.org/web/20190430102239/https://papiro.unizar.es/ojs/index.php/ais/article/download/2187/1971"
        },
    }
    example_fe = entity_from_dict(example_line["file_entity"], FileEntity)

    fe1 = copy.copy(example_fe)
    fe1.urls[
        1
    ].url = "https://web.archive.org/web/20190430102239/https://papiro.unizar.es/ojs/index.php/ais/article/download/2187/1971"
    assert fswtc.parse_record(example_line) == fe1

    # update code path; requires a known file ident and API running locally
    assert fswtc.counts["update"] == 0
    dummy_fe = api.get_file("aaaaaaaaaaaaamztaaaaaaaaai")
    fe1.ident = dummy_fe.ident

    assert fswtc.try_update(fe1) is False
    assert fswtc.counts["skip-existing-mismatch"] == 1

    fe1.sha1 = dummy_fe.sha1
    assert fswtc.try_update(fe1) is False
    assert fswtc.counts["skip-revision-changed"] == 1

    fe1.revision = dummy_fe.revision
    assert fswtc.try_update(fe1) is False
    print(fswtc.counts)
    assert fswtc.counts["update"] == 1


def main() -> None:
    parser = argparse.ArgumentParser(formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument(
        "--host-url", default="http://localhost:9411/v0", help="connect to this host/port"
    )
    parser.add_argument("--batch-size", help="size of batch to send", default=50, type=int)
    parser.set_defaults(
        auth_var="FATCAT_API_AUTH_TOKEN",
    )
    parser.add_argument(
        "json_file",
        help="File with jsonlines from file_meta schema to import from",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )

    args = parser.parse_args()
    api = authenticated_api(
        args.host_url,
        # token is an optional kwarg (can be empty string, None, etc)
        token=os.environ.get(args.auth_var),
    )

    fswtc = FileShortWaybackTimestampCleanup(
        api,
        edit_batch_size=args.batch_size,
    )
    JsonLinePusher(fswtc, args.json_file).run()


if __name__ == "__main__":
    main()
