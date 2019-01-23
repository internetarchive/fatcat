
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

from .common import FatcatImporter, make_kafka_consumer
from .crossref import CrossrefImporter, CROSSREF_TYPE_MAP
from .grobid_metadata import GrobidMetadataImporter
from .journal_metadata import JournalMetadataImporter
from .matched import MatchedImporter
from .orcid import OrcidImporter
from .kafka_source import KafkaSource
from .file_source import FileSource
