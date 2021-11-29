#!/bin/bash

set -eu

# sudo mkdir /var/run/fatcat-web
# sudo chown bnewbold:bnewbold /var/run/fatcat-web/

FATCAT_WEB_DIR=python
cd $FATCAT_WEB_DIR

uwsgi \
    -s /var/run/fatcat-web/uwsgi.sock \
    --manage-script-name --mount \
    --plugin python3 \
    --virtualenv .venv \
    --mount /:fatcat:app
