#!/bin/bash

# This script re-generates the fatcat API client (fatcat-openapi-client) from the
# swagger/openapi2 spec file, using automated tools ("codegen")

set -exu
set -o pipefail

OUTPUT=`pwd`/codegen-out
mkdir -p $OUTPUT
# Strip tags, so entire API is under a single class
# Also strip long part of description so it doesn't clutter source headers
cat ../fatcat-openapi2.yml | grep -v "TAGLINE$" | perl -0777 -pe "s/<\!-- STARTLONGDESCRIPTION -->.*<\!-- ENDLONGDESCRIPTION -->//s" > $OUTPUT/api.yml

docker run \
    -v $OUTPUT:/tmp/swagger/ \
    openapitools/openapi-generator-cli:v4.1.2 \
    generate \
    --generator-name python \
    --input-spec /tmp/swagger/api.yml \
    --output /tmp/swagger/ \
    --package-name=fatcat_openapi_client \
    -p packageVersion="0.5.0"

sudo chown -R `whoami`:`whoami` $OUTPUT
mkdir -p fatcat_openapi_client
cp -r $OUTPUT/fatcat_openapi_client/* fatcat_openapi_client
cp $OUTPUT/README.md README.md.new

# I don't know what they were thinking with this TypeWithDefault stuff, but it
# caused really gnarly config cross-contamination issues when running mulitple
# clients in parallel.
# See also: https://github.com/swagger-api/swagger-codegen/issues/9117
patch -p0 << END_PATCH
--- fatcat_openapi_client/configuration.py
+++ fatcat_openapi_client/configuration.py
@@ -37,7 +37,7 @@ class TypeWithDefault(type):
         cls._default = copy.copy(default)
 
 
-class Configuration(six.with_metaclass(TypeWithDefault, object)):
+class Configuration(object):
     """NOTE: This class is auto generated by OpenAPI Generator
 
     Ref: https://openapi-generator.tech
END_PATCH

# Another patch to fix nasty auth cross-contamination between instances of
# Configuration object.
patch -p0 << END_PATCH
--- fatcat_openapi_client/configuration.py
+++ fatcat_openapi_client/configuration.py
@@ -44,14 +44,11 @@ class Configuration(object):
     Do not edit the class manually.
 
     :param host: Base url
-    :param api_key: Dict to store API key(s)
-    :param api_key_prefix: Dict to store API prefix (e.g. Bearer)
     :param username: Username for HTTP basic authentication
     :param password: Password for HTTP basic authentication
     """
 
     def __init__(self, host="https://api.fatcat.wiki/v0",
-                 api_key={}, api_key_prefix={},
                  username="", password=""):
         """Constructor
         """
@@ -62,10 +59,10 @@ class Configuration(object):
         """Temp file folder for downloading files
         """
         # Authentication Settings
-        self.api_key = api_key
+        self.api_key = {}
         """dict to store API key(s)
         """
-        self.api_key_prefix = api_key_prefix
+        self.api_key_prefix = {}
         """dict to store API prefix (e.g. Bearer)
         """
         self.refresh_api_key_hook = None
END_PATCH

# these tests are basically no-ops
mkdir -p tests/codegen
cp -r $OUTPUT/test/* tests/codegen

# ooo, this makes me nervous
rm -rf $OUTPUT
