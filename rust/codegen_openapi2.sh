#!/bin/sh

cargo swagger fatcat-openapi2.yml fatcat-api --docker-tag=v2.3.1
sudo chown `whoami`:`whoami` -R fatcat-api
git checkout fatcat-api/Cargo.toml

# Hack to handle "extra" JSON fields
sed -i 's/Object/serde_json::Value/g' fatcat-api/src/models.rs
sed -i 's/extern crate uuid;/extern crate serde_json;\nextern crate uuid;/g' fatcat-api/src/models.rs

cargo fmt
