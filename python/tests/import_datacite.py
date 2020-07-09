"""
Test datacite importer.
"""

import collections
import datetime
import pytest
import gzip
from fatcat_tools.importers import DataciteImporter, JsonLinePusher
from fatcat_tools.importers.datacite import (
    find_original_language_title,
    parse_datacite_titles,
    parse_datacite_dates,
    clean_doi,
    index_form_to_display_name,
    lookup_license_slug,
    contributor_list_contains_contributor,
)
from fatcat_tools.transforms import entity_to_dict
import fatcat_openapi_client
from fixtures import api
import json


@pytest.fixture(scope="function")
def datacite_importer(api):
    with open("tests/files/ISSN-to-ISSN-L.snip.txt", "r") as issn_file:
        yield DataciteImporter(
            api,
            issn_file,
            extid_map_file="tests/files/example_map.sqlite3",
            bezerk_mode=True,
        )


@pytest.fixture(scope="function")
def datacite_importer_existing(api):
    with open("tests/files/ISSN-to-ISSN-L.snip.txt", "r") as issn_file:
        yield DataciteImporter(
            api,
            issn_file,
            extid_map_file="tests/files/example_map.sqlite3",
            bezerk_mode=False,
        )


@pytest.mark.skip(reason="larger datacite import slows tests down")
def test_datacite_importer_huge(datacite_importer):
    last_index = datacite_importer.api.get_changelog(limit=1)[0].index
    with gzip.open("tests/files/datacite_1k_records.jsonl.gz", "rt") as f:
        datacite_importer.bezerk_mode = True
        counts = JsonLinePusher(datacite_importer, f).run()
    assert counts["insert"] == 998
    change = datacite_importer.api.get_changelog_entry(index=last_index + 1)
    release = datacite_importer.api.get_release(
        change.editgroup.edits.releases[0].ident
    )
    assert len(release.contribs) == 3


def test_find_original_language_title():
    """
    Original language might be included, in various ways.
    """
    Case = collections.namedtuple("Case", "about input result")
    cases = [
        Case("defaults to None", {}, None),
        Case("ignore unknown keys", {"broken": "kv"}, None),
        Case("just a title", {"title": "Noise Reduction"}, None),
        Case(
            "same title should be ignored",
            {"title": "Noise Reduction", "original_language_title": "Noise Reduction"},
            None,
        ),
        Case(
            "empty subdict is ignored",
            {"title": "Noise Reduction", "original_language_title": {},},
            None,
        ),
        Case(
            "unknown subdict keys are ignored",
            {"title": "Noise Reduction", "original_language_title": {"broken": "kv"},},
            None,
        ),
        Case(
            "original string",
            {"title": "Noise Reduction", "original_language_title": "Подавление шума",},
            "Подавление шума",
        ),
        Case(
            "language tag is ignored, since its broken",
            {
                "title": "Noise Reduction",
                "original_language_title": {
                    "language": "ja",
                    "__content__": "Noise Reduction",
                },
            },
            None,
        ),
        Case(
            "do not care about language",
            {
                "title": "Noise Reduction",
                "original_language_title": {
                    "language": "ja",
                    "__content__": "Rauschunterdrückung",
                },
            },
            "Rauschunterdrückung",
        ),
        Case(
            "ignore excessive questionmarks",
            {
                "title": "Noise Reduction",
                "original_language_title": {
                    "language": "ja",
                    "__content__": "???? However",
                },
            },
            None,
        ),
    ]

    for case in cases:
        result = find_original_language_title(case.input)
        assert result == case.result


