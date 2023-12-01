Not allowed to PUT edits to the same entity in the same editgroup. If you want
to update an edit, need to delete the old one first.

The state depends only on the current entity state, not any redirect. This
means that if the target of a redirect is deleted, the redirecting entity is
still "redirect", not "deleted".

Redirects-to-redirects are not allowed; this is enforced when the editgroup is
accepted, to prevent race conditions.

Redirects to "work-in-progress" (WIP) rows are disallowed at update time (and
not re-checked at accept time).

"ident table" parameters are ignored for entity updates. This is so clients can
simply re-use object instantiations.

The "state" parameter of an entity body is used as a flag when deciding whether
to do non-normal updates (eg, redirect or undelete, as opposed to inserting a
new revision).

In the API, if you, eg, expand=files on a redirected release, you will get
files that point to the *target* release entity. If you use the /files endpoint
(instead of expand), you will get the files pointing to the redirected entity
(which probably need updating!). Also, if you expand=files on the target
entity, you *won't* get the files pointing to the redirected release. A
high-level merge process might make these changes at the same time? Or at least
tag at edit review time. A sweeper task can look for and auto-correct such
redirects after some delay period.

=> it would not be too hard to update get_release_files to check for such
   redirects; could be handled by request flag?

`prev_rev` is naively set to the most-recent previous state. If the current
state was deleted or a redirect, it is set to null.

This parameter is not checked/enforced at edit accept time (but could be, and
maybe introduce `prev_redirect`, for race detection). Or, could have ident
point to most-recent edit, and have edits point to prev, for firmer control.


fatcat misc:

- opencitations: https://arxiv.org/abs/1906.11964
- https://pub.uni-bielefeld.de/record/2934907
- re-read: scratch/issn/web_archiving.md
- should expansion of 'wip' entities be allowed?
- could now just not show 'wip' entities (unless part of editgroup)
-  release_ref | 19904400 | Missing Index? |  4141039616 | 81833687 |  61929287
- privacy/security issue with libmacaroon logging failed caveat verification
- blank box on editgroup pages when not logged in
- don't have "Editable catalog of bibliographic and fulltext file metadata" be the thing in snippets?
- web: '|dictsort' in a bunch of places (for stability)
- example HTML paper: https://andrewgyork.github.io/rescan_line_sted/
- pubmed importer should include section in ALLCAPS: for multi-part abstracts
- https://github.com/rholder/retrying
- feature: push-button "update metadata from crossref"
- demo ORCID: 0000-0002-1825-0097
- link: https://www.jstor.org/dfr/about/technical-specifications
- after indexing, optimise the Elasticsearch index by merging into a single segment: curl -XPOST 'http://localhost:9200/scholar/_forcemerge?max_num_segments=1'

