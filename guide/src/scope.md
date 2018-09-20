# Scope

The goal is to capture the "scholarly web": the graph of written works that
cite other works. Any work that is both cited more than once and cites more
than one other work in the catalog is very likely to be in scope. "Leaf nodes"
and small islands of intra-cited works may or may not be in scope.

Overall focus is on written works, with some exceptions. The expected core
focus (for which we would pursue "completeness") is:

    journal articles
    academic books
    conference proceedings
    technical memos
    dissertations
    monographs
    well-researched blog posts
    web pages (that have citations)
    "white papers"

Possibly in scope:

    reports
    magazine articles
    essays
    notable mailing list postings
    government documents
    presentations (slides, video)
    datasets
    well-researched wiki pages
    patents

Probably not:

    court cases and legal documents
    newspaper articles
    social media
    manuals
    datasheets
    courses
    published poetry

Definitely not:

    audio recordings
    tv show episodes
    musical scores
    advertisements

Author, citation, and work disambiguation would be core tasks. Linking
pre-prints to final publication is in scope.

I'm much less interested in altmetrics, funding, and grant relationships than
most existing databases in this space.

fatcat would not include any fulltext content itself, even for cleanly licensed
(open access) works, but would have "strong" (verified) links to fulltext
content, and would include file-level metadata (like hashes and fingerprints)
to help discovery and identify content from any source. File-level URLs with
context ("repository", "author-homepage", "web-archive") should make fatcat
more useful for both humans and machines to quickly access fulltext content of
a given mimetype than existing redirect or landing page systems. So another
factor in deciding scope is whether a work has "digital fixity" and can be
contained in a single immutable file.
