#!/bin/bash

# Run this script from the ../rust/ directory, only once.

CMD_PATH="./target/debug"

$CMD_PATH/fatcat-auth create-editor --admin --bot crossref-bot > /dev/null
$CMD_PATH/fatcat-auth create-editor --admin --bot pubmed-bot > /dev/null
$CMD_PATH/fatcat-auth create-editor --admin --bot datacite-bot > /dev/null
$CMD_PATH/fatcat-auth create-editor --admin --bot orcid-bot > /dev/null
$CMD_PATH/fatcat-auth create-editor --admin --bot journal-metadata-bot > /dev/null
$CMD_PATH/fatcat-auth create-editor --admin --bot sandcrawler-bot > /dev/null

echo -n "FATCAT_AUTH_WORKER_CROSSREF="
$CMD_PATH/fatcat-auth create-token `$CMD_PATH/fatcat-auth list-editors | grep crossref-bot | cut -f1`
echo -n "FATCAT_AUTH_WORKER_PUBMED="
$CMD_PATH/fatcat-auth create-token `$CMD_PATH/fatcat-auth list-editors | grep pubmed-bot | cut -f1`
echo -n "FATCAT_AUTH_WORKER_DATACITE="
$CMD_PATH/fatcat-auth create-token `$CMD_PATH/fatcat-auth list-editors | grep datacite-bot | cut -f1`
echo -n "FATCAT_AUTH_WORKER_ORCID="
$CMD_PATH/fatcat-auth create-token `$CMD_PATH/fatcat-auth list-editors | grep orcid-bot | cut -f1`
echo -n "FATCAT_AUTH_WORKER_JOURNAL_METADATA="
$CMD_PATH/fatcat-auth create-token `$CMD_PATH/fatcat-auth list-editors | grep journal-metadata-bot | cut -f1`
echo -n "FATCAT_AUTH_SANDCRAWLER="
$CMD_PATH/fatcat-auth create-token `$CMD_PATH/fatcat-auth list-editors | grep sandcrawler-bot | cut -f1`
