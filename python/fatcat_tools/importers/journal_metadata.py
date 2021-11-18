from typing import Any, Dict, List, Optional

import fatcat_openapi_client
from fatcat_openapi_client import DefaultApi, ContainerEntity

from fatcat_tools.normal import clean_str

from .common import EntityImporter


def or_none(s: Optional[str]) -> Optional[str]:
    if s is None:
        return None
    if len(s) == 0:
        return None
    return s


def truthy(s: Optional[str]) -> Optional[bool]:
    if s is None:
        return None
    s = s.lower()

    if s in ("true", "t", "yes", "y", "1"):
        return True
    elif s in ("false", "f", "no", "n", "0"):
        return False
    else:
        return None


class JournalMetadataImporter(EntityImporter):
    """
    Imports journal metadata ("containers") by ISSN, currently from a custom
    munged JSON format (see ../extra/journal_metadata/).

    See guide for details on the many 'extra' fields used here.
    """

    def __init__(self, api: DefaultApi, **kwargs) -> None:

        eg_desc = kwargs.get(
            "editgroup_description",
            "Automated import of container-level metadata, by ISSN. Metadata from Internet Archive munging.",
        )
        eg_extra = kwargs.get("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.JournalMetadataImporter")
        super().__init__(api, editgroup_description=eg_desc, editgroup_extra=eg_extra, **kwargs)

    def want(self, raw_record: Any) -> bool:
        if raw_record.get("issnl") and raw_record.get("name"):
            return True
        return False

    def parse_record(self, row: Dict[str, Any]) -> Optional[ContainerEntity]:
        """
        row is a python dict (parsed from JSON).

        returns a ContainerEntity (or None if invalid or couldn't parse)
        """

        if not row.get("name"):
            # Name is required (by schema)
            return None

        extra = dict()
        for key in (
            "issne",
            "issnp",
            "languages",
            "country",
            "urls",
            "abbrev",
            "coden",
            "aliases",
            "original_name",
            "first_year",
            "last_year",
            "platform",
            "default_license",
            "road",
            "mimetypes",
            "sherpa_romeo",
            "kbart",
        ):
            if row.get(key):
                extra[key] = row[key]
        # TODO: not including for now: norwegian, dois/crossref, ia

        extra_doaj = dict()
        if row.get("doaj"):
            if row["doaj"].get("as_of"):
                extra_doaj["as_of"] = row["doaj"]["as_of"]
            if row["doaj"].get("works"):
                extra_doaj["works"] = row["doaj"]["works"]
        if extra_doaj:
            extra["doaj"] = extra_doaj

        extra_ia = dict()
        # TODO: would like an ia.longtail_ia flag
        if row.get("sim"):
            # NB: None case of the .get() here is blech, but othrwise
            # extra['ia'].get('sim') would be false-y, breaking 'any_ia_sim' later on
            extra_ia["sim"] = {
                "year_spans": row["sim"].get("year_spans"),
            }
        if extra_ia:
            extra["ia"] = extra_ia

        name = clean_str(row.get("name"))
        if not name:
            return None

        ce = ContainerEntity(
            issnl=row["issnl"],
            issne=row.get("issne"),
            issnp=row.get("issnp"),
            container_type=None,  # TODO
            name=name,
            publisher=clean_str(row.get("publisher")),
            wikidata_qid=None,  # TODO
            extra=extra,
        )
        return ce

    def try_update(self, ce: ContainerEntity) -> bool:

        existing = None
        try:
            existing = self.api.lookup_container(issn=ce.issnl)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        if not existing:
            # doesn't exist, create it
            return True

        # for now, only update KBART, and only if there is new content
        if not existing.extra:
            existing.extra = dict()
        if ce.extra.get("kbart") and (existing.extra.get("kbart") != ce.extra["kbart"]):
            if not existing.extra.get("kbart"):
                existing.extra["kbart"] = {}
            existing.extra["kbart"].update(ce.extra["kbart"])
            self.api.update_container(self.get_editgroup_id(), existing.ident, existing)
            self.counts["update"] += 1
            return False
        else:
            self.counts["exists"] += 1
            return False

        # if we got this far, it's a bug
        raise NotImplementedError

    def insert_batch(self, batch: List[ContainerEntity]) -> None:
        self.api.create_container_auto_batch(
            fatcat_openapi_client.ContainerAutoBatch(
                editgroup=fatcat_openapi_client.Editgroup(
                    description=self.editgroup_description, extra=self.editgroup_extra
                ),
                entity_list=batch,
            )
        )
