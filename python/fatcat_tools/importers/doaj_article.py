"""
Importer for DOAJ article-level metadata, schema v1.

DOAJ API schema and docs: https://doaj.org/api/v1/docs
"""

import warnings
import datetime
from typing import List, Optional

import fatcat_openapi_client
from fatcat_tools.normal import (clean_doi, clean_str, parse_month,
    clean_orcid, detect_text_lang, parse_lang_name, parse_country_name,
    clean_pmid, clean_pmcid)
from fatcat_tools.importers.common import EntityImporter

# Cutoff length for abstracts.
MAX_ABSTRACT_LENGTH = 2048


class DoajArticleImporter(EntityImporter):

    def __init__(self,
                 api,
                 issn_map_file,
                 **kwargs):

        eg_desc = kwargs.get(
            'editgroup_description',
            "Automated import of DOAJ article metadata, harvested from REST API or bulk dumps"
        )
        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent',
                                         'fatcat_tools.DoajArticleImporter')
        # ensure default is to not do updates with this worker (override super() default)
        kwargs['do_updates'] = kwargs.get("do_updates", False)
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
        """

        if not obj or not isinstance(obj, dict) or 'bibjson' not in obj:
            self.counts['skip-empty'] += 1
            return None

        bibjson = obj['bibjson']

        title = clean_str(bibjson.get('title'), force_xml=True)
        if not title:
            self.counts['skip-title'] += 1
            return False

        container_name = clean_str(bibjson['journal']['title'])
        container_id = None
        # NOTE: 'issns' not documented in API schema
        for issn in bibjson['journal']['issns']:
            issnl = self.issn2issnl(issn)
            if issnl:
                container_id = self.lookup_issnl(self.issn2issnl(issn))
            if container_id:
                # don't store container_name when we have an exact match
                container_name = None
                break

        volume = clean_str(bibjson['journal'].get('volume'))
        # NOTE: this schema seems to use "number" as "issue number"
        issue = clean_str(bibjson['journal'].get('number'))
        publisher = clean_str(bibjson['journal'].get('publisher'))

        try:
            release_year = int(bibjson.get('year'))
        except (TypeError, ValueError):
            release_year = None
        release_month = parse_month(clean_str(bibjson.get('month')))

        # block bogus far-future years/dates
        if release_year is not None and (release_year > (self.this_year + 5) or release_year < 1000):
            release_month = None
            release_year = None

        license_slug = self.doaj_license_slug(bibjson['journal'].get('license'))
        country = parse_country_name(bibjson['journal'].get('country'))
        language = None
        for raw in bibjson['journal'].get('language') or []:
            language = parse_lang_name(raw)
            if language:
                break

        # pages
        # NOTE: error in API docs? seems like start_page not under 'journal' object
        start_page = clean_str(bibjson['journal'].get('start_page')) or clean_str(bibjson.get('start_page'))
        end_page = clean_str(bibjson['journal'].get('end_page')) or clean_str(bibjson.get('end_page'))
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
            doaj_extra['keywords'] = [k for k in [clean_str(s) for s in bibjson.get('keywords')] if k]

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
            issue=issue,
            pages=pages,
            language=language,
            abstracts=abstracts,
            extra=extra,
            license_slug=license_slug,
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

        # lookup existing release by DOAJ article id
        existing = None
        try:
            existing = self.api.lookup_release(doaj=re.ext_ids.doaj)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        # then try other ext_id lookups
        if not existing:
            for extid_type in ('doi', 'pmid', 'pmcid'):
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
                    if existing.ext_ids.doaj:
                        warn_str = f"unexpected DOAJ ext_id match after lookup failed doaj={re.ext_ids.doaj} ident={existing.ident}"
                        warnings.warn(warn_str)
                        self.counts["skip-doaj-id-mismatch"] += 1
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

        # if no fuzzy existing match, create entity
        if not existing:
            return True

        # other logic could go here about skipping updates
        if not self.do_updates or existing.ext_ids.doaj:
            self.counts['exists'] += 1
            return False

        # fields to copy over for update
        existing.ext_ids.doaj = existing.ext_ids.doaj or re.ext_ids.doaj
        existing.release_type = existing.release_type or re.release_type
        existing.release_stage = existing.release_stage or re.release_stage
        existing.container_id = existing.container_id or re.container_id
        existing.abstracts = existing.abstracts or re.abstracts
        existing.extra['doaj'] = re.extra['doaj']
        existing.volume = existing.volume or re.volume
        existing.issue = existing.issue or re.issue
        existing.pages = existing.pages or re.pages
        existing.language = existing.language or re.language

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

    def doaj_abstracts(self, bibjson: dict) -> List[fatcat_openapi_client.ReleaseAbstract]:
        text = clean_str(bibjson.get('abstract'))
        if not text or len(text) < 10:
            return []
        if len(text) > MAX_ABSTRACT_LENGTH:
            text = text[:MAX_ABSTRACT_LENGTH] + " [...]"

        lang = detect_text_lang(text)

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
        index = 0
        for author in authors:
            if not author.get('name'):
                continue
            creator_id = None
            orcid = clean_orcid(author.get('orcid_id'))
            if orcid:
                creator_id = self.lookup_orcid(orcid)
            contribs.append(fatcat_openapi_client.ReleaseContrib(
                raw_name=author.get('name'),
                role='author',
                index=index,
                creator_id=creator_id,
                raw_affiliation=clean_str(author.get('affiliation')),
            ))
            index += 1
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
            if not id_obj.get('id'):
                continue
            if id_obj['type'].lower() == 'doi':
                doi = clean_doi(id_obj['id'])
            elif id_obj['type'].lower() == 'pmid':
                pmid = clean_pmid(id_obj['id'])
            elif id_obj['type'].lower() == 'pmcid':
                pmcid = clean_pmcid(id_obj['id'])

        return fatcat_openapi_client.ReleaseExtIds(
            doaj=doaj_article_id,
            doi=doi,
            pmid=pmid,
            pmcid=pmcid,
        )

    def doaj_license_slug(self, license_list: List[dict]) -> Optional[str]:
        """
        bibjson.journal.license {
            open_access (boolean, optional),
            title (string, optional),
            type (string, optional),
            url (string, optional),
            version (string, optional)
        }
        """
        if not license_list:
            return None
        for license in license_list:
            if not license.get('open_access'):
                continue
            slug = license.get('type')
            if slug.startswith('CC '):
                slug = slug.replace('CC ', 'cc-').lower()
                return slug
        return None
