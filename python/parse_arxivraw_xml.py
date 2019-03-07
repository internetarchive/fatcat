
import sys
import json
import datetime
from bs4 import BeautifulSoup
from bs4.element import NavigableString
from pylatexenc.latex2text import LatexNodes2Text


latex2text = LatexNodes2Text()

def parse_arxiv_authors(raw):
    if not raw:
        return []
    authors = raw.split(', ')
    if authors:
        last = authors[-1].split(" and ")
        if len(last) == 2:
            authors[-1] = last[0]
            authors.append(last[1])
    authors = [latex2text.latex_to_text(a).strip() for a in authors]
    return authors

def test_parse_arxiv_authors():

    assert parse_arxiv_authors("Raphael Chetrite, Shamik Gupta, Izaak Neri and \\'Edgar Rold\\'an") == [
        "Raphael Chetrite",
        "Shamik Gupta",
        "Izaak Neri",
        "Édgar Roldán",
    ]
    assert parse_arxiv_authors("Izaak Neri and \\'Edgar Rold\\'an") == [
        "Izaak Neri",
        "Édgar Roldán",
    ]
    assert parse_arxiv_authors("Raphael Chetrite Shamik Gupta") == [
        "Raphael Chetrite Shamik Gupta",
    ]

class ArxivRawXmlParser():
    """
    Converts arxiv.org "arXivRaw" OAI-PMH XML records to fatcat release entities

    TODO: this will require a special importer that keeps works together
    TODO: arxiv_id lookup in API (rust) with no version specified should select
          the "most recent" version; can be a simple sort?
    """

    def __init__(self):
        pass

    def parse_file(self, handle):

        # 1. open with beautiful soup
        soup = BeautifulSoup(handle, "xml")

        # 2. iterate over articles, call parse_article on each
        for article in soup.find_all("record"):
            resp = self.parse_record(article)
            print(json.dumps(resp))
            #sys.exit(-1)


    def parse_record(self, record):

        metadata = record.arXivRaw
        extra = dict()
        extra_arxiv = dict()

        base_id = metadata.id.string
        doi = None
        if metadata.doi and metadata.doi.string:
            doi = metadata.doi.string.lower().strip()
            assert doi.startswith('10.')
        title = latex2text.latex_to_text(metadata.title.string)
        authors = parse_arxiv_authors(metadata.authors.string)
        contribs = [dict(raw_name=a, role='author') for a in authors]

        lang = "en"     # the vast majority in english
        if metadata.comments and metadata.comments.string:
            comments = metadata.comments.string.strip()
            extra_arxiv['comments'] = comments
            if 'in french' in comments.lower():
                lang = 'fr'
            elif 'in spanish' in comments.lower():
                lang = 'es'
            elif 'in portuguese' in comments.lower():
                lang = 'pt'
            elif 'in hindi' in comments.lower():
                lang = 'hi'
            elif 'in japanese' in comments.lower():
                lang = 'ja'
            elif 'in german' in comments.lower():
                lang = 'de'
            elif 'simplified chinese' in comments.lower():
                lang = 'zh'
            elif 'in russian' in comments.lower():
                lang = 'ru'
            # more languages?

        release_type = "article-journal"

        if metadata.find('journal-ref') and metadata.find('journal-ref').string:
            journal_ref = metadata.find('journal-ref').string.strip()
            extra_arxiv['journal_ref'] = journal_ref
            if "conf." in journal_ref.lower() or "proc." in journal_ref.lower():
                release_type = "conference-paper"
        if metadata.find('report-no') and metadata.find('report-no').string:
            extra['number'] = metadata.find('report-no').string.strip()
            release_type = "report"
        if metadata.find('acm-class') and metadata.find('acm-class').string:
            extra_arxiv['acm_class'] = metadata.find('acm_class').string.strip()
        if metadata.categories and metadata.categories.string:
            extra_arxiv['categories'] = metadata.categories.string.split()
        license_slug = None
        if metadata.license and metadata.license.string:
            # XXX: convert URL to slug
            license_slug = metadata.license.string.strip()
        abstracts = None
        if metadata.abstract:
            # TODO: test for this multi-abstract code path
            abstracts = []
            abst = metadata.abstract.string.strip()
            orig = None
            if '-----' in abst:
                both = abst.split('-----')
                abst = both[0].strip()
                orig = both[1].strip()
            if '$' in abst or '{' in abstr:
                mime = "application/x-latex"
                abst_plain = latex2text.latex_to_text(abst)
                abstracts.append(dict(content=abst_plain, mime="text/plain", lang="en"))
            else:
                mime = "text/plain"
            abstracts.append(dict(content=abst, mime=mime, lang="en"))
            if orig:
                abstracts.append(dict(content=orig, mime=mime))

        if extra_arxiv:
            extra['arxiv'] = extra_arxiv
        if not extra:
            extra = None

        versions = []
        for version in metadata.find_all('version'):
            arxiv_id = base_id + version['version']
            release_date = version.date.string.strip()
            release_date = datetime.datetime.strptime(release_date, "%a, %d %b %Y %H:%M:%S %Z").date()
            versions.append(dict(
                work_id=None,
                title=title,
                #original_title
                release_type="article-journal",
                release_status='submitted', # XXX: source_type?
                release_date=release_date.isoformat(),
                release_year=release_date.year,
                arxiv_id=arxiv_id,
                #doi (see below)
                #pmid
                #pmcid
                #isbn13     # never in Article
                #volume
                #issue
                #pages
                #publisher
                language=lang,
                #license_slug   # not in MEDLINE

                # content, mimetype, lang
                abstracts=abstracts,

                # raw_name, role, raw_affiliation, extra
                contribs=contribs,

                #   name, type, publisher, issnl
                #   extra: issnp, issne, original_name, languages, country
                #container=container,   # very little/none; resolve via DOI?

                # extra:
                #   withdrawn_date
                #   translation_of
                #   subtitle
                #   aliases
                #   container_name
                #   group-title
                #   pubmed: retraction refs
                extra=extra,
            ))

        # only apply DOI to most recent version (HACK)
        if doi:
            versions[-1]['doi'] = doi
            versions[-1]['release_status'] = "published"
        return base_id, versions

if __name__=='__main__':
    parser = ArxivRawXmlParser()
    parser.parse_file(open(sys.argv[1]))
