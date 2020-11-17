"""
Importer for DOAJ article-level metadata, schema v1.

DOAJ API schema and docs: https://doaj.org/api/v1/docs
"""

import collections
import datetime
import sys
from typing import List, Dict, Optional

import langdetect

import fatcat_openapi_client
from fatcat_tools.normal import clean_doi
from fatcat_tools.transforms import entity_to_dict
from fatcat_tools.importers.common import EntityImporter, clean

# Cutoff length for abstracts.
MAX_ABSTRACT_LENGTH = 2048


class DoajArticleImporter(EntityImporter):

    def __init__(self,
                 api,
                 issn_map_file,
                 debug=False,
                 insert_log_file=None,
                 **kwargs):

        eg_desc = kwargs.get(
            'editgroup_description',
            "Automated import of DOAJ article metadata, harvested from REST API or bulk dumps"
        )
        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent',
                                         'fatcat_tools.DoajArticleImporter')
        super().__init__(api,
                         issn_map_file=issn_map_file,
                         editgroup_description=eg_desc,
                         editgroup_extra=eg_extra,
                         **kwargs)

        self.this_year = datetime.datetime.now().year
        self.read_issn_map_file(issn_map_file)

    def want(self, obj):
        return True


    def parse_record(self, obj):
        """
        bibjson {
            abstract (string, optional),
            author (Array[bibjson.author], optional),
            identifier (Array[bibjson.identifier]),
            journal (bibjson.journal, optional),
            keywords (Array[string], optional),
            link (Array[bibjson.link], optional),
            month (string, optional),
            subject (Array[bibjson.subject], optional),
            title (string),
            year (string, optional)
        }
        bibjson.journal {
            country (string, optional),
            end_page (string, optional),
            language (Array[string], optional),
            license (Array[bibjson.journal.license], optional),
            number (string, optional),
            publisher (string, optional),
            start_page (string, optional),
            title (string, optional),
            volume (string, optional)
        }

        TODO:
        - release_date
        - container_id
        - issue (number?)
        - license is article license; import as slug
        - "open_access" flag in doaj_meta
        - container lookup from issns ("issns" key)
        """

        if not obj or not isinstance(obj, dict) or not 'bibjson' in obj:
            self.counts['skip-empty'] += 1
            return None

        bibjson = obj['bibjson']

        title = clean(bibjson.get('title'))
        if not title:
            self.counts['skip-title'] += 1
            return False

        container_id = None
        container_name = None

        volume = clean(bibjson['journal'].get('volume'))
        number = clean(bibjson['journal'].get('number'))
        publisher = clean(bibjson['journal'].get('publisher'))

        try:
            release_year = int(bibjson.get('year'))
        except (TypeError, ValueError):
            release_year = None
        # XXX: parse_month
        release_month = clean(bibjson.get('year'))

        # block bogus far-future years/dates
        if release_year is not None and (release_year > (self.this_year + 5) or release_year < 1000):
            release_month = None
            release_year = None

        # country
        country = None
        # XXX: country = parse_country(bibjson['journal'].get('country'))

        # language
        language = None
        # XXX: language = parse_language(bibjson['journal'].get('language'))

        # pages
        # TODO: error in API docs? seems like start_page not under 'journal' object
        start_page = clean(bibjson['journal'].get('start_page')) or clean(bibjson.get('start_page'))
        end_page = clean(bibjson['journal'].get('end_page')) or clean(bibjson.get('end_page'))
        pages: Optional[str] = None
        if start_page and end_page:
            pages = f"{start_page}-{end_page}"
        elif start_page:
            pages = start_page

        doaj_article_id = obj['id'].lower()
        ext_ids = self.doaj_ext_ids(bibjson['identifier'], doaj_article_id)
        abstracts = self.doaj_abstracts(bibjson)
        contribs = self.doaj_contribs(bibjson.get('author') or [])
            
        # DOAJ-specific extra
        doaj_extra = dict()
        if bibjson.get('subject'):
            doaj_extra['subject'] = bibjson.get('subject')
        if bibjson.get('keywords'):
            doaj_extra['keywords'] = [k for k in [clean(s) for s in bibjson.get('keywords')] if k]

        # generic extra
        extra = dict()
        if country:
            extra['country'] = country
        if not container_id and container_name:
            extra['container_name'] = container_name
        if release_year and release_month:
            # TODO: schema migration
            extra['release_month'] = release_month

        if doaj_extra:
            extra['doaj'] = doaj_extra
        if not extra:
            extra = None

        re = fatcat_openapi_client.ReleaseEntity(
            work_id=None,
            container_id=container_id,
            release_type='article-journal',
            release_stage='published',
            title=title,
            release_year=release_year,
            #release_date,
            publisher=publisher,
            ext_ids=ext_ids,
            contribs=contribs,
            volume=volume,
            number=number, # XXX
            #issue,
            pages=pages,
            language=language,
            abstracts=abstracts,
            extra=extra,
            #license_slug=license_slug,
        )
        re = self.biblio_hacks(re)
        return re

    @staticmethod
    def biblio_hacks(re):
        """
        This function handles known special cases. For example,
        publisher-specific or platform-specific workarounds.
        """
        return re

    def try_update(self, re):

        # lookup existing DOI (don't need to try other ext idents for crossref)
        existing = None
        try:
            existing = self.api.lookup_release(doaj=re.ext_ids.doaj)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err
            # doesn't exist, need to update
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

    def doaj_abstracts(self, bibjson: dict) -> List[fatcat_openapi_client.ReleaseAbstract]:
        text = clean(bibjson['abstract'])
        if not text or len(text) < 10:
            return []
        if len(text) > MAX_ABSTRACT_LENGTH:
            text = text[:MAX_ABSTRACT_LENGTH] + " [...]"

        # Detect language. This is fuzzy and may be removed, if too unreliable.
        lang = None
        try:
            lang = langdetect.detect(text)
        except (langdetect.lang_detect_exception.LangDetectException, TypeError) as err:
            #print('[{}] language detection failed with {} on {}'.format(doi, err, text), file=sys.stderr)
            pass

        abstract = fatcat_openapi_client.ReleaseAbstract(
            mimetype="text/plain",
            content=text,
            lang=lang,
        )

        return [abstract,]

    def doaj_contribs(self, authors: List[dict]) -> List[fatcat_openapi_client.ReleaseContrib]:
        """
        bibjson.author {
            affiliation (string, optional),
            name (string),
            orcid_id (string, optional)
        }
        """
        contribs = []
        # TODO: index?
        for author in authors:
            if not author.get('name'):
                continue
            contribs.append(fatcat_openapi_client.ReleaseContrib(
                raw_name=author.get('name'),
                # XXX: orcid_id=author.get('orcid_id') or None,
                # XXX: affiliation=author.get('affiliation') or None,
            ))
        return contribs

    def doaj_ext_ids(self, identifiers: List[dict], doaj_article_id: str) -> fatcat_openapi_client.ReleaseExtIds:
        """
        bibjson.identifier {
            id (string),
            type (string)
        }
        """

        assert doaj_article_id.isalnum() and len(doaj_article_id) == 32

        doi: Optional[str] = None
        pmid: Optional[str] = None
        pmcid: Optional[str] = None
        for id_obj in identifiers:
            if id_obj['type'].lower() == 'doi':
                doi = clean_doi(id_obj['id'])
            elif id_obj['type'].lower() == 'pmid':
                pmid = id_obj['id']
            elif id_obj['type'].lower() == 'pmcid':
                pmcid = id_obj['id']

        return fatcat_openapi_client.ReleaseExtIds(
            doaj=doaj_article_id,
            doi=doi,
            pmid=pmid,
            pmcid=pmcid,
        )
