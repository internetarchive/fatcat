
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

## Simplifications

In early development, we'll make at least the following simplifications:

- authentication (authn and authz) are not enforced and don't have user
  interfaces. actual authentication will be eased in via a microservice and/or
  oauth to gitlab/github/orcid.org
- "extra" metadata is stored in-entity as JSONB. In the future this might be
  broken out to a separate table
- libraries won't be vendored; in the future they will be via a git submodule


## OpenAPI Code Generation

Install the go-swagger tool:

    go get -u github.com/go-swagger/go-swagger/cmd/swagger


    swagger generate server -A Fatcat -f fatcat-openapi2.yml
