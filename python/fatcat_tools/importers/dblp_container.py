"""
Importer for DBLP container-level (journal/conference/series) metadata,
pre-scraped in to JSON from HTML pages.
"""

import sys  # noqa: F401
from typing import Any, Dict, List, Optional, Sequence

import fatcat_openapi_client
from fatcat_openapi_client import ApiClient, ContainerEntity

from fatcat_tools.importers.common import EntityImporter
from fatcat_tools.normal import clean_str


class DblpContainerImporter(EntityImporter):
    def __init__(
        self,
        api: ApiClient,
        issn_map_file: Sequence,
        dblp_container_map_file: Sequence,
        dblp_container_map_output: Any,
        **kwargs
    ) -> None:

        eg_desc = kwargs.get(
            "editgroup_description",
            "Automated import of container-level metadata scraped from dblp HTML",
        )
        eg_extra = kwargs.get("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.DblpContainerImporter")
        super().__init__(api, editgroup_description=eg_desc, editgroup_extra=eg_extra, **kwargs)

        self.dblp_container_map_output = dblp_container_map_output
        self.read_dblp_container_map_file(dblp_container_map_file)
        self.read_issn_map_file(issn_map_file)
        print("\t".join(["dblp_prefix", "container_id"]), file=self.dblp_container_map_output)

    def read_dblp_container_map_file(self, dblp_container_map_file: Sequence) -> None:
        self._dblp_container_map = dict()
        print("Loading existing dblp prefix container map file...", file=sys.stderr)
        for line in dblp_container_map_file:
            if line.startswith("dblp_prefix") or len(line) == 0:
                continue
            (prefix, container_id) = line.split()[0:2]
            assert len(container_id) == 26
            self._dblp_container_map[prefix] = container_id
            print("\t".join([prefix, container_id]), file=self.dblp_container_map_output)
        print(
            f"Got {len(self._dblp_container_map)} existing dblp container mappings.",
            file=sys.stderr,
        )

    def lookup_dblp_prefix(self, prefix: str) -> Optional[str]:
        if not prefix:
            return None
        return self._dblp_container_map.get(prefix)

    def want(self, raw_record: Any) -> bool:
        return True

    def parse_record(self, row: Dict[str, Any]) -> Optional[ContainerEntity]:
        """
        row is a python dict (parsed from JSON).

        returns a ContainerEntity (or None if invalid or couldn't parse)
        """

        dblp_prefix = row.get("key") or row.get("dblp_prefix")
        assert dblp_prefix
        assert row["title"]

        container_type = None
        if dblp_prefix.startswith("conf/"):
            container_type = "conference-series"
        elif dblp_prefix.startswith("journals/"):
            container_type = "journal"
        elif dblp_prefix.startswith("series/"):
            container_type = "book-series"

        issnl = None
        for issn in row.get("issns", []):
            issnl = self.issn2issnl(issn)
            if issnl:
                break

        extra: Dict[str, Any] = {
            "dblp": {
                "prefix": dblp_prefix,
            },
        }

        if row.get("homepage_url"):
            extra["urls"] = [row["homepage_url"]]

        if row.get("acronym"):
            extra["acronym"] = row["acronym"]

        ce = fatcat_openapi_client.ContainerEntity(
            name=clean_str(row["title"]),
            container_type=container_type,
            issnl=issnl,
            wikidata_qid=row.get("wikidata_qid"),
            extra=extra,
        )
        return ce

    def try_update(self, ce: ContainerEntity) -> bool:

        dblp_prefix = ce.extra["dblp"]["prefix"]
        existing = None
        existing_container_id = self.lookup_dblp_prefix(dblp_prefix)
        if existing_container_id:
            existing = self.api.get_container(existing_container_id)
        if not existing and ce.issnl:
            # check if existing by ISSN-L
            try:
                existing = self.api.lookup_container(issnl=ce.issnl)
            except fatcat_openapi_client.rest.ApiException as err:
                if err.status != 404:
                    raise err
        if not existing and ce.wikidata_qid:
            try:
                existing = self.api.lookup_container(wikidata_qid=ce.wikidata_qid)
            except fatcat_openapi_client.rest.ApiException as err:
                if err.status != 404:
                    raise err

        # TODO: plan to add a fuzzy match check here

        if not existing:
            return True

        if existing:
            self.counts["exists"] += 1
            print(
                "\t".join([ce.extra["dblp"]["prefix"], existing.ident]),
                file=self.dblp_container_map_output,
            )
            return False

        # shouldn't get here
        raise NotImplementedError()

    def insert_batch(self, batch: List[ContainerEntity]) -> None:
        """
        Because we want to print a prefix/container_id match for each row, we
        require a special batch insert method
        """
        eg = self.api.create_container_auto_batch(
            fatcat_openapi_client.ContainerAutoBatch(
                editgroup=fatcat_openapi_client.Editgroup(
                    description=self.editgroup_description, extra=self.editgroup_extra
                ),
                entity_list=batch,
            )
        )
        for c_edit in eg.edits.containers:
            c = self.api.get_container(c_edit.ident)
            print(
                "\t".join([c.extra["dblp"]["prefix"], c.ident]),
                file=self.dblp_container_map_output,
            )
