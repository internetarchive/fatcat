
import sys
import json
import sqlite3
import datetime
import itertools
import subprocess
from bs4 import BeautifulSoup

import fatcat_client
from .common import EntityImporter, clean, is_cjk, DATE_FMT


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
            if not doi.startswith('10.'):
                sys.stderr.write("bogus JALC DOI: {}\n".format(doi))
                doi = None
        if not doi:
            return None

        contribs = []
        people = record.find_all("Person")
        if (people and (len(people) % 2 == 0)
                and not is_cjk(people[0].find('name').string)
                and is_cjk(people[1].find('name').string)):
            # both english and japanese names are usually included for every author
            # TODO: turns out this isn't always the case; see
            # 10.18948/shasetaikai.1990.0_601 as an example with 4 actual
            # authors, but 5 Person entries; all 4 authors in japanese, a
            # single author in both japanese in english. Ugh!
            for i in range(int(len(people)/2)):
                eng = people[i*2]
                jpn = people[i*2 + 1]
                # there isn't always an english name though? TODO
                name = eng
                if not name.find('name'):
                    name = jpn
                surname = name.find('familyName')
                if surname:
                    surname = surname.string
                given_name = name.find('givenName')
                if given_name:
                    given_name = given_name.string
                contrib = fatcat_client.ReleaseContrib(
                    raw_name=clean(name.find('name').string),
                    given_name=clean(given_name),
                    surname=clean(surname),
                    role='author',
                )
                if eng.find('name') and jpn.find('name'):
                    surname = jpn.find('familyName')
                    if surname:
                        surname = surname.string
                    given_name = jpn.find('givenName')
                    if given_name:
                        given_name = given_name.string
                    contrib.extra = {
                        'original_name': {
                            'lang': 'ja',
                            'raw_name': clean(jpn.find('name').string),
                            'given_name': clean(given_name),
                            'surname': clean(surname),
                        }}
                contribs.append(contrib)
        elif people:
            # TODO: test for this codepath?
            for eng in people:
                surname = eng.find('familyName')
                if surname:
                    surname = surname.string
                given_name = eng.find('givenName')
                if given_name:
                    given_name = given_name.string
                contrib = dict(
                    raw_name=clean(eng.find('name').string),
                    given_name=clean(given_name),
                    surname=clean(surname),
                    role='author',
                )
                contribs.append(contrib)

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
            pubs = [p.string.strip() for p in record.find_all("publicationName")]
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
            pubs = [p.string.strip() for p in record.find_all("publisher")]
            pubs = [p for p in pubs if p]
            if len(pubs) > 1 and pubs[0] == pubs[1]:
                pubs = [pubs[0]]
            if len(pubs) > 1 and is_cjk(pubs[0]):
                # ordering is not reliable
                pubs = [pubs[1], pubs[0]]
            publisher = clean(pubs[0])
            if len(pubs) > 1:
                container_extra['publisher_aliases'] = pubs[1:]

        if (container_id is None and self.create_containers and (issnl is not None)
                and container_name):
            # name, type, publisher, issnl
            # extra: issnp, issne, original_name, languages, country
            container_extra['country'] = 'jp'
            container_extra['languages'] = ['ja']
            ce = fatcat_client.ContainerEntity(
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

        re = fatcat_client.ReleaseEntity(
            work_id=None,
            title=title,
            original_title=clean(original_title),
            release_type="article-journal",
            release_stage='published',
            release_date=release_date,
            release_year=release_year,
            ext_ids=fatcat_client.ReleaseExtIds(
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
        except fatcat_client.rest.ApiException as err:
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
        self.api.create_release_auto_batch(fatcat_client.ReleaseAutoBatch(
            editgroup=fatcat_client.Editgroup(
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
