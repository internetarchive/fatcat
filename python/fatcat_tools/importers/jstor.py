
import sys
import json
import sqlite3
import datetime
import itertools
import subprocess
from bs4 import BeautifulSoup

import fatcat_client
from .common import EntityImporter, clean

# is this just ISO 3-char to ISO 2-char?
# XXX: more entries
JSTOR_LANG_MAP = {
    'eng': 'en',
}

# XXX: more entries
JSTOR_CONTRIB_MAP = {
    'author': 'author',
    'editor': 'editor',
    'translator': 'translator',
    'illustrator': 'illustrator',
}

class JstorImporter(EntityImporter):
    """
    Importer for JSTOR bulk XML metadata (eg, from their Early Journals
    Collection)
    """

    def __init__(self, api, issn_map_file, **kwargs):

        eg_desc = kwargs.get('editgroup_description',
            "Automated import of JSTOR XML metadata")
        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.JstorImporter')
        super().__init__(api,
            issn_map_file=issn_map_file,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)

        self.create_containers = kwargs.get('create_containers')

        self.read_issn_map_file(issn_map_file)

    def want(self, obj):
        return True

    def parse_record(self, article):

        journal_meta = article.front.find("journal-meta")
        article_meta = article.front.find("article-meta")

        extra = dict()
        extra_jstor = dict()

        journal_title = journal_meta.find("journal-title").string
        publisher = journal_meta.find("publisher-name").string
        issn = journal_meta.find("issn")
        if issn:
            issn = issn.string
            if len(issn) == 8:
                issn = "{}-{}".format(issn[0:4], issn[4:8])
            else:
                assert len(issn) == 9
        # XXX:
        container_id = None
        container = dict(
            name=journal_title,
            publisher=publisher,
            issn=issn,   # TODO: ISSN-L lookup...
        )

        doi = article_meta.find("article-id", {"pub-id-type": "doi"})
        if doi:
            doi = doi.string.lower().strip()

        jstor_id = article_meta.find("article-id", {"pub-id-type": "jstor"})
        if jstor_id:
            jstor_id = jstor_id.string

        title = article_meta.find("article-title")
        if title:
            title = title.string.strip()
            if title.endswith('.'):
                title = title[:-1]

        contribs = []
        cgroup = article_meta.find("contrib-group")
        if cgroup:
            for c in cgroup.find_all("contrib"):
                given = c.find("given-names")
                surname = c.find("surname")
                if given and surname:
                    name = "{} {}".format(given.string, surname.string)
                elif surname:
                    name = surname.string
                else:
                    name = None
                contribs.append(fatcat_client.ReleaseContrib(
                    role=JSTOR_CONTRIB_MAP.get(c['contrib-type']),
                    raw_name=clean(name),
                    given_name=clean(given.string),
                    surname=clean(surname.string),
                ))

        release_year = None
        release_date = None
        pub_date = article_meta.find('pub-date')
        if pub_date and pub_date.year:
            release_year = int(pub_date.year.string)
            if pub_date.month and pub_date.day:
                release_date = datetime.date(
                    release_year,
                    int(pub_date.month.string),
                    int(pub_date.day.string))
        
        volume = None
        if article_meta.volume:
            volume = article_meta.volume.string or None

        issue = None
        if article_meta.issue:
            issue = article_meta.issue.string or None

        pages = None
        if article_meta.find("page-range"):
            pages = article_meta.find("page-range").string
        elif article_meta.fpage:
            pages = article_meta.fpage.string

        language = None
        cm = article_meta.find("custom-meta")
        if cm.find("meta-name").string == "lang":
            language = cm.find("meta-value").string
            language = JSTOR_LANG_MAP.get(language)

        release_type = "article-journal"
        if "[Abstract]" in title:
            # TODO: strip the "[Abstract]" bit?
            release_type = "abstract"
        elif "[Editorial" in title:
            release_type = "editorial"
        elif "[Letter" in title:
            release_type = "letter"
        elif "[Poem" in title or "[Photograph" in title:
            release_type = None

        if title.startswith("[") and title.endswith("]"):
            # strip brackets if that is all that is there (eg, translation or non-english)
            title = title[1:-1]

        # JSTOR issue-id
        if article_meta.find('issue-id'):
            issue_id = clean(article_meta.find('issue-id').string)
            if issue_id:
                extra_jstor['issue_id'] = issue_id

        # JSTOR journal-id
        # XXX:

        # everything in JSTOR is published
        release_stage = "published"

        # extra:
        #   withdrawn_date
        #   translation_of
        #   subtitle
        #   aliases
        #   container_name
        #   group-title
        #   pubmed: retraction refs
        if extra_jstor:
            extra['jstor'] = extra_jstor
        if not extra:
            extra = None

        re = fatcat_client.ReleaseEntity(
            #work_id
            title=title,
            #original_title
            release_type=release_type,
            release_stage=release_stage,
            release_date=release_date.isoformat(),
            release_year=release_year,
            ext_ids=fatcat_client.ReleaseExtIds(
                doi=doi,
                jstor=jstor_id,
            ),
            volume=volume,
            issue=issue,
            pages=pages,
            publisher=publisher,
            language=language,
            #license_slug

            # content, mimetype, lang
            #abstracts=abstracts,

            contribs=contribs,

            # key, year, container_name, title, locator
            # extra: volume, authors, issue, publisher, identifiers
            #refs=refs,

            #   name, type, publisher, issnl
            #   extra: issnp, issne, original_name, languages, country
            container_id=container_id,

            extra=extra,
        )
        return re

    def try_update(self, re):

        # first, lookup existing by JSTOR id (which much be defined)
        existing = None
        try:
            existing = self.api.lookup_release(jstor=re.ext_ids.jstor)
        except fatcat_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        # then try DOI lookup if there is one
        if not existing and re.ext_ids.doi:
            try:
                existing = self.api.lookup_release(doi=re.ext_ids.doi)
            except fatcat_client.rest.ApiException as err:
                if err.status != 404:
                    raise err

        if existing and existing.ext_ids.jstor:
            # don't update if it already has JSTOR ID
            self.counts['exists'] += 1
            return False
        elif existing:
            # but do update if only DOI was set
            existing.ext_ids.jstor = re.jstor_id
            existing.extra['jstor'] = re.extra['jstor']
            self.api.update_release(self.get_editgroup_id(), existing.ident, existing)
            self.counts['update'] += 1
            return False

        return True

    def insert_batch(self, batch):
        self.api.create_release_auto_batch(fatcat_client.ReleaseAutoBatch(
            editgroup=fatcat_client.Editgroup(
                description=self.editgroup_description,
                extra=self.editgroup_extra),
            entity_list=batch))

    def parse_file(self, handle):

        # 1. open with beautiful soup
        soup = BeautifulSoup(handle, "xml")

        # 2. iterate over articles, call parse_article on each
        for article in soup.find_all("article"):
            resp = self.parse_record(article)
            print(json.dumps(resp))
            #sys.exit(-1)

if __name__=='__main__':
    parser = JstorImporter()
    parser.parse_file(open(sys.argv[1]))
