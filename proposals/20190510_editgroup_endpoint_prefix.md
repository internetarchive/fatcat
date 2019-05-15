
# Editgroup API Endpoint Prefixes

In summary, change the API URL design such that entity mutations (create,
update, delete) happen under the URL path of an editgroup, with the
`editgroup_id` as a path component, instead of being REST actions on the
canonical URL with `editgroup_id` as a query parameter.

This is a fairly large/systemic breaking change, though it should not change
code *structure* much (just argument order/passing), and requires no SQL
changes. It may remove corner-case features (like non-auto match operations?).


## Renamed API URLs

For all entity types:

    /editgroup/<editgroup_id>/release/<ident>
        PUT: update_release(editgroup_id, ident, entity)
        DELETE: delete_release(editgroup_id, ident)

    /editgroup/<editgroup_id>/release
        POST: create_release(editgroup_id, entity)

    /editgroup/auto/release/batch
        POST: create_release_auto_batch(editgroup, [entity])
            => actually new ReleaseAutoBatch(editgroup, entity_list) body

    /editgroup/<editgroup_id>/release/edit/<edit_uuid>
        DELETE: delete_release_edit(editgroup_id, edit_uuid)


## New Webface URLs

    /editgroup/<editgroup_id>/release/<ident>
        GET: shows the (potentially WIP) entity as of this editgroup

    Some way to delete an edit from an editgroup


## Future: New API URLs

Will not actually implement these for now.

For all entity types:

    /editgroup/<editgroup_id>/release/<ident>
        GET: get_editgroup_release(editgroup_id, ident) => entity revision

    /editgroup/<editgroup_id>/release/batch
        POST: create_release_batch([entity])


## SCRATCH

container
creator
file
fileset
webcapture
release
work

per entity:
x auto_batch type
x create_* path and editgroup_id parameter
x   delete old parameter
x batch:
x   url
x   operationId
x   new single parameter
x   return type
x put/delete
x   new url section
x   remove old editgroup_id parameters (2x)
x delete edit
x   new url section
x   remove old edit_id

