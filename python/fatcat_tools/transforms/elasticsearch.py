

import collections
from fatcat_openapi_client import ApiClient


def check_kbart(year, archive):
    if not archive or not archive.get('year_spans'):
        return None
    for span in archive['year_spans']:
        if year >= span[0] and year <= span[1]:
            return True
    return False

def test_check_kbart():

    assert check_kbart(1990, dict()) == None
    assert check_kbart(1990, dict(year_spans=[[2000, 2000]])) == False
    assert check_kbart(2000, dict(year_spans=[[2000, 2000]])) == True
    assert check_kbart(1950, dict(year_spans=[[1900, 1920], [1990, 2000]])) == False
    assert check_kbart(1950, dict(year_spans=[[1900, 1920], [1930, 2000]])) == True


def release_to_elasticsearch(entity, force_bool=True):
    """
    Converts from an entity model/schema to elasticsearch oriented schema.

    Returns: dict
    Raises exception on error (never returns None)
    """

    if entity.state in ('redirect', 'deleted'):
        return dict(
            ident = entity.ident,
            state = entity.state,
        )
    elif entity.state != 'active':
        raise ValueError("Unhandled entity state: {}".format(entity.state))

    # First, the easy ones (direct copy)
    release = entity
    t = dict(
        ident = release.ident,
        state = release.state,
        revision = release.revision,
        work_id = release.work_id,
        title = release.title,
        subtitle = release.subtitle,
        original_title = release.original_title,
        release_type = release.release_type,
        release_stage = release.release_stage,
        withdrawn_status = release.withdrawn_status,
        language = release.language,
        volume = release.volume,
        issue = release.issue,
        pages = release.pages,
        number = release.number,
        license = release.license_slug,
        doi = release.ext_ids.doi,
        pmid = release.ext_ids.pmid,
        pmcid = release.ext_ids.pmcid,
        isbn13 = release.ext_ids.isbn13,
        wikidata_qid = release.ext_ids.wikidata_qid,
        core_id = release.ext_ids.core,
        arxiv_id = release.ext_ids.arxiv,
        jstor_id = release.ext_ids.jstor,
        ark_id = release.ext_ids.ark,
        mag_id = release.ext_ids.mag,
    )

    is_oa = None
    is_preserved = None
    is_longtail_oa = None
    in_kbart = None
    in_jstor = False
    in_web = False
    in_dweb = False
    in_ia = False
    in_ia_sim = False
    in_shadows = False

    release_year = release.release_year
    if release.release_date:
        # .isoformat() results in, eg, '2010-10-22' (YYYY-MM-DD)
        t['release_date'] = release.release_date.isoformat()
        if not release_year:
            release_year = release.release_date.year
    if release_year:
        t['release_year'] = release_year

    t['any_abstract'] = len(release.abstracts or []) > 0
    t['ref_count'] = len(release.refs or [])
    ref_release_ids = []
    for r in (release.refs or []):
        if r.target_release_id:
            ref_release_ids.append(r.target_release_id)
    t['ref_release_ids'] = ref_release_ids
    t['ref_linked_count'] = len(ref_release_ids)
    t['contrib_count'] = len(release.contribs or [])
    contrib_names = []
    contrib_affiliations = []
    creator_ids = []
    for c in (release.contribs or []):
        if c.raw_name:
            contrib_names.append(c.raw_name)
        elif c.surname:
            contrib_names.append(c.surname)
        if c.creator_id:
            creator_ids.append(c.creator_id)
        if c.raw_affiliation:
            contrib_affiliations.append(c.raw_affiliation)
    t['contrib_names'] = contrib_names
    t['creator_ids'] = creator_ids
    t['affiliations'] = contrib_affiliations

    # TODO: mapping... probably by lookup?
    t['affiliation_rors'] = None

    container = release.container
    if container:
        t['publisher'] = container.publisher
        t['container_name'] = container.name
        # this is container.ident, not release.container_id, because there may
        # be a redirect involved
        t['container_id'] = container.ident
        t['container_issnl'] = container.issnl
        t['container_type'] = container.container_type
        if container.extra:
            c_extra = container.extra
            if c_extra.get('kbart') and release_year:
                in_jstor = check_kbart(release_year, c_extra['kbart'].get('jstor'))
                in_kbart = in_jstor
                for archive in ('portico', 'lockss', 'clockss'):
                    in_kbart = in_kbart or check_kbart(release_year, c_extra['kbart'].get(archive))

            if c_extra.get('ia'):
                if c_extra['ia'].get('sim') and release_year:
                    in_ia_sim = check_kbart(release_year, c_extra['ia']['sim'])
                if c_extra['ia'].get('longtail_oa'):
                    is_longtail_oa = True
            if c_extra.get('sherpa_romeo'):
                if c_extra['sherpa_romeo'].get('color') == 'white':
                    is_oa = False
            if c_extra.get('default_license') and c_extra.get('default_license').startswith('CC-'):
                is_oa = True
            if c_extra.get('doaj'):
                if c_extra['doaj'].get('as_of'):
                    is_oa = True
            if c_extra.get('road'):
                if c_extra['road'].get('as_of'):
                    is_oa = True
            if c_extra.get('ezb'):
                if c_extra['ezb'].get('color') == 'green':
                    is_oa = True
            if c_extra.get('szczepanski'):
                if c_extra['szczepanski'].get('as_of'):
                    is_oa = True

    # fall back to release-level container metadata if container not linked or
    # missing context
    if not t.get('publisher'):
        t['publisher'] = release.publisher
    if not t.get('container_name') and release.extra:
        t['container_name'] = release.extra.get('container_name')

    if release.ext_ids.jstor or (release.ext_ids.doi and release.ext_ids.doi.startswith('10.2307/')):
        in_jstor = True

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
        for release_url in (f.urls or []):
            if not f.mimetype and 'pdf' in release_url.url.lower():
                is_pdf = True
            if release_url.url.lower().startswith('http'):
                in_web = True
            if release_url.rel in ('dweb', 'p2p', 'ipfs', 'dat', 'torrent'):
                # not sure what rel will be for this stuff
                in_dweb = True
            if is_pdf:
                any_pdf_url = release_url.url
            if is_pdf and release_url.rel in ('webarchive', 'repository') and is_pdf:
                is_preserved = True
                good_pdf_url = release_url.url
            if '//www.jstor.org/' in release_url.url:
                in_jstor = True
            if '//web.archive.org/' in release_url.url or '//archive.org/' in release_url.url:
                in_ia = True
                if is_pdf:
                    best_pdf_url = release_url.url
                    ia_pdf_url = release_url.url
    # here is where we bake-in priority; IA-specific
    t['best_pdf_url'] = best_pdf_url or good_pdf_url or any_pdf_url
    t['ia_pdf_url'] = ia_pdf_url

    if release.license_slug:
        # TODO: more/better checks here, particularly strict *not* OA licenses
        if release.license_slug.startswith("CC-"):
            is_oa = True

    extra = release.extra or dict()
    if extra:
        if extra.get('is_oa'):
            # NOTE: not actually setting this anywhere... but could
            is_oa = True
        if extra.get('longtail_oa'):
            # sometimes set by GROBID/matcher
            is_oa = True
            is_longtail_oa = True
        if not t.get('container_name'):
            t['container_name'] = extra.get('container_name')
        if extra.get('crossref'):
            if extra['crossref'].get('archive'):
                # all crossref archives are KBART, I believe
                in_kbart = True
        # backwards compatible subtitle fetching
        if not t['subtitle'] and extra.get('subtitle'):
            if type(extra['subtitle']) == list:
                t['subtitle'] = extra['subtitle'][0]
            else:
                t['subtitle'] = extra['subtitle']

    t['first_page'] = None
    if release.pages:
        first = release.pages.split('-')[0]
        first = first.replace('p', '')
        if first.isdigit():
            t['first_page'] = first
        # TODO: non-numerical first pages

    t['ia_microfilm_url'] = None
    if in_ia_sim:
        # TODO: determine URL somehow? I think this is in flux. Will probably
        # need extra metadata in the container extra field.
        # special case as a demo for now.
        if release.container_id == "hl5g6d5msjcl7hlbyyvcsbhc2u" \
                and release.release_year in (2011, 2013) \
                and release.issue.isdigit() \
                and t['first_page']:
            t['ia_microfilm_url'] = "https://archive.org/details/sim_bjog_{}-{:02d}/page/n{}".format(
                release.release_year,
                int(release.issue) - 1,
                t['first_page'],
            )

    t['doi_registrar'] = None
    if extra and t['doi']:
        for k in ('crossref', 'datacite', 'jalc'):
            if k in extra:
                t['doi_registrar'] = k
        if not 'doi_registrar' in t:
            t['doi_registrar'] = 'crossref'

    if t['doi']:
        t['doi_prefix'] = t['doi'].split('/')[0]

    if is_longtail_oa:
        is_oa = True

    if force_bool:
        t['is_oa'] = bool(is_oa)
        t['is_longtail_oa'] = bool(is_longtail_oa)
        t['in_kbart'] = bool(in_kbart)
        t['in_ia_sim'] = bool(in_ia_sim)
        t['in_jstor'] = bool(in_jstor)
        t['in_web'] = bool(in_web)
        t['in_dweb'] = bool(in_dweb)
        t['in_shadows'] = bool(in_shadows)
    else:
        t['is_oa'] = is_oa
        t['is_longtail_oa'] = is_longtail_oa
        t['in_kbart'] = in_kbart
        t['in_ia_sim'] = in_ia_sim
        t['in_jstor'] = in_jstor
        t['in_web'] = in_web
        t['in_dweb'] = in_dweb
        t['in_shadows'] = in_shadows

    t['in_ia'] = bool(in_ia)
    t['is_preserved'] = bool(is_preserved or in_ia or in_kbart or in_jstor)

    if in_ia:
        t['preservation'] = 'bright'
    elif in_kbart or in_jstor:
        t['preservation'] = 'dark_only'
    elif in_shadows:
        t['preservation'] = 'shadows_only'
    else:
        t['preservation'] = 'none'

    return t