def test_parse_datacite_titles():
    """
    Given a list of titles, find title, original_language_title and subtitle.
    Result is a 3-tuple of title, original_language_title, subtitle.
    """
    Case = collections.namedtuple("Case", "about input result")
    cases = [
        Case("handle None", None, (None, None, None)),
        Case("empty list", [], (None, None, None)),
        Case("empty item", [{}], (None, None, None)),
        Case("broken keys", [{"broken": "kv"}], (None, None, None)),
        Case(
            "title only",
            [{"title": "Total carbon dioxide"}],
            ("Total carbon dioxide", None, None),
        ),
        Case(
            "title and subtitle",
            [
                {"title": "Total carbon dioxide"},
                {"title": "Station TT043_7-9", "titleType": "Subtitle"},
            ],
            ("Total carbon dioxide", None, "Station TT043_7-9"),
        ),
        Case(
            "title, subtitle order does not matter",
            [
                {"title": "Station TT043_7-9", "titleType": "Subtitle"},
                {"title": "Total carbon dioxide"},
            ],
            ("Total carbon dioxide", None, "Station TT043_7-9"),
        ),
        Case(
            "multiple titles, first wins",
            [{"title": "Total carbon dioxide"}, {"title": "Meeting Heterogeneity"},],
            ("Total carbon dioxide", None, None),
        ),
        Case(
            "multiple titles, plus sub",
            [
                {"title": "Total carbon dioxide"},
                {"title": "Meeting Heterogeneity"},
                {"title": "Station TT043_7-9", "titleType": "Subtitle"},
            ],
            ("Total carbon dioxide", None, "Station TT043_7-9"),
        ),
        Case(
            "multiple titles, multiple subs",
            [
                {"title": "Total carbon dioxide"},
                {"title": "Meeting Heterogeneity"},
                {"title": "Station TT043_7-9", "titleType": "Subtitle"},
                {"title": "Some other subtitle", "titleType": "Subtitle"},
            ],
            ("Total carbon dioxide", None, "Station TT043_7-9"),
        ),
        Case(
            "title, original, sub",
            [
                {
                    "title": "Total carbon dioxide",
                    "original_language_title": "Всего углекислого газа",
                },
                {"title": "Station TT043_7-9", "titleType": "Subtitle"},
            ],
            ("Total carbon dioxide", "Всего углекислого газа", "Station TT043_7-9"),
        ),
        Case(
            "title, original same as title, sub",
            [
                {
                    "title": "Total carbon dioxide",
                    "original_language_title": {"__content__": "Total carbon dioxide",},
                },
                {"title": "Station TT043_7-9", "titleType": "Subtitle"},
            ],
            ("Total carbon dioxide", None, "Station TT043_7-9"),
        ),
        Case(
            "title, original dict, sub",
            [
                {
                    "title": "Total carbon dioxide",
                    "original_language_title": {
                        "__content__": "Всего углекислого газа",
                    },
                },
                {"title": "Station TT043_7-9", "titleType": "Subtitle"},
            ],
            ("Total carbon dioxide", "Всего углекислого газа", "Station TT043_7-9"),
        ),
    ]

    for case in cases:
        result = parse_datacite_titles(case.input)
        assert result == case.result, case.about


def test_parse_datacite_dates():
    """
    Test datacite date parsing.
    """
    Case = collections.namedtuple("Case", "about input result")
    cases = [
        Case("None is None", None, (None, None, None)),
        Case("empty list is None", [], (None, None, None)),
        Case("empty item is None", [{}], (None, None, None)),
        Case("year only yields year only", [{"date": "2019"}], (None, None, 2019)),
        Case("int year", [{"date": 2019}], (None, None, 2019)),
        Case("first wins", [{"date": "2019"}, {"date": "2020"}], (None, None, 2019)),
        Case(
            "skip bogus year", [{"date": "abc"}, {"date": "2020"}], (None, None, 2020)
        ),
        Case(
            "first with type",
            [{"date": "2019", "dateType": "Accepted"}, {"date": "2020"}],
            (None, None, 2019),
        ),
        Case(
            "full date",
            [{"date": "2019-12-01", "dateType": "Valid"},],
            (datetime.date(2019, 12, 1), 12, 2019),
        ),
        Case(
            "date type prio",
            [
                {"date": "2000-12-01", "dateType": "Valid"},
                {"date": "2010-01-01", "dateType": "Updated"},
            ],
            (datetime.date(2000, 12, 1), 12, 2000),
        ),
        Case(
            "date type prio, Available > Updated",
            [
                {"date": "2010-01-01", "dateType": "Updated"},
                {"date": "2000-12-01", "dateType": "Available"},
            ],
            (datetime.date(2000, 12, 1), 12, 2000),
        ),
        Case(
            "allow different date formats, Available > Updated",
            [
                {"date": "2010-01-01T10:00:00", "dateType": "Updated"},
                {"date": "2000-12-01T10:00:00", "dateType": "Available"},
            ],
            (datetime.date(2000, 12, 1), 12, 2000),
        ),
        Case(
            "allow different date formats, Available > Updated",
            [
                {"date": "2010-01-01T10:00:00Z", "dateType": "Updated"},
                {"date": "2000-12-01T10:00:00Z", "dateType": "Available"},
            ],
            (datetime.date(2000, 12, 1), 12, 2000),
        ),
        Case(
            "allow fuzzy date formats, Available > Updated",
            [
                {"date": "2010", "dateType": "Updated"},
                {"date": "2000 Dec 01", "dateType": "Available"},
            ],
            (datetime.date(2000, 12, 1), 12, 2000),
        ),
        Case(
            "fuzzy year only",
            [{"date": "Year 2010", "dateType": "Issued"},],
            (None, None, 2010),
        ),
        Case(
            "fuzzy year and month",
            [{"date": "Year 2010 Feb", "dateType": "Issued"},],
            (None, 2, 2010),
        ),
        Case(
            "fuzzy year, month, day",
            [{"date": "Year 2010 Feb 24", "dateType": "Issued"},],
            (datetime.date(2010, 2, 24), 2, 2010),
        ),
        Case(
            "ignore broken date",
            [{"date": "Febrrr 45", "dateType": "Updated"},],
            (None, None, None),
        ),
    ]
    for case in cases:
        result = parse_datacite_dates(case.input)
        assert result == case.result, case.about


