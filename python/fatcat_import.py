#!/usr/bin/env python3

import argparse
import os
import sys

import sentry_sdk

from fatcat_tools import authenticated_api
from fatcat_tools.importers import (
    ARABESQUE_MATCH_WHERE_CLAUSE,
    ArabesqueMatchImporter,
    ArxivRawImporter,
    Bs4XmlFileListPusher,
    Bs4XmlFilePusher,
    Bs4XmlLargeFilePusher,
    Bs4XmlLinesPusher,
    ChoculaImporter,
    CrossrefImporter,
    DataciteImporter,
    DblpContainerImporter,
    DblpReleaseImporter,
    DoajArticleImporter,
    FileMetaImporter,
    FilesetImporter,
    GrobidMetadataImporter,
    IngestFileResultImporter,
    IngestFilesetResultImporter,
    IngestWebResultImporter,
    JalcImporter,
    JournalMetadataImporter,
    JsonLinePusher,
    JstorImporter,
    KafkaBs4XmlPusher,
    KafkaJsonPusher,
    LinePusher,
    MatchedImporter,
    OrcidImporter,
    PubmedImporter,
    SavePaperNowFileImporter,
    SavePaperNowFilesetImporter,
    SavePaperNowWebImporter,
    ShadowLibraryImporter,
    SqlitePusher,
)

# Yep, a global. Gets DSN from `SENTRY_DSN` environment variable
sentry_client = sentry_sdk.init()


def run_crossref(args: argparse.Namespace) -> None:
    fci = CrossrefImporter(
        args.api,
        args.issn_map_file,
        edit_batch_size=args.batch_size,
        bezerk_mode=args.bezerk_mode,
    )
    if args.kafka_mode:
        KafkaJsonPusher(
            fci,
            args.kafka_hosts,
            args.kafka_env,
            "api-crossref",
            "fatcat-{}-import-crossref".format(args.kafka_env),
            consume_batch_size=args.batch_size,
        ).run()
    else:
        JsonLinePusher(fci, args.json_file).run()


def run_jalc(args: argparse.Namespace) -> None:
    ji = JalcImporter(args.api, args.issn_map_file)
    Bs4XmlLinesPusher(ji, args.xml_file, "<rdf:Description").run()


def run_arxiv(args: argparse.Namespace) -> None:
    ari = ArxivRawImporter(args.api, edit_batch_size=args.batch_size)
    if args.kafka_mode:
        KafkaBs4XmlPusher(
            ari,
            args.kafka_hosts,
            args.kafka_env,
            "oaipmh-arxiv",
            "fatcat-{}-import-arxiv".format(args.kafka_env),
        ).run()
    else:
        if args.xml_file == sys.stdin:
            print("note: reading from stdin", file=sys.stderr)
        Bs4XmlFilePusher(ari, args.xml_file, "record").run()


def run_pubmed(args: argparse.Namespace) -> None:
    pi = PubmedImporter(
        args.api,
        args.issn_map_file,
        edit_batch_size=args.batch_size,
        do_updates=args.do_updates,
        lookup_refs=(not args.no_lookup_refs),
    )
    if args.kafka_mode:
        KafkaBs4XmlPusher(
            pi,
            args.kafka_hosts,
            args.kafka_env,
            "ftp-pubmed",
            "fatcat-{}-import-pubmed".format(args.kafka_env),
        ).run()
    else:
        Bs4XmlLargeFilePusher(
            pi,
            args.xml_file,
            ["PubmedArticle"],
        ).run()


def run_jstor(args: argparse.Namespace) -> None:
    ji = JstorImporter(args.api, args.issn_map_file, edit_batch_size=args.batch_size)
    Bs4XmlFileListPusher(ji, args.list_file, "article").run()


def run_orcid(args: argparse.Namespace) -> None:
    foi = OrcidImporter(args.api, edit_batch_size=args.batch_size)
    JsonLinePusher(foi, args.json_file).run()


def run_journal_metadata(args: argparse.Namespace) -> None:
    fii = JournalMetadataImporter(args.api, edit_batch_size=args.batch_size)
    JsonLinePusher(fii, args.json_file).run()


