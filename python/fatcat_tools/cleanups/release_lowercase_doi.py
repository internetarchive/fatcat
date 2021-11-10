import argparse
import os
import sys

from fatcat_openapi_client import ApiClient, ReleaseEntity, ReleaseExtIds

from fatcat_tools import authenticated_api, public_api
from fatcat_tools.importers.common import EntityImporter, LinePusher


class ReleaseLowercaseDoiCleanup(EntityImporter):
    """
    This is a one-off / one-time cleanup script for release entities, to fix
    upper-case DOIs. In fatcat, all DOIs should be normalized to lowercase.

    While this calls itself a cleanup, it is based on the import code path. It
    is not integrated into the `fatcat_import` or `fatcat_cleanup` controller;
    instead it has a __main__ function and is invoked like:

        python -m fatcat_tools.cleans.release_lowercase_doi - < blah.tsv

    It expects to get a simple text line on stdin, which is a release entity.
    The correction is implemented by fetching the current version of the
    entity, verifying the issue, and updating if it is still a problem.

    This does not try to do any merging, just corrects the case in a single
    update.
    """

    def __init__(self, api: ApiClient, **kwargs):

        eg_desc = (
            kwargs.pop("editgroup_description", None)
            or "Normalize release DOIs (extid) to lower-case"
        )
        eg_extra = kwargs.pop("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.ReleaseLowercaseDoiCleanup")
        super().__init__(
            api,
            do_updates=True,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs,
        )
        self.testing_mode = False

    def want(self, row: str) -> bool:
        row = row.strip().split()[0]
        if len(row) == 26:
            return True
        else:
            return False

    def parse_record(self, row: str) -> ReleaseEntity:

        # bezerk mode doesn't make sense for this importer
        assert self.bezerk_mode is False

        ident = row.strip().split()[0]
        assert len(ident) == 26

        return ReleaseEntity(
            ident=ident,
            ext_ids=ReleaseExtIds(),
        )

    def try_update(self, re: ReleaseEntity) -> bool:

        # should always be existing
        existing = self.api.get_release(re.ident)

        if not existing:
            self.counts["skip-existing-not-found"] += 1
            return False

        if existing.status != "active":
            self.counts["skip-existing-entity-status"] += 1
            return False

        if not existing.ext_ids.doi:
            self.counts["skip-existing-no-doi"] += 1
            return False

        if existing.ext_ids.doi == existing.ext_ids.doi.lower():
            self.counts["skip-existing-doi-fine"] += 1
            return False

        existing.ext_ids.doi = existing.ext_ids.doi.lower()

        # not doing a check for "in current editgroup", because the source of
        # these corrections (entity dump) contains no dupes

        if not self.testing_mode:
            self.api.update_release(self.get_editgroup_id(), re.ident, re)
        self.counts["update"] += 1
        return False


def test_lowercase_doi() -> None:
    api = public_api("http://localhost:9411/v0")
    rldc = ReleaseLowercaseDoiCleanup(api=api)
    rldc.testing_mode = True

    assert rldc.want("") is False
    assert rldc.want("aaaaaaaaaaaaarceaaaaaaaaai") is True
    assert rldc.want("aaaaaaaaaaaaarceaaaaaaaaai\t10.1234/ABCD") is True
    rldc.parse_record("aaaaaaaaaaaaarceaaaaaaaaai")

    dummy_re = api.get_release("aaaaaaaaaaaaarceaaaaaaaaai")
    assert rldc.try_update(dummy_re) is False
    assert rldc.counts["skip-existing-doi-fine"] == 1
    # this isn't a very complete test, doesn't get to update part


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
        "idents_file",
        help="File with release identifier to try updating",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )

    args = parser.parse_args()
    api = authenticated_api(
        args.host_url,
        # token is an optional kwarg (can be empty string, None, etc)
        token=os.environ.get(args.auth_var),
    )

    rldc = ReleaseLowercaseDoiCleanup(
        api,
        edit_batch_size=args.batch_size,
    )
    LinePusher(rldc, args.idents_file).run()


if __name__ == "__main__":
    main()
