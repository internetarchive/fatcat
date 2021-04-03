
status: wip

Crawlability Improvements
--------------------------

We are interested in making the fatcat corpus more crawlable/indexable by
aggregators and academic search enginges. For example, CiteseerX, Google
Scholar, or Microsoft Academic (when themselves get used by other projects).

Some open questions:

- is the web.archive.org iframe for PDFs ok, or should we redirect to PDFs with `id_` in the datetime?


## Redirect URLs and `citation_pdf_url`

We suspect that some crawlers do not like that fatcat.wiki landing pages have
`citation_pdf_url` fields that point to a different registered domain.
`www.semanticscholar.org -> pdfs.semanticscholar.org` is presumably ok, but
maybe `fatcat.wiki -> web.archive.org` and `fatcat.wiki -> archive.org` are
not.

Google Scholar docs also request the PDF link be "in the same subdirectory"
(though this obviously isn't true on, eg, semanticscholar.org):

> If this page shows only the abstract of the paper and you have the full text
> in a separate file, e.g., in the PDF format, please specify the locations of
> all full text versions using citation_pdf_url or DC.identifier tags. The
> content of the tag is the absolute URL of the PDF file; for security reasons,
> it must refer to a file in the same subdirectory as the HTML abstract.

Also suspect that a redirect is probably find. If a journal links from a
landing page to a `.pdf` URL on the same domain, often there is an HTTP
redirect to, eg, amazon AWS. These seem to get indexed fine.

So, potentially have `citation_pdf_url` point to something on `fatcat.wiki`,
which then redirects to `web.archive.org` or `archive.org`, would be
sufficient. This would also be a reasonable URL for external services to point
to, in that which specific access mechanism is redirected would vary as the
catalog is improved.

So, proposing two new web fatcat.wiki endpoints:

    /release/<ident>/access-redirect
    /file/<ident>/access-redirect

Both of these would use an HTTP 302 "temporary" redirect to the "best" archival
fulltext copy.

If somebody wants to link to a specific file (by hash), they should use the
file link. If they want to link to any fulltext access copy, then should use
the release link.

Open questions:

- should the redirect only ever go to archive.org properties?
- for releases, should the file type and access type be filtered? maybe with a
  query parameter, or a `.pdf` suffix?


## "Browsable" Site

Another improvement would be to make the site more "browsable". To start, an
index of journals (by first letter, publisher, country, or similar), then
organize papers under the journal by volume, year, etc. This would give
crawlers a way to spider all papers in the index.