def test_datacite_importer(datacite_importer):
    last_index = datacite_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/datacite_sample.jsonl", "r") as f:
        datacite_importer.bezerk_mode = True
        counts = JsonLinePusher(datacite_importer, f).run()
    assert counts["insert"] == 1
    assert counts["exists"] == 0
    assert counts["skip"] == 0

    # fetch most recent editgroup
    change = datacite_importer.api.get_changelog_entry(index=last_index + 1)
    eg = change.editgroup
    assert eg.description
    assert "datacite" in eg.description.lower()
    assert eg.extra["git_rev"]
    assert "fatcat_tools.DataciteImporter" in eg.extra["agent"]

    last_index = datacite_importer.api.get_changelog(limit=1)[0].index
    with open("tests/files/datacite_sample.jsonl", "r") as f:
        datacite_importer.bezerk_mode = False
        datacite_importer.reset()
        counts = JsonLinePusher(datacite_importer, f).run()
    assert counts["insert"] == 0
    assert counts["exists"] == 1
    assert counts["skip"] == 0
    assert last_index == datacite_importer.api.get_changelog(limit=1)[0].index


def test_datacite_dict_parse(datacite_importer):
    with open("tests/files/datacite_sample.jsonl", "r") as f:
        raw = json.load(f)
        r = datacite_importer.parse_record(raw)
        # ensure the API server is ok with format
        JsonLinePusher(datacite_importer, [json.dumps(raw)]).run()

        print(r.extra)
        assert r.title == "Triticum turgidum L. subsp. durum (Desf.) Husn. 97090"
        assert (
            r.publisher == "International Centre for Agricultural Research in Dry Areas"
        )
        assert r.release_type == "article"
        assert r.release_stage == "published"
        assert r.license_slug == None
        assert r.original_title == None
        assert r.ext_ids.doi == "10.18730/8dym9"
        assert r.ext_ids.isbn13 == None
        assert r.language == "en"
        assert r.subtitle == None
        assert r.release_date == None
        assert r.release_year == 1986
        assert "subtitle" not in r.extra
        assert "subtitle" not in r.extra["datacite"]
        assert "funder" not in r.extra
        assert "funder" not in r.extra["datacite"]
        # matched by ISSN, so shouldn't be in there
        # assert extra['container_name'] == "International Journal of Quantum Chemistry"
        assert r.extra["datacite"]["subjects"] == [
            {"subject": "Plant Genetic Resource for Food and Agriculture"}
        ]
        assert len(r.abstracts) == 1
        assert len(r.abstracts[0].content) == 421
        assert len(r.contribs) == 2
        assert r.contribs[0].raw_name == "GLIS Of The ITPGRFA"
        assert r.contribs[0].given_name == None
        assert r.contribs[0].surname == None
        assert len(r.refs) == 0


def test_datacite_conversions(datacite_importer):
    """
    Datacite JSON to release entity JSON representation. The count is hardcoded
    for now.
    """
    datacite_importer.debug = True
    for i in range(35):
        src = "tests/files/datacite/datacite_doc_{0:02d}.json".format(i)
        dst = "tests/files/datacite/datacite_result_{0:02d}.json".format(i)
        with open(src, "r") as f:
            re = datacite_importer.parse_record(json.load(f))
            result = entity_to_dict(re)
        with open(dst, "r") as f:
            expected = json.loads(f.read())

        assert result == expected, "output mismatch in {}".format(dst)


def test_index_form_to_display_name():
    Case = collections.namedtuple("Case", "input output")
    cases = [
        Case("", ""),
        Case("ABC", "ABC"),
        Case("International Space Station", "International Space Station"),
        Case("Jin, Shan", "Shan Jin"),
        Case(
            "Volkshochschule Der Bundesstadt Bonn",
            "Volkshochschule Der Bundesstadt Bonn",
        ),
        Case("Solomon, P. M.", "P. M. Solomon"),
        Case("Sujeevan Ratnasingham", "Sujeevan Ratnasingham"),
        Case(
            "Paul Stöckli (1906-1991), Künstler", "Paul Stöckli (1906-1991), Künstler"
        ),
    ]

    for c in cases:
        assert c.output == index_form_to_display_name(c.input)


