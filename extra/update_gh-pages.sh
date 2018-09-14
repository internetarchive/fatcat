#!/bin/bash

set -e -u -o pipefail

cd rust
cargo doc
mkdir -p /tmp/fatcat-ghpages
cp -r release/doc/fatcat release/doc/fatact_api_spec /tmp/fatcat-ghpages
cd ..
git checkout gh-pages
mv /tmp/fatcat-ghpages/* .
git add -u fatcat fatact_api_spec
git commit -m "updating rendered manpage for github docs" || true
git checkout master
rm -r /tmp/fatcat-ghpages

echo "DONE"
