#!/usr/bin/env bash

set -e -u -o pipefail

PIPENV_VENV_IN_PROJECT=true pipenv install --dev --deploy

rm -f fatcat-python.tar.gz
tar czf fatcat-python.tar.gz .venv *.py fatcat_tools fatcat_web Pipfile* *.ini README*
