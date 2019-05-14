
# SQL (and API) schema changes

Intend to make these changes at the same time as bumping OpenAPI schema from
0.2 to 0.3, along with `20190510_editgroup_endpoint_prefix` and
`20190510_release_ext_ids`.

Also adding some indices to speed up entity edit history views, but those are
just a performance change, not visible in API schema.

### Structured Contrib Names

`creator` entities already have "structured" names: in addition to
`display_name`, there are `given_name` and `surname` fields. This change is to
add these two fields to release contribs as well (to join `raw_name`).

The two main motivations are:

1. make various representations (eg, citation formats) of release entities
   easier. CSL and many display formats require given/surname distinctions
2. improve algorithmic matching between release entities, raw metadata (eg,
   from GROBID), and citation strings. Eg, biblio-glutton wants "first author
   surname"; we can't provide this from existing `raw_name` field

The status quo is that many large metadata sources often include structured
names, and we munge them into a single name.

Some arguments against this change are:

1. should be "normalizing" this structure into creator entities. However,
   display/representation of a contributor might change between publications
2. structure isn't always deterministic from what is visible in published
   documents. AKA, raw name is unambiguous (it's what is "printed" on the
   document), but given/sur decomposition can be ambiguous (for individauls, or
   entire locales/cultures)
3. could just stash in contrib `extra_json`. However, seems common enough to
   include as full fields

Questions/Decisions:

- should contrib `raw_name` be changed to `display_name` for consistency with
  `creator`? `raw_name` should probably always be what is in/on the document
  itself, thus no.
- should we still munge a `raw_name` at insert time (we we only have structured
  names), or push this on to client code to always create something for
  display?

### Rename `release_status` to `release_stage`

Describes the field better. I think this is uncontroversial and not too
disruptive at this point.

### New release fields: subtitle, number, version

`subtitle`: mostly for books. could have a flat-out style guide policy against
use for articles? Already frequently add subtitle metadata as an `extra_json`
field.

`number`: intended to represent, eg, a report number ("RFC ..."). Not to be
confused with `container-number`, `chapter`, `edition`

`version`: intended to be a short string ("v3", "2", "third", "3.9") to
disambiguate which among multiple versions. CSL has a separate `edition` field.

These are somewhat hard to justify as dedicated fields vs. `extra_json`.

`subtitle` is a pretty core field for book metadata, but raises ambiguity for
other release types.

Excited to include many reports and memos (as grey lit), for which the number
is a pretty major field, and we probably want to include in elasticsearch but
not as part of the title field, and someday perhaps an index on `number`, so
that's easier to justify.

TODO:

- `version` maybe should be dropped. arXiv is one possible justification, as is
  sorting by this field in display.

### Withdrawn fields

As part of a plan to represent retractions and other "unpublishing", decided to
track when and whether a release has been "withdrawn", distinct from the
`release_stage`.

To motivate this, consider a work that has been retracted. There are multiple
releases of different stages; should not set the `release_stage` for all to
`withdrawn` or `retracted`, because then hard to disambiguate between the
release entities. Also maybe the pre-print hasn't been formally withdrawn and
is still in the pre-print server, or maybe only the pre-print was withdrawn
(for being partial/incorrect?) while the final version is still "active".

As with `release_date`, just `withdrawn_date` is insufficient, so we get
`withdrawn_year` also...  and `withdrawn_month` in the future? Also
`withdrawn_state` for cases where we don't know even the year. This could
probably be a bool (`is_withdrawn` or `withdrawn`), but the flexibility of a
TEXT/ENUM has been nice.

TODO:

- boolean (`is_withdrawn`, default False) or text (`withdrawn_status`). Let's
  keep text to allow evolution in the future; if the field is defined at all
  it's "withdrawn" (true), if not it isn't

### New release extids: `mag_id`, `ark_id`

See also: `20190510_release_ext_ids`.

- `mag_id`: Microsoft Academic Graph identifier.
- `ark_id`: ARK identifier.

These will likely be the last identifiers added as fields on `release`; a
future two-stage refactor will be to move these out to a child table (something
like `extid_type`, `extid_value`, with a UNIQ index for lookups).

Perhaps the `extid` table should be implemented now, starting with these
identifiers?

### Web Capture CDX `size_bytes`

Pretty straight-forward. 

Considered adding `extra_json` as well, to be consistent with other tables, but
feels too heavy for the CDX case. Can add later if there is an actual need;
adding fields easier than removing (for backwards compat).

### Object/Class Name Changes

TODO

### Rust/Python Library Name Changes

Do these as separate commits, after merging back in to master, for v0.3:

- rust `fatcat-api-spec` => `fatcat-openapi`
- python `fatcat_client` => `fatcat_openapi_client`

### More?

`release_month`: apprently pretty common to know the year and month but not
date. I have avoided so far, seems like unnecessary complexity. Could start
as an `extra_json` field?
