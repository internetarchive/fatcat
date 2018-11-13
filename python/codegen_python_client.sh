#!/bin/bash

set -exu
set -o pipefail

OUTPUT=`pwd`/codegen-out
mkdir -p $OUTPUT
# Strip tags, so entire API is under a single class
cat ../fatcat-openapi2.yml | grep -v "TAGLINE$" | sed 's/\[https\]/\[http\]/g' > $OUTPUT/api.yml

docker run \
    -v $OUTPUT:/tmp/swagger/ \
    swaggerapi/swagger-codegen-cli:v2.3.1 \
    generate \
    --lang python \
    --input-spec /tmp/swagger/api.yml \
    --output /tmp/swagger/ \
    -DpackageName=fatcat_client

sudo chown -R `whoami`:`whoami` $OUTPUT
mkdir -p fatcat_client
mkdir -p tests/codegen_tests
cp -r $OUTPUT/fatcat_client/* fatcat_client
cp -r $OUTPUT/test/* tests/codegen_tests
cp $OUTPUT/README.md fatcat_client/README.md
# ooo, this makes me nervous
rm -rf $OUTPUT
