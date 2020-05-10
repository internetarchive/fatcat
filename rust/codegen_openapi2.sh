#!/bin/bash

set -exu
set -o pipefail

echo "Running openapi-generator..."
OUTPUT=`pwd`/fatcat-openapi
mkdir -p $OUTPUT
cat ../fatcat-openapi2.yml | grep -v "TAGLINE$" | perl -0777 -pe "s/<\!-- STARTLONGDESCRIPTION -->.*<\!-- ENDLONGDESCRIPTION -->//s" > $OUTPUT/api.yaml

export OPENAPI_GENERATOR_VERSION=5.0.0-SNAPSHOT
./openapi-generator-cli.sh \
    generate \
    --generator-name rust-server \
    --input-spec $OUTPUT/api.yaml \
    --output $OUTPUT \
    --package-name=fatcat-openapi \
    --generate-alias-as-model

cd fatcat-openapi

echo "Running cargo-fmt (first time)..."
cargo fmt

echo "Patching..."

# Hack to fix "release_date" (and "withdrawn_date") as Date, not DateTime
sed -i 's/_date: Option<chrono::DateTime<chrono::Utc>>/_date: Option<chrono::NaiveDate>/g' src/models.rs
sed -i 's/_date: Vec<chrono::DateTime<chrono::Utc>>/_date: Vec<chrono::NaiveDate>/g' src/models.rs
perl -0777 -pi -e 's/_date\.push\(\n\s+chrono::DateTime::<chrono::Utc>::from_str/_date\.push\(chrono::NaiveDate::from_str/gs' src/models.rs

# unnecessary duplicate copies of API spec
rm api.yaml
rm -rf api/

echo "Running cargo-fmt (final time)..."
cargo fmt

cp Cargo.toml Cargo.toml.codegen
git checkout Cargo.toml
diff Cargo.toml Cargo.toml.codegen && true
echo "Check the above diff for non-metadata changes to fatcat-openapi/Cargo.toml"
