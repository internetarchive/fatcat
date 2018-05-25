#!/bin/sh

cargo swagger fatcat-openapi2.yml fatcat-api --docker-tag=v2.3.1
sudo chown `whoami`:`whoami` -R fatcat-api
git checkout fatcat-api/Cargo.toml
cargo fmt