def run_chocula(args: argparse.Namespace) -> None:
    fii = ChoculaImporter(args.api, edit_batch_size=args.batch_size, do_updates=args.do_updates)
    JsonLinePusher(fii, args.json_file).run()


def run_matched(args: argparse.Namespace) -> None:
    fmi = MatchedImporter(
        args.api,
        edit_batch_size=args.batch_size,
        editgroup_description=args.editgroup_description_override,
        default_link_rel=args.default_link_rel,
        default_mimetype=args.default_mimetype,
    )
    JsonLinePusher(fmi, args.json_file).run()


def run_arabesque_match(args: argparse.Namespace) -> None:
    if (args.sqlite_file and args.json_file) or not (args.sqlite_file or args.json_file):
        print("Supply one of --sqlite-file or --json-file")
    ami = ArabesqueMatchImporter(
        args.api,
        editgroup_description=args.editgroup_description_override,
        do_updates=args.do_updates,
        require_grobid=(not args.no_require_grobid),
        extid_type=args.extid_type,
        crawl_id=args.crawl_id,
        default_link_rel=args.default_link_rel,
        edit_batch_size=args.batch_size,
    )
    if args.sqlite_file:
        SqlitePusher(ami, args.sqlite_file, "crawl_result", ARABESQUE_MATCH_WHERE_CLAUSE).run()
    elif args.json_file:
        JsonLinePusher(ami, args.json_file).run()


def run_ingest_file(args: argparse.Namespace) -> None:
    ifri = IngestFileResultImporter(
        args.api,
        editgroup_description=args.editgroup_description_override,
        skip_source_allowlist=args.skip_source_allowlist,
        do_updates=args.do_updates,
        default_link_rel=args.default_link_rel,
        require_grobid=(not args.no_require_grobid),
        edit_batch_size=args.batch_size,
    )
    if args.kafka_mode:
        KafkaJsonPusher(
            ifri,
            args.kafka_hosts,
            args.kafka_env,
            "ingest-file-results",
            "fatcat-{}-ingest-file-result".format(args.kafka_env),
            kafka_namespace="sandcrawler",
            consume_batch_size=args.batch_size,
        ).run()
    else:
        JsonLinePusher(ifri, args.json_file).run()


def run_ingest_web(args: argparse.Namespace) -> None:
    iwri = IngestWebResultImporter(
        args.api,
        editgroup_description=args.editgroup_description_override,
        skip_source_allowlist=args.skip_source_allowlist,
        do_updates=args.do_updates,
        default_link_rel=args.default_link_rel,
        edit_batch_size=args.batch_size,
    )
    if args.kafka_mode:
        KafkaJsonPusher(
            iwri,
            args.kafka_hosts,
            args.kafka_env,
            "ingest-file-results",
            "fatcat-{}-ingest-web-result".format(args.kafka_env),
            kafka_namespace="sandcrawler",
            consume_batch_size=args.batch_size,
            force_flush=True,
        ).run()
    else:
        JsonLinePusher(iwri, args.json_file).run()


def run_ingest_fileset(args: argparse.Namespace) -> None:
    ifri = IngestFilesetResultImporter(
        args.api,
        editgroup_description=args.editgroup_description_override,
        skip_source_allowlist=args.skip_source_allowlist,
        do_updates=args.do_updates,
        default_link_rel=args.default_link_rel,
        edit_batch_size=args.batch_size,
    )
    if args.kafka_mode:
        KafkaJsonPusher(
            ifri,
            args.kafka_hosts,
            args.kafka_env,
            "ingest-fileset-results",
            "fatcat-{}-ingest-fileset-result".format(args.kafka_env),
            kafka_namespace="sandcrawler",
            consume_batch_size=args.batch_size,
            force_flush=True,
        ).run()
    else:
        JsonLinePusher(ifri, args.json_file).run()


