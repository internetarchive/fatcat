#!/bin/sh

echo "Running cargo-swagger..."
cargo swagger ../fatcat-openapi2.yml fatcat-openapi --docker-tag=v2.3.1
sudo chown `whoami`:`whoami` -R fatcat-openapi
git checkout fatcat-openapi/Cargo.toml

echo "Patching..."

# Hacks to fix crate naming
sed -i 's/fatcat::/fatcat_openapi::/g' fatcat-openapi/examples/client.rs fatcat-openapi/examples/server.rs fatcat-openapi/examples/server_lib/server.rs
sed -i 's/extern crate fatcat;/extern crate fatcat_openapi;/g' fatcat-openapi/examples/client.rs fatcat-openapi/examples/server.rs fatcat-openapi/examples/server_lib/server.rs

# Squash many warnings ("error: unused doc comment")
sed -i 's/\/\/\/ Create Mime objects/\/\/ Create Mime objects/g' fatcat-openapi/src/mimetypes.rs

# 2019 edition crate imports
sed -i 's/use models;/use crate::models;/g' fatcat-openapi/src/client.rs fatcat-openapi/src/server.rs fatcat-openapi/src/models.rs
sed -i 's/use mimetypes;/use crate::mimetypes;/g' fatcat-openapi/src/client.rs fatcat-openapi/src/server.rs
sed -i 's/use {/use crate::{/g' fatcat-openapi/src/client.rs fatcat-openapi/src/server.rs

# weird broken auth example
sed -i 's/chain.link_before(AllowAllMiddleware/\/\/chain.link_before(AllowAllMiddleware/g' fatcat-openapi/examples/server.rs

# Hack to handle "extra" JSON fields
sed -i 's/Object/serde_json::Value/g' fatcat-openapi/src/models.rs
sed -i 's/extern crate uuid;/extern crate serde_json;\nextern crate uuid;/g' fatcat-openapi/src/models.rs

# Hack to fix "release_date" as Date, not DateTime
sed -i 's/release_date: Option<chrono::DateTime<chrono::Utc>>/release_date: Option<chrono::NaiveDate>/g' fatcat-openapi/src/models.rs
sed -i 's/withdrawn_date: Option<chrono::DateTime<chrono::Utc>>/withdrawn_date: Option<chrono::NaiveDate>/g' fatcat-openapi/src/models.rs

# Hack to optionally log unknown request fields (not actually needed)
#sed -i 's/\(response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));\)/\1 warn!("unknown fields in request body: {:?}", unused_elements);/g' fatcat-openapi/src/server.rs

# Hack to require that optional params parse correctly (boolean, integer, datetime)
# If we reformat, this this should basically go from, eg:
#    .and_then(|x| x.parse::<i64>()
#    .ok());
# To:
#    .and_then(|x| Some(x.parse::<i64>()))
#    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
#    .map_err(|x| Response::with((status::InternalServerError, "unparsable query parameter (expected integer)".to_string())))?;
sed -i 's/.and_then(|x| x.parse::<i64>().ok());$/.and_then(|x| Some(x.parse::<i64>())).map_or_else(|| Ok(None), |x| x.map(|v| Some(v))).map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected integer)".to_string())))?;/g' fatcat-openapi/src/server.rs
sed -i 's/.and_then(|x| x.parse::<bool>().ok());$/.and_then(|x| Some(x.to_lowercase().parse::<bool>())).map_or_else(|| Ok(None), |x| x.map(|v| Some(v))).map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected boolean)".to_string())))?;/g' fatcat-openapi/src/server.rs
sed -i 's/.and_then(|x| x.parse::<chrono::DateTime<chrono::Utc>>().ok());$/.and_then(|x| Some(x.parse::<chrono::DateTime<chrono::Utc>>())).map_or_else(|| Ok(None), |x| x.map(|v| Some(v))).map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected UTC datetime in ISO\/RFC format)".to_string())))?;/g' fatcat-openapi/src/server.rs

# unnecessary duplicate copies of API spec
rm fatcat-openapi/api.yaml
rm -rf fatcat-openapi/api/

cd fatcat-openapi

echo "Running cargo-fix (slow)..."
cargo fix --allow-dirty

echo "Running cargo-fmt..."
cargo fmt
