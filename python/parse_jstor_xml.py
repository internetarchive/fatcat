
import sys
import json
import datetime
from bs4 import BeautifulSoup
from bs4.element import NavigableString


class JstorXmlParser():
    """
    Converts JSTOR bulk XML metadata (eg, from their Early Journals Collection)
    """

    def __init__(self):
        pass

    def parse_file(self, handle):

        # 1. open with beautiful soup
        soup = BeautifulSoup(handle, "xml")

        # 2. iterate over articles, call parse_article on each
        for article in soup.find_all("article"):
            resp = self.parse_article(article)
            print(json.dumps(resp))
            #sys.exit(-1)

    def parse_article(self, article):

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
        container = dict(
            name=journal_title,
            publisher=publisher,
            issn=issn,   # TODO: ISSN-L lookup...
        )

        doi = article_meta.find("article-id", attr={"pub-id-type": "doi"})
        if doi:
            doi = doi.string.lower().strip()

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
                contribs.append(dict(
                    role=c['contrib-type'],   # XXX: types? mapping?
                    raw_name=name,
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

        release_type = "article-journal"
        if "[Abstract]" in title:
            release_type = "abstract"
        elif "[Editorial" in title:
            release_type = "editorial"
        elif "[Letter" in title:
            release_type = "letter"
        elif "[Poem" in title or "[Photograph" in title:
            release_type = None
        else if title.startswith("[") and title.endswith("]"):
            # strip brackets if that is all that is there (eg, translation or non-english)
            title = title[1:-1]

        # everything in JSTOR is published
        release_status = "published"

        if extra_jstor:
            extra['jstor'] = extra_jstor
        if not extra:
            extra = None

        re = dict(
            issn=issn, # not an entity field
            #work_id
            title=title,
            #original_title
            release_type=release_type,
            release_status=release_status,
            release_date=release_date.isoformat(),
            release_year=release_year,
            doi=doi,
            #pmid
            #pmcid
            #isbn13     # TODO: ?
            volume=volume,
            issue=issue,
            pages=pages,
            publisher=publisher,
            language=language,
            #license_slug   # TODO: ?

            # content, mimetype, lang
            #abstracts=abstracts,

            # raw_name, role, raw_affiliation, extra
            contribs=contribs,

            # key, year, container_name, title, locator
            # extra: volume, authors, issue, publisher, identifiers
            #refs=refs,

            #   name, type, publisher, issnl
            #   extra: issnp, issne, original_name, languages, country
            container=container,

            # extra:
            #   withdrawn_date
            #   translation_of
            #   subtitle
            #   aliases
            #   container_name
            #   group-title
            #   pubmed: retraction refs
            extra=extra,
        )
        return re

if __name__=='__main__':
    parser = JstorXmlParser()
    parser.parse_file(open(sys.argv[1]))