def test_lookup_license_slug():
    Case = collections.namedtuple("Case", "input output")
    cases = [
        Case("https://opensource.org/licenses/MIT", "MIT"),
        Case("creativecommons.org/licenses/by-nc-nd/3.0/", "CC-BY-NC-ND"),
        Case("http://creativecommons.org/licences/by-nc-sa/4.0", "CC-BY-NC-SA"),
        Case("http://creativecommons.org/licenses/by-nc-nd/2.5/co", "CC-BY-NC-ND"),
        Case("http://creativecommons.org/licenses/by-nd/4.0/legalcode", "CC-BY-ND"),
        Case("http://creativecommons.org/licenses/by/2.0/uk/legalcode", "CC-BY"),
        Case("http://creativecommons.org/publicdomain/zero/1.0/legalcode", "CC-0"),
        Case("http://doi.wiley.com/10.1002/tdm_license_1.1", "WILEY-TDM-1.1"),
        Case("http://homepage.data-planet.com/terms-use", "SAGE-DATA-PLANET"),
        Case("http://www.springer.com/tdm", "SPRINGER-TDM"),
        Case(
            "https://archaeologydataservice.ac.uk/advice/termsOfUseAndAccess.xhtml",
            "ADS-UK",
        ),
        Case(
            "https://archaeologydataservice.ac.uk/advice/termsOfUseAndAccess", "ADS-UK"
        ),
        Case("https://creativecommons.org/public-domain/cc0", "CC-0"),
        Case("https://creativecommons.org/publicdomain/zero/1.0", "CC-0"),
        Case("https://creativecommons.org/share-your-work/public-domain/cc0", "CC-0"),
        Case("https://www.elsevier.com/tdm/userlicense/1.0", "ELSEVIER-USER-1.0"),
        Case("https://www.gnu.org/licenses/gpl-3.0.html", "GPL-3.0"),
        Case("http://rightsstatements.org/page/InC/1.0?language=en", "RS-INC"),
        Case("http://onlinelibrary.wiley.com/termsAndConditions", "WILEY"),
        Case("https://publikationen.bibliothek.kit.edu/kitopen-lizenz", "KIT-OPEN"),
        Case(
            "http://journals.sagepub.com/page/policies/text-and-data-mining-license",
            "SAGE-TDM",
        ),
        Case(
            "https://creativecommons.org/publicdomain/mark/1.0/deed.de",
            "CC-PUBLICDOMAIN",
        ),
        Case("http://creativecommons.org/publicdomain/mark/1.0", "CC-PUBLICDOMAIN"),
        Case("https://creativecommons.org/publicdomain/mark/1.0", "CC-PUBLICDOMAIN"),
        Case("https://creativecommons.org/publicdomain/mark/1.0/", "CC-PUBLICDOMAIN"),
        Case(
            "https://creativecommons.org/publicdomain/mark/1.0/deed.de",
            "CC-PUBLICDOMAIN",
        ),
        Case("https://creativecommons.org/share-your-work/public-domain/cc0/", "CC-0"),
        Case("http://spdx.org/licenses/CC0-1.0.json", "CC-0"),
        Case("http://spdx.org/licenses/CC-BY-1.0.json", "CC-BY"),
        Case("http://spdx.org/licenses/CC-BY-4.0.json", "CC-BY"),
        Case("http://spdx.org/licenses/CC-BY-NC-4.0.json", "CC-BY-NC"),
        Case("http://spdx.org/licenses/CC-BY-SA-3.0.json", "CC-BY-SA"),
        Case("http://spdx.org/licenses/CC-BY-SA-4.0.json", "CC-BY-SA"),
        Case("http://spdx.org/licenses/MIT.json", "MIT"),
        Case("http://spdx.org/licenses/OGL-Canada-2.0.json", "OGL-CANADA"),
    ]

    for c in cases:
        got = lookup_license_slug(c.input)
        assert c.output == got, "{}: got {}, want {}".format(c.input, got, c.output)


def test_contributor_list_contains_contributor():
    Case = collections.namedtuple("Case", "contrib_list contrib want")
    cases = [
        Case([], fatcat_openapi_client.ReleaseContrib(raw_name="Paul Katz"), False),
    ]
    for c in cases:
        got = contributor_list_contains_contributor(c.contrib_list, c.contrib)
        assert got == c.want
