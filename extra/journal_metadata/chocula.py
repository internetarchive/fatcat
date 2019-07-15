#!/usr/bin/env python3

"""
Count Chocula - online serials metadata and stats

  "one, two, three, un-preserved web-native open-access long-tail indie
  journals, hah, hah, hah!"

  (yeah, I know, this name isn't very good)
  (see also: https://teamyacht.com/ernstchoukula.com/Ernst-Choukula.html)

Commands:

    everything
    init_db
    summarize

    index_doaj
    index_road
    index_crossref
    index_entrez
    index_norwegian
    index_szczepanski
    index_ezb

    load_fatcat
    load_fatcat_stats

    export_urls
    update_url_status

Future commands:

    fatcat_edits
    index_jurn
    index_wikidata
    index_datacite
    preserve_kbart --keeper SLUG
    preserve_sim

TODO:
- KBART imports (with JSON, so only a single row per slug)
- check that all fields actually getting imported reasonably
- imprint/publisher distinction (publisher is big group)
- summary table should be superset of fatcat table
- add timestamp columns to enable updates?
- "GOLD" importer (for scopus/WoS)
- fatcat export (filters for changes to make, writes out as JSON)
- homepage crawl/status script
- update_url_status (needs re-write)
- index -> directory
- log out index issues (duplicate ISSN-L, etc)
- validate against GOLD OA list
- decide what to do with JURN... match? create missing fatcat?
x load_fatcat
x fatcat_stats (also add timestamp column)
x export_url
"""

import sys, csv, json
from collections import Counter
import sqlite3
import argparse

import ftfy
import urlcanon
import surt
import tldextract
import pycountry


################### File Config

ISSNL_FILE = 'data/20190220.ISSN-to-ISSN-L.txt'

ENTREZ_FILE = 'data/entrez-journals.csv'
ROAD_FILE = 'data/road-2018-01-24.tsv'
ROAD_DATE = '2018-01-24'
DOAJ_FILE = 'data/doaj_20190124.csv'
DOAJ_DATE = '2019-01-24'
CROSSREF_FILE = 'data/doi_titles_file_2019-01-24.csv'
SHERPA_ROMEO_JOURNAL_FILE = 'data/romeo-journals.csv'
SHERPA_ROMEO_POLICY_FILE = 'data/romeo-policies.csv'
NORWEGIAN_FILE = 'data/2018-03-02 Norwegian Register for Scientific Journals and Series.csv'
NORWEGIAN_DATE = '2018-03-02'
LOCKSS_FILE = 'data/kbart_LOCKSS.txt'
CLOCKSS_FILE = 'data/kbart_CLOCKSS.txt'
PORTICO_FILE = 'data/Portico_Holding_KBart.txt'
JSTOR_FILE = 'data/jstor_all-archive-titles.txt'
SIM_FILE = 'data/MASTER TITLE_METADATA_LIST_20171019.converted.csv'
IA_CRAWL_FILE = 'data/journal_homepage_results.partial.tsv'
SZCZEPANSKI_FILE = 'data/Jan-Szczepanski-Open-Access-Journals-2018_0.fixed.json'
EZB_FILE = 'data/ezb_metadata.json'
FATCAT_CONTAINER_FILE = 'data/container_export.json'
FATCAT_STATS_FILE = 'data/container_stats.json'


################### Utilities

# NOTE: this is a partial list, focusing on non-publisher hosted platforms and
# software frameworks
PLATFORM_MAP = {
    'OJS': 'ojs',
    'BMC': 'bmc',
    'SciELO Brazil': 'scielo',
    'SciELO Argentina': 'scielo',
    'SciELO': 'scielo',
    'SciELO Mexico': 'scielo',
    'SciELO Spain': 'scielo',
    'SciELO Portugal': 'scielo',
    'WordPress': 'wordpress',
    'Sciendo': 'sciendo',
    'Drupal': 'drupal',
    'revues.org': 'openedition',
}

MIMETYPE_MAP = {
    'PDF': 'application/pdf',
    'HTML': 'text/html',
    'XML': 'application/xml',
}

def unquote(s):
    if s.startswith('"'):
        s = s[1:]
    if s.endswith('"'):
        s = s[:-1]
    if s.endswith('.'):
        s = s[:-1]
    return s.strip()

def parse_lang(s):
    if not s or s in ('Not applicable', 'Multiple languages', 'Unknown'):
        return None
    try:
        if len(s) == 2:
            lang = pycountry.languages.get(alpha2=s.lower())
        elif len(s) == 3:
            lang = pycountry.languages.get(alpha3=s.lower())
        else:
            lang = pycountry.languages.get(name=s)
        return lang.alpha2.lower()
    except KeyError:
        return None
    except AttributeError:
        return None

