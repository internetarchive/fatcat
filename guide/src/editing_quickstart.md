
# Editing Quickstart

This tutorial describes how to make edits to the Fatcat catalog using the web
interface. We will add a new `file` to an existing `release`, then update the
release to point at a different `container`. You can follow these directions on
either the [QA](https://qa.fatcat.wiki) or [production](https://fatcat.wiki)
public catalogs. You will:

- create an editor account and log-in
- create a new `file` entity
- update an existing `release` entity
- submit editgroup for review

First [create an editor account](https://fatcat.wiki/auth/login) and log-in. If
you don't have an account with any of the existing federated log-in services
(eg, Wikipedia, ORCID, Github), you can
[create a few Internet Archive account](https://archive.org/account/signup),
confirm your email, and then log-in to Fatcat using that. You should see your
username in the upper right-hand corner of every page when you are successfully
logged in.

Next find the release's fatcat identifer for the paper we want to add a file
to. You can [search](https://fatcat.wiki/release/search) by title, or
[lookup](https://fatcat.wiki/release/lookup) a paper by an identifier (such
as a DOI or arXiv ID). If the release you are looking for doesn't exist yet,
you'll need to [create](https://fatcat.wiki/release/create) a new one. All of
these actions are linked from the Fatcat front page for each entity type.

The release fatcat identifier is the garbled looking string like
`hsmo6p4smrganpb3fndaj2lon4` which you can find under the title of the paper's
[entity page](https://fatcat.wiki/release/hsmo6p4smrganpb3fndaj2lon4), and also
in the URL. You'll need this identifier to link the file to the release.

Before creating a new file entity (or any entity for that matter), check that
there isn't already an entity referencing the exact same file. Download the
file (eg, PDF) that you want to add to your local computer, and calculate the
SHA-1 hash of the file using a tool like `sha1sum` on the command line. If you
aren't familiar with command line tools, you can upload to a [free online
service](http://onlinemd5.com/). The SHA-1 hash will look like
`de9aefc4522b385121e72faaee75bda9fbb8bf6e`, and you can do a [file
lookup](https://fatcat.wiki/file/lookup). If a file already exists, you could
edit it to add new URLs (locations), or add/update any release links.

Assuming a file entity doesn't already exist, go to [create
file](https://fatcat.wiki/file/create). We will want to start a new "editgroup"
for these changes. If you don't have any editgroups in progress, you can just
enter a description sentance and a new one will be created; if you did have
edits in progress, you'll need to select the "create new editgroup" option from
the drop-down of your existing editgroups.

Enter the basic file metadata in the fields provided. The red stared fields are
required (size in bytes and SHA-1). Add a URL on the public web where the file
can be found. It's best if PDFs are uploaded to repositories (eg,
[Zenodo](https://zenodo.org)) or hosted on the publisher's website. A second
archival location can be added (eg, using the Wayback Machine's ["save page
now"](http://web.archive.org/save) feature), or you could skip this and wait
for a bot to verify and archive the URL later. The left drop-down menu lets you
set the "type" of each URL. Add the release identifier you found earlier to the
"Releases" list.

Add a one-sentance description of your change, and submit the form. You will be
redirected to a provisional ("work in progress") view of the new entity. Edits
are not immediately merged into the catalog proper; the first need to be
"submitted" and then accepted (eg, by a human moderator or robot).

Let's add a second edit to the same editgroup before continuing. The new file
view should have a link to the release entity; follow that link, then click the
"edit" button (either the tab or the blue link at the bottom of the infobox).
This time, the most recent editgroup should already be selected, so you don't
need to enter a description at the top. If there are any problems with basic
metadata, go ahead and fix them, but otherwise skip down to the "Container"
section and update the fatcat identifer ("FCID") to point to the correct
journal. You can [lookup journals](https://fatcat.wiki/container/lookup) by
ISSN-L, or [search](https://fatcat.wiki/container/search) by title. Add a short
description of your change ("Updated journal to XYZ") and then submit.

You now have two edits in your editgroup. There should be links to the
editgroup itself from the "work-in-progress" pages, or you can find all your
editgroups from the drop-down link in the upper right-hand corner of every
page (your username, then "Edit History"). The editgroup page shows all the
entities created, updated, or deleted, and allows you to make tweaks (re-edit)
or remove changes. If the release/container update you made was bogus (just as
a learning exersize), you could remove it here. It's a good practice to group
related edits into the same editgroup, but only up to 50 or so edits at a time
(more than that becomes difficult hard to review).

If things look good, click the "submit" button on the editgroup page. This will
mark your changes as "ready for review", and they will show up on the [global
reviewable editgroups](https://fatcat.wiki/reviewable) list. If you change your
mind, you can "unsubmit" the editgroup and make more changes. Humans and bots
can make annotations to editgroups, recommending changes. At the current time
there are no email or other update notifications, so you need to check in on
annotations and other status manually.

When your changes have been reviewed, a moderator will "accept" them, and the
entities will be updated in the catalog. Every accepted editgroup ends up in
[the changelog](https://fatcat.wiki/changelog).

And then you're done, thanks for your contribution!

