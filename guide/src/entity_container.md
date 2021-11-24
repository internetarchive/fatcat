
# Container Entity Reference

## Fields

- `name` (string, required): The title of the publication, as used in
  international indexing services. Eg, "Journal of Important Results". Not
  necessarily in the native language, but also not necessarily in English.
  Alternative titles (and translations) can be stored in "extra" metadata (see
  below)
- `container_type` (string): eg, journal vs. conference vs. book series.
  Controlled vocabulary is described below.
- `publication_status` (string): whether actively publishing, never published
  anything, or discontinued. Controlled vocabularity is described below.
- `publisher` (string): The name of the publishing organization. Eg, "Society
  of Curious Students".
- `issnl` (string): an external identifier, with registration controlled by the
  [ISSN organization](http://www.issn.org/). Registration is relatively
  inexpensive and easy to obtain (depending on world region), so almost all
  serial publications have one. The ISSN-L ("linking ISSN") is one of either
  the print (`issp`) or electronic (`issne`) identifiers for a serial
  publication; not all publications have both types of ISSN, but many do, which
  can cause confusion. The ISSN master list is not gratis/public, but the
  ISSN-L mapping is.
- `issne` (string): Electronic ISSN ("ISSN-E")
- `issnp` (string): Print ISSN ("ISSN-P")
- `wikidata_qid` (string): external linking identifier to a Wikidata entity.

#### `extra` Fields

- `abbrev` (string): a commonly used abbreviation for the publication, as used
  in citations, following the [ISO 4][] standard. Eg, "Journal of Polymer
  Science Part A" -> "J. Polym. Sci. A"
- `acronym` (string): acronym of publication name. Usually all upper-case, but
  sometimes a very terse, single-word truncated form of the name (eg, a pun).
- `coden` (string): an external identifier, the [CODEN code][]. 6 characters,
  all upper-case.
- `default_license` (string, slug): short name (eg, "CC-BY-SA") for the
  default/recommended license for works published in this container
- `original_name` (string): native name (if `name` is translated)
- `platform` (string): hosting platform: OJS, wordpress, scielo, etc
- `mimetypes` (array of string): formats that this container publishes all works
  under (eg, 'application/pdf', 'text/html')
- `first_year` (integer): first year of publication
- `last_year` (integer): final year of publication (implies that container is no longer active)
- `languages` (array of strings): ISO codes; the first entry is considered the
  "primary" language (if that makes sense)
- `country` (string): ISO abbreviation (two characters) for the country this
  container is published in
- `aliases` (array of strings): significant alternative names or abbreviations
  for this container (not just capitalization/punctuation)
- `region` (string, slug): continent/world-region (vocabulary is TODO)
- `discipline` (string, slug): highest-level subject aread (vocabulary is TODO)
- `urls` (array of strings): known homepage URLs for this container (first in array is default)
- `issnp` (deprecated; string): Print ISSN; deprecated now that there is a top-level field
- `issne` (deprecated; string): Electronic ISSN; deprecated now that there is a top-level field

Additional fields used in analytics and "curration" tracking:

- `doaj` (object)
  - `as_of` (string, ISO datetime): datetime of most recent check; if not set,
    not actually in DOAJ
  - `seal` (bool): has DOAJ seal
  - `work_level` (bool): whether work-level publications are registered with DOAJ
  - `archive` (array of strings): preservation archives
- `road` (object)
  - `as_of` (string, ISO datetime): datetime of most recent check; if not set,
    not actually in ROAD
- `kbart` (object)
  - `lockss`, `clockss`, `portico`, `jstor` etc (object)
    - `year_spans` (array of arrays of integers (pairs)): year spans (inclusive)
      for which the given archive has preserved this container
    - `volume_spans` (array of arrays of integers (pairs)): volume spans (inclusive)
      for which the given archive has preserved this container
- `sherpa_romeo` (object):
    - `color` (string): the SHERPA/RoMEO "color" of the publisher of this container
- `doi`: TODO: include list of prefixes and which (if any) DOI registrar is used
- `dblp` (object):
  - `prefix` (string): prefix of dblp keys published as part of this container
    (eg, 'journals/blah' or 'conf/xyz')
- `ia` (object): Internet Archive specific fields
  - `sim` (object): same format as `kbart` preservation above; coverage in microfilm collection
  - `longtail` (bool): is this considered a "long-tail" open access venue
- `publisher_type` (string): controlled vocabulary

For KBART and other "coverage" fields, we "over-count" on the assumption that
works with "in-progress" status will soon actually be preserved. Elements of
these arrays are either an integer (means that single year is preserved), or an
array of length two (meaning everything between the two numbers (inclusive) is
preserved).

[CODEN]: https://en.wikipedia.org/wiki/CODEN

#### `container_type` Vocabulary

- `journal`
- `proceedings`
- `conference-series`
- `book-series`
- `blog`
- `magazine`
- `trade`
- `test`

#### `publication_status` Vocabulary

- `active`: ongoing publication of new releases
- `suspended`: publication has stopped, but may continue in the future
- `discontinued`: publication has permanently ceased
- `vanished`: publication has stopped, and public traces have vanished (eg,
  publisher website has disappeared with no notice)
- `never`: no works were ever published under this container
- `one-time`: releases were all published as a one-time even. for example, a
  single instance of a conference, or a fixed-size book series
