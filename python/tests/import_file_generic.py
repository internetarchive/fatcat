
import pytest
from fatcat_openapi_client import *

from fatcat_tools.importers.common import EntityImporter


def test_file_update_generic():

    f1 = FileEntity(
        size=89238,
        md5="7ce6615b2a5904939576d9567bd5f68e",
        sha1="027e7ed3ea1a40e92dd2657a1e3c992b5dc45dd2",
        sha256="f1f4f18a904e76818863ccbc6141fce92b0dcb47b0d6041aec98bc6806e393c3",
        mimetype="application/pdf",
        urls=[],
        release_ids=[],
        extra=dict(a=2, b=5),
        edit_extra=dict(test_key="files rule"),
    )
    assert f1 == EntityImporter.generic_file_cleanups(f1)

    url_sets = [
        # dummy
        {
            'before': [],
            'after': [],
        },
        # social => academicsocial
        {
            'before': [
                FileUrl(url="https://academic.edu/blah.pdf", rel="social"),
            ],
            'after': [
                FileUrl(url="https://academic.edu/blah.pdf", rel="academicsocial"),
            ],
        },
        # archive.org repository => archive
        {
            'before': [
                FileUrl(url="https://archive.org/download/item/blah.pdf", rel="repository"),
            ],
            'after': [
                FileUrl(url="https://archive.org/download/item/blah.pdf", rel="archive"),
            ],
        },
        # :80 in URL is redundant
        {
            'before': [
                FileUrl(url="http://homepages.math.uic.edu/~rosendal/PapersWebsite/BanachMinimalExamples.pdf", rel="web"),
                FileUrl(url="http://homepages.math.uic.edu:80/~rosendal/PapersWebsite/BanachMinimalExamples.pdf", rel="web"),
                FileUrl(url="http://mit.edu/item/blah.pdf", rel="web"),
                FileUrl(url="http://mit.edu:80/item/blah.pdf", rel="web"),
            ],
            'after': [
                FileUrl(url="http://homepages.math.uic.edu/~rosendal/PapersWebsite/BanachMinimalExamples.pdf", rel="web"),
                FileUrl(url="http://mit.edu/item/blah.pdf", rel="web"),
            ],
        },
        {
            'before': [
                FileUrl(url="http://mit.edu:80/item/blah.pdf", rel="web"),
            ],
            'after': [
                FileUrl(url="http://mit.edu:80/item/blah.pdf", rel="web"),
            ],
        },
        # http/https redundant
        {
            'before': [
                FileUrl(url="https://eo1.gsfc.nasa.gov/new/validationReport/Technology/JoeCD/asner_etal_PNAS_20041.pdf", rel="web"),
                FileUrl(url="http://eo1.gsfc.nasa.gov/new/validationReport/Technology/JoeCD/asner_etal_PNAS_20041.pdf", rel="web"),
                FileUrl(url="https://mit.edu/item/blah.pdf", rel="web"),
                FileUrl(url="https://web.archive.org/web/12345542/http://mit.edu/item/blah.pdf", rel="webarchive"),
                FileUrl(url="http://mit.edu/item/blah.pdf", rel="web"),
                FileUrl(url="https://web.archive.org/web/12345542/something.com/blah.pdf", rel="webarchive"),
            ],
            'after': [
                FileUrl(url="https://eo1.gsfc.nasa.gov/new/validationReport/Technology/JoeCD/asner_etal_PNAS_20041.pdf", rel="web"),
                FileUrl(url="https://mit.edu/item/blah.pdf", rel="web"),
                FileUrl(url="https://web.archive.org/web/12345542/http://mit.edu/item/blah.pdf", rel="webarchive"),
                FileUrl(url="https://web.archive.org/web/12345542/something.com/blah.pdf", rel="webarchive"),
            ],
        },
        # short /2017/ wayback datetime
        {
            'before': [
                FileUrl(url="https://web.archive.org/web/2017/http://www.geoamazonia.net/index.php/revista/article/download/51/pdf_38", rel="webarchive"),
                FileUrl(url="https://web.archive.org/web/20170922010835/http://www.geoamazonia.net/index.php/revista/article/download/51/pdf_38", rel="webarchive"),
            ],
            'after': [
                FileUrl(url="https://web.archive.org/web/20170922010835/http://www.geoamazonia.net/index.php/revista/article/download/51/pdf_38", rel="webarchive"),
            ],
        },
    ]

    for pair in url_sets:
        f1.urls = pair['before']
        assert EntityImporter.generic_file_cleanups(f1).urls == pair['after']