def parse_country(s):
    if not s or s in ('Unknown'):
        return None
    try:
        if len(s) == 2:
            country = pycountry.countries.get(alpha2=s.lower())
        else:
            country = pycountry.countries.get(name=s)
    except KeyError:
        return None
    if country:
        return country.alpha_2.lower()
    else:
        return None

def parse_mimetypes(val):
    # XXX: multiple mimetypes?
    if not val:
        return
    mimetype = None
    if '/' in val:
        mimetype = val
    else:
        mimetype = MIMETYPE_MAP.get(val)
    if not mimetype:
        return None
    return [mimetype]

def gaps_to_spans(first, last, gaps):
    if not gaps:
        return [[first, last]]
    if not (last >= first and max(gaps) < last and min(gaps) > first):
        # mangled
        print("mangled years: {}".format((first, last, gaps)))
        return []
    full = list(range(first, last+1))
    for missing in gaps:
        full.remove(missing)
    spans = []
    low = None
    last = None
    for year in full:
        if not low:
            low = year
            last = year
            continue
        if year != last+1:
            spans.append([low, last])
            low = year
            last = year
        last = year
    if low:
        spans.append([low, last])
    return spans

def test_gaps():
    assert gaps_to_spans(1900, 1900, None) == \
        [[1900, 1900]]
    assert gaps_to_spans(1900, 1903, None) == \
        [[1900, 1903]]
    assert gaps_to_spans(1900, 1902, [1901]) == \
        [[1900, 1900], [1902, 1902]]
    assert gaps_to_spans(1950, 1970, [1955, 1956, 1965]) == \
        [[1950, 1954], [1957, 1964], [1966, 1970]]

def merge_spans(old, new):
    if not new:
        return old
    if not old:
        old = []
    old.extend(new)
    years = set()
    for span in old:
        for y in range(span[0], span[1]+1):
            years.add(y)
    if not years:
        return []
    spans = []
    start = None
    last = None
    todo = False
    for y in sorted(list(years)):
        if start == None:
            # very first
            start = y
            last = y
            todo = True
            continue
        if y == last + 1:
            # span continues
            last = y
            todo = True
            continue
        # a gap just happened!
        spans.append([start, last])
        start = y
        last = y
        todo = True
    if todo:
        spans.append([start, last])
    return spans

def test_merge_spans():
    assert merge_spans([[5, 10]], [[10, 20]]) == \
        [[5, 20]]
    assert merge_spans([[5, 9]], [[10, 20]]) == \
        [[5, 20]]
    assert merge_spans([[5, 11]], [[10, 20]]) == \
        [[5, 20]]
    assert merge_spans([], []) == \
        []
    assert merge_spans([[9, 11]], []) == \
        [[9,11]]
    assert merge_spans([[2000, 2000]], [[1450, 1900]]) == \
        [[1450, 1900], [2000, 2000]]


################### Main Class

