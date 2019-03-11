

import collections
from fatcat_client import ApiClient

def entity_to_dict(entity, api_client=None):
    """
    Hack to take advantage of the code-generated serialization code.

    Initializing/destroying ApiClient objects is surprisingly expensive
    (because it involves a threadpool), so we allow passing an existing
    instance. If you already have a full-on API connection `api`, you can
    access the ApiClient object as `api.api_client`. This is such a speed-up
    that this argument may become mandatory.
    """
    if not api_client:
        api_client = ApiClient()
    return api_client.sanitize_for_serialization(entity)

def entity_from_json(json_str, entity_type, api_client=None):
    """
    Hack to take advantage of the code-generated deserialization code

    See not on `entity_to_dict()` about api_client argument.
    """
    if not api_client:
        api_client = ApiClient()
    thing = collections.namedtuple('Thing', ['data'])
    thing.data = json_str
    return api_client.deserialize(thing, entity_type)

def check_kbart(year, archive):
    if not archive or not archive.get('year_spans'):
        return None
    for span in archive['year_spans']:
        if year >= span[0] and year <= span[1]:
            return True
    return False

def test_check_kbart():

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
    is_preserved = None
    is_longtail_oa = None
    in_kbart = None
    in_jstor = False
    in_web = False
    in_dweb = False
    in_ia = False
    in_ia_sim = False
    in_shadow = False

    release_year = release.release_year
    if release.release_date:
        # .isoformat() results in, eg, '2010-10-22' (YYYY-MM-DD)
        t['release_date'] = release.release_date.isoformat()
        if not release_year:
            release_year = release.release_date.year
    if release_year:
        t['release_year'] = release_year

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
    else:
        t['publisher'] = release.publisher

    if release.jstor_id or (release.doi and release.doi.startswith('10.2307/')):
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
        for url in (f.urls or []):
            if url.url.lower().startswith('http'):
                in_web = True
            if url.rel in ('dweb', 'p2p', 'ipfs', 'dat', 'torrent'):
                # not sure what rel will be for this stuff
                in_dweb = True
            if is_pdf:
                any_pdf_url = url.url
            if is_pdf and url.rel in ('webarchive', 'repository') and is_pdf:
                is_preserved = True
                good_pdf_url = url.url
            if '//www.jstor.org/' in url.url:
                in_jstor = True
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
    else:
        t['is_oa'] = is_oa
        t['is_longtail_oa'] = is_longtail_oa
        t['in_kbart'] = in_kbart
        t['in_ia_sim'] = in_ia_sim
        t['in_jstor'] = in_jstor
        t['in_web'] = in_web
        t['in_dweb'] = in_dweb

    t['in_ia'] = bool(in_ia)
    t['is_preserved'] = bool(is_preserved or in_ia or in_kbart or in_jstor)
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

    # TODO: region, discipline
    # TODO: single primary language?
    if not entity.extra:
        entity.extra = dict()
    for key in ('country', 'languages', 'mimetypes', 'first_year', 'last_year'):
        if entity.extra.get(key):
            t[key] = entity.extra[key]

    in_doaj = None
    in_road = None
    # TODO: not currently implemented
    in_doi = None
    # TODO: would be nice to have 'in_doaj_works', or maybe just "any_pid"
    #in_doaj_works = None
    in_sherpa_romeo = None
    is_oa = None
    # TODO: not actually set/stored anywhere?
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
    if extra.get('default_license'):
        if extra['default_license'].startswith('CC-'):
            is_oa = True
    if extra.get('sherpa_romeo'):
        in_sherpa_romeo = True
        if extra['sherpa_romeo'].get('color') == 'white':
            is_oa = False
    if extra.get('kbart'):
        any_kbart = True
        if extra['kbart'].get('jstor'):
            any_jstor = True
    if extra.get('ia'):
        if extra['ia'].get('sim'):
            any_ia_sim = True

    t['in_doaj'] = bool(in_doaj)
    t['in_road'] = bool(in_road)
    t['in_sherpa_romeo'] = bool(in_sherpa_romeo)
    t['any_kbart'] = bool(any_kbart)
    t['is_longtail_oa'] = bool(is_longtail_oa)
    if force_bool:
        t['in_doi'] = bool(in_doi)
        t['is_oa'] = bool(in_doaj or in_road or is_longtail_oa or is_oa)
        t['any_jstor'] = bool(any_jstor)
        t['any_ia_sim'] = bool(any_ia_sim)
    else:
        t['in_doi'] = in_doi
        t['is_oa'] = in_doaj or in_road or is_longtail_oa or is_oa
        t['any_jstor'] = any_jstor
        t['any_ia_sim'] = any_ia_sim
    return t


def contribs_by_role(contribs, role):
    ret = [c.copy() for c in contribs if c['role'] == role]
    [c.pop('role') for c in ret]
    # XXX:
    [c.pop('literal') for c in ret]
    if not ret:
        return None
    else:
        return ret


