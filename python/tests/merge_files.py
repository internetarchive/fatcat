from fatcat_openapi_client import FileEntity, FileUrl
from fixtures import api

from fatcat_tools.mergers.files import FileMerger


def test_choose_primary_file(api) -> None:

    fm = FileMerger(api=api)
    fe_partial = FileEntity(
        ident="aaaasb5apzfhbbxxc7rgu2yw6m",
        sha1="b1beebb5f979121cd234c69b08e3f42af3aaaaaa",
    )
    fe_norelease = FileEntity(
        ident="bbbbsb5apzfhbbxxc7rgu2yw6m",
        sha1="b1beebb5f979121cd234c69b08e3f42af3bbbbbb",
        md5="d2c7318315bfc7d3aab0db933e95e632",
        sha256="528064c7664a96c79c80c423210f6f9f4fafe949dd59dfd1572a04b906d5e163",
        size=60719,
        mimetype="application/pdf",
    )
    fe_nourls = FileEntity(
        ident="ccccsb5apzfhbbxxc7rgu2yw6m",
        sha1="b1beebb5f979121cd234c69b08e3f42af3bbbbbb",
        md5="d2c7318315bfc7d3aab0db933e95e632",
        sha256="528064c7664a96c79c80c423210f6f9f4fafe949dd59dfd1572a04b906d5e163",
        size=60719,
        mimetype="application/pdf",
        release_ids=["dlrxjg7mxrayxfltget7fqcrjy"],
    )
    fe_complete = FileEntity(
        ident="ddddsb5apzfhbbxxc7rgu2yw6m",
        sha1="b1beebb5f979121cd234c69b08e3f42af3bbbbbb",
        md5="d2c7318315bfc7d3aab0db933e95e632",
        sha256="528064c7664a96c79c80c423210f6f9f4fafe949dd59dfd1572a04b906d5e163",
        size=60719,
        mimetype="application/pdf",
        release_ids=["dlrxjg7mxrayxfltget7fqcrjy"],
        urls=[
            FileUrl(rel="web", url="http://aughty.org/pdf/future_open.pdf"),
        ],
        extra=dict(asdf=123),
    )
    fe_pseudo_complete = FileEntity(
        ident="eeeesb5apzfhbbxxc7rgu2yw6m",
        sha1="b1beebb5f979121cd234c69b08e3f42af3bbbbbb",
        sha256="528064c7664a96c79c80c423210f6f9f4fafe949dd59dfd1572a04b906d5e163",
        size=60719,
        mimetype="application/pdf",
        release_ids=["dlrxjg7mxrayxfltget7fqcrjy"],
        urls=[
            FileUrl(rel="web", url="http://aughty.org/pdf/future_open.pdf"),
        ],
        extra=dict(asdf=123),
    )

    assert fm.choose_primary_file([fe_partial, fe_norelease]) == "bbbbsb5apzfhbbxxc7rgu2yw6m"
    assert (
        fm.choose_primary_file([fe_partial, fe_nourls, fe_norelease])
        == "ccccsb5apzfhbbxxc7rgu2yw6m"
    )
    assert (
        fm.choose_primary_file([fe_partial, fe_complete, fe_nourls, fe_norelease])
        == "ddddsb5apzfhbbxxc7rgu2yw6m"
    )
    assert (
        fm.choose_primary_file([fe_partial, fe_pseudo_complete, fe_nourls, fe_norelease])
        == "ccccsb5apzfhbbxxc7rgu2yw6m"
    )


