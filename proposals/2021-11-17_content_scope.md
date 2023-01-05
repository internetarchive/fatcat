
status: work-in-progress

Content Scope Fields
======================

Usually, "artifact" entities (file, fileset, webcapture) should not contain
bibliographic metadata about their contents. For example, a file entity
describing a PDF of a journal article should not indicate the publication
stage, retraction status, publication type, journal ISSN, or other metadata
about that article; the `release` entity should contain that information.
Additionally, it is usually assumed that a single "artifact" entity is a
complete representation of any associated release entities: the complete
dataset, or complete article.

This document describes a new metadata field to handle some special cases that
go against this principle: the `content_scope` of a file, fileset, or
webcapture. It is intended to be used when there is an exception to the
assumption that a single "artifact" is a complete representation of a release.
It is particularly useful when there is a problem with the artifact, resulting
with it being disassociated with all releases.


## Values

This section will get copied to the guide.

- if not set, assume that the artifact entity is valid and represents a
  complete copy of the release
- `issue`: artifact contains an entire issue of a serial publication (eg, issue
  of a journal), representing several releases in full
- `abstract`: contains only an abstract (short description) of the release, not
  the release itself (unless the `release_type` itself is `abstract`, in which
  case it is the entire release)
- `index`: index of a journal, or series of abstracts from a conference (TODO:
  separate value for conference abstract lists?)
- `slides`: slide deck (usually in "landscape" orientation)
- `front-matter`: non-article content from a journal, such as editorial policies
- `supplement`: usually a file entity which is a supplement or appendix, not
  the entire work
- `component`: a sub-component of a release, which may or may not be associated
  with a `component` release entity. For example, a single figure or table as
  part of an article
- `poster`: digital copy of a poster, eg as displayed at conference poster sessions
- `sample`: a partial sample of the entire work. eg, just the first page of an
  article. distinct from `truncated`
- `truncated`: the file has been truncated at a binary level, and may also be
  corrupt or invalid. distinct from `sample`
- `corrupt`: broken, mangled, or corrupt file (at the binary level)
- `stub`: any other out-of-scope artifact situations, where the artifact
  represents something which would not link to any possible in-scope release in
  the catalog (except a `stub` release)
- `landing-page`: for webcapture, the landing page of a work, as opposed to the
  work itself
- `spam`: content is spam. articles, webpages, or issues which include
  incidental advertisements within them are not counted as `spam`


## Implementation

The string field `content_scope` will be added to file, fileset, and webcapture
entities.

By default, this field does not need to be set. If it is empty, it can be
assumed that the artifact represents an appropriate copy of the full release.
If it is set, and the artifact is associated with one or more releases,
downstream users/code may want to verify that the `content_scope` and
`release_type` values are consistent. For example, `slides` is not consistent
with `article-journal`, so such a file should be marked for review, and not
considered a valid access option or preservation copy for the purposes of
coverage analysis.


## Removing Release Linkage

In cases where the "artifact" entity is not an acceptable representation of any
release (eg, truncation, corruption, spam), the entity should have the
`release_ids` field cleared.

Optionally, the new `extra` field `related_release_ids` can be used to indicate
that an artifact entity has something to do with specific releases, but is not
a full representation of them. This can be useful for corrupt or partial
content to link to releases it is a partial representation of.