def release_to_csl(entity):
    """
    Returns a python dict which can be json.dumps() to get a CSL-JSON (aka,
    citeproc-JSON, aka Citation Style Language JSON)

    This function will likely become an API method/endpoint

    Follows, but not enforced by: https://github.com/citation-style-language/schema/blob/master/csl-data.json
    """
    contribs = []
    for contrib in (entity.contribs or []):
        if contrib.creator:
            # TODO: should we actually be pulling creator metadata? or just
            # using release-local raw metadata?
            c = dict(
                family=contrib.creator.surname,
                given=contrib.creator.given_name,
                #dropping-particle
                #non-dropping-particle
                #suffix
                #comma-suffix
                #static-ordering
                literal=contrib.raw_name, # or display_name?
                #parse-names,
                role=contrib.role,
            )
        else:
            c = dict(
                # XXX: possible inclusion of full name metadata in release_contrib
                family=contrib.raw_name.split()[-1],
                literal=contrib.raw_name,
                role=contrib.role,
            )
        for k in list(c.keys()):
            if not c[k]:
                c.pop(k)
        contribs.append(c)
    abstract = None
    if entity.abstracts:
        abstract = entity.abstracts[0].content

    issued_date = None
    if entity.release_date:
        issued_date = {"date-parts": [[
            entity.release_date.year,
            entity.release_date.month,
            entity.release_date.day,
        ]]}
    elif entity.release_year:
        issued_date = {"date-parts": [[entity.release_year]]}

    csl = dict(
        #id,
        #categories
        type=entity.release_type or "article", # XXX: can't be blank
        language=entity.language,
        #journalAbbreviation
        #shortTitle
        ## see below for all contrib roles
        #accessed
        #container
        #event-date
        issued=issued_date,
        #original-date
        #submitted
        abstract=abstract,
        #annote
        #archive
        #archive_location
        #archive-place
        #authority
        #call-number
        #chapter-number
        #citation-number
        #citation-label
        #collection-number
        #collection-title
        container_title=entity.container and entity.container.name,
        #container-title-short
        #dimensions
        DOI=entity.doi,
        #edition
        #event
        #event-place
        #first-reference-note-number
        #genre
        ISBN=entity.isbn13,
        ISSN=entity.container and entity.container.issnl,
        issue=entity.issue,
        #jurisdiction
        #keyword
        #locator
        #medium
        #note
        #number
        #number-of-pages
        #number-of-volumes
        #original-publisher
        #original-publisher-place
        #original-title
        # XXX: page=entity.pages,
        page_first=entity.pages.split('-')[0],
        PMCID=entity.pmcid,
        PMID=entity.pmid,
        publisher=(entity.container and entity.container.publisher) or entity.publisher,
        #publisher-place
        #references
        #reviewed-title
        #scale
        #section
        #source
        #status
        title=entity.title,
        #title-short
        #URL
        #version
        volume=entity.volume,
        #year-suffix
    )
    for role in ['author', 'collection-editor', 'composer', 'container-author',
            'director', 'editor', 'editorial-director', 'interviewer',
            'illustrator', 'original-author', 'recipient', 'reviewed-author',
            'translator']:
        cbr = contribs_by_role(contribs, role)
        if cbr:
            csl[role] = cbr
    # underline-to-dash
    csl['container-title'] = csl.pop('container_title')
    csl['page-first'] = csl.pop('page_first')
    empty_keys = [k for k,v in csl.items() if not v]
    for k in empty_keys:
        csl.pop(k)
    return csl


def refs_to_csl(entity):
    ret = []
    for ref in entity.refs:
        if ref.release_id and False:
            # TODO: fetch full entity from API and convert with release_to_csl
            raise NotImplementedError
        else:
            issued_date = None
            if ref.year:
                issued_date = [[ref.year]]
            csl = dict(
                title=ref.title,
                issued=issued_date,
            )
        csl['id'] = ref.key or ref.index, # zero- or one-indexed?
        ret.append(csl)
    return ret


def changelog_to_elasticsearch(entity):

    editgroup = entity.editgroup
    t = dict(
        index=entity.index,
        editgroup_id=entity.editgroup_id,
        timestamp=entity.timestamp,
        editor_id=editgroup.editor_id,
    )

    extra = editgroup.extra or dict()
    if extra.get('agent'):
        t['agent'] = extra['agent']

    t['containers'] = len(editgroup.edits.containers)
    t['creators'] = len(editgroup.edits.containers)
    t['files'] = len(editgroup.edits.containers)
    t['filesets'] = len(editgroup.edits.containers)
    t['webcaptures'] = len(editgroup.edits.containers)
    t['releases'] = len(editgroup.edits.containers)
    t['works'] = len(editgroup.edits.containers)

    # TODO: parse and pull out counts
    #created = 0
    #updated = 0
    #deleted = 0
    #t['created'] = created
    #t['updated'] = updated
    #t['deleted'] = deleted
    #t['total'] = created + updated + deleted
    return t
