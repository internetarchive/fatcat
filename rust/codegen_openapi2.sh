#!/bin/sh

cargo swagger ../fatcat-openapi2.yml fatcat-api-spec --docker-tag=v2.3.1
sudo chown `whoami`:`whoami` -R fatcat-api-spec
git checkout fatcat-api-spec/Cargo.toml

# Hack to handle "extra" JSON fields
sed -i 's/Object/serde_json::Value/g' fatcat-api-spec/src/models.rs
sed -i 's/extern crate uuid;/extern crate serde_json;\nextern crate uuid;/g' fatcat-api-spec/src/models.rs

cargo fmt
