
from .elasticsearch import release_to_elasticsearch

def release_ingest_request(release, oa_only=False, ingest_request_source='fatcat', ingest_type=None):
    """
    Takes a full release entity object and returns an ingest request (as dict),
    or None if it seems like this release shouldn't be ingested.

    The release entity should have the container, file, fileset, and webcapture
    fields set.

    The 'oa_only' boolean flag indicates that we should only return an ingest
    request if we have reason to believe this is an OA release (or, eg, in
    arxiv or pubmed central). Respecting this flag means we are likely to miss
    a lot of "hybrid" and "bronze" content, but could reduce crawl load
    significantly.

    The type of the ingest request may depend on release type and container
    metadata (eg, as to whether we expect a PDF, datasets, web page), so
    calling code should check the returned type field.
    """

    if release.state != 'active':
        return None

    # generate a URL where we expect to find fulltext
    url = None
    link_source = None
    link_source_id = None
    if release.ext_ids.arxiv:
        url = "https://arxiv.org/pdf/{}.pdf".format(release.ext_ids.arxiv)
        link_source = "arxiv"
        link_source_id = release.ext_ids.arxiv
    elif release.ext_ids.doi:
        url = "https://doi.org/{}".format(release.ext_ids.doi)
        link_source = "doi"
        link_source_id = release.ext_ids.doi
    elif release.ext_ids.pmcid and release.ext_ids.pmid:
        # TODO: how to tell if an author manuscript in PMC vs. published?
        #url = "https://www.ncbi.nlm.nih.gov/pmc/articles/{}/pdf/".format(release.ext_ids.pmcid)
        url = "http://europepmc.org/backend/ptpmcrender.fcgi?accid={}&blobtype=pdf".format(release.ext_ids.pmcid)
        link_source = "pubmed"
        link_source_id = release.ext_ids.pmid

    if not url:
        return None

    ext_ids = release.ext_ids.to_dict()
    ext_ids = dict([(k, v) for (k, v) in ext_ids.items() if v])

    if oa_only and link_source not in ('arxiv', 'pubmed'):
        es = release_to_elasticsearch(release)
        if not es['is_oa']:
            return None

    # TODO: infer ingest type based on release_type or container metadata?
    if not ingest_type:
        ingest_type = 'pdf'

    ingest_request = {
        'ingest_type': ingest_type,
        'ingest_request_source': ingest_request_source,
        'base_url': url,
        'release_stage': release.release_stage,
        'fatcat': {
            'release_ident': release.ident,
            'work_ident': release.work_id,
        },
        'ext_ids': ext_ids,
    }

    if link_source and link_source_id:
        ingest_request['link_source'] = link_source
        ingest_request['link_source_id'] = link_source_id

    return ingest_request

