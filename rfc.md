
fatcat is a half-baked idea to build an open, independent, collaboratively
editable bibliographic database of most written works.

## Technical Architecture

The canonical backend datastore would be a very large transactional SQL server.
A relatively simple and stable back-end daemon would expose an API (could be
REST, GraphQL, gRPC, etc). As little "application logic" as possible would be
embedded in this back-end; as much as possible would be pushed to bots which
could be authored and operated by anybody. A separate web interface project
would talk to the API backend and could be developed more rapidly.

## Editing Workflow and Bots

Both human editors and bots would have edits go through the same API, with
humans using either the default web interface or arbitrary integrations or
client software.

The usual workflow would be to create edits (or creations, merges, deletions)
to individual entities one at a time, all under a single "edit group" of
related edits (eg, correcting authorship info for multiple works related to a
single author). When ready, the editor would "submit" the edit group for
review. During the review period, humans could vote (or veto/approve if they
have higher permissions), and bots can perform automated checks. During this
period the editor can make tweaks if necessary. After some fixed time period
(72 hours?) with no changes and no blocking issues, the edit group would be
auto-accepted, if no auto-resolvable merge-conflicts have arisen. This process
balances editing labor (reviews are easy, but optional) against quality
(cool-down period makes it easier to detect and prevent spam or out-of-control
bots). Advanced permissions could allow some trusted human and bot editors to
push through edits more rapidly.

Bots would need to be tuned to have appropriate edit group sizes (eg, daily
batches, instead of millions of works in a single edit) to make human QA and
reverts possible.

Data progeny and citation would be left to the edit history. In the case of
importing external databases, the expectation would be that special-purpose
bot accounts would be used. Human editors would leave edit messages to clarify
their sources.

A style guide (wiki), chat room, and discussion forum would be hosted as
separate stand-alone services for editors to propose projects and debate
process or scope changes. It would be best if these could use federated account
authorization (oauth?) to have consistent account IDs across mediums.

## Itentifiers

A fixed number of first class "entities" would be definied, with common
behavior and schema layouts. These would all be semantic entities like "work",
"edition", "container", and "person".

fatcat identifiers would be semanticly meaningless fixed length random numbers,
usually represented in case-insensitive base32 format. Each entity type would
have it's own identifier namespace. Eg, 96 bit identifiers would have 26
characters and look like:

    fcwork_rzga5b9cd7efgh04iljk

As a URL:

    https://fatcat.org/work/rzga5b9cd7efgh04iljk

A 64 bit namespace is probably plenty though:

    fcwork_rzga5b9cd7efg
    https://fatcat.org/work/rzga5b9cd7efg

The idea would be to only have fatcat identifiers be used to interlink between
databases, *not* to supplant DOIs, ISBNs, handle, ARKs, and other "registered"
persistant identifiers.

## Entities and Internal Schema

Internally, identifiers would be lightweight pointers to actual metadata
objects, which can be thought of as "versions". The metadata objects themselves
would be immutable once commited; the edit process is one of creating new
objects and, if the edit is approved, pointing the identifier to the new
version. Entities would reference between themselves by identifier.

Edit objects represent a change to a single entity; edits get batched together
into edit groups (like "commits" and "pull requests" in git parlance).

SQL tables would probably look something like the following, though be specific
to each entity type (eg, there would be an actual `work_revision` table, but
not an actual `entity_revision` table):

    entity_id
        uuid
        current_revision

    entity_revision
        entity_id (bi-directional?)
        previous: entity_revision or none
        state: normal, redirect, deletion
        redirect_entity_id: optional
        extra: json blob
        edit_id

    edit
        mutable: boolean
        edit_group
        editor

    edit_group

Additional type-specific columns would hold actual metadata. Additional tables
(which would reference both `entity_revision` and `entity_id` foreign keys as
appropriate) would represent things like external identifiers, ordered
author/work relationships, citations between works, etc. Every revision of an
entity would require duplicating all of these associated rows, which could end
up being a large source of inefficiency, but is necessary to represent the full
history of an object.

## Scope

Want the "scholarly web": the graph of works that cite other works. Certainly
every work that is cited more than once and every work that both cites and is
cited; "leaf nodes" and small islands might not be in scope.

Focusing on written works, with some exceptions. Expect core media (for which we would pursue "completeness") to be:

    journal articles
    books
    conference proceedings
    technical memos
    dissertations

Probably in scope:

    reports
    magazine articles
    published poetry
    essays
    government documents
    conference
    presentations (slides, video)

Probably not:

    patents
    court cases and legal documents
    manuals
    datasheets
    courses

Definitely not:

    audio recordings
    tv show episodes
    musical scores
    advertisements

