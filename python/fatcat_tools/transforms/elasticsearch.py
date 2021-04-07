
import datetime
from typing import Optional

import tldextract

from fatcat_openapi_client import ReleaseEntity, ContainerEntity


def check_kbart(year: int, archive: dict) -> Optional[bool]:
    if not archive or not archive.get('year_spans'):
        return None
    for span in archive['year_spans']:
        if year >= span[0] and year <= span[1]:
            return True
    return False

def test_check_kbart() -> None:

    assert check_kbart(1990, dict()) is None
    assert check_kbart(1990, dict(year_spans=[[2000, 2000]])) is False
    assert check_kbart(2000, dict(year_spans=[[2000, 2000]])) is True
    assert check_kbart(1950, dict(year_spans=[[1900, 1920], [1990, 2000]])) is False
    assert check_kbart(1950, dict(year_spans=[[1900, 1920], [1930, 2000]])) is True


def release_to_elasticsearch(entity: ReleaseEntity, force_bool: bool = True) -> dict:
    """
    Converts from an entity model/schema to elasticsearch oriented schema.

    This is a large/complex transform, so subsets are split out into helper
    functions.

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
        doc_index_ts=datetime.datetime.utcnow(),
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
        dblp_id = release.ext_ids.dblp,
        doaj_id = release.ext_ids.doaj,
    )

    t.update(dict(
        is_oa = None,
        is_longtail_oa = None,
        is_preserved = None,
        in_web = False,
        in_dweb = False,
        in_ia = False,
        in_ia_sim = False,
        in_kbart = None,
        in_jstor = False,
        in_doaj= bool(release.ext_ids.doaj),
        in_shadows = False,
    ))

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

    if release.container:
        t.update(_rte_container_helper(release.container, release_year))

    # fall back to release-level container metadata if container not linked or
    # missing context
    if not t.get('publisher'):
        t['publisher'] = release.publisher
    if not t.get('container_name') and release.extra:
        t['container_name'] = release.extra.get('container_name')

    if release.ext_ids.jstor or (release.ext_ids.doi and release.ext_ids.doi.startswith('10.2307/')):
        t['in_jstor'] = True

    # transform file/fileset/webcapture related fields
    t.update(_rte_content_helper(release))

    if release.ext_ids.doaj:
        t['is_oa'] = True

    if release.license_slug:
        # TODO: more/better checks here, particularly strict *not* OA licenses
        if release.license_slug.startswith("CC-"):
            t['is_oa'] = True
        if release.license_slug.startswith("ARXIV-"):
            t['is_oa'] = True

    extra = release.extra or dict()
    if extra:
        if extra.get('is_oa'):
            # NOTE: not actually setting this anywhere... but could
            t['is_oa'] = True
        if extra.get('longtail_oa'):
            # sometimes set by GROBID/matcher
            t['is_oa'] = True
            t['is_longtail_oa'] = True
        if not t.get('container_name'):
            t['container_name'] = extra.get('container_name')
        if extra.get('crossref'):
            if extra['crossref'].get('archive'):
                # all crossref archives are KBART, I believe
                t['in_kbart'] = True
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
    if t['in_ia_sim']:
        # TODO: determine URL somehow? I think this is in flux. Will probably
        # need extra metadata in the container extra field.
        # special case as a demo for now.
        if release.container_id == "hl5g6d5msjcl7hlbyyvcsbhc2u" \
                and release.release_year in (2011, 2013) \
                and release.issue \
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

    if t['is_longtail_oa']:
        t['is_oa'] = True

    # optionally coerce all flags from Optional[bool] to bool
    if force_bool:
        for k in ('is_oa', 'is_longtail_oa', 'in_kbart', 'in_ia_sim',
                  'in_jstor', 'in_web', 'in_dweb', 'in_shadows'):
            t[k] = bool(t[k])

    t['in_ia'] = bool(t['in_ia'])
    t['is_preserved'] = bool(
        t['is_preserved']
        or t['in_ia']
        or t['in_kbart']
        or t['in_jstor']
        or t.get('pmcid')
        or t.get('arxiv_id')
    )

    if t['in_ia']:
        t['preservation'] = 'bright'
    elif t['is_preserved']:
        t['preservation'] = 'dark'
    elif t['in_shadows']:
        t['preservation'] = 'shadows_only'
    else:
        t['preservation'] = 'none'

    return t

def _rte_container_helper(container: ContainerEntity, release_year: Optional[int]) -> dict:
    """
    Container metadata sub-section of release_to_elasticsearch()
    """
    this_year = datetime.date.today().year
    t = dict()
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
            if check_kbart(release_year, c_extra['kbart'].get('jstor')):
                t['in_jstor'] = True
            if t.get('in_kbart') or t.get('in_jstor'):
                t['in_kbart'] = True
            for archive in ('portico', 'lockss', 'clockss', 'pkp_pln',
                            'hathitrust', 'scholarsportal', 'cariniana'):
                t['in_kbart'] = t.get('in_kbart') or check_kbart(release_year, c_extra['kbart'].get(archive))
                # recent KBART coverage is often not updated for the
                # current year. So for current-year publications, consider
                # coverage from *last* year to also be included in the
                # Keeper
                if not t.get('in_kbart') and release_year == this_year:
                    t['in_kbart'] = check_kbart(this_year - 1, c_extra['kbart'].get(archive))

        if c_extra.get('ia'):
            if c_extra['ia'].get('sim') and release_year:
                t['in_ia_sim'] = check_kbart(release_year, c_extra['ia']['sim'])
            if c_extra['ia'].get('longtail_oa'):
                t['is_longtail_oa'] = True
        if c_extra.get('sherpa_romeo'):
            if c_extra['sherpa_romeo'].get('color') == 'white':
                t['is_oa'] = False
        if c_extra.get('default_license') and c_extra.get('default_license').startswith('CC-'):
            t['is_oa'] = True
        if c_extra.get('doaj'):
            if c_extra['doaj'].get('as_of'):
                t['is_oa'] = True
                t['in_doaj'] = True
        if c_extra.get('road'):
            if c_extra['road'].get('as_of'):
                t['is_oa'] = True
        if c_extra.get('szczepanski'):
            if c_extra['szczepanski'].get('as_of'):
                t['is_oa'] = True
        if c_extra.get('country'):
            t['country_code'] = c_extra['country']
            t['country_code_upper'] = c_extra['country'].upper()
        if c_extra.get('publisher_type'):
            t['publisher_type'] = c_extra['publisher_type']
        if c_extra.get('discipline'):
            t['discipline'] = c_extra['discipline']
    return t

def _rte_content_helper(release: ReleaseEntity) -> dict:
    """
    File/FileSet/WebCapture sub-section of release_to_elasticsearch()

    The current priority order for "best_pdf_url" is:
    - internet archive urls (archive.org or web.archive.org)
    - other webarchive or repository URLs
    - any other URL
    """
    t = dict(
        file_count = len(release.files or []),
        fileset_count = len(release.filesets or []),
        webcapture_count = len(release.webcaptures or []),
    )

    any_pdf_url = None
    good_pdf_url = None
    best_pdf_url = None
    ia_pdf_url = None

    for f in release.files or []:
        if f.extra and f.extra.get('shadows'):
            t['in_shadows'] = True
        is_pdf = 'pdf' in (f.mimetype or '')
        for release_url in (f.urls or []):
            # first generic flags
            t.update(_rte_url_helper(release_url))

            # then PDF specific stuff (for generating "best URL" fields)
            if not f.mimetype and 'pdf' in release_url.url.lower():
                is_pdf = True
            if is_pdf:
                any_pdf_url = release_url.url
                if release_url.rel in ('webarchive', 'repository', 'repo'):
                    good_pdf_url = release_url.url
                if '//web.archive.org/' in release_url.url or '//archive.org/' in release_url.url:
                    best_pdf_url = release_url.url
                    ia_pdf_url = release_url.url

    # here is where we bake-in PDF url priority; IA-specific
    t['best_pdf_url'] = best_pdf_url or good_pdf_url or any_pdf_url
    t['ia_pdf_url'] = ia_pdf_url

    for fs in release.filesets or []:
        for url_obj in (fs.urls or []):
            t.update(_rte_url_helper(url_obj))

    for wc in release.webcaptures or []:
        for url_obj in (wc.archive_urls or []):
            t.update(_rte_url_helper(url_obj))

    return t

def _rte_url_helper(url_obj) -> dict:
    """
    Takes a location URL ('url' and 'rel' keys) and returns generic preservation status.

    Designed to work with file, webcapture, or fileset URLs.

    Returns a dict; should *not* include non-True values for any keys because
    these will be iteratively update() into the overal object.
    """
    t = dict()
    if url_obj.rel in ('webarchive', 'repository', 'archive', 'repo'):
        t['is_preserved'] = True
    if '//web.archive.org/' in url_obj.url or '//archive.org/' in url_obj.url:
        t['in_ia'] = True
    if url_obj.url.lower().startswith('http') or url_obj.url.lower().startswith('ftp'):
        t['in_web'] = True
    if url_obj.rel in ('dweb', 'p2p', 'ipfs', 'dat', 'torrent'):
        # not sure what rel will be for this stuff
        t['in_dweb'] = True
    if '//www.jstor.org/' in url_obj.url:
        t['in_jstor'] = True
    return t


def container_to_elasticsearch(entity, force_bool=True, stats=None):
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
        doc_index_ts=datetime.datetime.utcnow(),
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
                'discipline', 'publisher_type'):
        if entity.extra.get(key):
            t[key] = entity.extra[key]

    if entity.extra.get('dblp') and entity.extra['dblp'].get('prefix'):
        t['dblp_prefix'] = entity.extra['dblp']['prefix']

    if 'country' in t:
        t['country_code'] = t.pop('country')

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
    keepers = []

    extra = entity.extra
    if extra.get('doaj'):
        if extra['doaj'].get('as_of'):
            in_doaj = True
    if extra.get('road'):
        if extra['road'].get('as_of'):
            in_road = True
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
        for k, v in extra['kbart'].items():
            if v and isinstance(v, dict):
                keepers.append(k)
    if extra.get('ia'):
        if extra['ia'].get('sim'):
            any_ia_sim = True
        if extra['ia'].get('longtail_oa'):
            is_longtail_oa = True
    t['is_superceded'] = bool(extra.get('superceded'))

    t['keepers'] = keepers
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

    # mix in stats, if provided
    if stats:
        t['releases_total'] = stats['total']
        t['preservation_bright'] = stats['preservation']['bright']
        t['preservation_dark'] = stats['preservation']['dark']
        t['preservation_shadows_only'] = stats['preservation']['shadows_only']
        t['preservation_none'] = stats['preservation']['none']
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
    """
    Note that this importer requires expanded fill info to work. Calling code
    may need to re-fetch editgroup from API to get the 'editor' field. Some of
    the old kafka feed content doesn't includes editor in particular.
    """

    editgroup = entity.editgroup
    t = dict(
        doc_index_ts=datetime.datetime.utcnow(),
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
        doc_index_ts=datetime.datetime.utcnow(),
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
    )

    parsed_urls = [tldextract.extract(u.url) for u in entity.urls]
    t['hosts'] = list(set(['.'.join([seg for seg in pu if seg]) for pu in parsed_urls]))
    t['domains'] = list(set([pu.registered_domain for pu in parsed_urls]))
    t['rels'] = list(set([u.rel for u in entity.urls]))

    t['in_ia'] = bool('archive.org' in t['domains'])
    t['in_ia_petabox'] = bool('archive.org' in t['hosts'])

    any_url = None
    good_url = None
    best_url = None
    for release_url in (entity.urls or []):
        any_url = release_url.url
        if release_url.rel in ('webarchive', 'repository'):
            good_url = release_url.url
        if '//web.archive.org/' in release_url.url or '//archive.org/' in release_url.url:
            best_url = release_url.url
    # here is where we bake-in priority; IA-specific
    t['best_url'] = best_url or good_url or any_url

    return t
