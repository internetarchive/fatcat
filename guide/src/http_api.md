# REST API

The fatcat HTTP API is mostly a classic REST CRUD (Create, Read, Update,
Delete) API, with a few twists.

A declarative specification of all API endpoints, JSON data models, and
response types is available in OpenAPI 2.0 format. Code generation tools are
used to generate both server-side type-safe endpoint routes and client-side
libraries. Auto-generated reference documentation is, for now, available at
<https://api.qa.fatcat.wiki>.

All API traffic is over HTTPS; there is no insecure HTTP endpoint, even for
read-only operations. To start, all endpoints accept and return only JSON
serialized content.

## Editgroups

All mutating entity operations (create, update, delete) accept an
`editgroup_id` query parameter. If the parameter isn't set, the editor's
"currently active" editgroup will be used, or a new editgroup will be created
from scratch. It's generally preferable to manually create an editgroup and use
the `id` in edit requests; the allows appropriate metadata to be set. The
"currently active" editgroup behavior may be removed in the future.

## Sub-Entity Expansion

To reduce the need for multiple GET queries when looking for common related
metadata, it is possible to include linked entities in responses using the
`expand` query parameter. For example, by default the `release` model only
includes an optional `container_id` field which points to a container entity.
If the `expand` parameter is set:

    https://api.qa.fatcat.wiki/v0/release/aaaaaaaaaaaaarceaaaaaaaaam?expand=container

Then the full container model will be included under the `container` field.
Multiple expand parameters can be passed, comma-separated.

## Authentication and Authorization

There are two editor types: bots and humans. Additionally, either type of
editor may have additional privileges which allow them to, eg, directly accept
editgroups (as opposed to submitting edits for review).

All mutating API calls (POST, PUT, DELETE HTTP verbs) require token-based
authentication using an HTTP Bearer token. If you can't generate such a token
from the web interface (because that feature hasn't been implemented), look for
a public demo token for experimentation, or ask an administrator for a token.

## QA Instance

The intent is to run a public "sandbox" QA instance of the catalog, using a
subset of the full catalog, running the most recent development branch of the
API specification. This instance can be used by developers for prototyping and
experimentation, though note that all data is periodically wiped, and this
endpoint is more likely to have bugs or be offline.

