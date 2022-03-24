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

from .arabesque import ARABESQUE_MATCH_WHERE_CLAUSE, ArabesqueMatchImporter
from .arxiv import ArxivRawImporter
from .chocula import ChoculaImporter
from .common import (
    Bs4XmlFileListPusher,
    Bs4XmlFilePusher,
    Bs4XmlLargeFilePusher,
    Bs4XmlLinesPusher,
    CsvPusher,
    EntityImporter,
    JsonLinePusher,
    KafkaBs4XmlPusher,
    KafkaJsonPusher,
    LinePusher,
    SqlitePusher,
)
from .crossref import CrossrefImporter
from .datacite import DataciteImporter
from .dblp_container import DblpContainerImporter
from .dblp_release import DblpReleaseImporter
from .doaj_article import DoajArticleImporter
from .file_meta import FileMetaImporter
from .fileset_generic import FilesetImporter
from .grobid_metadata import GrobidMetadataImporter
from .ingest import (
    IngestFileResultImporter,
    IngestFilesetFileResultImporter,
    IngestFilesetResultImporter,
    IngestWebResultImporter,
    SavePaperNowFileImporter,
    SavePaperNowFilesetImporter,
    SavePaperNowWebImporter,
)
from .jalc import JalcImporter
from .journal_metadata import JournalMetadataImporter
from .jstor import JstorImporter
from .matched import MatchedImporter
from .orcid import OrcidImporter
from .pubmed import PubmedImporter
from .shadow import ShadowLibraryImporter