class ChoculaDatabase():

    def __init__(self, db_file):
        self._issn_issnl_map = dict()
        self.db = sqlite3.connect(db_file, isolation_level='EXCLUSIVE')
        self.data = dict()
        self.c = None

    def read_issn_map_file(self, issn_map_path):
        print("##### Loading ISSN map file...")
        with open(issn_map_path, 'r') as issn_map_file:
            self._issn_issnl_map = dict()
            for line in issn_map_file:
                if line.startswith("ISSN") or len(line) == 0:
                    continue
                (issn, issnl) = line.split()[0:2]
                self._issn_issnl_map[issn] = issnl
                # double mapping makes lookups easy
                self._issn_issnl_map[issnl] = issnl
        print("Got {} ISSN-L mappings.".format(len(self._issn_issnl_map)))

    def issn2issnl(self, issn):
        if issn is None:
            return None
        return self._issn_issnl_map.get(issn)

    def add_issn(self, index_slug, raw_issn=None, issne=None, issnp=None, identifier=None, name=None, publisher=None, extra=None):

        # do ISSN => ISSN-L mappings for any raw ISSNs
        issnl = None
        if not (raw_issn or issne or issnp):
            return None, 'no-issn'
        for lookup in (issnp, issne, raw_issn):
            if not lookup:
                continue
            lookup = lookup.strip().upper()
            #if not (len(lookup) == 9 and lookup[4] == '-'):
            #    print(lookup)
            #    print(len(lookup))
            #    print(lookup[4])
            #    return None, 'invalid-issn'
            #assert len(lookup) == 9 and lookup[4] == '-'
            issnl = self.issn2issnl(lookup)
            if issnl:
                break
        if not issnl:
            return None, 'no-issnl'
            #print((raw_issn, issne, issnp))
            # UGH.
            #issnl = issne or issnp or raw_issn
            #if not issnl:
            #issnl = issnl.strip().upper()
            #assert len(issnl) == 9 and issnl[4] == '-'
            #status = 'found-munge'
        else:
            status = 'found'

        if extra == None:
            extra = dict()

        if issne:
            extra['issne'] = issne
        if issnp:
            extra['issnp'] = issnp

        if publisher:
            publisher = unquote(ftfy.fix_text(publisher))
        if publisher:
            extra['publisher'] = publisher

        if extra:
            extra = json.dumps(extra)
        else:
            extra = None

        try:
            self.c.execute("INSERT INTO journal_index VALUES (?,?,?,?,?,?)",
                (issnl, index_slug, identifier, name, None, extra))
            status = 'inserted'
        except sqlite3.IntegrityError as ie:
            if str(ie).startswith("UNIQUE"):
                return None, "duplicate-issnl"
            raise ie

        return issnl, status

    def add_url(self, issnl, url):
        if not (url and issnl) or 'mailto:' in url.lower() or url in ('http://n/a', 'http://N/A'):
            return
        if url.startswith('www.'):
            url = "http://" + url
        url.replace('Http://', 'http://')

        url = str(urlcanon.semantic_precise(url))
        url_surt = surt.surt(url)
        tld = tldextract.extract(url)
        domain = '.'.join(tld[:])

        self.c.execute("INSERT OR REPLACE INTO homepage (issnl, surt, url, host, domain, suffix) VALUES (?,?,?,?,?,?)",
            (issnl, url_surt, url, tld.domain, tld.registered_domain, tld.suffix))

    def index_entrez(self, args):
        path = args.input_file or ENTREZ_FILE
        print("##### Loading Entrez...")
        # JrId,JournalTitle,MedAbbr,"ISSN (Print)","ISSN (Online)",IsoAbbr,NlmId
        reader = csv.DictReader(open(path))
        counts = Counter()
        self.c = self.db.cursor()
        for row in reader:
            if not (row.get('ISSN (Online)') or row.get('ISSN (Print)')):
                counts['skipped'] += 1
                continue
            extra = dict()
            if row['IsoAbbr']:
               extra['abbrev'] = row['IsoAbbr'].strip()
            issnl, status = self.add_issn(
                'entrez',
                issne=row.get('ISSN (Online)'),
                issnp=row.get('ISSN (Print)'),
                name=row['JournalTitle'],
                extra=extra,
            )
            counts[status] += 1
        self.c.close()
        self.db.commit()
        print(counts)

    def index_road(self, args):
        path = args.input_file or ROAD_FILE
        print("##### Loading ROAD...")
        reader = csv.DictReader(open(path), delimiter='\t',
            fieldnames=("ISSN", "ISSN-L", "Short Title", "Title", "Publisher", "URL1", "URL2", "Region", "Lang1", "Lang2")
        )
        counts = Counter()
        self.c = self.db.cursor()
        for row in reader:
            extra = dict()
            if row['Lang1']:
                extra['langs'] = [row['Lang1']]
            if row['Lang2']:
                extra['langs'].append(row['Lang2'])
            # TODO: region mapping: "Europe and North America"
            # TODO: lang mapping: already alpha-3
            issnl, status = self.add_issn(
                'road',
                raw_issn=row['ISSN-L'],
                name=row['Short Title'],
                publisher=row['Publisher'],
            )
            counts[status] += 1
            if not issnl:
                continue
            if row['URL1']:
                self.add_url(issnl, row['URL1'])
            if row['URL2']:
                self.add_url(issnl, row['URL2'])
        self.c.close()
        self.db.commit()
        print(counts)

    def index_doaj(self, args):
        path = args.input_file or DOAJ_FILE
        print("##### Loading DOAJ...")
        #Journal title,Journal URL,Alternative title,Journal ISSN (print version),Journal EISSN (online version),Publisher,Society or institution,"Platform, host or aggregator",Country of publisher,Journal article processing charges (APCs),APC information URL,APC amount,Currency,Journal article submission fee,Submission fee URL,Submission fee amount,Submission fee currency,Number of articles publish in the last calendar year,Number of articles information URL,Journal waiver policy (for developing country authors etc),Waiver policy information URL,Digital archiving policy or program(s),Archiving: national library,Archiving: other,Archiving infomation URL,Journal full-text crawl permission,Permanent article identifiers,Journal provides download statistics,Download statistics information URL,First calendar year journal provided online Open Access content,Full text formats,Keywords,Full text language,URL for the Editorial Board page,Review process,Review process information URL,URL for journal's aims & scope,URL for journal's instructions for authors,Journal plagiarism screening policy,Plagiarism information URL,Average number of weeks between submission and publication,URL for journal's Open Access statement,Machine-readable CC licensing information embedded or displayed in articles,URL to an example page with embedded licensing information,Journal license,License attributes,URL for license terms,Does this journal allow unrestricted reuse in compliance with BOAI?,Deposit policy directory,Author holds copyright without restrictions,Copyright information URL,Author holds publishing rights without restrictions,Publishing rights information URL,DOAJ Seal,Tick: Accepted after March 2014,Added on Date,Subjects
        reader = csv.DictReader(open(path))
        counts = Counter()
        self.c = self.db.cursor()
        for row in reader:

            extra = dict(as_of=DOAJ_DATE)
            extra['mimetypes'] = parse_mimetypes(row['Full text formats'])
            platform = PLATFORM_MAP.get(row['Platform, host or aggregator'])
            if platform:
                extra['platform'] = platform
            if row['DOAJ Seal']:
                extra['seal'] = {"no": False, "yes": True}[row['DOAJ Seal'].lower()]
            if row['Country of publisher']:
                extra['country'] = parse_country(row['Country of publisher'])
            row['lang'] = parse_lang(row['Full text language'])
            # TODO: work_level: bool (are work-level publications deposited with DOAJ?)

            if row['Digital archiving policy or program(s)']:
                extra['archive'] = [a.strip() for a in row['Digital archiving policy or program(s)'].split(',') if a.strip()]
            elif row['Archiving: national library']:
                extra['archive'] = ['national-library']

            crawl_permission = row['Journal full-text crawl permission']
            if crawl_permission:
                extra['crawl-permission'] = dict(Yes=True, No=False)[crawl_permission]
            # TODO: Permanent article identifiers
            default_license = row['Journal license']
            if default_license and default_license.startswith('CC'):
                extra['default_license'] = default_license.replace('CC ', 'CC-').strip()

            issnl, status = self.add_issn(
                'doaj',
                issnp=row['Journal ISSN (print version)'],
                issne=row['Journal EISSN (online version)'],
                name=row['Journal title'],
                publisher=row['Publisher'],
                extra=extra,
            )
            if row['Journal URL']:
                self.add_url(issnl, row['Journal URL'])
            counts[status] += 1

            # TODO: Subjects
        self.c.close()
        self.db.commit()
        print(counts)

    def index_sherpa_romeo(self, args):
        journal_path = args.input_file or SHERPA_ROMEO_JOURNAL_FILE
        policy_path = SHERPA_ROMEO_POLICY_FILE
        # first load policies
        print("##### Loading SHERPA/ROMEO policies...")
        #RoMEO Record ID,Publisher,Policy Heading,Country,RoMEO colour,Published Permission,Published Restrictions,Published Max embargo,Accepted Prmission,Accepted Restrictions,Accepted Max embargo,Submitted Permission,Submitted Restrictions,Submitted Max embargo,Open Access Publishing,Record Status,Updated
        policies = dict()
        fixed_policy_file = ftfy.fix_file(open(policy_path, 'rb'))
        policy_reader = csv.DictReader(fixed_policy_file)
        for row in policy_reader:
            policies[row['RoMEO Record ID']] = row
        print("##### Loading SHERPA/ROMEO journal metadata...")
        #Journal Title,ISSN,ESSN,URL,RoMEO Record ID,Updated
        # super mangled :(
        raw_file = open(journal_path, 'rb').read().decode(errors='replace')
        fixed_file = ftfy.fix_text(raw_file)
        reader = csv.DictReader(fixed_file.split('\n'))
        counts = Counter()
        self.c = self.db.cursor()
        for row in reader:
            #row['Journal Title'] = row.pop('\ufeffJournal Title')
            row.update(policies[row['RoMEO Record ID']])
            extra = dict()
            if row['RoMEO colour']:
                extra['color'] = row['RoMEO colour']
            # row['Open Access Publishing']
            if row['Country']:
                extra['country'] = parse_country(row['Country'])
            issnl, status = self.add_issn(
                'sherpa_romeo',
                issnp=row['ISSN'],
                issne=row['ESSN'],
                name=row['Journal Title'],
                publisher=row['Publisher'],
                extra=extra,
            )
            counts[status] += 1
            if not issnl:
                continue
        self.c.close()
        self.db.commit()
        print(counts)

    def index_norwegian(self, args):
        path = args.input_file or NORWEGIAN_FILE
        print("##### Loading Norwegian Registry...")
        #pandas.read_csv(NORWEGIAN_FILE, sep=';', encoding="ISO-8859-1")
        #NSD tidsskrift_id;Original title;International title;Present Level (2018);Print ISSN;Online ISSN;Open Access;NPI Scientific Field;NPI Academic Discipline;URL;Publishing Company;Publisher;Country of publication;Language;Level 2019;Level 2018;Level 2017;Level 2016;Level 2015;Level 2014;Level 2013;Level 2012;Level 2011;Level 2010;Level 2009;Level 2008;Level 2007;Level 2006;Level 2005;Level 2004;itar_id
        reader = csv.DictReader(open(path, encoding="ISO-8859-1"), delimiter=";")
        counts = Counter()
        self.c = self.db.cursor()
        for row in reader:
            issnp = row['Print ISSN']
            issne = row['Online ISSN']
            if issne and len(issne.strip()) != 9:
                issne = None
            if issnp and len(issnp.strip()) != 9:
                issnp = None
            if not (issnp or issne):
                counts['no-issn'] += 1
                continue
            extra = dict(as_of=NORWEGIAN_DATE)
            extra['level'] = int(row['Present Level (2018)'])
            if row['Original title'] != row['International title']:
                extra['original_name'] = row['Original title']
            if row['Country of publication']:
                extra['country'] = parse_country(row['Country of publication'])
            if row['Language']:
                extra['lang'] = parse_lang(row['Language'])
            issnl, status = self.add_issn(
                'norwegian',
                issnp=issnp,
                issne=issne,
                identifier=row['NSD tidsskrift_id'],
                name=row['International title'],
                publisher=row['Publisher'],
                extra=extra,
            )
            counts[status] += 1
            if not issnl:
                continue
            if row['URL']:
                self.add_url(issnl, row['URL'])
        self.c.close()
        self.db.commit()
        print(counts)

    def index_szczepanski(self, args):
        path = args.input_file or SZCZEPANSKI_FILE
        print("##### Loading Szczepanski...")
        # JSON
        json_file = open(path, 'r')
        counts = Counter()
        self.c = self.db.cursor()
        for row in json_file:
            if not row:
                continue
            row = json.loads(row)
            if not (row.get('issne') or row.get('issnp') or row.get('issn')):
                #print(row)
                counts['no-issn'] += 1
                continue
            extra = dict()
            if row.get('extra'):
                extra['notes'] = row.get('extra')
            for k in ('other_titles', 'year_spans', 'ed'):
                if row.get(k):
                    extra[k] = row[k]
            issnl, status = self.add_issn(
                'szczepanski',
                issne=row.get('issne'),
                issnp=row.get('issnp'),
                raw_issn=row.get('issn'),
                name=row['title'],
                publisher=row.get('ed'),
                extra=extra,
            )
            counts[status] += 1
            if not issnl:
                continue
            for url in row.get('urls', []):
                self.add_url(issnl, url['url'])
        self.c.close()
        self.db.commit()
        print(counts)

    def index_ezb(self, args):
        path = args.input_file or EZB_FILE
        print("##### Loading EZB...")
        # JSON
        json_file = open(path, 'r')
        counts = Counter()
        self.c = self.db.cursor()
        for row in json_file:
            if not row:
                continue
            row = json.loads(row)
            if not (row.get('issne') or row.get('issnp')):
                #print(row)
                counts['no-issn'] += 1
                continue
            extra = dict()
            for k in ('ezb_color', 'subjects', 'keywords', 'zdb_id',
                      'first_volume', 'first_issue', 'first_year',
                      'appearance', 'costs'):
                if row.get(k):
                    extra[k] = row[k]
            issnl, status = self.add_issn(
                'ezb',
                issne=row.get('issne'),
                issnp=row.get('issnp'),
                identifier=row['ezb_id'],
                name=row['title'],
                publisher=row.get('publisher'),
                extra=extra,
            )
            counts[status] += 1
            if not issnl:
                continue
            if row.get('url'):
                self.add_url(issnl, row['url'])
        self.c.close()
        self.db.commit()
        print(counts)

    def index_kbart(self, name, path):
        print("##### Loading KBART file for {}...".format(name))
        #publication_title      print_identifier        online_identifier       date_first_issue_online num_first_vol_online    num_first_issue_online  date_last_issue_online  num_last_vol_online     num_last_issue_online   title_url       first_author    title_id        embargo_info    coverage_depth  coverage_notes  publisher_name
        raw_file = open(path, 'rb').read().decode(errors='replace')
        fixed_file = ftfy.fix_text(raw_file)
        reader = csv.DictReader(fixed_file.split('\n'), delimiter='\t')
        counts = Counter()
        self.c = self.db.cursor()
        for row in reader:
            if not row['print_identifier'] and not row['online_identifier']:
                counts['no-issn'] += 1
                continue
            issnl, status = self.add_issn(
                issnp=row['print_identifier'],
                issne=row['online_identifier'],
                name=row['publication_title'],
                publisher=row['publisher_name'],
                extra=extra,
            )
            counts[status] += 1
            if not issnl:
                continue
            d = self.data[issnl]
            if not 'kbart' in d:
                self.data[issnl]['kbart'] = dict()
                d = self.data[issnl]
            if not name in d['kbart']:
                self.data[issnl]['kbart'][name] = dict()
            old_spans = self.data[issnl]['kbart'].get(name, dict()).get('year_spans', [])
            kbart = dict()
            if row['date_first_issue_online'] and row['date_last_issue_online']:
                start = int(row['date_first_issue_online'][:4])
                end = int(row['date_last_issue_online'][:4])
                if not start <= end:
                    print("{}: {} not before {}! er, mangling".format(
                        issnl,
                        row['date_first_issue_online'],
                        row['date_last_issue_online']))
                    new_spans = [[end, start]]
                else:
                    new_spans = [[start, end]]
                self.data[issnl]['kbart'][name]['year_spans'] = merge_spans(old_spans, new_spans)
        self.c.close()
        self.db.commit()
        print(counts)

    def index_crossref(self, args):
        path = args.input_file or CROSSREF_FILE
        print("##### Loading Crossref...")
        #"JournalTitle","JournalID","Publisher","pissn","eissn","additionalIssns","doi","(year1)[volume1]issue1,issue2,issue3(year2)[volume2]issue4,issues5"
        reader = csv.DictReader(open(path))
        counts = Counter()
        self.c = self.db.cursor()
        for row in reader:
            if row['pissn'] and len(row['pissn']) == 8:
                row['pissn'] = row['pissn'][:4] + '-' + row['pissn'][4:]
            if row['eissn'] and len(row['eissn']) == 8:
                row['eissn'] = row['eissn'][:4] + '-' + row['eissn'][4:]
            if row['additionalIssns'] and len(row['additionalIssns']) == 8:
                row['additionalIssns'] = row['additionalIssns'][:4] + '-' + row['additionalIssns'][4:]
            if not (row['pissn'] or row['eissn'] or row['additionalIssns']):
                #print(row)
                counts['no-issn'] += 1
                continue
            extra = dict()
            issnl, status = self.add_issn(
                'crossref',
                issnp=row['pissn'],
                issne=row['eissn'],
                raw_issn=row['additionalIssns'],
                identifier=row.get('doi'),
                name=row['JournalTitle'],
                publisher=row['Publisher'],
                extra=extra,
            )
            counts[status] += 1
        self.c.close()
        self.db.commit()
        print(counts)

    def index_sim(self, args):
        path = args.input_file or SIM_FILE
        print("##### Loading SIM Metadata...")
        #NA Pub Cat ID,Title,Publisher,ISSN,Impact Rank,Total Cities,Journal Impact Factor,Eigenfact or Score,First Volume,Last Volume,NA Gaps,"Scholarly / Peer-\n Reviewed","Peer-\n Reviewed",Pub Type,Pub Language,Subjects
        reader = csv.DictReader(open(path))
        counts = Counter()
        self.c = self.db.cursor()
        for row in reader:
            if not row['ISSN'] or row['ISSN'] == "NULL":
                counts['no-issn'] += 1
                continue
            issnl, status = self.add_issn(
                'ia_sim',
                raw_issn=row['ISSN'][:9],
                name=row['Title'],
                publisher=row['Publisher'],
                extra=extra,
            )
            counts[status] += 1
            if not issnl:
                continue
            d = self.data[issnl]
            sim = dict()
            sim['id'] = row['NA Pub Cat ID']
            first_year = row['First Volume']
            if first_year:
                first_year = int(first_year)
                sim['first_year'] = int(row['First Volume'])
            else:
                first_year = None
            last_year = row['Last Volume']
            if last_year:
                last_year = int(last_year)
                sim['last_year'] = last_year
            else:
                last_year = None
            gaps = [int(g) for g in row['NA Gaps'].split(';') if g.strip()]
            if gaps:
                sim['gaps'] = gaps
            if first_year and last_year:
                sim['year_spans'] = gaps_to_spans(first_year, last_year, gaps)
            if row['Pub Language']:
                self.add_lang(issnl, row['Pub Language'])
            # TODO: 'Pub Type'
            all_keys = list(sim.keys())
            for k in all_keys:
                if not sim[k]:
                    sim.pop(k)
            self.data[issnl]['sim'] = sim
        self.c.close()
        self.db.commit()
        print(counts)

    def update_url_status(self, args):
        path = args.input_file or IA_CRAWL_FILE
        print("##### Loading IA Homepage Crawl Results...")
        reader = csv.DictReader(open(path), delimiter='\t',
            fieldnames=("ISSN", "first_url", "first_status", "last_status", "last_url")
        )
        counts = Counter()
        self.c = self.db.cursor()
        for row in reader:
            counts['total'] += 1
            url = row['first_url']
            assert(url)
            self.c.execute("UPDATE homepage SET status_code=?, terminal_url=?, terminal_status_code=? WHERE url=?",
                (row['first_status'], row['last_url'], row['last_status'], url))
            counts['updated'] += 1
        self.c.close()
        self.db.commit()
        print(counts)

    def load_fatcat(self, args):
        path = args.input_file or FATCAT_CONTAINER_FILE
        print("##### Loading Fatcat Container Entities...")
        # JSON
        json_file = open(path, 'r')
        counts = Counter()
        self.c = self.db.cursor()
        for row in json_file:
            if not row:
                continue
            row = json.loads(row)
            if row['state'] != 'active':
                continue
            counts['total'] += 1
            extra = row.get('extra', dict())
            issne = extra.get('issne')
            issnp = extra.get('issnp')
            country = extra.get('country')
            languages = extra.get('languages', [])
            lang = None
            if languages:
                lang = languages[0]
            try:
                self.c.execute("INSERT OR REPLACE INTO fatcat_container (issnl, ident, revision, issne, issnp, wikidata_qid, name, container_type, publisher, country, lang) VALUES (?,?,?,?,?,?,?,?,?,?,?)",
                    (row['issnl'], row['ident'], row['revision'], issne, issnp,
                     row.get('wikidata_qid'), row['name'],
                     row.get('container_type'), extra.get('publisher'), country,
                     lang))
            except sqlite3.IntegrityError as ie:
                if str(ie).startswith("UNIQUE"):
                    return None, "duplicate-issnl"
                raise ie
            counts['inserted'] += 1
            if row.get('issnl'):
                urls = extra.get('urls', [])
                for url in urls:
                    self.add_url(row['issnl'], url)
        self.c.close()
        self.db.commit()
        print(counts)

    def load_fatcat_stats(self, args):
        path = args.input_file or FATCAT_STATS_FILE
        print("##### Loading Fatcat Container Stats...")
        # JSON
        json_file = open(path, 'r')
        counts = Counter()
        self.c = self.db.cursor()
        for row in json_file:
            if not row:
                continue
            row = json.loads(row)
            total = int(row['total'])
            if total > 0:
                ia_frac = float(row['in_web'])/total
                preserved_frac = float(row['is_preserved'])/total
            else:
                ia_frac = None
                preserved_frac = None
            self.c.execute("UPDATE fatcat_container SET release_count = ?, ia_count = ?, ia_frac = ?, preserved_count = ?, preserved_frac = ? WHERE issnl = ?",
                (total, row['in_web'], ia_frac, row['is_preserved'], preserved_frac, row['issnl']))
            counts['updated'] += 1
        self.c.close()
        self.db.commit()
        print(counts)

    def export_urls(self, args):
        self.c = self.db.cursor()
        self.db.row_factory = sqlite3.Row
        cur = self.db.execute("SELECT issnl, url FROM homepage;")
        for hrow in cur:
            assert(hrow['url'])
            assert(len(hrow['url'].split()) == 1)
            print('\t'.join((hrow['issnl'], hrow['url'])))

    def summarize(self, args):
        print("##### Summarizing Everything...")
        counts = Counter()
        self.c = self.db.cursor()
        self.db.row_factory = sqlite3.Row
        index_issnls = list(self.c.execute('SELECT DISTINCT issnl FROM journal_index'))
        fatcat_issnls = list(self.c.execute('SELECT DISTINCT issnl FROM fatcat_container'))
        all_issnls = set([i[0] for i in index_issnls + fatcat_issnls])
        print("{} total ISSN-Ls".format(len(all_issnls)))
        for issnl in list(all_issnls):
            #print(issnl)
            counts['total'] += 1

            out = dict()

            fatcat_row = list(self.db.execute("SELECT * FROM fatcat_container WHERE issnl = ?;", [issnl]))
            if fatcat_row:
                frow = fatcat_row[0]
                out['fatcat_ident'] = frow['ident']
                for k in ('name', 'publisher', 'issne', 'issnp', 'lang', 'country'):
                    if not out.get(k) and frow[k]:
                        out[k] = frow[k]
                any_preservation = bool(frow['preserved_count'])
                any_ia = bool(frow['ia_count'])

            cur = self.db.execute("SELECT * FROM journal_index WHERE issnl = ?;", [issnl])
            for irow in cur:
                if irow['slug'] in ('crossref',):
                    out['has_dois'] = True
                if irow['slug'] in ('doaj','road','szczepanski'):
                    out['is_oa'] = True
                # TODO: or if sz color is green
                # TODO: define longtail, based on publisher_type?
                for k in ('name',):
                    if not out.get(k) and irow[k]:
                        out[k] = irow[k]
                if irow['extra']:
                    extra = json.loads(irow['extra'])
                    for k in ('country', 'lang', 'issne', 'issnp', 'publisher'):
                        if not out.get(k) and extra.get(k):
                            out[k] = extra[k]

            cur = self.db.execute("SELECT * FROM homepage WHERE issnl = ?;", [issnl])
            for hrow in cur:
                if hrow['terminal_status_code'] == 200:
                    out['any_live_homepage'] = True

            self.c.execute("INSERT OR REPLACE INTO journal_summary (issnl, issne, issnp, fatcat_ident, name, publisher, country, lang, is_oa, is_longtail, has_dois, any_live_homepage, any_preservation, any_ia) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
                (issnl, out.get('issne'), out.get('issnp'),
                 out.get('fatcat_ident'), out.get('name'),
                 out.get('publisher'), out.get('country'), out.get('lang'),
                 out.get('is_oa'), out.get('is_longtail'), out.get('has_dois', False),
                 out.get('any_live_homepage', False), out.get('any_preservation', False),
                 out.get('any_ia')))
        self.c.close()
        self.db.commit()
        print(counts)

    def everything(self, args):
        self.init_db(args)
        self.index_doaj(args)
        self.index_norwegian(args)
        self.index_crossref(args)
        self.index_sherpa_romeo(args)
        self.index_road(args)
        self.index_entrez(args)
        self.index_ezb(args)
        self.load_fatcat(args)
        self.load_fatcat_stats(args)
        self.update_url_status(args)
        #self.load_kbart('lockss', LOCKSS_FILE)
        #self.load_kbart('clockss', CLOCKSS_FILE)
        #self.load_kbart('portico', PORTICO_FILE)
        #self.load_kbart('jstor', JSTOR_FILE)
        #self.index_sim(args)
        #self.load_homepage_crawl(IA_CRAWL_FILE)
        self.summarize(args)
        print("### Done with everything!")

    def init_db(self, args):
        print("### Creating Database...")
        self.db.executescript("""
            PRAGMA main.page_size = 4096;
            PRAGMA main.cache_size = 20000;
            PRAGMA main.locking_mode = EXCLUSIVE;
            PRAGMA main.synchronous = OFF;
        """)
        with open('chocula_schema.sql', 'r') as fschema:
            self.db.executescript(fschema.read())
        print("Done!")

