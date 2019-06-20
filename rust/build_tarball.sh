#!/usr/bin/env bash

set -e -u -o pipefail

cargo build --release
mkdir -p ./bin/
cp target/release/{fatcatd,fatcat-auth,fatcat-export} bin

rm -f fatcat-rust.tar.gz
tar czf fatcat-rust.tar.gz bin migrations README.md example.env