def test_merge_file_metadata_from(api) -> None:
    fm = FileMerger(api=api)
    fe_partial = FileEntity(
        ident="aaaasb5apzfhbbxxc7rgu2yw6m",
        sha1="b1beebb5f979121cd234c69b08e3f42af3aaaaaa",
    )
    fe_norelease = FileEntity(
        ident="bbbbsb5apzfhbbxxc7rgu2yw6m",
        sha1="b1beebb5f979121cd234c69b08e3f42af3bbbbbb",
        md5="d2c7318315bfc7d3aab0db933e95e632",
        sha256="528064c7664a96c79c80c423210f6f9f4fafe949dd59dfd1572a04b906d5e163",
        size=60719,
        mimetype="application/pdf",
    )
    fe_nourls = FileEntity(
        ident="ccccsb5apzfhbbxxc7rgu2yw6m",
        sha1="b1beebb5f979121cd234c69b08e3f42af3bbbbbb",
        md5="d2c7318315bfc7d3aab0db933e95e632",
        sha256="528064c7664a96c79c80c423210f6f9f4fafe949dd59dfd1572a04b906d5e163",
        size=60719,
        mimetype="application/pdf",
        release_ids=["dlrxjg7mxrayxfltget7fqcrjy"],
    )
    fe_complete = FileEntity(
        ident="ddddsb5apzfhbbxxc7rgu2yw6m",
        sha1="b1beebb5f979121cd234c69b08e3f42af3bbbbbb",
        md5="ddddddd315bfc7d3aab0db933e95e632",
        sha256="528064c7664a96c79c80c423210f6f9f4fafe949dd59dfd1572a04b906d5e163",
        size=60719,
        mimetype="application/pdf",
        release_ids=["dlrxjg7mxrayxfltget7fqcrjy"],
        urls=[
            FileUrl(rel="web", url="http://aughty.org/pdf/future_open.pdf"),
        ],
        extra=dict(asdf=123),
    )
    fe_pseudo_complete = FileEntity(
        ident="eeeesb5apzfhbbxxc7rgu2yw6m",
        sha1="b1beebb5f979121cd234c69b08e3f42af3bbbbbb",
        sha256="528064c7664a96c79c80c423210f6f9f4fafe949dd59dfd1572a04b906d5e163",
        size=60719,
        mimetype="application/pdf",
        release_ids=["dlrxjg7mxrayxfltget7fqcrjy"],
        urls=[
            FileUrl(rel="web", url="http://aughty.org/pdf/future_open.pdf"),
        ],
        extra=dict(asdf=123),
    )
    fe_another_release_id = FileEntity(
        ident="fffffffapzfhbbxxc7rgu2yw6m",
        release_ids=["qqqqqg7mxrayxfltget7fqcrjy"],
    )
    fe_another_url = FileEntity(
        ident="zzzzzzzapzfhbbxxc7rgu2yw6m",
        urls=[
            FileUrl(rel="repository", url="http://someuni.edu/repo/file.pdf"),
        ],
    )
    fe_more_extra = FileEntity(
        ident="fffffffapzfhbbxxc7rgu2yw6m",
        release_ids=["qqqqqg7mxrayxfltget7fqcrjy"],
        extra=dict(thang=456),
    )

    assert fm.merge_file_metadata_from(fe_nourls, fe_partial) is False
    assert fm.merge_file_metadata_from(fe_complete, fe_pseudo_complete) is False
    assert fm.merge_file_metadata_from(fe_complete, fe_complete) is False
    assert fm.merge_file_metadata_from(fe_partial, fe_norelease) is True
    assert fe_partial.md5 == fe_norelease.md5
    assert fe_partial.size == fe_norelease.size
    assert fm.merge_file_metadata_from(fe_partial, fe_complete) is True
    assert fe_partial.md5 != fe_complete.md5
    assert fe_partial.extra == fe_complete.extra
    assert {(u.rel, u.url) for u in fe_partial.urls or []} == {
        (u.rel, u.url) for u in fe_complete.urls or []
    }
    assert fe_partial.release_ids == fe_complete.release_ids
    assert fm.merge_file_metadata_from(fe_partial, fe_another_release_id) is True
    assert fe_partial.release_ids == [
        "dlrxjg7mxrayxfltget7fqcrjy",
        "qqqqqg7mxrayxfltget7fqcrjy",
    ]
    assert fm.merge_file_metadata_from(fe_partial, fe_another_release_id) is False
    assert fm.merge_file_metadata_from(fe_partial, fe_more_extra) is True
    assert fe_partial.extra == dict(asdf=123, thang=456)
    assert fm.merge_file_metadata_from(fe_partial, fe_more_extra) is False
    assert fm.merge_file_metadata_from(fe_partial, fe_another_url) is True
    assert fe_partial.urls[-1].url == "http://someuni.edu/repo/file.pdf"
    assert fm.merge_file_metadata_from(fe_partial, fe_another_url) is False
