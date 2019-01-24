

import collections
from fatcat_client import ReleaseEntity, ApiClient

def entity_to_dict(entity):
    """
    Hack to take advantage of the code-generated serialization code
    """
    ac = ApiClient()
    return ac.sanitize_for_serialization(entity)

def entity_from_json(json_str, entity_type):
    """
    Hack to take advantage of the code-generated deserialization code
    """
    ac = ApiClient()
    thing = collections.namedtuple('Thing', ['data'])
    thing.data = json_str
    return ac.deserialize(thing, entity_type)

def release_to_elasticsearch(release):
    """
    Converts from an entity model/schema to elasticsearch oriented schema.

    Returns: dict
    Raises exception on error (never returns None)
    """

    if release.state in ('redirect', 'deleted'):
        return dict(
            ident = release.ident,
            state = release.state,
        )
    elif release.state != 'active':
        raise ValueError("Unhandled release state: {}".format(release.state))

    # First, the easy ones (direct copy)
    t = dict(
        ident = release.ident,
        state = release.state,
        revision = release.revision,
        title = release.title,
        original_title = release.original_title,
        release_type = release.release_type,
        release_status = release.release_status,
        language = release.language,
        license = release.license_slug,
        doi = release.doi,
        pmid = release.pmid,
        pmcid = release.pmcid,
        isbn13 = release.isbn13,
        wikidata_qid = release.wikidata_qid,
        core_id = release.core_id,
        arxiv_id = release.core_id,
        jstor_id = release.jstor_id,
    )

    is_oa = None
    is_longtail_oa = None
    in_kbart = None
    in_web = False
    in_dweb = False
    in_ia = False
    in_shadow = False

    if release.release_date:
        # .isoformat() results in, eg, '2010-10-22' (YYYY-MM-DD)
        t['release_date'] = release.release_date.isoformat()
        if release.release_year is None:
            t['release_year'] = release.release_date.year
    if release.release_year is not None:
        t['release_year'] = release.release_year

    t['any_abstract'] = len(release.abstracts) > 0
    t['ref_count'] = len(release.refs or [])
    t['contrib_count'] = len(release.contribs or [])
    contrib_names = []
    for c in (release.contribs or []):
        if c.raw_name:
            contrib_names.append(c.raw_name)
    t['contrib_names'] = contrib_names

    container = release.container
    if container:
        t['publisher'] = container.publisher
        t['container_name'] = container.name
        t['container_issnl'] = container.issnl
        t['container_type'] = container.container_type
        if container.extra:
            if container.extra.get('is_oa') or container.extra.get('in_doaj'):
                is_oa = True
            if container.extra.get('in_kbart'):
                # TODO: better KBART check goes here
                in_kbart = True
            if container.extra.get('ia'):
                # TODO: container longtail check goes here
                # TODO: sim/microfilm check goes here
                pass
            # TODO: SHERPA/Romeo goes here
    else:
        t['publisher'] = release.publisher

    files = release.files or []
    t['file_count'] = len(files)
    t['fileset_count'] = len(release.filesets or [])
    t['webcapture_count'] = len(release.webcaptures or [])
    any_pdf_url = None
    good_pdf_url = None
    best_pdf_url = None
    ia_pdf_url = None
    for f in files:
        if f.extra and f.extra.get('shadows'):
            # TODO: shadow check goes here
            in_shadows = True
        is_pdf = 'pdf' in (f.mimetype or '')
        for url in (f.urls or []):
            if url.url.lower().startswith('http'):
                in_web = True
            if url.rel in ('dweb', 'p2p', 'ipfs', 'dat', 'torrent'):
                # TODO: not sure what rel will be
                in_dweb = True
            if is_pdf:
                any_pdf_url = url.url
            if is_pdf and url.rel in ('webarchive', 'repository') and is_pdf:
                is_preserved = True
                good_pdf_url = url.url
            if '//web.archive.org/' in url.url or '//archive.org/' in url.url:
                in_ia = True
                if is_pdf:
                    best_pdf_url = url.url
                    ia_pdf_url = url.url
    # here is where we bake-in priority; IA-specific
    t['best_pdf_url'] = best_pdf_url or good_pdf_url or any_pdf_url
    t['ia_pdf_url'] = ia_pdf_url

    if release.license_slug:
        # TODO: more/better checks here, particularly strict *not* OA licenses
        if release.license_slug.startswith("CC-"):
            is_oa = True

    extra = release.extra or dict()
    if extra:
        # TODO: longtail OA check from GROBID here
        if extra.get('in_kbart'):
            # NOTE: not actually setting this anywhere
            in_kbart = True
        if extra.get('is_oa'):
            # NOTE: not actually setting this anywhere
            is_oa = True
        if extra.get('grobid'):
            if not t.get('container_name'):
                t['container_name'] = extra['grobid'].get('container_name')
            if extra['grobid'].get('longtail_oa'):
                is_longtail_oa = True
        if extra.get('crossref'):
            if extra['crossref'].get('archive'):
                # all crossref archives are KBART, I believe
                in_kbart = True

    if is_longtail_oa:
        is_oa = True
    t['is_oa'] = is_oa
    t['is_longtail_oa'] = is_longtail_oa
    t['in_kbart'] = in_kbart
    t['in_web'] = in_web
    t['in_dweb'] = in_dweb
    t['in_ia'] = in_ia
    t['is_preserved'] = in_ia or in_kbart
    return t