def main():
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers()

    parser.add_argument("--db-file",
        help="run in mode that considers only terminal HTML success",
        default='chocula.sqlite',
        type=str)
    parser.add_argument("--input-file",
        help="override default input file path",
        default=None,
        type=str)

    sub = subparsers.add_parser('everything')
    sub.set_defaults(func='everything')

    sub = subparsers.add_parser('init_db')
    sub.set_defaults(func='init_db')

    sub = subparsers.add_parser('summarize')
    sub.set_defaults(func='summarize')

    # TODO: 'jurn'
    for ind in ('doaj', 'road', 'crossref', 'entrez', 'norwegian', 'szczepanski', 'ezb'):
        sub = subparsers.add_parser('index_{}'.format(ind))
        sub.set_defaults(func='index_{}'.format(ind))

    sub = subparsers.add_parser('load_fatcat')
    sub.set_defaults(func='load_fatcat')

    sub = subparsers.add_parser('load_fatcat_stats')
    sub.set_defaults(func='load_fatcat_stats')

    sub = subparsers.add_parser('export_urls')
    sub.set_defaults(func='export_urls')

    sub = subparsers.add_parser('update_url_status')
    sub.set_defaults(func='update_url_status')

    args = parser.parse_args()
    if not args.__dict__.get("func"):
        print("tell me what to do! (try --help)")
        sys.exit(-1)

    cdb = ChoculaDatabase(args.db_file)
    if args.func.startswith('index_') or args.func in ('everything',):
        cdb.read_issn_map_file(ISSNL_FILE)
    func = getattr(cdb, args.func)
    func(args)

if __name__ == '__main__':
    main()

