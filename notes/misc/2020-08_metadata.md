
## Artificial Containers

    biorxiv
    medrxiv
        doi_prefix:10.1101
        publisher:"Cold Spring Harbor Laboratory"
    -> article-journal? article? should match "paper" filter
    -> status: draft? submitted?
    -> there is some flag in crossref metadata...

    arxiv
    -> article-journal?
    -> set container_name?

    protocols.io
        doi_prefix:10.17504/protocols.io.
        container_name:protocols.io

    10.25384/sage. -> sage.figshare.com
    -> at least set container_name

    figshare
        doi_prefix:10.6084
    -> at least set container_name

    zenodo
    -> at least set container_name

Maybe? Later?

    PsycEXTRA
        container_name:"PsycEXTRA Dataset"
        doi_prefix:10.1037
        crossref
    => 300k+ releases
    => subtitle is 'number' (like "(577982012-038)")
    => dataset
    => publication status unknown

    f1000 reviews
        container_name:"F1000 - Post-publication peer review of the biomedical literature"
        title:"Faculty of 1000 evaluation for "[...]
        doi_prefix:10.3410/
        crossref
    => 222k releases
    => type -> peer-review (?)

    IUPAC Standards Online

    GBIF
        doi_prefix: 10.15468/dl.
    => 838k releases

==================


later fatcat:
- pmid+crossref pre-prints
    https://fatcat.wiki/release/d4lrxugtqbapxgi4jrrlmzjily
- zenodo: handle "repost from another ISSN" case (drop issn/container_id)
- doi_prefix:10.18720 no container metadata; should be thesis type?
- research square (10.21203) metadata (journal articles, pre-print or published?)
- journals.ub.uni-heidelberg.de metadata is poor? no journal link
- try_work_lookup() -> part of try update?
    => zenodo "isidentical"
    => zenodo "isversionof"
    => figshare "isversionof"
    => later, try_work_fuzzy()
- biorxiv, medrxiv container name (and/or `container_id`?)
    => and "article" not "post"
- datacite container:"microPublication Biology" -> micropub type?
- ES container index: `publisher_type` (?)
- arxiv: remove release_type="report" logic
- arxiv: don't include DOI, just merge under work
- datacite release_type: resourceType=SaComponent -> 'component'
    https://api.datacite.org/dois/10.1371/journal.pbio.0020429.g004
- datacite title `{:unav}` (PLOS)
    https://fatcat.wiki/release/search?q=doi_prefix%3A10.1371+unav