def run_savepapernow_file(args: argparse.Namespace) -> None:
    ifri = SavePaperNowFileImporter(
        args.api,
        editgroup_description=args.editgroup_description_override,
        edit_batch_size=args.batch_size,
    )
    if args.kafka_mode:
        KafkaJsonPusher(
            ifri,
            args.kafka_hosts,
            args.kafka_env,
            "ingest-file-results",
            "fatcat-{}-savepapernow-file-result".format(args.kafka_env),
            kafka_namespace="sandcrawler",
            consume_batch_size=args.batch_size,
            force_flush=True,
        ).run()
    else:
        JsonLinePusher(ifri, args.json_file).run()


def run_savepapernow_web(args: argparse.Namespace) -> None:
    ifri = SavePaperNowWebImporter(
        args.api,
        editgroup_description=args.editgroup_description_override,
        edit_batch_size=args.batch_size,
    )
    if args.kafka_mode:
        KafkaJsonPusher(
            ifri,
            args.kafka_hosts,
            args.kafka_env,
            "ingest-file-results",
            "fatcat-{}-savepapernow-web-result".format(args.kafka_env),
            kafka_namespace="sandcrawler",
            consume_batch_size=args.batch_size,
            force_flush=True,
        ).run()
    else:
        JsonLinePusher(ifri, args.json_file).run()


def run_savepapernow_fileset(args: argparse.Namespace) -> None:
    ifri = SavePaperNowFilesetImporter(
        args.api,
        editgroup_description=args.editgroup_description_override,
        edit_batch_size=args.batch_size,
    )
    if args.kafka_mode:
        KafkaJsonPusher(
            ifri,
            args.kafka_hosts,
            args.kafka_env,
            "ingest-file-results",
            "fatcat-{}-savepapernow-fileset-result".format(args.kafka_env),
            kafka_namespace="sandcrawler",
            consume_batch_size=args.batch_size,
            force_flush=True,
        ).run()
    else:
        JsonLinePusher(ifri, args.json_file).run()


def run_grobid_metadata(args: argparse.Namespace) -> None:
    fmi = GrobidMetadataImporter(
        args.api,
        edit_batch_size=args.batch_size,
        longtail_oa=args.longtail_oa,
        bezerk_mode=args.bezerk_mode,
    )
    LinePusher(fmi, args.tsv_file).run()


def run_shadow_lib(args: argparse.Namespace) -> None:
    fmi = ShadowLibraryImporter(args.api, edit_batch_size=100)
    JsonLinePusher(fmi, args.json_file).run()


def run_datacite(args: argparse.Namespace) -> None:
    dci = DataciteImporter(
        args.api,
        args.issn_map_file,
        edit_batch_size=args.batch_size,
        bezerk_mode=args.bezerk_mode,
        debug=args.debug,
        insert_log_file=args.insert_log_file,
    )
    if args.kafka_mode:
        KafkaJsonPusher(
            dci,
            args.kafka_hosts,
            args.kafka_env,
            "api-datacite",
            "fatcat-{}-import-datacite".format(args.kafka_env),
            consume_batch_size=args.batch_size,
        ).run()
    else:
        JsonLinePusher(dci, args.json_file).run()


def run_doaj_article(args: argparse.Namespace) -> None:
    dai = DoajArticleImporter(
        args.api,
        args.issn_map_file,
        edit_batch_size=args.batch_size,
        do_updates=args.do_updates,
    )
    if args.kafka_mode:
        KafkaJsonPusher(
            dai,
            args.kafka_hosts,
            args.kafka_env,
            "api-doaj",
            "fatcat-{}-import-doaj".format(args.kafka_env),
            consume_batch_size=args.batch_size,
        ).run()
    else:
        JsonLinePusher(dai, args.json_file).run()


def run_dblp_release(args: argparse.Namespace) -> None:
    dri = DblpReleaseImporter(
        args.api,
        dblp_container_map_file=args.dblp_container_map_file,
        edit_batch_size=args.batch_size,
        do_updates=args.do_updates,
        dump_json_mode=args.dump_json_mode,
    )
    Bs4XmlLargeFilePusher(
        dri,
        args.xml_file,
        DblpReleaseImporter.ELEMENT_TYPES,
        use_lxml=True,
    ).run()


