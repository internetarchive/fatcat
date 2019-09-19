
## Fatcat Release Process

This roughly describes the process of pushing a new minor (or patch) release of
Fatcat to various targets.

The order of operations here hasn't been reviewed yet, this is mostly just a
checklist of things that need to happen. Eg, maybe the tag shouldn't be created
until after client libraries have been pushed and the project deployed
successfully.

- merge/push/deploy any outstanding patches ahead of the version bump
- update the version number in fatcat-openapi2.yml
- codegen the rust libraries
- edit both the `fatcat` and `fatcat-openapi` Cargo.toml files and update the
  version numbers. Make sure you commit the `fatcat-openapi` one before
  re-codegen or it will get clobbered (!)
- codegen python client; may need to hand-tweak setup.py and README changes
- edit `python_openapi_client/fatcat_openapi_client/__version__.py` and set the
  correct version number
- under `python`, wipe either the entire local `.venv` or just the
  `fatcat_openapi_client` library copy. quit any pipenv shells, and re-run
  `pipenv --install`.
- grep the repository with the old version number to see if anything was missed
  (you should expect at least CHANGELOG to match)
- run all the tests locally and commit everything
- update CHANGELOG.md in it's own commit
- push for remote CI to run, eg on a separate branch. don't proceed until CI
  runs successfully!
- after CI passes, locally merge to master, then tag the CHANGELOG commit with
  the new version number. sign the tag. this looks like `git tag v0.1.2 -s`
- push master and the tags to all notable git remotes
- deploy changes, remembering to rebuild rust and pipenv

Great! Now we can push new python package (`fatcat-openapi-client`):

- `rm -rf dist/`
- `python3 setup.py sdist bdist_wheel` 
- Upload to test.pypi.org first, check that things look ok:
  `python3 -m twine upload --repository-url https://test.pypi.org/legacy/ dist/*`
- Then real pypi.org:
  `python3 -m twine upload dist/*`

and rust (`fatcat-openapi`):

- `cd rust/fatcat-openapi/`
- try first with: `cargo publish --dry-run`
- then: `cargo publish`
