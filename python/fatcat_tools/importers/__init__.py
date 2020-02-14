
"""
To run an import you combine two classes; one each of:

- RecordSource: somehow iterates over a source of raw records (eg, from a
  database, Kafka, files on disk, stdin) and pushes into an entity importer.
- EntityImporter: class that a record iterator pushes raw (unparsed) records
  into. The entity importer parses and decides what to do (ignore, update,
  insert, etc). There is usually a primary entity type, though related entities
  can be created along the way. Maintains API connection and editgroup/batch
  state.

"""

from .common import EntityImporter, JsonLinePusher, LinePusher, CsvPusher, SqlitePusher, Bs4XmlFilePusher, Bs4XmlLargeFilePusher, Bs4XmlLinesPusher, Bs4XmlFileListPusher, KafkaJsonPusher, KafkaBs4XmlPusher, make_kafka_consumer, clean, is_cjk, LANG_MAP_MARC
from .crossref import CrossrefImporter, CROSSREF_TYPE_MAP, lookup_license_slug
from .datacite import DataciteImporter
from .jalc import JalcImporter
from .jstor import JstorImporter
from .arxiv import ArxivRawImporter
from .pubmed import PubmedImporter
from .grobid_metadata import GrobidMetadataImporter
from .journal_metadata import JournalMetadataImporter
from .chocula import ChoculaImporter
from .matched import MatchedImporter
from .orcid import OrcidImporter
from .arabesque import ArabesqueMatchImporter, ARABESQUE_MATCH_WHERE_CLAUSE
from .wayback_static import auto_wayback_static
from .cdl_dash_dat import auto_cdl_dash_dat
from .ingest import IngestFileResultImporter, SavePaperNowFileImporter
