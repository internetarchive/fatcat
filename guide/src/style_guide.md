# Cataloging Style Guide

## Language and Translation of Metadata

The Fatcat data model does not include multiple titles or names for the same
entity, or even a "native"/"international" representation as seems common in
other bibliographic systems. This most notably applies to release titles, but
also to container and publisher names, and likely other fields.

For now, editors must use their own judgment over whether to use the title of
the release listed in the work itself

This is not to be confused with *translations* of entire works, which should be
treated as an entirely separate `release`.

## External Identifiers

"Fake identifiers", which are actually registered and used in examples and
documentation (such as DOI `10.5555/12345678`) are allowed (and the entity
should be tagged as a fake or example). Non-registered "identifier-like
strings", which are semantically valid but not registered, should not exist in
Fatcat metadata in an identifier column. Invalid identifier strings can be
stored in "extra" metadata. Crossref has [blogged][] about this distinction.

[blogged]: https://www.crossref.org/blog/doi-like-strings-and-fake-dois/

## Editgroups and Meta-Meta-Data

Editors are expected to group their edits in semantically meaningful editgroups
of a reasonable size for review and acceptance. For example, merging two
`creators` and updating related `releases` could all go in a single editgroup.
Large refactors, conversions, and imports, which may touch thousands of
entities, should be grouped into reasonable size editgroups; extremely large
editgroups may cause technical issues, and make review unmanageable. 50 edits is
a decent batch size, and 100 is a good upper limit (and may be enforced by the
server).