def container_to_elasticsearch(entity, force_bool=True):
    """
    Converts from an entity model/schema to elasticsearch oriented schema.

    Returns: dict
    Raises exception on error (never returns None)
    """

    if entity.state in ('redirect', 'deleted'):
        return dict(
            ident = entity.ident,
            state = entity.state,
        )
    elif entity.state != 'active':
        raise ValueError("Unhandled entity state: {}".format(entity.state))

    # First, the easy ones (direct copy)
    t = dict(
        ident = entity.ident,
        state = entity.state,
        revision = entity.revision,

        name = entity.name,
        publisher = entity.publisher,
        container_type = entity.container_type,
        issnl = entity.issnl,
        wikidata_qid = entity.wikidata_qid,
    )

    if not entity.extra:
        entity.extra = dict()
    for key in ('country', 'languages', 'mimetypes', 'original_name',
                'first_year', 'last_year', 'aliases', 'abbrev', 'region',
                'discipline'):
        if entity.extra.get(key):
            t[key] = entity.extra[key]

    t['issns'] = []
    if entity.issnl:
        t['issns'].append(entity.issnl)
    for key in ('issnp', 'issne'):
        if entity.extra.get(key):
            t['issns'].append(entity.extra[key])

    in_doaj = None
    in_road = None
    is_oa = None
    is_longtail_oa = None
    any_kbart = None
    any_jstor = None
    any_ia_sim = None

    extra = entity.extra
    if extra.get('doaj'):
        if extra['doaj'].get('as_of'):
            in_doaj = True
    if extra.get('road'):
        if extra['road'].get('as_of'):
            in_road = True
    if extra.get('ezb'):
        if extra['ezb'].get('color') == 'green':
            is_oa = True
    if extra.get('szczepanski'):
        if extra['szczepanski'].get('as_of'):
            is_oa = True
    if extra.get('default_license'):
        if extra['default_license'].startswith('CC-'):
            is_oa = True
    t['sherpa_romeo_color'] = None
    if extra.get('sherpa_romeo'):
        t['sherpa_romeo_color'] = extra['sherpa_romeo'].get('color')
        if extra['sherpa_romeo'].get('color') == 'white':
            is_oa = False
    if extra.get('kbart'):
        any_kbart = True
        if extra['kbart'].get('jstor'):
            any_jstor = True
    if extra.get('ia'):
        if extra['ia'].get('sim'):
            any_ia_sim = True
        if extra['ia'].get('longtail_oa'):
            is_longtail_oa = True
    t['is_superceded'] = bool(extra.get('superceded'))

    t['in_doaj'] = bool(in_doaj)
    t['in_road'] = bool(in_road)
    t['any_kbart'] = bool(any_kbart)
    if force_bool:
        t['is_oa'] = bool(in_doaj or in_road or is_oa)
        t['is_longtail_oa'] = bool(is_longtail_oa)
        t['any_jstor'] = bool(any_jstor)
        t['any_ia_sim'] = bool(any_ia_sim)
    else:
        t['is_oa'] = in_doaj or in_road or is_oa
        t['is_longtail_oa'] = is_longtail_oa
        t['any_jstor'] = any_jstor
        t['any_ia_sim'] = any_ia_sim
    return t


