#!/bin/sh

set -ex

OUTPUT=`pwd`/codegen-out
mkdir -p $OUTPUT
cp ../rust/fatcat-openapi2.yml $OUTPUT/api.yml

docker run \
    -v $OUTPUT:/tmp/swagger/ \
    swaggerapi/swagger-codegen-cli:v2.3.1 \
    generate \
    --lang python \
    --input-spec /tmp/swagger/api.yml \
    --output /tmp/swagger/ \
    -DpackageName=fatcat_client

sudo chown -R `whoami`:`whoami` $OUTPUT
cp -r $OUTPUT/fatcat_client fatcat_client
cp -r $OUTPUT/test tests/fatcat_client
