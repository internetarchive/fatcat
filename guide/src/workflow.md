# Workflow

## Basic Editing Workflow and Bots

Both human editors and bots should have edits go through the same API, with
humans using either the default web interface, integration, or client
software.

The normal workflow is to create edits (or updates, merges, deletions) on
individual entities. Individual changes are bundled into an "edit group" of
related edits (eg, correcting authorship info for multiple works related to a
single author). When ready, the editor "submits" the edit group for
review. During the review period, human editors vote and bots can perform
automated checks. During this period the editor can make tweaks if necessary.
After some fixed time period (72 hours?) with no changes and no blocking
issues, the edit group would be auto-accepted if no merge conflicts have
be created by other edits to the same entities. This process balances editing
labor (reviews are easy, but optional) against quality (cool-down period makes
it easier to detect and prevent spam or out-of-control bots). More
sophisticated roles and permissions could allow some certain humans and bots to
push through edits more rapidly (eg, importing new works from a publisher API).

Bots need to be tuned to have appropriate edit group sizes (eg, daily batches,
instead of millions of works in a single edit) to make human QA review and
reverts manageable.

Data provenance and source references are captured in the edit metadata, instead
of being encoded in the entity data model itself. In the case of importing
external databases, the expectation is that special-purpose bot accounts
are be used, and tag timestamps and external identifiers in the edit metadata.
Human editors can leave edit messages to clarify their sources.

A [style guide](./style_guide.md) and discussion forum are intended to be be
hosted as separate stand-alone services for editors to propose projects and
debate process or scope changes. These services should have unified accounts
and logins (OAuth?) for consistent account IDs across all services.

