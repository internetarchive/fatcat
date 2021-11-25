
status: implemented

Entity Mergers
===============

One category of type of catalog metadata cleanup is merging multiple duplicate
entries into a single record. The fatcat catalog allows this via during the
duplicate entities into "redirect records" which point at the single merged
record.

This proposal briefly describes the process for doing bulk merges.


## External Identifier Duplicates

The easiest category of entity duplicates to discover is cases where multiple
entities have the same external (persistent) identifier. For example, releases
with the exact same DOI, containers with the same ISSN-L, or creators with the
same ORCiD. Files with the same SHA-1 hash is a similar issue. The catalog does
not block the creation of such entities, though it is assumed that editors and
bots will do their best to prevent creating duplicates, and that this is
checked and monitored via review bots (auto-annotation) and bulk quality
checks.

In these cases, it is simple enough to use the external identifier dumps (part
of the fatcat bulk exports), find duplicates by identifier, and create merge
requests.


## Merge Requests JSON Schema

Proposed JSON schema for bulk entity merging:

    entity_type: str, required. eg: "file"
    primary_id: str, optional, entity ident
    duplicate_ids: [str], required, entity idents
    evidence: dict, optional, merger/entity specific
        # evidence fields for external identifier dupes
        extid: str, the identifier value
        extid_type: str, eg "doi" or "sha1"

The merge request generation process might indicate which of the entities
should end up as the "primary", or it might leave that determination to the
merger itself. `primary_id` should not be set arbitrarily or randomly if there
is not a good reason for a specific entity to be the "primary" which others
redirect to.

The `primary_id` should not be included in `duplicate_ids`, but the merger code
will remove it if included accidentally.

The `evidence` fields are flexible. By default they will all be included as
top-level "edit extra" metadata on each individual entity redirected, but not
on the primary entity (if it gets updated).


## Merge Process and Semantics

The assumption is that all the entities indicated in `duplicate_ids` will be
redirected to the `primary_id`. Any metadata included in the duplicates which
is not included in the primary will be copied in to the primary, but existing
primary metadata fields will not be "clobbered" (overwritten) by duplicate
metadata. This includes top-level fields of the `extra` metadata dict, if
appropriate. If there is no unique metadata in the redirected entities, the
primary does not need to be updated and will not be.


## Work/Release Grouping and Merging

Work and Release entities are something of a special case.

Merging two release entities will result in all artifact entities (files,
filesets, webcaptures) being updated which previously pointed at the duplicate
entity to point to the primary entity. If the work entities associated with the
duplicate releases have no other releases associated with them, they also will
be merged (redirected) to the primary release's work entity.

"Grouping" releases is the same as merging their works. In this situation, the
number of distinct release entities stays the same, but the duplicates are
updated to be under the same work as the primary. This is initially implemented
by merging the work entities, and then updating *all* the releases under each
merged work towards the primary work identifier. No artifact entities need to
be updated in this scenario.

A currently planned option would be to pull a single release out of a group of
releases under a work, and point it to a new work. This would be a form of
"regrouping". For now this can only be achieved by updating the release
entities individually, not in a bulk/automated manner.


## Container Merging

Because many releases point to containers, it is not practical to update all
the releases at the same time as merging the containers. In the long run it is
good for the health of the catalog to have all the releases updated to point at
the the primary container, but these updates can be delayed.

To keep statistics and functionality working before release updates happen,
downstream users of release entities should "expand" container sub-entities and
use the "redirect" ident of the container entity instead of "ident", if the
"redirect" is set. For example, when linking in web interfaces, or when doing a
schema transform in the fatcat and scholar.archive.org search index.


## Background Reading

"The Lens MetaRecord and LensID: An open identifier system for aggregated
metadata and versioning of knowledge artefacts"
https://osf.io/preprints/lissa/t56yh/

