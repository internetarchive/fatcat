#!/bin/bash

set -xeo pipefail

cat fatcat-openapi2.yml | python3 -c 'import sys, yaml, json; json.dump(yaml.load(sys.stdin), sys.stdout, indent=4)' > fatcat-openapi2.json
swagger generate server -A fatcat -f ./fatcat-openapi2.json --exclude-main -t gen
rm fatcat-openapi2.json
