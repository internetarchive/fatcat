
status: implemented

TOML Editing of Entity Metadata
===============================

Goal is to enable full-power editing through the web interface, of the raw
entity schema and "extra" metadata, for all entity types.

A side-effect of this should be enabling redirect editing between entities, as
well as "undeleting" or other state transitions (other than deletion).

Plan:

- find and add a toml transform library to pipenv deps. preferably with good
  human-readable parsing errors
- TOML/JSON transform helpers, with tests
- implement generic TOML entity editing view (HTML)
- implement generic TOML entity editing endpoints

Some metadata fields are removed before displaying TOML to edit. For example,
the ident, revision, and redirect fields for 'active' entities. It should still
be possible to do redirects by entering only the redirect field in the TOML
form.

## UI Integration

For existing edit forms, add a link to the "advanced" editing option.

For endpoints without a form-based option (yet), do an HTTP redirect to the
TOML editing option.

## New Webface Views

`/<entity>/create/toml`
    GET: display template to be filled in
    POST: form submit for creation
`/<entity>/<ident>/edit/toml`
    GET: transform entity for editing
    POST: form submit for edit
`/editgroup/<editgroup_id>/<entity>/<ident>/edit/toml`
    GET: transform entity for editing
    POST: form submit for edit
