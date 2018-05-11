
This folder contains source for the fatcat API daemon ("fatcatd"), written in
golang.


## Structure

fatcatd is essentially just glue between two declarative schemas:

- a postgres-flavor SQL database schema
- an OpenAPI/Swagger REST API definition

## Dev Setup

- postgres 9.6+ running locally
- golang environment configured
    - https://github.com/golang/dep
- checkout (or symlink) this repo to $GOPATH/src/git.archive.org/bnewbold/fatcat
- dep ensure

On debian/ubuntu:

    sudo -u postgres createuser -s `whoami`
    createdb -O `whoami` fatcat
    psql fatcat -f fatcat-schema.sql

Build with:

    go build ./cmd/*/

## Simplifications

In early development, we'll make at least the following simplifications:

- authentication (authn and authz) are not enforced and don't have user
  interfaces. actual authentication will be eased in via a microservice and/or
  oauth to gitlab/github/orcid.org
- "extra" metadata is stored in-entity as JSONB. In the future this might be
  broken out to a separate table
- libraries won't be vendored; in the future they will be via a git submodule


## OpenAPI Code Generation

    cat fatcat-openapi2.yml | python3 -c 'import sys, yaml, json; json.dump(yaml.load(sys.stdin), sys.stdout, indent=4)' > fatcat-openapi2.json

Install the go-swagger tool:

    go get -u github.com/go-swagger/go-swagger/cmd/swagger


"Simple" server:

    swagger generate server -A Fatcat -f fatcat-openapi2.json

"Custom" server:

    swagger generate server -A fatcat -f ./fatcat-openapi2.json --exclude-main -t gen

## Future

Could refactor the API side to use gRPC and grpc-gateway instead of swagger
(which would result in a compatible REST JSON interface). For faster bots and
import, and lower latency between webface and backend.

