#!/usr/bin/env python3

import sys, csv, json
import ftfy
import pycountry
from collections import Counter

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
    return country.alpha2.lower()

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

class Munger():
    """
    Top-level fields we'd like to fill in if possible:

        issnp: string
        issne: string
        first_year: year (integer)
        last_year: if publishing has stopped
        languages: array of ISO codes; first is the "primary" language
        country: ISO shortcode of country published from
        urls: homepage links
        abbrev: string
        default_license: slug
        original_name: native name (if name is translated)
        platform: hosting platform: OJS, wordpress, scielo, etc
        mimetypes: array of strings (eg, 'application/pdf', 'text/html')
        aliases: array of "also known as"

    Lower priority (TODO/later):
        coden: string
        oclc_id: string (lookup?)
        lccn_id: string (lookup?)
        dblb_id: string
        region: TODO: continent/world-region
        discipline: TODO: highest-level subject; "life science", "humanities", etc
        field: TODO: narrower description of field
        subjects: TODO?

    TODO: so many missing ISSN/ISSN-L
    TODO: abbrev
    """

    def __init__(self):
        self.data = dict()
        with open(ISSNL_FILE, 'r') as f:
            self.read_issn_map_file(f)

    def run(self, out_path):
        self.load_doaj(DOAJ_FILE)
        self.load_norwegian(NORWEGIAN_FILE)
        self.load_crossref(CROSSREF_FILE)
        self.load_sherpa_romeo(SHERPA_ROMEO_JOURNAL_FILE, SHERPA_ROMEO_POLICY_FILE)
        self.load_road(ROAD_FILE)
        self.load_kbart('lockss', LOCKSS_FILE)
        self.load_kbart('clockss', CLOCKSS_FILE)
        self.load_kbart('portico', PORTICO_FILE)
        self.load_kbart('jstor', JSTOR_FILE)
        self.load_entrez(ENTREZ_FILE)
        self.load_sim(SIM_FILE)
        self.load_homepage_crawl(IA_CRAWL_FILE)
        self.summarize()
        self.dump(out_path)
        print("Done!")

    def dump(self, out_path):
        print("#### Dumping to {}".format(out_path))
        with open(out_path, 'w') as out:
            for issnl in self.data:
                out.write(json.dumps(self.data[issnl]) + "\n")

    def summarize(self):
        print("##### Loaded {} unique entries".format(len(self.data)))

    def read_issn_map_file(self, issn_map_file):
        print("##### Loading ISSN map file...")
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

    def add_issn(self, raw_issn=None, issne=None, issnp=None, name=None, publisher=None):
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
            #print((raw_issn, issne, issnp))
            # UGH.
            issnl = issne or issnp or raw_issn
            if not issnl:
                return None, 'no-issnl'
            issnl = issnl.strip().upper()
            assert len(issnl) == 9 and issnl[4] == '-'
            status = 'found-munge'
        else:
            status = 'found'
        # lookup ISSN-Ls in data (or create one)
        if not issnl in self.data:
            status = 'created'
            self.data[issnl] = dict(issnl=issnl)
        d = self.data[issnl]
        # if name/publisher not set, do so
        if name and not 'name' in d:
            name = unquote(ftfy.fix_text(name))
            if name:
                self.data[issnl]['name'] = name
        if publisher and not 'publisher' in d:
            publisher = unquote(ftfy.fix_text(publisher))
            if publisher:
                self.data[issnl]['publisher'] = publisher
        if issne and not 'issne' in d:
            self.data[issnl]['issne'] = issne
        if issnp and not 'issnp' in d:
            self.data[issnl]['issnp'] = issnp
        # always return ISSN-L
        return issnl, status

    def add_lang(self, issnl, lang):
        if not (lang and issnl):
            return
        lang = parse_lang(lang)
        if not lang:
            return
        if 'languages' not in self.data[issnl]:
            self.data[issnl]['languages'] = [lang]
        elif lang not in self.data[issnl]['languages']:
            self.data[issnl]['languages'].append(lang)

    def add_url(self, issnl, url):
        if not (url and issnl) or 'mailto:' in url.lower() or url in ('http://n/a', 'http://N/A'):
            return
        if url.startswith('www.'):
            url = "http://" + url
        url.replace('Http://', 'http://')
        if 'urls' not in self.data[issnl]:
            self.data[issnl]['urls'] = [url]
        elif url not in self.data[issnl]['urls']:
            self.data[issnl]['urls'].append(url)

    def add_country(self, issnl, country):
        if not (country and issnl):
            return
        country = parse_country(country)
        if not country:
            return
        if 'country' not in self.data[issnl]:
            self.data[issnl]['country'] = country

    def add_mimetype(self, issnl, val):
        if not (val and issnl):
            return
        mimetype = None
        if '/' in val:
            mimetype = val
        else:
            mimetype = MIMETYPE_MAP.get(val)
        if not mimetype:
            return
        if 'mimetypes' not in self.data[issnl]:
            self.data[issnl]['mimestypes'] = [mimetype]
        elif mimetype not in self.data[issnl]['mimestypes']:
            self.data[issnl]['mimestypes'].append(mimetype)

    def load_entrez(self, path):
        print("##### Loading Entrez...")
        # JrId,JournalTitle,MedAbbr,"ISSN (Print)","ISSN (Online)",IsoAbbr,NlmId
        reader = csv.DictReader(open(path))
        counts = Counter()
        for row in reader:
            if not (row.get('ISSN (Online)') or row.get('ISSN (Print)')):
                counts['skipped'] += 1
                continue
            issnl, status = self.add_issn(
                issne=row.get('ISSN (Online)'),
                issnp=row.get('ISSN (Print)'),
                name=row['JournalTitle'],
            )
            if row['IsoAbbr'] and not 'abbrev' in self.data[issnl]:
                self.data[issnl]['abbrev'] = row['IsoAbbr'].strip()
            counts[status] += 1
        print(counts)

    def load_road(self, path):
        print("##### Loading ROAD...")
        reader = csv.DictReader(open(path), delimiter='\t',
            fieldnames=("ISSN", "ISSN-L", "Short Title", "Title", "Publisher", "URL1", "URL2", "Region", "Lang1", "Lang2")
        )
        counts = Counter()
        for row in reader:
            issnl, status = self.add_issn(
                raw_issn=row['ISSN-L'],
                name=row['Short Title'],
                publisher=row['Publisher'],
            )
            counts[status] += 1
            if not issnl:
                continue
            d = self.data[issnl]
            if row['URL1']:
                self.add_url(issnl, row['URL1'])
            if row['URL2']:
                self.add_url(issnl, row['URL2'])
            if row['Lang1']:
                self.add_lang(issnl, row['Lang1'])
            if row['Lang2']:
                self.add_lang(issnl, row['Lang2'])
            # TODO: region mapping: "Europe and North America"
            # TODO: lang mapping: already alpha-3
            self.data[issnl]['road'] = dict(as_of=ROAD_DATE)
        print(counts)

    def load_doaj(self, path):
        print("##### Loading DOAJ...")
        #Journal title,Journal URL,Alternative title,Journal ISSN (print version),Journal EISSN (online version),Publisher,Society or institution,"Platform, host or aggregator",Country of publisher,Journal article processing charges (APCs),APC information URL,APC amount,Currency,Journal article submission fee,Submission fee URL,Submission fee amount,Submission fee currency,Number of articles publish in the last calendar year,Number of articles information URL,Journal waiver policy (for developing country authors etc),Waiver policy information URL,Digital archiving policy or program(s),Archiving: national library,Archiving: other,Archiving infomation URL,Journal full-text crawl permission,Permanent article identifiers,Journal provides download statistics,Download statistics information URL,First calendar year journal provided online Open Access content,Full text formats,Keywords,Full text language,URL for the Editorial Board page,Review process,Review process information URL,URL for journal's aims & scope,URL for journal's instructions for authors,Journal plagiarism screening policy,Plagiarism information URL,Average number of weeks between submission and publication,URL for journal's Open Access statement,Machine-readable CC licensing information embedded or displayed in articles,URL to an example page with embedded licensing information,Journal license,License attributes,URL for license terms,Does this journal allow unrestricted reuse in compliance with BOAI?,Deposit policy directory,Author holds copyright without restrictions,Copyright information URL,Author holds publishing rights without restrictions,Publishing rights information URL,DOAJ Seal,Tick: Accepted after March 2014,Added on Date,Subjects
        reader = csv.DictReader(open(path))
        counts = Counter()
        for row in reader:
            issnl, status = self.add_issn(
                issnp=row['Journal ISSN (print version)'],
                issne=row['Journal EISSN (online version)'],
                name=row['Journal title'],
                publisher=row['Publisher'],
            )
            counts[status] += 1
            if not issnl:
                continue
            d = self.data[issnl]
            doaj = dict(as_of=DOAJ_DATE)
            # TODO: work_level: bool (are work-level publications deposited with DOAJ?)

            if row['Digital archiving policy or program(s)']:
                doaj['archive'] = [a.strip() for a in row['Digital archiving policy or program(s)'].split(',') if a.strip()]
            elif row['Archiving: national library']:
                doaj['archive'] = ['national-library']

            crawl_permission = row['Journal full-text crawl permission']
            if crawl_permission:
                doaj['crawl-permission'] = dict(Yes=True, No=False)[crawl_permission]
            # TODO: Permanent article identifiers
            default_license = row['Journal license']
            if default_license and default_license.startswith('CC'):
                self.data[issnl]['default_license'] = default_license.replace('CC ', 'CC-').strip()

            self.add_mimetype(issnl, row['Full text formats'])
            platform = PLATFORM_MAP.get(row['Platform, host or aggregator'])
            if platform:
                self.data[issnl]['platform'] = platform
            if row['DOAJ Seal']:
                doaj['seal'] = {"no": False, "yes": True}[row['DOAJ Seal'].lower()]
            if row['Country of publisher']:
                self.add_country(issnl, row['Country of publisher'])
            if row['Full text language']:
                self.add_lang(issnl, row['Full text language'])
            if row['Journal URL']:
                self.add_url(issnl, row['Journal URL'])
            # TODO: Subjects
            self.data[issnl]['doaj'] = doaj
        print(counts)

    def load_sherpa_romeo(self, journal_path, policy_path):
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
        for row in reader:
            #row['Journal Title'] = row.pop('\ufeffJournal Title')
            row.update(policies[row['RoMEO Record ID']])
            issnl, status = self.add_issn(
                issnp=row['ISSN'],
                issne=row['ESSN'],
                name=row['Journal Title'],
                publisher=row['Publisher'],
            )
            counts[status] += 1
            if not issnl:
                continue
            d = self.data[issnl]
            sherpa_romeo = dict()
            if row['RoMEO colour']:
                sherpa_romeo['color'] = row['RoMEO colour']
            # row['Open Access Publishing']
            if row['Country']:
                self.add_country(issnl, row['Country'])
            self.data[issnl]['sherpa_romeo'] = sherpa_romeo
        print(counts)

    def load_norwegian(self, path):
        print("##### Loading Norwegian Registry...")
        #pandas.read_csv(NORWEGIAN_FILE, sep=';', encoding="ISO-8859-1")
        #NSD tidsskrift_id;Original title;International title;Present Level (2018);Print ISSN;Online ISSN;Open Access;NPI Scientific Field;NPI Academic Discipline;URL;Publishing Company;Publisher;Country of publication;Language;Level 2019;Level 2018;Level 2017;Level 2016;Level 2015;Level 2014;Level 2013;Level 2012;Level 2011;Level 2010;Level 2009;Level 2008;Level 2007;Level 2006;Level 2005;Level 2004;itar_id
        reader = csv.DictReader(open(path, encoding="ISO-8859-1"), delimiter=";")
        counts = Counter()
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
            issnl, status = self.add_issn(
                issnp=issnp,
                issne=issne,
                name=row['International title'],
                publisher=row['Publisher'],
            )
            counts[status] += 1
            if not issnl:
                continue
            d = self.data[issnl]
            norwegian = dict(as_of=NORWEGIAN_DATE)
            norwegian['level'] = int(row['Present Level (2018)'])
            norwegian['id'] = int(row['NSD tidsskrift_id'])

            if row['Original title'] != row['International title'] and not 'original_name' in d:
                self.data[issnl]['original_name'] = row['Original title']
            if row['Country of publication']:
                self.add_country(issnl, row['Country of publication'])
            if row['Language']:
                self.add_lang(issnl, row['Language'])
            if row['URL']:
                self.add_url(issnl, row['URL'])
            self.data[issnl]['norwegian'] = norwegian
        print(counts)

    def load_kbart(self, name, path):
        print("##### Loading KBART file for {}...".format(name))
        #publication_title      print_identifier        online_identifier       date_first_issue_online num_first_vol_online    num_first_issue_online  date_last_issue_online  num_last_vol_online     num_last_issue_online   title_url       first_author    title_id        embargo_info    coverage_depth  coverage_notes  publisher_name
        raw_file = open(path, 'rb').read().decode(errors='replace')
        fixed_file = ftfy.fix_text(raw_file)
        reader = csv.DictReader(fixed_file.split('\n'), delimiter='\t')
        counts = Counter()
        for row in reader:
            if not row['print_identifier'] and not row['online_identifier']:
                counts['no-issn'] += 1
                continue
            issnl, status = self.add_issn(
                issnp=row['print_identifier'],
                issne=row['online_identifier'],
                name=row['publication_title'],
                publisher=row['publisher_name'],
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
        print(counts)

    def load_crossref(self, path):
        print("##### Loading Crossref...")
        #"JournalTitle","JournalID","Publisher","pissn","eissn","additionalIssns","doi","(year1)[volume1]issue1,issue2,issue3(year2)[volume2]issue4,issues5"
        reader = csv.DictReader(open(path))
        counts = Counter()
        for row in reader:
            if row['pissn'] and len(row['pissn']) == 8:
                row['pissn'] = row['pissn'][:4] + '-' + row['pissn'][4:]
            if row['eissn'] and len(row['eissn']) == 8:
                row['eissn'] = row['eissn'][:4] + '-' + row['eissn'][4:]
            if not (row['pissn'] or row['eissn']):
                counts['no-issn'] += 1
                continue
            issnl, status = self.add_issn(
                issnp=row['pissn'],
                issne=row['eissn'],
                name=row['JournalTitle'],
                publisher=row['Publisher'],
            )
            counts[status] += 1
            if not issnl:
                continue
            d = self.data[issnl]
            crossref = dict()
            if row['doi']:
                crossref['doi'] = row['doi']
            crossref['any'] = True
            self.data[issnl]['crossref'] = crossref
        print(counts)

    def load_sim(self, path):
        print("##### Loading SIM Metadata...")
        #NA Pub Cat ID,Title,Publisher,ISSN,Impact Rank,Total Cities,Journal Impact Factor,Eigenfact or Score,First Volume,Last Volume,NA Gaps,"Scholarly / Peer-\n Reviewed","Peer-\n Reviewed",Pub Type,Pub Language,Subjects
        reader = csv.DictReader(open(path))
        counts = Counter()
        for row in reader:
            if not row['ISSN'] or row['ISSN'] == "NULL":
                counts['no-issn'] += 1
                continue
            issnl, status = self.add_issn(
                raw_issn=row['ISSN'][:9],
                name=row['Title'],
                publisher=row['Publisher'],
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
        print(counts)

    def load_homepage_crawl(self, path):
        print("##### Loading IA Homepage Crawl Results...")
        reader = csv.DictReader(open(path), delimiter='\t',
            fieldnames=("ISSN", "first_url", "first_status", "last_status", "last_url")
        )
        counts = Counter()
        for row in reader:
            issnl, status = self.add_issn(
                raw_issn=row['ISSN'],
            )
            counts[status] += 1
            if not issnl:
                continue
            d = self.data[issnl]
            ia = d.get('ia', dict())
            ia['homepage_status'] = int(row['last_status'])
            if ia['homepage_status'] == 200:
                ia['homepage_url'] = row['last_url']
            else:
                ia['homepage_url'] = row['first_url']
            self.data[issnl]['ia'] = ia
        print(counts)

if __name__=='__main__':
    if len(sys.argv) != 2 or sys.argv[1].startswith('-'):
        print("pass me path for an output JSON lines file")
        sys.exit(-1)
    munger = Munger()
    munger.run(sys.argv[1])

