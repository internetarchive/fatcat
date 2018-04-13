
## Cookbook

To create a new works:

    login/create account

    match/lookup on first work; if found, modify not create
    TODO: match/lookup on files?

    new edit group (under account; wip)
    new edit (under group)

    match/lookup on all creators
    match/lookup on all refs
      TODO: match/lookup on files?
    match/lookup container

    new work_rev (linked to edit)
    new work_ident (linked to rev; not-live)
    contributor stubs/links
    new release_rev (linked to work_ident)
    new release_ident (linked to rev; not-live)
    contributor stubs/links
    ref stubs/links
    new file_rev
    new file_ident

    set edit group state to "review"

    set edit group state to "accepted"
    set all ident flags to "live"


To edit, eg, a contributor:

    new edit group (under account; wip)
    new edit (under group)

    create contributor_rev row (and relationship rows)

    set edit group state to "review"

    set edit group state to "accepted"
    point ident row to new rev


Accept edit group:

    for each edit:
      update entity ident state (activate, redirect, delete)
      ? mark revision/edit as immutable
      ? append log/changelog row
    update edit group state

## Entity Schema

Each entity type has tables:

    _rev
      core representation of a version of the entity

    _ident
      persistant, external identifier
      allows merging, unmerging, stable cross-entity references

    _edit
      represents change metadata for a single change to one ident
      needed because an edit alwasy changes ident, but might not change rev

    _log
      history of when edits were actually applied
      allows fast lookups of history of an entity (ident)
      unnecessary if we keep a log of edit group accepts?
      punt on this for now

## Entity States

    wip (not live; not redirect; has rev)
      activate
    active (live; not redirect; has rev)
      redirect
      delete
    redirect (live; redirect; rev or not)
      split
      delete
    deleted (live; not redirect; no rev)
      redirect
      activate

    "wip redirect" or "wip deleted" are invalid states

