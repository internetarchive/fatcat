#!/bin/bash

set -e -u -o pipefail

# Run this script from the ../rust/ directory, only once.

CMD_PATH="./target/debug"

$CMD_PATH/fatcat-auth create-editor --admin --bot crossref-bot > /dev/null
$CMD_PATH/fatcat-auth create-editor --admin --bot pubmed-bot > /dev/null
$CMD_PATH/fatcat-auth create-editor --admin --bot datacite-bot > /dev/null
$CMD_PATH/fatcat-auth create-editor --admin --bot orcid-bot > /dev/null
$CMD_PATH/fatcat-auth create-editor --admin --bot journal-metadata-bot > /dev/null
$CMD_PATH/fatcat-auth create-editor --admin --bot sandcrawler-bot > /dev/null
$CMD_PATH/fatcat-auth create-editor --admin --bot crawl-bot > /dev/null
$CMD_PATH/fatcat-auth create-editor --admin --bot archive-org-bot > /dev/null
$CMD_PATH/fatcat-auth create-editor --admin --bot jstor-bot > /dev/null
$CMD_PATH/fatcat-auth create-editor --admin --bot jalc-bot > /dev/null
$CMD_PATH/fatcat-auth create-editor --admin --bot arxiv-bot > /dev/null

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
echo -n "FATCAT_AUTH_WORKER_CRAWL="
$CMD_PATH/fatcat-auth create-token `$CMD_PATH/fatcat-auth list-editors | grep crawl-bot | cut -f1`
echo -n "FATCAT_AUTH_WORKER_ARCHIVE_ORG="
$CMD_PATH/fatcat-auth create-token `$CMD_PATH/fatcat-auth list-editors | grep archive-org-bot | cut -f1`
echo -n "FATCAT_AUTH_WORKER_JSTOR="
$CMD_PATH/fatcat-auth create-token `$CMD_PATH/fatcat-auth list-editors | grep jstor-bot | cut -f1`
echo -n "FATCAT_AUTH_WORKER_JALC="
$CMD_PATH/fatcat-auth create-token `$CMD_PATH/fatcat-auth list-editors | grep jalc-bot | cut -f1`
echo -n "FATCAT_AUTH_WORKER_ARXIV="
$CMD_PATH/fatcat-auth create-token `$CMD_PATH/fatcat-auth list-editors | grep arxiv-bot | cut -f1`

echo -n "FATCAT_AUTH_WEBFACE="
$CMD_PATH/fatcat-auth create-token `$CMD_PATH/fatcat-auth list-editors | grep webface-bot | cut -f1`
echo -n "FATCAT_API_AUTH_TOKEN="
$CMD_PATH/fatcat-auth create-token `$CMD_PATH/fatcat-auth list-editors | grep 'admin' | grep -v 'editor_id' | cut -f1`
