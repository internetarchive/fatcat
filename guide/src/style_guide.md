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
stored in "extra" metadata. Crossref has [blogged]() about this distinction.

[blogged]: https://www.crossref.org/blog/doi-like-strings-and-fake-dois/

#### DOIs

All DOIs stored in an entity column should be registered (aka, should be
resolvable from `doi.org`). Invalid identifiers may be cleaned up or removed by
bots.

DOIs should *always* be stored and transferred in lower-case form. Note that
there are almost no other constraints on DOIs (and handles in general): they
may have multiple forward slashes, whitespace, of arbitrary length, etc.
Crossref has a [number of examples]() of such "valid" but frustratingly
formatted strings.

[number of examples]: https://www.crossref.org/blog/dois-unambiguously-and-persistently-identify-published-trustworthy-citable-online-scholarly-literature-right/

In the Fatcat ontology, DOIs and release entities are one-to-one.

It is the intention to automatically (via bot) create a Fatcat release for
every Crossref-registered DOI from a whitelist of media types
("journal-article" etc, but not all), and it would be desirable to auto-create
entities for in-scope publications from all registrars. It is not the intention
to auto-create a release for every registered DOI. In particular,
"sub-component" DOIs (eg, for an individual figure or table from a publication)
aren't currently auto-created, but could be stored in "extra" metadata, or on a
case-by-case basis.

## Human Names

Representing names of human beings in databases is a fraught subject. For some
background reading, see:

- [Falsehoods Programmers Believe About Names](https://www.kalzumeus.com/2010/06/17/falsehoods-programmers-believe-about-names/) (blog post)
- [Personal names around the world](https://www.w3.org/International/questions/qa-personal-names) (W3C informational)
- [Hubert Blaine Wolfeschlegelsteinhausenbergerdorff Sr.](https://en.wikipedia.org/wiki/Hubert_Blaine_Wolfeschlegelsteinhausenbergerdorff_Sr.) (Wikipedia article)

Particular difficult issues in the context of a bibliographic database include
the non-universal concept of "family" vs.  "given" names and their relationship
to first and last names; the inclusion of honorary titles and other suffixes
and prefixes to a name; the distinction between "preferred", "legal", and
"bibliographic" names, or other situations where a person may not wish to be
known under the name they are commonly referred to under; language and character
set issues; and pseudonyms, anonymous publications, and fake personas (perhaps
representing a group, like Bourbaki).

The general guidance for Fatcat is to:

- not be a "source of truth" for representing a persona or human being; ORCID
  and Wikidata are better suited to this task
- represent author personas, not necessarily 1-to-1 with human beings
- prioritize the concerns of a reader or researcher over that of the author
- enable basic interoperability with external databases, file formats, schemas,
  and style guides
- when possible, respect the wishes of individuals

The data model for the `creator` entity has three name fields:

- `surname` and `given_name`: needed for "aligning" with external databases,
  and to export metadata to many standard formats
- `display_name`: the "preferred" representation for display of the entire name,
  in the context of international attribution of authorship of a written work
  
Names to not necessarily need to expressed in a Latin character set, but also
does not necessarily need to be in the native language of the creator or the
language of their notable works

Ideally all three fields are populated for all creators.

It seems likely that this schema and guidance will need review. "Extra"
metadata can be used to store aliases and alternative representations, which
may be useful for disambiguation and automated de-duplication.

## Editgroups and Meta-Meta-Data

Editors are expected to group their edits in semantically meaningful editgroups
of a reasonable size for review and acceptance. For example, merging two
`creators` and updating related `releases` could all go in a single editgroup.
Large refactors, conversions, and imports, which may touch thousands of
entities, should be grouped into reasonable size editgroups; extremely large
editgroups may cause technical issues, and make review unmanageable. 50 edits is
a decent batch size, and 100 is a good upper limit (and may be enforced by the
server).

