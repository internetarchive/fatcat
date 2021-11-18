import argparse
import copy
import os
import sys
from typing import Any, Dict

import fatcat_openapi_client
from fatcat_openapi_client import DefaultApi, FileEntity

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

        python -m fatcat_tools.cleans.file_short_wayback_ts - < blah.json
    """

    def __init__(self, api: DefaultApi, **kwargs):

        eg_desc = (
            kwargs.pop("editgroup_description", None)
            or "Expand trunacted timestamps in wayback URLs"
        )
        eg_extra = kwargs.pop("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get(
            "agent", "fatcat_tools.FileShortWaybackTimestampCleanup"
        )
        super().__init__(
            api,
            do_updates=True,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs,
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
                if not (
                    f"/web/{partial_ts}" in fix_url
                    and fe_url.url.endswith(original_url)
                    and fix_url.endswith(original_url)
                ):
                    print(
                        f"bad replacement URL: partial_ts={partial_ts} original={original_url} fix_url={fix_url}",
                        file=sys.stderr,
                    )
                    self.counts["skip-bad-replacement"] += 1
                    return None
                assert "://" in fix_url
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

        if existing.state != "active":
            self.counts["skip-existing-entity-state"] += 1
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
            # note: passing 'fe' instead of 'existing' here, which is not
            # usually how it goes!
            self.api.update_file(self.get_editgroup_id(), fe.ident, fe)
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

    # another example, which failed with an assertion in prod due to duplicated URLs
    example_line2: Dict[str, Any] = {
        "file_entity": {
            "release_ids": ["22jt7euq4fafhblzullmnesso4"],
            "mimetype": "application/pdf",
            "urls": [
                {
                    "url": "https://www.jstage.jst.go.jp/article/ibk/59/1/59_KJ00007115297/_pdf",
                    "rel": "repository",
                },
                {
                    "url": "https://web.archive.org/web/201811010021/https://www.jstage.jst.go.jp/article/ibk/59/1/59_KJ00007115297/_pdf",
                    "rel": "webarchive",
                },
                {
                    "url": "https://web.archive.org/web/20181101002154/https://www.jstage.jst.go.jp/article/ibk/59/1/59_KJ00007115297/_pdf",
                    "rel": "webarchive",
                },
            ],
            "sha256": "51ec58e7a2325d28d1deb0a4bc6422c0e4ae7b12ffb0b6298981a7b8b7730b19",
            "sha1": "ad96a584fc6073b9a23736bc61ae0ec4a5661433",
            "md5": "3d509743359649e34a27ae70c5cd3018",
            "size": 430665,
            "extra": {
                "shadows": {"scimag_doi": "10.4259/ibk.59.1_194", "scimag_id": "69089904"}
            },
            "revision": "f1fa11ff-d521-45cf-9db1-cb3c8bd3ea48",
            "ident": "duymhmxk3fgtzk37yp2pvthtxq",
            "state": "active",
        },
        "full_urls": {
            "https://web.archive.org/web/201811010021/https://www.jstage.jst.go.jp/article/ibk/59/1/59_KJ00007115297/_pdf": "https://web.archive.org/web/20181101002154/https://www.jstage.jst.go.jp/article/ibk/59/1/59_KJ00007115297/_pdf"
        },
        "status": "success-self",
    }

    fe2 = fswtc.parse_record(example_line2)
    assert len(fe2.urls) == 2
    assert fe2.urls[0].rel == "repository"
    assert (
        fe2.urls[1].url
        == "https://web.archive.org/web/20181101002154/https://www.jstage.jst.go.jp/article/ibk/59/1/59_KJ00007115297/_pdf"
    )

    # ensure URL order is stable
    example_line3: Dict[str, Any] = {
        "file_entity": {
            "release_ids": ["5rin7f2cdvc5hjkqqw53z7sr3i"],
            "mimetype": "application/pdf",
            "urls": [
                {"url": "https://pubs.usgs.gov/bul/1108/report.pdf", "rel": "web"},
                {
                    "url": "https://web.archive.org/web/201904291643/https://pubs.usgs.gov/bul/1108/report.pdf",
                    "rel": "webarchive",
                },
            ],
            "sha256": "714cd48c2577e9b058b8f16b4574765da685f67582cc53898a9d6933e45d6cc0",
            "sha1": "4efbdb517c0ff3f58136e4efbbec2bd9315400d3",
            "md5": "89b6e6cc4e0259317e26ddf1a9a336a0",
            "size": 41265,
            "revision": "926fcf73-e644-4446-a24b-4d0940a2cf65",
            "ident": "lvnz23nzijaapf5iti45zez6zu",
            "state": "active",
        },
        "full_urls": {
            "https://web.archive.org/web/201904291643/https://pubs.usgs.gov/bul/1108/report.pdf": "https://web.archive.org/web/20190429164342/https://pubs.usgs.gov/bul/1108/report.pdf"
        },
        "status": "success-db",
    }

    fe3 = fswtc.parse_record(example_line3)
    assert len(fe3.urls) == 2
    assert fe3.urls[0].rel == "web"
    assert fe3.urls[0].url == "https://pubs.usgs.gov/bul/1108/report.pdf"
    assert fe3.urls[1].rel == "webarchive"


def main() -> None:
    parser = argparse.ArgumentParser(formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument(
        "--host-url", default="http://localhost:9411/v0", help="connect to this host/port"
    )
    parser.add_argument("--batch-size", help="size of batch to send", default=100, type=int)
    parser.set_defaults(
        auth_var="FATCAT_AUTH_WORKER_CLEANUP",
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
