#!/bin/bash

# This script re-generates the fatcat API client (fatcat-openapi-client) from the
# swagger/openapi2 spec file, using automated tools ("codegen")

set -exu
set -o pipefail

OUTPUT=`pwd`/codegen-out
mkdir -p $OUTPUT
# Strip tags, so entire API is under a single class
# Also strip long part of description so it doesn't clutter source headers
cat ../fatcat-openapi2.yml | grep -v ' pattern: ' | perl -0777 -pe "s/<\!-- STARTLONGDESCRIPTION -->.*<\!-- ENDLONGDESCRIPTION -->//s" > $OUTPUT/api.yml

docker run \
    -v $OUTPUT:/tmp/swagger/ \
    openapitools/openapi-generator-cli:v6.0.1 \
    generate \
    --generator-name python \
    --input-spec /tmp/swagger/api.yml \
    --output /tmp/swagger/ \
    --package-name=fatcat_openapi_client \
    -p packageVersion="0.5.1"

sudo chown -R `whoami`:`whoami` $OUTPUT
mkdir -p fatcat_openapi_client
cp -r $OUTPUT/fatcat_openapi_client/* fatcat_openapi_client
cp $OUTPUT/README.md README.md.new

# these tests are basically no-ops
mkdir -p tests/codegen
cp -r $OUTPUT/test/* tests/codegen

# ooo, this makes me nervous
rm -rf $OUTPUT
