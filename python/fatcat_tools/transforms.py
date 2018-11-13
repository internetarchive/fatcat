
import collections
from fatcat_client import ReleaseEntity, ApiClient

def entity_to_json(entity):
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

def release_elastic_dict(release):
    """
    Converts from an entity model/schema to elasticsearch oriented schema.

    Returns: dict
    """

    if release.state != 'active':
        raise ValueError("Entity is not 'active'")

    # First, the easy ones (direct copy)
    t = dict(
        ident = release.ident,
        revision = release.revision,
        title = release.title,
        release_type = release.release_type,
        release_status = release.release_status,
        language = release.language,
        doi = release.doi,
        pmid = release.pmid,
        pmcid = release.pmcid,
        isbn13 = release.isbn13,
        core_id = release.core_id,
        wikidata_qid = release.wikidata_qid
    )

    if release.release_date:
        # TODO: resolve why this can be either a string or datetime
        if type(release.release_date) == str:
            t['release_date'] = release.release_date
        else:
            t['release_date'] = release.release_date.strftime('%F')

    container = release.container
    container_is_kept = False
    if container:
        t['publisher'] = container.publisher
        t['container_name'] = container.name
        t['container_issnl'] = container.issnl
        container_extra = container.extra
        if container_extra:
            t['container_is_oa'] = container_extra.get('is_oa')
            container_is_kept = container_extra.get('is_kept', False)
            t['container_is_longtail_oa'] = container_extra.get('is_longtail_oa')
    else:
        t['publisher'] = release.publisher

    files = release.files or []
    t['file_count'] = len(files)
    in_wa = False
    in_ia = False
    t['file_pdf_url'] = None
    for f in files:
        is_pdf = 'pdf' in f.get('mimetype', '')
        for url in f.get('urls', []):
            if url.get('rel', '') == 'webarchive':
                in_wa = True
            if '//web.archive.org/' in url['url'] or '//archive.org/' in url['url']:
                in_ia = True
                if is_pdf:
                    t['file_pdf_url'] = url['url']
            if not t['file_pdf_url'] and is_pdf:
                t['file_pdf_url'] = url['url']
    t['file_in_webarchive'] = in_wa
    t['file_in_ia'] = in_ia

    extra = release.extra or dict()
    if extra:
        t['in_shadow'] = extra.get('in_shadow')
        if extra.get('grobid') and extra['grobid'].get('is_longtail_oa'):
            t['container_is_longtail_oa'] = True
    t['any_abstract'] = bool(release.abstracts)
    t['is_kept'] = container_is_kept or extra.get('is_kept', False)

    t['ref_count'] = len(release.refs or [])
    t['contrib_count'] = len(release.contribs or [])
    contrib_names = []
    for c in (release.contribs or []):
        if c.raw_name:
            contrib_names.append(c.raw_name)
    t['contrib_names'] = contrib_names
    return t
