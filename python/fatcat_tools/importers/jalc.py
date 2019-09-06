
import sys
import json
import sqlite3
import datetime
import itertools
import subprocess
from bs4 import BeautifulSoup

import fatcat_openapi_client
from .common import EntityImporter, clean, is_cjk, DATE_FMT


def parse_jalc_persons(raw_persons):
    """
    For the most part, JALC DC names are in either japanese or english. The
    two common patterns are a list alternating between the two (in which case
    the names are translations), or all in one language or the other.

    Because dublin core is a projection tossing away a bunch of context, the
    other cases are hard to disambiguate. There are also some cases with Korean
    and other languages mixed in. This crude method doesn't handle everything
    right; it tries to just get the two common patterns correct. Sorry humans!

    Edge cases for this function:
    - 10.11316/jpsgaiyo.56.1.4.0_757_3 <= all english, some japanese, works
    - 10.14988/pa.2017.0000013531 <= complex, not japanese/english, mixed
    - 10.15036/arerugi.62.1407_1 <= one japanese, two english; fails
    - 10.14988/pa.2017.0000007327 <= ambiguous; translator in jpn/eng
    """

    persons = []

    # first parse out into language-agnostic dics
    for raw in raw_persons:
        name = raw.find('name') or None
        if name:
            name = clean(name.string)
        surname = raw.find('familyName') or None
        if surname:
            surname = clean(surname.string)
        given_name = raw.find('givenName') or None
        if given_name:
            given_name = clean(given_name.string)
        lang = 'en'
        if is_cjk(name):
            lang = 'ja'
        if lang == 'en' and surname and given_name:
            # english names order is flipped
            name = "{} {}".format(given_name, surname)
        rc = fatcat_openapi_client.ReleaseContrib(
            raw_name=name,
            surname=surname,
            given_name=given_name,
            role="author")
        # add an extra hint field; won't end up in serialized object
        rc._lang = lang
        persons.append(rc)

    if not persons:
        return []

    if all([p._lang == 'en' for p in persons]) or all([p._lang == 'ja' for p in persons]):
        # all english names, or all japanese names
        return persons

    # for debugging
    #if len([1 for p in persons if p._lang == 'en']) != len([1 for p in persons if p._lang == 'ja']):
    #    print("INTERESTING: {}".format(persons[0]))

    start_lang = persons[0]._lang
    contribs = []
    for p in persons:
        if p._lang == start_lang:
            contribs.append(p)
        else:
            if p._lang == 'en' and contribs[-1]._lang == 'ja':
                eng = p
                jpn = contribs[-1]
            elif p._lang == 'ja' and contribs[-1]._lang == 'en':
                eng = contribs[-1]
                jpn = p
            else:
                # give up and just add as another author
                contribs.append(p)
                continue
            eng.extra = {
                'original_name': {
                    'lang': jpn._lang,
                    'raw_name': jpn.raw_name,
                    'given_name': jpn.given_name,
                    'surname': jpn.surname,
                },
            }
            contribs[-1] = eng
    return contribs


