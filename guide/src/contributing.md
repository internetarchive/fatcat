
# Contributing

Our aspiration is for this to be an open, collaborative project, with
individuals and organization of all sizes able to participate. There is not
much structure or documentation on how volunteers can get started or be most
helpful, but perhaps we can work together on that as well!

The best place to organize and coordinate right now is the
[gitter chatroom](https://gitter.im/internetarchive/fatcat). Gitter is
described as "for developers", but we use it for everybody, and you don't need
an invitation.

Want to help out? Below are a few example roles you could play.


#### Anybody: Find Bugs, Suggest Improvements

The user sign-up and editing workflow on fatcat.wiki is currently pretty poor.
How could this experience be improved and better documented? Specific ideas,
suggestions and diagrams would be very helpful. You don't need to know how to
program or about web technologies to contribute; hand drawings and example text
can be sufficient.


#### Community Organizer: Partner and Volunteer Organizing

Are you passionate about Open Access and want to help build a community around
preservation and universal access to knowledge? We could use help structuring
an editing community, and communicating with partner projects like Wikidata to
ensure we are not duplicating efforts.

A good example of a project to organize would be improving journal-level
metadata in wikidata, including journal homepages, and linking to fatcat
"container" entities.


#### Research Librarian: Identify Missing Content

If you have an interest in a specific scholarly field, you could give us
feedback on how good of a job fatcat is doing preserving at-risk open access
content. We know we have a lot of work to do, but both specific examples of
missing publications, as well as broader patterns and missing holes are helpful
to know about. Some missing content we know we don't have, but there are surely
entire categories of in-scope content that we do not even know are missing!


#### Metadata Librarian: Schema Improvements

Are you an experienced wrangler of BibFrame, MARC, bibtext, RDF, OAI-PMH, and
Citation Style Language? Our data model and entity schemas are bespoke (sorry!)
and designed to evolved over time. There might be related efforts and new
controlled vocabularies we could adopt or align with, or small changes to the
schema might enable new use cases. It could be as simple as identifying and
prioritizing new external identifiers (PIDs) to allow. Let us know what we got
right and what needs improvement!


#### Power Editor: Better Interfaces

Are you super experienced with data entry, editing, and corrections? Do you
have ideas on how our interface could be improved, or what kinds of new
interfaces and tools could be build to support effective editing? Our open API
allows third-party interfaces to make edits on individuals' behalf, meaning new
tools can be build for specific patterns of editing or user contribution.


#### Data Scientist: Wrangling and Visualization

We have hundreds of gigabytes of metadata to transform and normalize before
importing, and already have a rich open dataset with millions of linked
entities. Our elasticsearch analytics database has an open read-only endpoint
(<https://search.fatcat.wiki>), which are used to power our [coverage
interface](https://fatcat.wiki/coverage/search). What other interactive
visualizations could be built? What tools should we be using to wrangle
bibliographic metadata better and faster?


#### Author: Verify Metadata

Do you publish research documents, and want to ensure it is accessible to the
broadest audience today and in the future? Like many academic search engines,
you can add papers and link an author profile to specific publications. Unlike
others, you can also ensure uploaded pre-prints and other open versions of your
research are found and linked using the "save paper now" feature, and you can
any errors made by publishers and bots.


#### Translation and Accessibility Advocate

Some of our web interfaces have existing internationalization infrastructure,
and translations can be
[contributed directly](https://hosted.weblate.org/projects/internetarchive/).

Other projects need help getting translation infrastructure in place, and all
of our projects could use review and recommendations for improvement by experts
in web accessibility. For example, if you use a screen reader, feedback on
which parts of our services are most difficult to use are very helpful.


#### Software Developer: Bot Wrangling

Fatcat is structured such that all changes to the catalog go through an open
API. This includes human edits through the web interface, but the large
majority of edits are made by bots. You could write a new bot to help...

- review human edits (from the "reviewable" queue) to "lint" for typos, missing
  fields, or other problems, and then leave an annotation
- harvest, transform, and import metadata from addition subject- and
  region-specific sources
- find and clean-up patterns of poor or incorrect metadata already in the
  catalog


#### SQL Expert: Database Scaling

We have a large (500+ GByte) PostgreSQL database backing the catalog. This is
working great so far, but we have concerns about how the catalog will scale
further, especially if bots start making multiple updates per entity. You could
review our SQL schema and recommend improvements, or give feedback and advice
on how to switch to a distributed primary datastore.


#### Financial Supporter

Short on time? As a US 501(c)(3) non-profit, the Internet Archive always
appreciates and makes good use of [donations](https://archive.org/donate/).


## Software Contributions

Bugs and patches can be filed on Github at: <https://github.com/internetarchive/fatcat>

When considering making a non-trivial contribution, it can save review time and
duplicated work to post an issue with your intentions and plan. New code and
features must include unit tests before being merged, though we can help with
writing them.
