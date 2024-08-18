For Authors
===============

This page addresses common questions and concerns from individual authors of
works indexed in Fatcat, as well as the Internet Archive Scholar service built
on top of it.

For help in exceptional cases, contact Internet Archive through our usual
support channels.


## Updating Works

A frequent request from authors is to remove outdated versions of works.

The philosophy of the catalog is to go beyond "the version of record" and
instead collect "the record of versions". This means that drafts, manuscripts,
working papers, and other alternative versions of works can be fully included
and differentiated using metadata in the catalog. Even in the case of
retractions, expressions of concern, or other serious issues with earlier
versions, it is valuable to keep out-of-date versions in the catalog. Corrected
or updated versions will generally be preferred and linked to publicly, for
example on scholar.archive.org. Outright removing content reduces context and
can result in additional confusion for readers and librarians.

Because of this, it is strongly preferred to add new updated content instead of
requesting the removal of old out-of-date content. Depending on the situation,
this could involve creating a new post-publication `release` entity with the
date of update, with status `updated` or `retracted`; or a new pre-publication
`release`; or crawling an updated PDF and adding to an existing `release`
entity.


## Correcting Metadata

Sometimes the bibliographic metadata in fatcat is incorrect, incomplete, or out
of date. This is a particularly sensitive subject when it comes to representing
information about individuals. While we aspire to automating metadata updates
and improvements as much as possible, often a human touch is best.

Any person can contribute to the catalog directly by creating an account and
submitting changes for review. This includes, but is not limited to, authors or
a person acting on their behalf submitting corrections. The [editing
quickstart](./editing_quickstart.md) is a good place to start. Please remember that
corrections are considered part of the public record of the catalog and will be
preserved even if a contributor later deletes their account. Editor *usernames*
can be changed at any time.

Fatcat is in some sense a non-authoritive catalog, which means that it is
usually best if corrections are made in "upstream" sources first (or at the
same time) as being corrected in fatcat. For example, updating metadata in
publisher databases, repositories, or ORCiD in addition to in fatcat.


### Name Changes

The preferred workflow for author name changes depends on the author's
sensitivity to having prior names accessible and searchable.

If "also known as" behavior is desirable, contributor names on the release
record should remain unchanged (matching what the publication at the time
indicated), and a linked `creator` entity should include the
currently-preferred name for display.

If "also known as" is not acceptable, and the work has already been updated in
authoritative publication catalogs, then the contributor name can be updated on
`release` records as well.

See also the [`creator` style guide](./entity_creator.md).


### Author Relation Completeness

`creator` records are not always generated when importing `release` records;
the current practice is to create and/or link them if there is ORCiD metadata
linking specific authors to a published work.

This means that author/work is often very incomplete or non-existent. At this
time we would recommend using other services like dblp.org or openalex.org for
more complete (but possibly less accurate) author/work metadata.


## Resolving Publication Disputes

Authorship and publication ethics disputes should generally be resolved with
the original publisher first, then updated in fatcat.
