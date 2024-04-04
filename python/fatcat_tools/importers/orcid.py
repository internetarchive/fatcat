import sys
from typing import Any, Dict, List, Optional

import fatcat_openapi_client
from fatcat_openapi_client import ApiClient, CreatorEntity

from fatcat_tools.normal import clean_str, clean_orcid

from .common import EntityImporter


def value_or_none(e: Any) -> Any:
    if type(e) == dict:
        e = e.get("value")
    if type(e) == str and len(e) == 0:
        e = None
    # TODO: this is probably bogus; patched in desperation; remove?
    if e:
        try:
            e.encode()
        except UnicodeEncodeError:
            # Invalid JSON?
            print("BAD UNICODE")
            return None
    return e


class OrcidImporter(EntityImporter):
    def __init__(self, api: ApiClient, **kwargs) -> None:

        eg_desc = kwargs.get(
            "editgroup_description",
            "Automated import of ORCID metadata, from official bulk releases.",
        )
        eg_extra = kwargs.get("editgroup_extra", dict())
        eg_extra["agent"] = eg_extra.get("agent", "fatcat_tools.OrcidImporter")
        super().__init__(api, editgroup_description=eg_desc, editgroup_extra=eg_extra, **kwargs)

    def want(self, raw_record: Any) -> bool:
        return True

    def parse_record(self, obj: Dict[str, Any]) -> Optional[CreatorEntity]:
        """
        obj is a python dict (parsed from json).
        returns a CreatorEntity
        """

        if "person" not in obj:
            return False

        name = obj["person"]["name"]
        if not name:
            return None
        extra = None
        given = value_or_none(name.get("given-names"))
        sur = value_or_none(name.get("family-name"))
        display = value_or_none(name.get("credit-name"))
        if display is None:
            # TODO: sorry human beings
            if given and sur:
                display = "{} {}".format(given, sur)
            elif sur:
                display = sur
            elif given:
                display = given
        orcid = clean_orcid(obj["orcid-identifier"]["path"])
        if orcid is None:
            sys.stderr.write("Bad ORCID: {}\n".format(orcid))
            return None
        display = clean_str(display)
        if not display:
            # must have *some* name
            return None
        ce = CreatorEntity(
            orcid=orcid,
            given_name=clean_str(given),
            surname=clean_str(sur),
            display_name=display,
            extra=extra,
        )
        return ce

    def try_update(self, ce: CreatorEntity) -> bool:
        existing = None
        try:
            existing = self.api.lookup_creator(orcid=ce.orcid)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        # eventually we'll want to support "updates", but for now just skip if
        # entity already exists
        if existing:
            self.counts["exists"] += 1
            return False

        return True

    def insert_batch(self, batch: List[CreatorEntity]) -> None:
        self.api.create_creator_auto_batch(
            fatcat_openapi_client.CreatorAutoBatch(
                editgroup=fatcat_openapi_client.Editgroup(
                    description=self.editgroup_description, extra=self.editgroup_extra
                ),
                entity_list=batch,
            )
        )
