
import json

from citeproc import CitationStylesStyle, CitationStylesBibliography
from citeproc import Citation, CitationItem
from citeproc import formatter
from citeproc.source.json import CiteProcJSON
from citeproc_styles import get_style_filepath


def contribs_by_role(contribs, role):
    ret = [c.copy() for c in contribs if c['role'] == role]
    [c.pop('role') for c in ret]
    # TODO: some note to self here
    [c.pop('literal') for c in ret if 'literal' in c]
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
            # Default to "local" (publication-specific) metadata; fall back to
            # creator-level
            family = contrib.creator.surname or contrib.surname  or (contrib.raw_name and contrib.raw_name.split()[-1])
            if not family:
                # CSL requires some surname (family name)
                continue
            c = dict(
                family=family,
                given=contrib.creator.given_name or contrib.given_name,
                #dropping-particle
                #non-dropping-particle
                #suffix
                #comma-suffix
                #static-ordering
                literal=contrib.creator.display_name or contrib.raw_name,
                #parse-names,
                # role must be defined; default to author
                role=contrib.role or 'author',
            )
        else:
            family = contrib.surname or (contrib.raw_name and contrib.raw_name.split()[-1])
            if not family:
                # CSL requires some surname (family name)
                continue
            c = dict(
                family=family,
                given=contrib.given_name,
                literal=contrib.raw_name,
                # role must be defined; default to author
                role=contrib.role or 'author',
            )
        for k in list(c.keys()):
            if not c[k]:
                c.pop(k)
        contribs.append(c)
    if not contribs:
        raise ValueError("citeproc requires at least one author with a surname")
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
        type=entity.release_type or "article", # can't be blank
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
        DOI=entity.ext_ids.doi,
        #edition
        #event
        #event-place
        #first-reference-note-number
        #genre
        ISBN=entity.ext_ids.isbn13,
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
        # TODO: page=entity.pages,
        page_first=entity.pages and entity.pages.split('-')[0],
        PMCID=entity.ext_ids.pmcid,
        PMID=entity.ext_ids.pmid,
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

def citeproc_csl(csl_json, style, html=False):
    """
    Renders a release entity to a styled citation.

    Notable styles include:
    - 'csl-json': special case to JSON encode the structured CSL object (via
      release_to_csl())
    - bibtext: multi-line bibtext format (used with LaTeX)

    Returns a string; if the html flag is set, and the style isn't 'csl-json'
    or 'bibtex', it will be HTML. Otherwise plain text.
    """
    if not csl_json.get('id'):
        csl_json['id'] = "unknown"
    if style == "csl-json":
        return json.dumps(csl_json)
    bib_src = CiteProcJSON([csl_json])
    form = formatter.plain
    if html:
        form = formatter.html
    style_path = get_style_filepath(style)
    bib_style = CitationStylesStyle(style_path, validate=False)
    bib = CitationStylesBibliography(bib_style, bib_src, form)
    bib.register(Citation([CitationItem(csl_json['id'])]))
    lines = bib.bibliography()[0]
    if style == "bibtex":
        out = ""
        for line in lines:
            if line.startswith(" @"):
                out += "@"
            elif line.startswith(" "):
                out += "\n " + line
            else:
                out += line
        return ''.join(out)
    else:
        return ''.join(lines)
