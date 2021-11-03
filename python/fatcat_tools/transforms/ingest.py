INGEST_TYPE_CONTAINER_MAP = {
    # Optica
    "twtpsm6ytje3nhuqfu3pa7ca7u": "html",
    # Optics Express
    "cg4vcsfty5dfvgmat5wm62wgie": "html",
    # First Monday
    "svz5ul6qozdjhjhk7d627avuja": "html",
    # D-Lib Magazine
    "ugbiirfvufgcjkx33r3cmemcuu": "html",
    # Distill (distill.pub)
    "lx7svdzmc5dl3ay4zncjjrql7i": "html",
    # NLM technical bulletin
    "lovwr7ladjagzkhmoaszg7efqu": "html",
}


def release_ingest_request(release, ingest_request_source="fatcat", ingest_type=None):
    """
    Takes a full release entity object and returns an ingest request (as dict),
    or None if it seems like this release shouldn't be ingested.

    The release entity should have the container, file, fileset, and webcapture
    fields set.

    The type of the ingest request may depend on release type and container
    metadata (eg, as to whether we expect a PDF, datasets, web page), so
    calling code should check the returned type field.
    """

    if release.state != "active":
        return None

    if (not ingest_type) and release.container_id:
        ingest_type = INGEST_TYPE_CONTAINER_MAP.get(release.container_id)

    if not ingest_type:
        if release.release_type == "stub":
            return None
        elif release.release_type in ["component", "graphic"]:
            ingest_type = "component"
        elif release.release_type == "dataset":
            ingest_type = "dataset"
        elif release.release_type == "software":
            ingest_type = "software"
        elif release.release_type == "post-weblog":
            ingest_type = "html"
        elif release.release_type in [
            "article-journal",
            "article",
            "chapter",
            "paper-conference",
            "book",
            "report",
            "thesis",
        ]:
            ingest_type = "pdf"
        else:
            ingest_type = "pdf"

    # generate a URL where we expect to find fulltext
    url = None
    link_source = None
    link_source_id = None
    if release.ext_ids.arxiv and ingest_type == "pdf":
        url = "https://arxiv.org/pdf/{}.pdf".format(release.ext_ids.arxiv)
        link_source = "arxiv"
        link_source_id = release.ext_ids.arxiv
    elif release.ext_ids.pmcid and ingest_type == "pdf":
        # TODO: how to tell if an author manuscript in PMC vs. published?
        # url = "https://www.ncbi.nlm.nih.gov/pmc/articles/{}/pdf/".format(release.ext_ids.pmcid)
        url = "http://europepmc.org/backend/ptpmcrender.fcgi?accid={}&blobtype=pdf".format(
            release.ext_ids.pmcid
        )
        link_source = "pmc"
        link_source_id = release.ext_ids.pmcid
    elif release.ext_ids.doi:
        url = "https://doi.org/{}".format(release.ext_ids.doi.lower())
        link_source = "doi"
        link_source_id = release.ext_ids.doi.lower()

    if not url:
        return None

    ext_ids = release.ext_ids.to_dict()
    ext_ids = dict([(k, v) for (k, v) in ext_ids.items() if v])

    ingest_request = {
        "ingest_type": ingest_type,
        "ingest_request_source": ingest_request_source,
        "base_url": url,
        "release_stage": release.release_stage,
        "fatcat": {
            "release_ident": release.ident,
            "work_ident": release.work_id,
        },
        "ext_ids": ext_ids,
    }

    if link_source and link_source_id:
        ingest_request["link_source"] = link_source
        ingest_request["link_source_id"] = link_source_id

    return ingest_request