def _type_of_edit(edit):
    if edit.revision == None and edit.redirect_ident == None:
        return 'delete'
    elif edit.redirect_ident:
        # redirect
        return 'update'
    elif edit.prev_revision == None and edit.redirect_ident == None and edit.revision:
        return 'create'
    else:
        return 'update'


def changelog_to_elasticsearch(entity):

    editgroup = entity.editgroup
    t = dict(
        index=entity.index,
        editgroup_id=entity.editgroup_id,
        timestamp=entity.timestamp.isoformat(),
        editor_id=editgroup.editor_id,
        username=editgroup.editor.username,
        is_bot=editgroup.editor.is_bot,
        is_admin=editgroup.editor.is_admin,
    )

    extra = editgroup.extra or dict()
    if extra.get('agent'):
        t['agent'] = extra['agent']

    containers = [_type_of_edit(e) for e in editgroup.edits.containers]
    creators = [_type_of_edit(e) for e in editgroup.edits.creators]
    files = [_type_of_edit(e) for e in editgroup.edits.files]
    filesets = [_type_of_edit(e) for e in editgroup.edits.filesets]
    webcaptures = [_type_of_edit(e) for e in editgroup.edits.webcaptures]
    releases = [_type_of_edit(e) for e in editgroup.edits.releases]
    works = [_type_of_edit(e) for e in editgroup.edits.works]

    t['containers'] = len(containers)
    t['new_containers'] = len([e for e in containers if e == 'create'])
    t['creators'] = len(creators)
    t['new_creators'] = len([e for e in creators if e == 'create'])
    t['files'] = len(files)
    t['new_files'] = len([e for e in files if e == 'create'])
    t['filesets'] = len(filesets)
    t['new_filesets'] = len([e for e in filesets if e == 'create'])
    t['webcaptures'] = len(webcaptures)
    t['new_webcaptures'] = len([e for e in webcaptures if e == 'create'])
    t['releases'] = len(releases)
    t['new_releases'] = len([e for e in releases if e == 'create'])
    t['works'] = len(works)
    t['new_works'] = len([e for e in works if e == 'create'])

    all_edits = containers + creators + files + filesets + webcaptures + releases + works

    t['created'] = len([e for e in all_edits if e == 'create'])
    t['updated'] = len([e for e in all_edits if e == 'update'])
    t['deleted'] = len([e for e in all_edits if e == 'delete'])
    t['total'] = len(all_edits)
    return t


def file_to_elasticsearch(entity):
    """
    Converts from an entity model/schema to elasticsearch oriented schema.

    Returns: dict
    Raises exception on error (never returns None)
    """

    if entity.state in ('redirect', 'deleted'):
        return dict(
            ident = entity.ident,
            state = entity.state,
        )
    elif entity.state != 'active':
        raise ValueError("Unhandled entity state: {}".format(entity.state))

    # First, the easy ones (direct copy)
    t = dict(
        ident = entity.ident,
        state = entity.state,
        revision = entity.revision,
        release_ids = entity.release_ids,
        release_count = len(entity.release_ids),
        mimetype = entity.mimetype,
        size_bytes = entity.size,
        sha1 = entity.sha1,
        sha256 = entity.sha256,
        md5 = entity.md5,
        rel = [u.rel for u in entity.urls],
    )

    # TODO: domain, hosts (from urls; use proper urlcanon)
    t['rel'] = list(set([u.rel for u in entity.urls]))
    t['host'] = []
    t['domain'] = []

    in_ia = False
    for u in entity.urls:
        if '://archive.org/' in u.url or '://web.archive.org/' in u.url:
            in_ia = True
    t['in_ia'] = bool(in_ia)

    return t
