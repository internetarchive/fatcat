
status: prototyping, side-project


Fatcat CLI Client
===================

    fatcat get release_awuvsvwrwzev7jcljyo34r6gem
    fatcat get --toml release_awuvsvwrwzev7jcljyo34r6gem

    fatcat search containers "elife"
    => pretty prints in terminal/interactive; JSON rows in ES schema for non-interactive
    => limit, offset

Editing commands:

    fatcat editgroup new

    fatcat editgroup list
    fatcat eg list

    fatcat update container_tupsi5ep7bhhraup4irzk6tpuy publisher="Taylor and Francis"
    => prints URL of revision, and mentioned editgroup id

    fatcat get container_tupsi5ep7bhhraup4irzk6tpuy --toml > wip.toml
    => --expand files, --hide as args or "expand==files"?
    # edit wip.toml
    fatcat update container_tupsi5ep7bhhraup4irzk6tpuy --toml < wip.toml

    fatcat delete release_awuvsvwrwzev7jcljyo34r6gem

    fatcat create release < some_release.json

    fatcat create release --bulk < release_set.json
    => makes editgroups automatically

    fatcat update container --bulk < container_set.json
    => or, "fatcat update containers"?

    fatcat edit container_tupsi5ep7bhhraup4irzk6tpuy
    => or "update"?
    => container is fetched, $EDITOR is opened with JSON or TOML format, on
       save tool validates/prettyprints (a diff?) and asks whether to make edit

Other:

    fatcat download file_wuyi7kl4njehpg3yyngaqxcqfa

    fatcat status
    => account info?
    => current editgroup?

For editgroup ergonomics, entity mutating commands (which require an
editgroup), tool should fetch recent open editgroups for the user, filter to
those created with the CLI tool, and use the most recent. Behavior and be tuned
to be more or less conservative (let's start conservative).

At least for prototyping, configure via environment variables (eg, API token,
specifying alternative API endpoints).

Clever but already taken names:

- `fcc` (FatCat Client) is a fortran compiler. Also the name of the USA Federal
  Communications Commission (a notable radio/internet/phone regulator)
- `fc` (FatCat) is a bash built-in.

Argument conventions:

    ':'     Lookup specifier for entity (eg, external identifier like `doi:10.123/abc`)

    '='     Assign field to value in create or update contexts. Non-string
            values often can be infered by field type

    ':='    Assign field to non-string value in create or update contexts

Small details (mostly TODO):

- pass through API warning headers to stderr

## Similar Tools / Interfaces

### `httpie`

    ':'     HTTP headers

    '=='    URL parameters

    '='     data fields serialized into JSON, or as form data

    ':='    non-string JSON data (eg, true (boolean), 42 (number), or lists)

    '@'     Form field

Output goes to stdout (pretty-printed), unless specified to `--download / -d`),
in which case output file is infered, or `--output` sets it explicitly.

### Internet Archive `ia` Tool

TODO

#### `jq` / `toml`

Rust `toml-cli` has a small DSL for making mutations.

#### `ripgrep`

## More Ideas

Some sort of pretty-printer for work/release/file structure. Eg, like `tree`
unix command. See `ptree` rust crate.

## Implementation

Rust libraries:

- `toml`
- `toml_edit`: format-preserving TOML loading/mutating/serializing
- `termcolor`
- `atty` ("are we connected to a terminal")
- `tabwriter` for tabular CLI output
- `human-panic`
- `synect` for highlighting
- `exitcode`

