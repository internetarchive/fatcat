#!/bin/sh

cargo swagger ../fatcat-openapi2.yml fatcat-api-spec --docker-tag=v2.3.1
sudo chown `whoami`:`whoami` -R fatcat-api-spec
git checkout fatcat-api-spec/Cargo.toml

# Hack to handle "extra" JSON fields
sed -i 's/Object/serde_json::Value/g' fatcat-api-spec/src/models.rs
sed -i 's/extern crate uuid;/extern crate serde_json;\nextern crate uuid;/g' fatcat-api-spec/src/models.rs

# Hack to fix "release_date" as Date, not DateTime
sed -i 's/release_date: Option<chrono::DateTime<chrono::Utc>>/release_date: Option<chrono::NaiveDate>/g' fatcat-api-spec/src/models.rs
sed -i 's/withdrawn_date: Option<chrono::DateTime<chrono::Utc>>/withdrawn_date: Option<chrono::NaiveDate>/g' fatcat-api-spec/src/models.rs

# Hack to optionally log unknown request fields (not actually needed)
#sed -i 's/\(response.headers.set(Warning(format!("Ignoring unknown fields in body: {:?}", unused_elements)));\)/\1 warn!("unknown fields in request body: {:?}", unused_elements);/g' fatcat-api-spec/src/server.rs

# Hack to require that optional params parse correctly (boolean, integer, datetime)
# If we reformat, this this should basically go from, eg:
#    .and_then(|x| x.parse::<i64>()
#    .ok());
# To:
#    .and_then(|x| Some(x.parse::<i64>()))
#    .map_or_else(|| Ok(None), |x| x.map(|v| Some(v)))
#    .map_err(|x| Response::with((status::InternalServerError, "unparsable query parameter (expected integer)".to_string())))?;
sed -i 's/.and_then(|x| x.parse::<i64>().ok());$/.and_then(|x| Some(x.parse::<i64>())).map_or_else(|| Ok(None), |x| x.map(|v| Some(v))).map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected integer)".to_string())))?;/g' fatcat-api-spec/src/server.rs
sed -i 's/.and_then(|x| x.parse::<bool>().ok());$/.and_then(|x| Some(x.to_lowercase().parse::<bool>())).map_or_else(|| Ok(None), |x| x.map(|v| Some(v))).map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected boolean)".to_string())))?;/g' fatcat-api-spec/src/server.rs
sed -i 's/.and_then(|x| x.parse::<chrono::DateTime<chrono::Utc>>().ok());$/.and_then(|x| Some(x.parse::<chrono::DateTime<chrono::Utc>>())).map_or_else(|| Ok(None), |x| x.map(|v| Some(v))).map_err(|x| Response::with((status::BadRequest, "unparsable query parameter (expected UTC datetime in ISO\/RFC format)".to_string())))?;/g' fatcat-api-spec/src/server.rs

cargo fmt
