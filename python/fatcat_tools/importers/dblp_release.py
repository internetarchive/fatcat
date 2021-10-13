
"""
Importer for DBLP release-level (article/paper/etc) XML metadata.

Works similarly to PubMed XML importer: expects to have a large XML file
iterated over quickly, with individual elements re-parsed into smaller objects
and passed to `parse_record()`.

There are two valuable pieces of relationship metadata in dblp:

- container linkages, especially to conferences which do not have ISSNs, and
  thus no existing fatcat containers
- author disambiguation, which is a work in progress but anecdotally higher
  quality than MAG, Semantic Scholar, etc

We are not going to do author (creator) ingest at this time. For containers,
import would be made much easier if we updated the database schema to include
dblp_prefix as a lookup key, but this is more difficult for containers than
with releases, so we are going to skip it for now. This leaves us with a
brittle/unreliable TSV lookup mechanism for prefix-to-container_id (as of
December 2020).
"""

import sys  # noqa: F401
import json
import warnings
import datetime
from typing import List, Optional, Any

import fatcat_openapi_client

from fatcat_tools.normal import (clean_doi, clean_str, parse_month,
    clean_orcid,
    clean_arxiv_id, clean_wikidata_qid, clean_isbn13)
from fatcat_tools.importers.common import EntityImporter
from fatcat_tools.transforms import entity_to_dict


