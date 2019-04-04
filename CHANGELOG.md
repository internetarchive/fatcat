
# CHANGELOG

Notable technical changes to the fatcat schemas, Rust API server, Python web
interface, and Python client library will be recorded here.

Intent is to follow semantic versioning, with all components kept in lock-step,
but note that everything is pre-1.0, so there aren't any actual backwards
compatibility guarantees. The API prefix (eg, `/v0/`) will probably roughly
track major version number, but TBD.

See also:

- [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
- [Semantic Versioning](https://semver.org/spec/v2.0.0.html)

## [Unreleased]

- expanded example entities in SQL schema
- updated rust (cargo) dependencies; depend on rust 1.32+
- many tweaks to webface and guide
- basic release, container, file webface editing
- basic editgroup annotation and control (submit/accept) in webface
- CSL/BibTeX citation endpoints

## [0.2.0] - 2019-02-05

First tagged release, and start of notable change tracking.
