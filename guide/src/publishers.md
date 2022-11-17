For Publishers
===================

This page addresses common questions and concerns from publishers of research
works indexed in Fatcat, as well as the Internet Archive Scholar service built
on top of it. The [for authors](./authors.md) has some information on updates
and metadata corrections that are also relevant to publishers.

For help in exceptional cases, contact Internet Archive through our usual
support channels.


## Metadata Indexing

Many publishers will find that metadata records are already included in fatcat
if they register persistent identifiers for their research works. This pipeline
is based on our automated harvesting of DOI, Pubmed, dblp, DOAJ, and other
metadata catalogs. This process can take some time (eg, days from
registration), does not (yet) cover all persistent identifiers, and will only
cover those works which get identifiers.

For publishers who find that they are not getting indexed in fatcat, our
primary advice is to register ISSNs for venues (journals, repositories,
conferences, etc), and to register DOIs for all current and back-catalog works.
DOIs are the most common and integrated identifier in the scholarly ecosystem,
and will result in automatic indexing in many other aggregators in addition to
fatcat/scholar. There may be funding or resources available for smaller
publishers to cover the cost of DOI registration, and ISSN registration is
usually no-cost or affordable through national institutions.

We *do not* recommend that journal or conference publishers use general-purpose
repositories like Zenodo to obtain no-cost DOIs for journal articles. These
platforms are a great place for pre-publication versions, datasets, software,
and other artifacts, but not for primary publication-version works (in our
opinion).

If DOI registration is not possible, one good alternative is to get included in
the Directory of Open Access Journals and deposit article metadata there. This
process may take some time, but is a good basic indicator of publication
quality. DOAJ article metadata is periodically harvested and indexed in fatcat,
after a de-duplication process.

Fatcat does not yet support OAI-PMH as an identifier and mechanism for
automated journal ingest, but we likely will in the future. This would
particularly help publishers using the Open Journal System (OJS). Fatcat also
does not yet support crawling journal sites and extracting bibliographic
metadata from HTML tags.

Lastly, publishers could use the fatcat catalog web interface or API to push
metadata records about their works programmatically. We don't know of any
publishers actually doing this today.


## Improving Automatic Preservation

In alignment with it's mission, Internet Archive makes basic automated attempts
to capture and preserve all open access research publications on the public
web, at no cost. This effort comes with no guarantees around completeness,
timeliness, or support communications.

Preservation coverage can be monitored through the journal-specific dashboards
or via the coverage search interface.

There are a few technical things publishers can do to increase their
preservation coverage, in addition to the metadata indexing tips above:

- use the `citation_pdf_url` HTML meta tag, when appropriate, to link directly
  from article landing pages to PDF URLs
- use simple HTML to represent landing pages and article content, and do not
  require Javascript to render page content or links
- ensure that hosting server `robots.txt` rules are not preventing or overly
  restricting automated crawling
- use simple, accessible PDF access links. Do not use time-limited or
  IP-limited URLs, require specific referrer headers, or use cookies to
  authenticate access to OA PDFs
- minimize the number of HTTP redirects and HTML hops between DOI and fulltext
  content
- paywalls, loginwalls, geofencing, and anti-bot measures are all obviously
  antithetical to open crawling and indexing

Publishers are also free to submit "Save Paper Now" requests, or edit the
catalog itself either manually or in bulk through the API. If an individual
work persistently fails to ingest, try running a "Save Page Now" request first
from web.archive.org and verify that the content is available through Wayback
replay, then submit the "Save Paper Now" request again.


## Official Preservation

Internet Archive is developing preservation services for scholarly content on
the web. Contact us at webservices@archive.org for details.

Existing web archiving services offered to universities, national libraries,
and other institutions may already be appropriate for some publications. Check
if your affiliated institutions already have an
[Archive-IT](https://archive-it.org) account or other existing relationship
with Internet Archive.

Small publishers using Open Journal System (OJS) should be aware of the PKP
preservation project.
