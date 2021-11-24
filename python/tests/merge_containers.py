from fatcat_openapi_client import ContainerEntity
from fixtures import api

from fatcat_tools.mergers.containers import ContainerMerger


def test_choose_primary_container(api) -> None:

    release_counts = dict()
    redirects = dict()
    em = ContainerMerger(api=api)

    ce_stub = ContainerEntity(
        ident="pppppp5apzfhbbxxc7rgu2yw6m",
        name="dummy journal",
    )
    release_counts[ce_stub.ident] = 0
    redirects[ce_stub.ident] = []

    ce_partial = ContainerEntity(
        ident="eeeeeeeapzfhbbxxc7rgu2yw6m",
        name="dummy complete journal",
        publisher="some publisher",
        issnl="1234-5678",
        publication_status="active",
        extra=dict(asdf=123, ia=dict(asdf=True)),
    )
    release_counts[ce_partial.ident] = 0
    redirects[ce_partial.ident] = []

    ce_partial_redirects = ContainerEntity(
        ident="rrrrrrrrrrfhbbxxc7rgu2yw6m",
        name="dummy complete journal",
        publisher="some publisher",
        issnl="1234-5678",
        publication_status="active",
        extra=dict(asdf=123, ia=dict(asdf=True)),
    )
    release_counts[ce_partial_redirects.ident] = 0
    redirects[ce_partial_redirects.ident] = [
        "zzzzzzzzrrfhbbxxc7rgu2yw6m",
    ]

    ce_complete_zero = ContainerEntity(
        ident="oooooooapzfhbbxxc7rgu2yw6m",
        name="dummy complete journal",
        publisher="some publisher",
        issnl="1234-5678",
        publication_status="active",
        extra=dict(asdf=123, ia=dict(asdf=True)),
    )
    release_counts[ce_complete_zero.ident] = 0
    redirects[ce_complete_zero.ident] = []

    ce_complete_small = ContainerEntity(
        ident="cccccccapzfhbbxxc7rgu2yw6m",
        name="dummy complete journal",
        publisher="some publisher",
        issnl="1234-5678",
        publication_status="active",
        extra=dict(asdf=123, ia=dict(asdf=True)),
    )
    release_counts[ce_complete_small.ident] = 10
    redirects[ce_complete_small.ident] = []

    ce_complete_big = ContainerEntity(
        ident="ddddddddpzfhbbxxc7rgu2yw6m",
        name="dummy complete journal",
        publisher="some publisher",
        issnl="1234-5678",
        publication_status="active",
        extra=dict(asdf=123, ia=dict(asdf=True)),
    )
    release_counts[ce_complete_big.ident] = 9999999
    redirects[ce_complete_big.ident] = []

    assert (
        em.choose_primary_container([ce_stub, ce_partial], redirects, release_counts)
        == ce_partial.ident
    )
    assert (
        em.choose_primary_container(
            [ce_stub, ce_complete_zero, ce_partial], redirects, release_counts
        )
        == ce_complete_zero.ident
    )
    assert (
        em.choose_primary_container(
            [ce_stub, ce_partial_redirects, ce_complete_zero, ce_partial],
            redirects,
            release_counts,
        )
        == ce_partial_redirects.ident
    )
    assert (
        em.choose_primary_container(
            [ce_stub, ce_complete_zero, ce_complete_small, ce_partial],
            redirects,
            release_counts,
        )
        == ce_complete_small.ident
    )
    assert (
        em.choose_primary_container(
            [ce_stub, ce_complete_big, ce_complete_zero, ce_complete_small, ce_partial],
            redirects,
            release_counts,
        )
        == ce_complete_big.ident
    )
    assert (
        em.choose_primary_container(
            [ce_complete_small, ce_complete_big], redirects, release_counts
        )
        == ce_complete_big.ident
    )
