from typing import Any, Dict, List, Optional

import fatcat_openapi_client
from fatcat_openapi_client import ApiClient, ContainerEntity

from fatcat_tools.normal import clean_str

from .common import EntityImporter


class ChoculaImporter(EntityImporter):
    """
    Creates or updates container entities based on output of "chocula" script,
    which munges/processes journal metadata from several sources, including
    fatcat itself.

    See guide for details on the many 'extra' fields used here.
    """

    def __init__(self, api: ApiClient, **kwargs) -> None:

        eg_desc = kwargs.get(
            "editgroup_description",
            "Automated import of container-level metadata from Chocula tool.",
        )
        eg_extra = kwargs.get("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.ChoculaImporter")
        super().__init__(api, editgroup_description=eg_desc, editgroup_extra=eg_extra, **kwargs)

    def want(self, raw_record: Any) -> bool:
        if not raw_record.get("ident") and not raw_record.get("_known_issnl"):
            self.counts["skip-unknown-new-issnl"] += 1
            return False

        if raw_record.get("issnl") and raw_record.get("name"):
            return True
        else:
            self.counts["skip-partial-metadata"] += 1
            return False

    def parse_record(self, row: Dict[str, Any]) -> Optional[ContainerEntity]:
        """
        row is a python dict (parsed from JSON).

        returns a ContainerEntity (or None if invalid or couldn't parse)
        """

        name = clean_str(row.get("name"))
        if name and name.endswith("."):
            name = name[:-1]
        if not name:
            # Name is required (by schema)
            return None

        extra = dict()
        for k in (
            "urls",
            "webarchive_urls",
            "country",
            "sherpa_romeo",
            "ezb",
            "szczepanski",
            "doaj",
            "languages",
            "ia",
            "scielo",
            "kbart",
            "publisher_type",
            "platform",
        ):
            if row["extra"].get(k):
                extra[k] = row["extra"][k]

        container_type = None
        if "proceedings" in name.lower():
            container_type = "proceedings"
        elif "journal " in name.lower():
            container_type = "journal"

        if row["extra"].get("issnp"):
            row["extra"]["issnp"] = row["extra"]["issnp"].upper()
        if row["extra"].get("issne"):
            row["extra"]["issne"] = row["extra"]["issne"].upper()
        ce = ContainerEntity(
            issnl=row["issnl"],
            issnp=row["extra"].get("issnp"),
            issne=row["extra"].get("issne"),
            ident=row["ident"],
            name=name,
            container_type=container_type,
            publisher=clean_str(row.get("publisher")),
            wikidata_qid=row.get("wikidata_qid"),
            extra=extra,
        )
        return ce

    def try_update(self, ce: ContainerEntity) -> bool:

        existing = None
        if ce.ident:
            try:
                existing = self.api.get_container(ce.ident)
            except fatcat_openapi_client.rest.ApiException as err:
                if err.status != 404:
                    raise err
                self.counts["exists"] += 1
                self.counts["exists-not-found"] += 1
                return False
            if existing.state != "active":
                self.counts["exists"] += 1
                self.counts["exists-inactive"] += 1
                return False

        if not existing:
            # check if existing by ISSN-L
            try:
                existing = self.api.lookup_container(issnl=ce.issnl)
            except fatcat_openapi_client.rest.ApiException as err:
                if err.status != 404:
                    raise err
            if existing:
                self.counts["exists"] += 1
                self.counts["exists-by-issnl"] += 1
                return False
            # doesn't exist, always create
            return True

        # decide whether to update
        do_update = False
        if not self.do_updates:
            self.counts["exists"] += 1
            return False
        if not existing.extra:
            existing.extra = dict()
        if ce.extra.get("urls") and not ce.extra.get("urls", []):
            do_update = True
        if ce.extra.get("webarchive_urls") and not ce.extra.get("webarchive_urls", []):
            do_update = True
        for k in ("kbart", "ia", "doaj"):
            # always update these fields if not equal (chocula override)
            if ce.extra.get(k) and ce.extra[k] != existing.extra.get(k):
                do_update = True
        if ce.wikidata_qid and not existing.wikidata_qid:
            do_update = True

        if do_update:
            existing.wikidata_qid = existing.wikidata_qid or ce.wikidata_qid
            existing.publisher = existing.publisher or ce.publisher
            existing.container_type = existing.container_type or ce.container_type
            existing.publication_status = existing.publication_status or ce.publication_status
            # move issne/issnp from "extra" to top-level fields (new schema)
            if not existing.issne:
                tmp_issn = existing.extra.pop("issne", None)
                # clean up bad ISSNs in extra metadata
                if tmp_issn and len(tmp_issn) == 9:
                    existing.issne = tmp_issn.upper()
            if not existing.issnp:
                tmp_issn = existing.extra.pop("issnp", None)
                if tmp_issn and len(tmp_issn) == 9:
                    existing.issnp = tmp_issn.upper()
            existing.issne = existing.issne or ce.issne
            existing.issnp = existing.issnp or ce.issnp
            for k in ("urls", "webarchive_urls"):
                # be conservative about URL updates; don't clobber existing URL lists
                # may want to make this behavior more sophisticated in the
                # future, or at least a config flag
                if ce.extra.get(k) and not existing.extra.get(k):
                    existing.extra[k] = ce.extra.get(k, [])
            for k in (
                "sherpa_romeo",
                "ezb",
                "szczepanski",
                "doaj",
                "ia",
                "scielo",
                "kbart",
            ):
                # always update (chocula over-rides)
                if ce.extra.get(k):
                    existing.extra[k] = ce.extra[k]
            for k in ("country", "publisher_type", "platform"):
                # only include if not set (don't clobber human edits)
                if ce.extra.get(k) and not existing.extra.get(k):
                    existing.extra[k] = ce.extra[k]
            if ce.extra.get("languages"):
                if not existing.extra.get("languages"):
                    existing.extra["languages"] = ce.extra["languages"]
                elif not ce.extra["languages"][0] in existing.extra["languages"]:
                    existing.extra["languages"].append(ce.extra["languages"][0])

            self.api.update_container(self.get_editgroup_id(), existing.ident, existing)
            self.counts["update"] += 1
            return False
        else:
            self.counts["exists"] += 1
            self.counts["exists-skip-update"] += 1
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
