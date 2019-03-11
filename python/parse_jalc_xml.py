
import sys
import json
import datetime
import unicodedata
from bs4 import BeautifulSoup
from bs4.element import NavigableString


DATE_FMT = "%Y-%m-%d"

def is_cjk(s):
    if not s:
        return False
    return unicodedata.name(s[0]).startswith("CJK")

class JalcXmlParser():
    """
    Converts JALC DOI metadata (in XML/RDF format) to fatcat release entity

    NOTE: some JALC DOIs seem to get cross-registered with Crossref
    """

    def __init__(self):
        pass

    def parse_file(self, handle):

        # 1. open with beautiful soup
        soup = BeautifulSoup(handle, "xml")

        # 2. iterate over articles, call parse_article on each
        for record in soup.find_all("Description"):
            resp = self.parse_record(record)
            print(json.dumps(resp))
            #sys.exit(-1)


    def parse_record(self, record):
        """
        In JALC metadata, both English and Japanese records are given for most
        fields.
        """

        #extra = dict()
        #extra_jalc = dict()

        titles = record.find_all("title")
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
            assert doi.startswith('10.')

        contribs = []
        people = record.find_all("Person")
        if people and (len(people) % 2 == 0) and is_cjk(people[1].find('name').string):
            # both english and japanese names are included
            for i in range(int(len(people)/2)):
                # both english and japanese names are included for every author
                eng = people[i*2]
                jpn = people[i*2 + 1]
                raw_name = eng.find('name')
                orig_name = jpn.find('name')
                if not raw_name:
                    raw_name = orig_name
                contrib = dict(
                    raw_name=raw_name.string,
                    role='author',
                )
                if raw_name and orig_name:
                    contrib['extra'] = dict(original_name=orig_name.string)
                contribs.append(contrib)
        elif people:
            for eng in people:
                raw_name = eng.find('name')
                contrib = dict(
                    raw_name=eng.find('name').string,
                    role='author',
                )
                contribs.append(contrib)

        release_year = None
        release_date = None
        date = record.date or None
        if date:
            date = date.string
            if len(date) is 10:
                release_date = datetime.datetime.strptime(state['completed-date'], DATE_FMT).date()
                release_year = release_date.year
                release_date = release_date.isoformat()
            elif len(date) is 4:
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

        issn = None
        issn_list = record.find_all("issn")
        if issn_list:
            # if we wanted the other ISSNs, would also need to uniq the list.
            # But we only need one to lookup ISSN-L/container
            issn = issn_list[0].string

        container = dict()
        container_extra = dict()
        container_name = None
        if record.publicationName:
            pubs = [p.string.strip() for p in record.find_all("publicationName")]
            pubs = [p for p in pubs if p]
            assert(pubs)
            if len(pubs) > 1 and pubs[0] == pubs[1]:
                pubs = [pubs[0]]
            elif len(pubs) > 1 and is_cjk(pubs[0]):
                # ordering is not reliable
                pubs = [pubs[1], pubs[0]]
            container_name = pubs[0]
            container['name'] = container_name
            if len(pubs) > 1:
                orig_container_name = pubs[1]
                container_extra['original_name'] = pubs[1]
        publisher = None
        if record.publisher:
            pubs = [p.string.strip() for p in record.find_all("publisher")]
            pubs = [p for p in pubs if p]
            if len(pubs) > 1 and pubs[0] == pubs[1]:
                pubs = [pubs[0]]
            elif len(pubs) > 1 and is_cjk(pubs[0]):
                # ordering is not reliable
                pubs = [pubs[1], pubs[0]]
            publisher = pubs[0]
            container['publisher'] = publisher
            if len(pubs) > 1:
                container_extra['publisher_alt_name'] = pubs[1]
        if container_extra:
            container['extra'] = container_extra
        if not container:
            container = None

        # the vast majority of works are in japanese
        # TODO: any indication when *not* in japanese?
        lang = "ja"

        # reasonable default for this collection
        release_type = "article-journal"

        re = dict(
            work_id=None,
            title=title,
            original_title=original_title,
            release_type="article-journal",
            release_status='submitted', # XXX: source_type?
            release_date=release_date,
            release_year=release_year,
            #arxiv_id
            doi=doi,
            #pmid
            #pmcid
            #isbn13     # never in Article
            volume=volume,
            issue=issue,
            pages=pages,
            publisher=publisher,
            language=lang,
            #license_slug   # not in MEDLINE

            # content, mimetype, lang
            #abstracts=abstracts,

            # raw_name, role, raw_affiliation, extra
            contribs=contribs,

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
            #extra=extra,
        )
        return re

if __name__=='__main__':
    parser = JalcXmlParser()
    parser.parse_file(open(sys.argv[1]))