class DblpReleaseImporter(EntityImporter):

    def __init__(self,
                 api,
                 dblp_container_map_file=None,
                 **kwargs):

        eg_desc = kwargs.get(
            'editgroup_description',
            "Automated import of dblp metadata via XML records"
        )
        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent',
                                         'fatcat_tools.DblpReleaseImporter')
        # ensure default is to not do updates with this worker (override super() default)
        kwargs['do_updates'] = kwargs.get("do_updates", False)
        super().__init__(api,
                         editgroup_description=eg_desc,
                         editgroup_extra=eg_extra,
                         **kwargs)

        self.dump_json_mode = kwargs.get("dump_json_mode", False)
        self.this_year = datetime.datetime.now().year
        self.read_dblp_container_map_file(dblp_container_map_file)

    ELEMENT_TYPES = [
        "article",
        "inproceedings",
        "book",
        "incollection",
        "phdthesis",
        "mastersthesis",
        "www",
        #"data",  # no instances in 2020-11 dump
    ]

    def read_dblp_container_map_file(self, dblp_container_map_file) -> None:
        self._dblp_container_map = dict()
        if not dblp_container_map_file:
            print("Not loading a dblp prefix container map file; entities will fail to import", file=sys.stderr)
            return
        print("Loading dblp prefix container map file...", file=sys.stderr)
        for line in dblp_container_map_file:
            if line.startswith("dblp_prefix") or len(line) == 0:
                continue
            (prefix, container_id) = line.split()[0:2]
            container_id = container_id.strip()
            assert len(container_id) == 26
            self._dblp_container_map[prefix] = container_id
        print("Got {} dblp container mappings.".format(len(self._dblp_container_map)), file=sys.stderr)

    def lookup_dblp_prefix(self, prefix):
        if not prefix:
            return None
        return self._dblp_container_map.get(prefix)

    def want(self, xml_elem):
        if not xml_elem.name in self.ELEMENT_TYPES:
            self.counts['skip-type'] += 1
            return False
        if not xml_elem.get('key'):
            self.counts['skip-no-key'] += 1
            return False
        if xml_elem['key'].startswith('homepage/'):
            self.counts['skip-type-homepage'] += 1
            return False
        return True

    def parse_record(self, xml_elem):
        """
        - title
            => may contain <i>, <sub>, <sup>, <tt>
        - journal (abbrev?)
        - volume, pages, number (number -> issue)
        - publisher
        - year
            => for conferences, year of conference not of publication
        - month
        - crossref (from inproceedings to specific proceedings volume)
        - booktitle
            => for inproceedings, this is the name of conference or workshop. acronym.
        - isbn
        """

        dblp_key = xml_elem.get('key')
        if not dblp_key:
            self.counts['skip-empty-key'] += 1
            return False
        dblp_key_type = dblp_key.split('/')[0]

        # dblp_prefix may be used for container lookup
        dblp_prefix = None
        if dblp_key_type in ('journals', 'conf'):
            dblp_prefix = '/'.join(dblp_key.split('/')[:2])
        elif dblp_key_type in ('series', 'reference', 'tr', 'books'):
            dblp_prefix = '/'.join(dblp_key.split('/')[:-1])

        publtype = xml_elem.get('publtype') or None

        dblp_type = xml_elem.name
        if dblp_type not in self.ELEMENT_TYPES:
            self.counts[f'skip-dblp-type:{dblp_type}'] += 1

        if dblp_key_type in ('homepages', 'persons', 'dblpnote'):
            self.counts['skip-key-type'] += 1
            return False

        if dblp_key.startswith('journals/corr/'):
            self.counts['skip-arxiv-corr'] += 1
            return False

        title = clean_str(" ".join(xml_elem.title.stripped_strings), force_xml=True)
        if not title:
            self.counts['skip-title'] += 1
            return False
        if title.endswith('.'):
            title = title[:-1]

        release_type = None
        release_stage = 'published'
        withdrawn_status = None

        # primary releae_type detection: type of XML element, then prefix of key for granularity
        if dblp_type == 'article':
            release_type = 'article'
            if dblp_key_type == 'journals' and publtype != 'informal':
                release_type = 'article-journal'
            elif dblp_key_type == 'tr':
                release_type = 'report'
            elif title.startswith("Review:"):
                release_type = 'review'
        elif dblp_type == 'inproceedings':
            release_type = 'paper-conference'
        elif dblp_type == 'book':
            release_type = 'book'
        elif dblp_type == 'incollection':
            # XXX: part vs. chapter?
            release_type = 'chapter'
        elif dblp_type == 'data':
            release_type = 'dataset'
        elif dblp_type in ('mastersthesis', 'phdthesis'):
            release_type = 'thesis'

        # overrides/extensions of the above
        if publtype == 'informal':
            # for conferences, seems to indicate peer-review status
            # for journals, seems to indicate things like book reviews; split out above
            pass
        elif publtype == 'encyclopedia':
            release_type = 'entry-encyclopedia'
        elif publtype == 'edited':
            # XXX: article?
            release_type = 'editorial'
        elif publtype == 'data':
            release_type = 'dataset'
        elif publtype == 'data':
            release_type = 'dataset'
        elif publtype == 'software':
            release_type = 'software'
        elif publtype == 'widthdrawn':
            withdrawn_status = 'widthdrawn'
        elif publtype == 'survey':
            # XXX: flag as a review/survey article?
            pass

        #print((release_type, dblp_type, dblp_key_type, publtype), file=sys.stderr)

        container_name = None
        booktitle = clean_str(xml_elem.booktitle and xml_elem.booktitle.text)
        series = clean_str(xml_elem.series and xml_elem.series.text)

        if xml_elem.journal:
            container_name = clean_str(xml_elem.journal.text)

        container_id = None
        if dblp_prefix:
            container_id = self.lookup_dblp_prefix(dblp_prefix)
            # note: we will skip later if couldn't find prefix

        publisher = clean_str(xml_elem.publisher and xml_elem.publisher.text)
        volume = clean_str(xml_elem.volume and xml_elem.volume.text)
        issue = clean_str(xml_elem.number and xml_elem.number.text)
        pages = clean_str(xml_elem.pages and xml_elem.pages.text)
        release_year = clean_str(xml_elem.year and xml_elem.year.text)
        if release_year and release_year.isdigit():
            release_year = int(release_year)
        else:
            release_year = None
        release_month = parse_month(clean_str(xml_elem.month and xml_elem.month.text))
        isbn = clean_isbn13(xml_elem.isbn and xml_elem.isbn.text)
        part_of_key = clean_str(xml_elem.crossref and xml_elem.crossref.text)

        # block bogus far-future years/dates
        if release_year is not None and (release_year > (self.this_year + 5) or release_year < 1000):
            release_month = None
            release_year = None

        contribs = self.dblp_contribs(xml_elem or [])
        ext_ids = self.dblp_ext_ids(xml_elem, dblp_key)
        if isbn:
            ext_ids.isbn13 = isbn
        if ext_ids.doi:
            self.counts['has-doi'] += 1

        # dblp-specific extra
        dblp_extra = dict(type=dblp_type)
        note = clean_str(xml_elem.note and xml_elem.note.text)
        if note and not 'base-search.net' in note:
            dblp_extra['note'] = note
        if part_of_key:
            dblp_extra['part_of_key'] = part_of_key

        # generic extra
        extra = dict()
        if not container_id and container_name:
            extra['container_name'] = container_name

        if series and (dblp_key_type == 'series' or dblp_type == 'book'):
            extra['series-title'] = series
        elif series:
            dblp_extra['series'] = series

        if booktitle and dblp_key_type == 'series':
            extra['container-title'] = booktitle
        elif booktitle and dblp_key_type == 'conf':
            extra['event'] = booktitle
        elif booktitle:
            dblp_extra['booktitle'] = booktitle

        if release_year and release_month:
            # TODO: release_month schema migration
            extra['release_month'] = release_month

        if dblp_extra:
            extra['dblp'] = dblp_extra
        if not extra:
            extra = None

        re = fatcat_openapi_client.ReleaseEntity(
            work_id=None,
            container_id=container_id,
            release_type=release_type,
            release_stage=release_stage,
            withdrawn_status=withdrawn_status,
            title=title,
            release_year=release_year,
            #release_date,
            publisher=publisher,
            ext_ids=ext_ids,
            contribs=contribs,
            volume=volume,
            issue=issue,
            pages=pages,
            extra=extra,
        )
        re = self.biblio_hacks(re)

        if self.dump_json_mode:
            re_dict = entity_to_dict(re, api_client=self.api.api_client)
            re_dict['_dblp_ee_urls'] = self.dblp_ext_urls(xml_elem)
            re_dict['_dblp_prefix'] = dblp_prefix
            print(json.dumps(re_dict, sort_keys=True))
            return False

        if not re.container_id:
            self.counts["skip-dblp-container-missing"] += 1
            return False
        return re

    @staticmethod
    def biblio_hacks(re):
        """
        This function handles known special cases. For example,
        publisher-specific or platform-specific workarounds.
        """
        return re

    def try_update(self, re):

        # lookup existing release by dblp article id
        existing = None
        try:
            existing = self.api.lookup_release(dblp=re.ext_ids.dblp)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        # Just skip all releases with an arxiv_id for now. Have not decided
        # what to do about grouping works and lookup of un-versioned arxiv_id
        # yet. Note that this means we will lack coverage of some works which
        # have an arxiv preprint, but in those cases we will presumably at
        # least have the pre-print copy/record.
        if re.ext_ids.arxiv:
            self.counts["skip-arxiv"] += 1
            return False

        # then try other ext_id lookups
        if not existing:
            for extid_type in ('doi', 'wikidata_qid', 'isbn13', 'arxiv'):
                extid_val = getattr(re.ext_ids, extid_type)
                if not extid_val:
                    continue
                #print(f"  lookup release type: {extid_type} val: {extid_val}")
                try:
                    existing = self.api.lookup_release(**{extid_type: extid_val})
                except fatcat_openapi_client.rest.ApiException as err:
                    if err.status != 404:
                        raise err
                if existing:
                    if existing.ext_ids.dblp:
                        warn_str = f"unexpected dblp ext_id match after lookup failed dblp={re.ext_ids.dblp} ident={existing.ident}"
                        warnings.warn(warn_str)
                        self.counts["skip-dblp-id-mismatch"] += 1
                        return False
                    break

        if not existing and self.do_fuzzy_match:
            fuzzy_result = self.match_existing_release_fuzzy(re)
            # TODO: in the future, could assign work_id for clustering, or for
            # "EXACT" match, set existing and allow (optional) update code path
            # to run
            if fuzzy_result is not None:
                self.counts["exists-fuzzy"] += 1
                return False

        # if no existing, then create entity
        if not existing:
            return True

        if not self.do_updates or existing.ext_ids.dblp:
            self.counts['exists'] += 1
            return False

        # logic for whether to do update or skip
        if (existing.container_id and existing.release_type and existing.release_stage) or existing.ext_ids.arxiv:
            self.counts['skip-update'] += 1
            return False

        # fields to copy over for update
        # TODO: granular contrib metadata
        existing.contribs = existing.contribs or re.contribs
        existing.ext_ids.dblp = existing.ext_ids.dblp or re.ext_ids.dblp
        existing.ext_ids.wikidata_qid = existing.ext_ids.wikidata_qid or re.ext_ids.wikidata_qid
        existing.release_type = existing.release_type or re.release_type
        existing.release_stage = existing.release_stage or re.release_stage
        existing.withdrawn_status = existing.withdrawn_status or re.withdrawn_status
        existing.container_id = existing.container_id or re.container_id
        existing.extra['dblp'] = re.extra['dblp']
        existing.volume = existing.volume or re.volume
        existing.issue = existing.issue or re.issue
        existing.pages = existing.pages or re.pages

        try:
            self.api.update_release(self.get_editgroup_id(), existing.ident, existing)
            self.counts['update'] += 1
        except fatcat_openapi_client.rest.ApiException as err:
            # there is a code path where we try to update the same release
            # twice in a row; if that happens, just skip
            # NOTE: API behavior might change in the future?
            if "release_edit_editgroup_id_ident_id_key" in err.body:
                self.counts['skip-update-conflict'] += 1
                return False
            else:
                raise err

        return False

    def insert_batch(self, batch):
        self.api.create_release_auto_batch(fatcat_openapi_client.ReleaseAutoBatch(
            editgroup=fatcat_openapi_client.Editgroup(
                description=self.editgroup_description,
                extra=self.editgroup_extra),
            entity_list=batch))

    def dblp_contribs(self, authors: List[dict]) -> List[fatcat_openapi_client.ReleaseContrib]:
        """
        - author (multiple; each a single string)
            => may have HTML entities
            => may have a number at the end, to aid with identifier creation
            => orcid
        - editor (same as author)
            => orcid
        """
        contribs = []
        index = 0
        for elem in authors.find_all('author'):
            contrib = self.dblp_contrib_single(elem)
            contrib.role = "author"
            contrib.index = index
            contribs.append(contrib)
            index += 1

        for elem in authors.find_all('editor'):
            contrib = self.dblp_contrib_single(elem)
            contrib.role = "editor"
            contribs.append(contrib)

        return contribs

    def dblp_contrib_single(self, elem: Any) -> fatcat_openapi_client.ReleaseContrib:
        """
        In the future, might try to implement creator key-ificiation and lookup here.

        Example rows:

            <author>Michael H. B&ouml;hlen</author>
            <author orcid="0000-0002-4354-9138">Nicolas Heist</author>
            <author orcid="0000-0001-9108-4278">Jens Lehmann 0001</author>
        """

        creator_id = None
        extra = None
        raw_name = clean_str(elem.text)

        # remove number in author name, if present
        if raw_name.split()[-1].isdigit():
            raw_name = ' '.join(raw_name.split()[:-1])

        if elem.get('orcid'):
            orcid = clean_orcid(elem['orcid'])
            if orcid:
                creator_id = self.lookup_orcid(orcid)
                if not creator_id:
                    extra = dict(orcid=orcid)
        return fatcat_openapi_client.ReleaseContrib(
            raw_name=raw_name,
            creator_id=creator_id,
            extra=extra,
        )

    def dblp_ext_ids(self, xml_elem: Any, dblp_key: str) -> fatcat_openapi_client.ReleaseExtIds:
        """
        Takes a full XML object and returns external identifiers.

        Currently these can be arixv identifiers, DOI, or wikidata QID

        - ee (electronic edition; often DOI?)
            => in some cases a "local" URL
            => publisher URL; often DOI
            => type attr
        - url
            => dblp internal link to table-of-contents
        """

        doi: Optional[str] = None
        wikidata_qid: Optional[str] = None
        arxiv_id: Optional[str] = None
        for ee in xml_elem.find_all('ee'):
            url = ee.text
            # convert DOI-like domains, which mostly have DOIs anyways
            if '://doi.acm.org/' in url:
                url = url.replace('://doi.acm.org/', '://doi.org/')
            elif '://doi.ieeecomputersociety.org/' in url:
                url = url.replace('://doi.ieeecomputersociety.org/', '://doi.org/')

            if 'doi.org/10.' in url and not doi:
                doi = clean_doi(url)
            elif 'wikidata.org/entity/Q' in url and not wikidata_qid:
                wikidata_qid = clean_wikidata_qid(url)
            elif '://arxiv.org/abs/' in url and not arxiv_id:
                arxiv_id = url.replace('http://', '').replace('https://', '').replace('arxiv.org/abs/', '')
                arxiv_id = clean_arxiv_id(arxiv_id)

        return fatcat_openapi_client.ReleaseExtIds(
            dblp=dblp_key,
            doi=doi,
            wikidata_qid=wikidata_qid,
            arxiv=arxiv_id,
        )

    def dblp_ext_urls(self, xml_elem: Any) -> List[str]:
        """
        Takes a full XML object and returns a list of possible-fulltext URLs.

        Used only in JSON dump mode, with the intent of transforming into
        sandcrawler ingest requests.
        """
        EXTID_PATTERNS = [
            '://doi.acm.org/',
            '://doi.ieeecomputersociety.org/',
            'doi.org/10.',
            'wikidata.org/entity/Q',
            '://arxiv.org/abs/',
        ]
        urls = []
        for ee in xml_elem.find_all('ee'):
            url = ee.text
            skip = False
            for pattern in EXTID_PATTERNS:
                if pattern in url:
                    skip = True
                    break
            if skip:
                break
            urls.append(url)
        return urls