def run_dblp_container(args: argparse.Namespace) -> None:
    dci = DblpContainerImporter(
        args.api,
        args.issn_map_file,
        dblp_container_map_file=args.dblp_container_map_file,
        dblp_container_map_output=args.dblp_container_map_output,
        edit_batch_size=args.batch_size,
        do_updates=args.do_updates,
    )
    JsonLinePusher(dci, args.json_file).run()


def run_file_meta(args: argparse.Namespace) -> None:
    # do_updates defaults to true for this importer
    fmi = FileMetaImporter(
        args.api,
        edit_batch_size=100,
        editgroup_description=args.editgroup_description_override,
    )
    JsonLinePusher(fmi, args.json_file).run()


def run_fileset(args: argparse.Namespace) -> None:
    fmi = FilesetImporter(
        args.api,
        edit_batch_size=100,
        skip_release_fileset_check=args.skip_release_fileset_check,
    )
    JsonLinePusher(fmi, args.json_file).run()


def main() -> None:
    parser = argparse.ArgumentParser(formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument(
        "--host-url", default="http://localhost:9411/v0", help="connect to this host/port"
    )
    parser.add_argument(
        "--kafka-hosts",
        default="localhost:9092",
        help="list of Kafka brokers (host/port) to use",
    )
    parser.add_argument(
        "--kafka-env", default="dev", help="Kafka topic namespace to use (eg, prod, qa)"
    )
    parser.add_argument("--batch-size", help="size of batch to send", default=50, type=int)
    parser.add_argument(
        "--editgroup-description-override",
        help="editgroup description override",
        default=None,
        type=str,
    )
    subparsers = parser.add_subparsers()

    sub_crossref = subparsers.add_parser(
        "crossref", help="import Crossref API metadata format (JSON)"
    )
    sub_crossref.set_defaults(
        func=run_crossref,
        auth_var="FATCAT_AUTH_WORKER_CROSSREF",
    )
    sub_crossref.add_argument(
        "json_file",
        help="crossref JSON file to import from",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_crossref.add_argument(
        "issn_map_file",
        help="ISSN to ISSN-L mapping file",
        default=None,
        type=argparse.FileType("r"),
    )
    sub_crossref.add_argument(
        "--no-lookup-refs", action="store_true", help="skip lookup of references (PMID or DOI)"
    )
    sub_crossref.add_argument(
        "--kafka-mode", action="store_true", help="consume from kafka topic (not stdin)"
    )
    sub_crossref.add_argument(
        "--bezerk-mode",
        action="store_true",
        help="don't lookup existing DOIs, just insert (clobbers; only for fast bootstrap)",
    )

    sub_jalc = subparsers.add_parser("jalc", help="import JALC DOI metadata from XML dump")
    sub_jalc.set_defaults(
        func=run_jalc,
        auth_var="FATCAT_AUTH_WORKER_JALC",
    )
    sub_jalc.add_argument(
        "xml_file",
        help="Jalc RDF XML file (record-per-line) to import from",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_jalc.add_argument(
        "issn_map_file",
        help="ISSN to ISSN-L mapping file",
        default=None,
        type=argparse.FileType("r"),
    )

    sub_arxiv = subparsers.add_parser("arxiv", help="import arxiv.org metadata from XML files")
    sub_arxiv.set_defaults(
        func=run_arxiv,
        auth_var="FATCAT_AUTH_WORKER_ARXIV",
    )
    sub_arxiv.add_argument(
        "xml_file",
        nargs="?",
        help="arXivRaw XML file to import from",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_arxiv.add_argument(
        "--kafka-mode", action="store_true", help="consume from kafka topic (not stdin)"
    )

    sub_pubmed = subparsers.add_parser(
        "pubmed", help="import MEDLINE/PubMed work-level metadata (XML)"
    )
    sub_pubmed.set_defaults(
        func=run_pubmed,
        auth_var="FATCAT_AUTH_WORKER_PUBMED",
    )
    sub_pubmed.add_argument(
        "xml_file",
        nargs="?",
        help="Pubmed XML file to import from",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_pubmed.add_argument(
        "issn_map_file",
        help="ISSN to ISSN-L mapping file",
        default=None,
        type=argparse.FileType("r"),
    )
    sub_pubmed.add_argument(
        "--no-lookup-refs", action="store_true", help="skip lookup of references (PMID or DOI)"
    )
    sub_pubmed.add_argument(
        "--do-updates", action="store_true", help="update pre-existing release entities"
    )
    sub_pubmed.add_argument(
        "--kafka-mode", action="store_true", help="consume from kafka topic (not stdin)"
    )

    sub_jstor = subparsers.add_parser(
        "jstor", help="import JSTOR work-level metadata from XML dump"
    )
    sub_jstor.set_defaults(
        func=run_jstor,
        auth_var="FATCAT_AUTH_WORKER_JSTOR",
    )
    sub_jstor.add_argument(
        "list_file",
        help="List of JSTOR XML file paths to import from",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_jstor.add_argument(
        "issn_map_file",
        help="ISSN to ISSN-L mapping file",
        default=None,
        type=argparse.FileType("r"),
    )

    sub_orcid = subparsers.add_parser(
        "orcid", help="import creator entities from ORCID XML dump"
    )
    sub_orcid.set_defaults(func=run_orcid, auth_var="FATCAT_AUTH_WORKER_ORCID")
    sub_orcid.add_argument(
        "json_file",
        help="orcid JSON file to import from (or stdin)",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )

    sub_journal_metadata = subparsers.add_parser(
        "journal-metadata",
        help="import/update container metadata from old manual munging format",
    )
    sub_journal_metadata.set_defaults(
        func=run_journal_metadata,
        auth_var="FATCAT_AUTH_WORKER_JOURNAL_METADATA",
    )
    sub_journal_metadata.add_argument(
        "json_file",
        help="Journal JSON metadata file to import from (or stdin)",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )

    sub_chocula = subparsers.add_parser(
        "chocula", help="import/update container metadata from chocula JSON export"
    )
    sub_chocula.set_defaults(
        func=run_chocula,
        auth_var="FATCAT_AUTH_WORKER_JOURNAL_METADATA",
    )
    sub_chocula.add_argument(
        "json_file",
        help="chocula JSON entities file (or stdin)",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_chocula.add_argument(
        "--do-updates", action="store_true", help="update pre-existing container entities"
    )

    sub_matched = subparsers.add_parser(
        "matched",
        help="add file entities matched against existing releases; custom JSON format",
    )
    sub_matched.set_defaults(
        func=run_matched,
        auth_var="FATCAT_API_AUTH_TOKEN",
    )
    sub_matched.add_argument(
        "json_file",
        help="JSON file to import from (or stdin)",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_matched.add_argument(
        "--default-mimetype",
        default=None,
        help="default mimetype for imported files (if not specified per-file)",
    )
    sub_matched.add_argument(
        "--bezerk-mode",
        action="store_true",
        help="don't lookup existing files, just insert (clobbers; only for fast bootstrap)",
    )
    sub_matched.add_argument(
        "--default-link-rel",
        default="web",
        help="default URL rel for matches (eg, 'publisher', 'web')",
    )

    sub_arabesque_match = subparsers.add_parser(
        "arabesque", help="add file entities matched to releases from crawl log analysis"
    )
    sub_arabesque_match.set_defaults(
        func=run_arabesque_match,
        auth_var="FATCAT_AUTH_WORKER_CRAWL",
    )
    sub_arabesque_match.add_argument(
        "--sqlite-file", help="sqlite database file to import from"
    )
    sub_arabesque_match.add_argument(
        "--json-file", help="JSON file to import from (or stdin)", type=argparse.FileType("r")
    )
    sub_arabesque_match.add_argument(
        "--do-updates",
        action="store_true",
        help="update pre-existing file entities if new match (instead of skipping)",
    )
    sub_arabesque_match.add_argument(
        "--no-require-grobid",
        action="store_true",
        help="whether postproc_status column must be '200'",
    )
    sub_arabesque_match.add_argument(
        "--extid-type",
        default="doi",
        help="identifier type in the database (eg, 'doi', 'pmcid'",
    )
    sub_arabesque_match.add_argument(
        "--crawl-id", help="crawl ID (optionally included in editgroup metadata)"
    )
    sub_arabesque_match.add_argument(
        "--default-link-rel",
        default="web",
        help="default URL rel for matches (eg, 'publisher', 'web')",
    )

    sub_ingest_file = subparsers.add_parser(
        "ingest-file-results",
        help="add/update file entities linked to releases based on sandcrawler ingest results",
    )
    sub_ingest_file.set_defaults(
        func=run_ingest_file,
        auth_var="FATCAT_AUTH_WORKER_CRAWL",
    )
    sub_ingest_file.add_argument(
        "json_file",
        help="ingest_file JSON file to import from",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_ingest_file.add_argument(
        "--skip-source-allowlist",
        action="store_true",
        help="don't filter import based on request source allowlist",
    )
    sub_ingest_file.add_argument(
        "--kafka-mode", action="store_true", help="consume from kafka topic (not stdin)"
    )
    sub_ingest_file.add_argument(
        "--do-updates",
        action="store_true",
        help="update pre-existing file entities if new match (instead of skipping)",
    )
    sub_ingest_file.add_argument(
        "--no-require-grobid",
        action="store_true",
        help="whether postproc_status column must be '200'",
    )
    sub_ingest_file.add_argument(
        "--default-link-rel",
        default="web",
        help="default URL rel for matches (eg, 'publisher', 'web')",
    )

    sub_ingest_web = subparsers.add_parser(
        "ingest-web-results",
        help="add/update web entities linked to releases based on sandcrawler ingest results",
    )
    sub_ingest_web.set_defaults(
        func=run_ingest_web,
        auth_var="FATCAT_AUTH_WORKER_CRAWL",
    )
    sub_ingest_web.add_argument(
        "json_file",
        help="ingest_web JSON file to import from",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_ingest_web.add_argument(
        "--skip-source-allowlist",
        action="store_true",
        help="don't filter import based on request source allowlist",
    )
    sub_ingest_web.add_argument(
        "--kafka-mode", action="store_true", help="consume from kafka topic (not stdin)"
    )
    sub_ingest_web.add_argument(
        "--do-updates",
        action="store_true",
        help="update pre-existing web entities if new match (instead of skipping)",
    )
    sub_ingest_web.add_argument(
        "--default-link-rel",
        default="web",
        help="default URL rel for matches (eg, 'publisher', 'web')",
    )

    sub_ingest_fileset = subparsers.add_parser(
        "ingest-fileset-results",
        help="add/update fileset entities linked to releases based on sandcrawler ingest results",
    )
    sub_ingest_fileset.set_defaults(
        func=run_ingest_fileset,
        auth_var="FATCAT_AUTH_WORKER_CRAWL",
    )
    sub_ingest_fileset.add_argument(
        "json_file",
        help="ingest_fileset JSON file to import from",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_ingest_fileset.add_argument(
        "--skip-source-allowlist",
        action="store_true",
        help="don't filter import based on request source allowlist",
    )
    sub_ingest_fileset.add_argument(
        "--kafka-mode", action="store_true", help="consume from kafka topic (not stdin)"
    )
    sub_ingest_fileset.add_argument(
        "--do-updates",
        action="store_true",
        help="update pre-existing fileset entities if new match (instead of skipping)",
    )
    sub_ingest_fileset.add_argument(
        "--default-link-rel",
        default="fileset",
        help="default URL rel for matches (eg, 'publisher', 'web')",
    )

    sub_savepapernow_file = subparsers.add_parser(
        "savepapernow-file-results",
        help="add file entities crawled due to async Save Paper Now request",
    )
    sub_savepapernow_file.set_defaults(
        func=run_savepapernow_file,
        auth_var="FATCAT_AUTH_WORKER_SAVEPAPERNOW",
    )
    sub_savepapernow_file.add_argument(
        "json_file",
        help="ingest-file JSON file to import from",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_savepapernow_file.add_argument(
        "--kafka-mode", action="store_true", help="consume from kafka topic (not stdin)"
    )

    sub_savepapernow_web = subparsers.add_parser(
        "savepapernow-web-results",
        help="add webcapture entities crawled due to async Save Paper Now request",
    )
    sub_savepapernow_web.set_defaults(
        func=run_savepapernow_web,
        auth_var="FATCAT_AUTH_WORKER_SAVEPAPERNOW",
    )
    sub_savepapernow_web.add_argument(
        "json_file",
        help="ingest-file JSON file to import from",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_savepapernow_web.add_argument(
        "--kafka-mode", action="store_true", help="consume from kafka topic (not stdin)"
    )

    sub_savepapernow_fileset = subparsers.add_parser(
        "savepapernow-fileset-results",
        help="add fileset entities crawled due to async Save Paper Now request",
    )
    sub_savepapernow_fileset.set_defaults(
        func=run_savepapernow_fileset,
        auth_var="FATCAT_AUTH_WORKER_SAVEPAPERNOW",
    )
    sub_savepapernow_fileset.add_argument(
        "json_file",
        help="ingest-file JSON file to import from",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_savepapernow_fileset.add_argument(
        "--kafka-mode", action="store_true", help="consume from kafka topic (not stdin)"
    )

    sub_grobid_metadata = subparsers.add_parser(
        "grobid-metadata",
        help="create release and file entities based on GROBID PDF metadata extraction",
    )
    sub_grobid_metadata.set_defaults(
        func=run_grobid_metadata,
        auth_var="FATCAT_API_AUTH_TOKEN",
    )
    sub_grobid_metadata.add_argument(
        "tsv_file",
        help="TSV file to import from (or stdin)",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_grobid_metadata.add_argument(
        "--group-size", help="editgroup group size to use", default=75, type=int
    )
    sub_grobid_metadata.add_argument(
        "--longtail-oa",
        action="store_true",
        help="if this is an import of longtail OA content (sets an 'extra' flag)",
    )
    sub_grobid_metadata.add_argument(
        "--bezerk-mode",
        action="store_true",
        help="don't lookup existing files, just insert (clobbers; only for fast bootstrap)",
    )

    sub_shadow_lib = subparsers.add_parser(
        "shadow-lib",
        help="create release and file entities based on GROBID PDF metadata extraction",
    )
    sub_shadow_lib.set_defaults(
        func=run_shadow_lib,
        auth_var="FATCAT_AUTH_WORKER_SHADOW",
    )
    sub_shadow_lib.add_argument(
        "json_file",
        help="JSON file to import from (or stdin)",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )

    sub_datacite = subparsers.add_parser("datacite", help="import datacite.org metadata")
    sub_datacite.add_argument(
        "json_file",
        help="File with jsonlines from datacite.org v2 API to import from",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_datacite.add_argument(
        "issn_map_file",
        help="ISSN to ISSN-L mapping file",
        default=None,
        type=argparse.FileType("r"),
    )
    sub_datacite.add_argument(
        "--kafka-mode", action="store_true", help="consume from kafka topic (not stdin)"
    )
    sub_datacite.add_argument(
        "--bezerk-mode",
        action="store_true",
        help="don't lookup existing DOIs, just insert (clobbers; only for fast bootstrap)",
    )
    sub_datacite.add_argument(
        "--debug", action="store_true", help="write converted JSON to stdout"
    )
    sub_datacite.add_argument(
        "--insert-log-file",
        default="",
        type=str,
        help="write inserted documents into file (for debugging)",
    )
    sub_datacite.set_defaults(
        func=run_datacite,
        auth_var="FATCAT_AUTH_WORKER_DATACITE",
    )

    sub_doaj_article = subparsers.add_parser(
        "doaj-article", help="import doaj.org article metadata"
    )
    sub_doaj_article.add_argument(
        "json_file",
        help="File with JSON lines from DOAJ API (or bulk dump) to import from",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_doaj_article.add_argument(
        "--issn-map-file",
        help="ISSN to ISSN-L mapping file",
        default=None,
        type=argparse.FileType("r"),
    )
    sub_doaj_article.add_argument(
        "--kafka-mode", action="store_true", help="consume from kafka topic (not stdin)"
    )
    sub_doaj_article.add_argument(
        "--do-updates", action="store_true", help="update any pre-existing release entities"
    )
    sub_doaj_article.set_defaults(
        func=run_doaj_article,
        auth_var="FATCAT_AUTH_WORKER_DOAJ",
    )

    sub_dblp_release = subparsers.add_parser(
        "dblp-release", help="import dblp release metadata"
    )
    sub_dblp_release.add_argument(
        "xml_file",
        help="File with DBLP XML to import from",
        default=sys.stdin,
        type=argparse.FileType("rb"),
    )
    sub_dblp_release.add_argument(
        "--dblp-container-map-file",
        help="file path to dblp prefix to container_id TSV file",
        default=None,
        type=argparse.FileType("r"),
    )
    sub_dblp_release.add_argument(
        "--do-updates", action="store_true", help="update any pre-existing release entities"
    )
    sub_dblp_release.add_argument(
        "--dump-json-mode",
        action="store_true",
        help="print release entities to stdout instead of importing",
    )
    sub_dblp_release.set_defaults(
        func=run_dblp_release,
        auth_var="FATCAT_AUTH_WORKER_DBLP",
    )

    sub_dblp_container = subparsers.add_parser(
        "dblp-container", help="import dblp container metadata"
    )
    sub_dblp_container.add_argument(
        "json_file",
        help="File with DBLP container JSON to import from (see extra/dblp/)",
        default=sys.stdin,
        type=argparse.FileType("rb"),
    )
    sub_dblp_container.add_argument(
        "--dblp-container-map-file",
        help="file path to dblp pre-existing prefix to container_id TSV file",
        default=None,
        type=argparse.FileType("r"),
    )
    sub_dblp_container.add_argument(
        "--dblp-container-map-output",
        help="file path to output new dblp container map TSV to",
        default=None,
        type=argparse.FileType("w"),
    )
    sub_dblp_container.add_argument(
        "--issn-map-file",
        help="ISSN to ISSN-L mapping file",
        default=None,
        type=argparse.FileType("r"),
    )
    sub_dblp_container.add_argument(
        "--do-updates", action="store_true", help="update any pre-existing container entities"
    )
    sub_dblp_container.set_defaults(
        func=run_dblp_container,
        auth_var="FATCAT_AUTH_WORKER_DBLP",
    )

    sub_file_meta = subparsers.add_parser(
        "file-meta", help="simple update-only importer for file metadata"
    )
    sub_file_meta.set_defaults(
        func=run_file_meta,
        auth_var="FATCAT_API_AUTH_TOKEN",
    )
    sub_file_meta.add_argument(
        "json_file",
        help="File with jsonlines from file_meta schema to import from",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )

    sub_fileset = subparsers.add_parser("fileset", help="generic fileset importer")
    sub_fileset.set_defaults(
        func=run_fileset,
        auth_var="FATCAT_API_AUTH_TOKEN",
    )
    sub_fileset.add_argument(
        "json_file",
        help="File with jsonlines of fileset entities to import",
        default=sys.stdin,
        type=argparse.FileType("r"),
    )
    sub_fileset.add_argument(
        "--skip-release-fileset-check",
        action="store_true",
        help="create without checking if releases already have related filesets",
    )

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do!")
        sys.exit(-1)

    # allow editgroup description override via env variable (but CLI arg takes
    # precedence)
    if not args.editgroup_description_override and os.environ.get(
        "FATCAT_EDITGROUP_DESCRIPTION"
    ):
        args.editgroup_description_override = os.environ.get("FATCAT_EDITGROUP_DESCRIPTION")

    args.api = authenticated_api(
        args.host_url,
        # token is an optional kwarg (can be empty string, None, etc)
        token=os.environ.get(args.auth_var),
    )
    args.func(args)


if __name__ == "__main__":
    main()