class JalcImporter(EntityImporter):
    """
    Importer for JALC DOI metadata.

    NOTE: some JALC DOIs seem to get cross-registered with Crossref
    """

    def __init__(self, api, issn_map_file, **kwargs):

        eg_desc = kwargs.get('editgroup_description',
            "Automated import of JALC DOI metadata")
        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.JalcImporter')
        super().__init__(api,
            issn_map_file=issn_map_file,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)

        self.create_containers = kwargs.get('create_containers', True)
        extid_map_file = kwargs.get('extid_map_file')
        self.extid_map_db = None
        if extid_map_file:
            db_uri = "file:{}?mode=ro".format(extid_map_file)
            print("Using external ID map: {}".format(db_uri))
            self.extid_map_db = sqlite3.connect(db_uri, uri=True)
        else:
            print("Not using external ID map")

        self.read_issn_map_file(issn_map_file)

    def lookup_ext_ids(self, doi):
        if self.extid_map_db is None:
            return dict(core_id=None, pmid=None, pmcid=None, wikidata_qid=None, arxiv_id=None, jstor_id=None)
        row = self.extid_map_db.execute("SELECT core, pmid, pmcid, wikidata FROM ids WHERE doi=? LIMIT 1",
            [doi.lower()]).fetchone()
        if row is None:
            return dict(core_id=None, pmid=None, pmcid=None, wikidata_qid=None, arxiv_id=None, jstor_id=None)
        row = [str(cell or '') or None for cell in row]
        return dict(
            core_id=row[0],
            pmid=row[1],
            pmcid=row[2],
            wikidata_qid=row[3],
            # TODO:
            arxiv_id=None,
            jstor_id=None,
        )

    def want(self, obj):
        return True

    def parse_record(self, record):
        """
        record is a beautiful soup object
        returns a ReleaseEntity, or None

        In JALC metadata, both English and Japanese records are given for most
        fields.
        """

        extra = dict()
        extra_jalc = dict()

        titles = record.find_all("title")
        if not titles:
            return None
        title = titles[0].string.strip()
        original_title = None
        if title.endswith('.'):
            title = title[:-1]
        if len(titles) > 1:
            original_title = titles[1].string.strip()
            if original_title.endswith('.'):
                original_title = original_title[:-1]

        doi = None
        if record.doi:
            doi = record.doi.string.lower().strip()
            if doi.startswith('http://dx.doi.org/'):
                doi = doi.replace('http://dx.doi.org/', '')
            elif doi.startswith('https://dx.doi.org/'):
                doi = doi.replace('https://dx.doi.org/', '')
            elif doi.startswith('http://doi.org/'):
                doi = doi.replace('http://doi.org/', '')
            elif doi.startswith('https://doi.org/'):
                doi = doi.replace('https://doi.org/', '')
            if not (doi.startswith('10.') and '/' in doi):
                sys.stderr.write("bogus JALC DOI: {}\n".format(doi))
                doi = None
        if not doi:
            return None

        people = record.find_all("Person")
        contribs = parse_jalc_persons(people)

        for i, contrib in enumerate(contribs):
            if contrib.raw_name != "et al.":
                contrib.index = i

        release_year = None
        release_date = None
        date = record.date or None
        if date:
            date = date.string
            if len(date) is 10:
                release_date = datetime.datetime.strptime(date['completed-date'], DATE_FMT).date()
                release_year = release_date.year
                release_date = release_date.isoformat()
            elif len(date) is 4 and date.isdigit():
                release_year = int(date)

        pages = None
        if record.startingPage:
            pages = record.startingPage.string
            if record.endingPage:
                pages = "{}-{}".format(pages, record.endingPage.string)
        volume = None
        if record.volume:
            volume = record.volume.string
        issue = None
        if record.number:
            # note: number/issue transform
            issue = record.number.string

        # container
        issn = None
        issn_list = record.find_all("issn")
        if issn_list:
            # if we wanted the other ISSNs, would also need to uniq the list.
            # But we only need one to lookup ISSN-L/container
            issn = issn_list[0].string
        issnl = self.issn2issnl(issn)
        container_id = None
        if issnl:
            container_id = self.lookup_issnl(issnl)

        publisher = None
        container_name = None
        container_extra = dict()

        if record.publicationName:
            pubs = [p.string.strip() for p in record.find_all("publicationName") if p.string]
            pubs = [clean(p) for p in pubs if p]
            assert(pubs)
            if len(pubs) > 1 and pubs[0] == pubs[1]:
                pubs = [pubs[0]]
            if len(pubs) > 1 and is_cjk(pubs[0]):
                # eng/jpn ordering is not reliable
                pubs = [pubs[1], pubs[0]]
            container_name = clean(pubs[0])
            if len(pubs) > 1:
                container_extra['original_name'] = clean(pubs[1])

        if record.publisher:
            pubs = [p.string.strip() for p in record.find_all("publisher") if p.string]
            pubs = [p for p in pubs if p]
            if len(pubs) > 1 and pubs[0] == pubs[1]:
                pubs = [pubs[0]]
            if len(pubs) > 1 and is_cjk(pubs[0]):
                # ordering is not reliable
                pubs = [pubs[1], pubs[0]]
            if pubs:
                publisher = clean(pubs[0])
                if len(pubs) > 1:
                    container_extra['publisher_aliases'] = pubs[1:]

        if (container_id is None and self.create_containers and (issnl is not None)
                and container_name):
            # name, type, publisher, issnl
            # extra: issnp, issne, original_name, languages, country
            container_extra['country'] = 'jp'
            container_extra['languages'] = ['ja']
            ce = fatcat_openapi_client.ContainerEntity(
                name=container_name,
                container_type='journal',
                publisher=publisher,
                issnl=issnl,
                extra=(container_extra or None))
            ce_edit = self.create_container(ce)
            container_id = ce_edit.ident
            # short-cut future imports in same batch
            self._issnl_id_map[issnl] = container_id

        # the vast majority of works are in japanese
        # TODO: any indication when *not* in japanese?
        lang = "ja"

        # reasonable default for this collection
        release_type = "article-journal"

        # external identifiers
        extids = self.lookup_ext_ids(doi=doi)

        # extra:
        #   translation_of
        #   aliases
        #   container_name
        #   group-title
        # always put at least an empty dict here to indicate the DOI registrar
        # (informally)
        extra['jalc'] = extra_jalc

        title = clean(title)
        if not title:
            return None

        re = fatcat_openapi_client.ReleaseEntity(
            work_id=None,
            title=title,
            original_title=clean(original_title),
            release_type="article-journal",
            release_stage='published',
            release_date=release_date,
            release_year=release_year,
            ext_ids=fatcat_openapi_client.ReleaseExtIds(
                doi=doi,
                pmid=extids['pmid'],
                pmcid=extids['pmcid'],
                wikidata_qid=extids['wikidata_qid'],
                core=extids['core_id'],
                arxiv=extids['arxiv_id'],
                jstor=extids['jstor_id'],
            ),
            volume=volume,
            issue=issue,
            pages=pages,
            publisher=publisher,
            language=lang,
            #license_slug
            container_id=container_id,
            contribs=contribs,
            extra=extra,
        )
        return re

    def try_update(self, re):

        # lookup existing DOI
        existing = None
        try:
            existing = self.api.lookup_release(doi=re.ext_ids.doi)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err
            # doesn't exist, need to insert
            return True

        # eventually we'll want to support "updates", but for now just skip if
        # entity already exists
        if existing:
            self.counts['exists'] += 1
            return False

        return True

    def insert_batch(self, batch):
        self.api.create_release_auto_batch(fatcat_openapi_client.ReleaseAutoBatch(
            editgroup=fatcat_openapi_client.Editgroup(
                description=self.editgroup_description,
                extra=self.editgroup_extra),
            entity_list=batch))

    def parse_file(self, handle):
        """
        Helper for testing; can run this file stand-alone instead of using a pusher
        """

        # 1. open with beautiful soup
        soup = BeautifulSoup(handle, "xml")

        # 2. iterate over articles, call parse_article on each
        for record in soup.find_all("Description"):
            resp = self.parse_record(record)
            #print(json.dumps(resp))
            print(resp)
            #sys.exit(-1)


if __name__=='__main__':
    parser = JalcImporter(None, None)
    parser.parse_file(open(sys.argv[1]))
