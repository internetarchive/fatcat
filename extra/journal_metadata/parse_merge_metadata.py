#!/usr/bin/env python3

import sys, csv, json
import ftfy
import pycountry

ISSNL_FILE = 'data/20181203.ISSN-to-ISSN-L.txt'

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


class Munger():
    """
    Top-level fields we'd like to fill in if possible:

        issnp: string
        issne: string
        first_year: year (integer)
        last_year: if publishing has stopped
        languages: array of ISO codes; first is the "primary" language
        nation: ISO shortcode of nation published from
        url: homepage
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

    TODO: more ftfy?
    TODO: remove surrounding quotes
    TODO: null ISSN-L?
    TODO: sherpa OA: 'Paid OA options' or 'All journals OA'
    TODO: mailto: in urls
    TODO: empty gaps (sim)
    """

    def __init__(self):
        self.data = dict()
        with open(ISSNL_FILE, 'r') as f:
            self.read_issn_map_file(f)

    def run(self, out_path):
        self.load_road(ROAD_FILE)
        self.load_doaj(DOAJ_FILE)
        self.load_crossref(CROSSREF_FILE)
        self.load_norwegian(NORWEGIAN_FILE)
        self.load_sherpa_romeo(SHERPA_ROMEO_JOURNAL_FILE, SHERPA_ROMEO_POLICY_FILE)
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
        lookup = raw_issn or issne or issnp
        lookup = lookup.strip()
        if not (len(lookup) == 9 and lookup[4] == '-'):
            print(lookup)
            print(len(lookup))
            print(lookup[4])
        assert len(lookup) == 9 and lookup[4] == '-'
        issnl = self.issn2issnl(lookup.upper())
        # lookup ISSN-Ls in data (or create one)
        if not issnl in self.data:
            self.data[issnl] = dict(issnl=issnl)
        d = self.data[issnl]
        # if name/publisher not set, do so
        if name and not 'name' in d:
            self.data[issnl]['name'] = ftfy.fix_text(name).strip()
        if publisher and not 'publisher' in d:
            self.data[issnl]['publisher'] = ftfy.fix_text(publisher).strip()
        if issne and not 'issne' in d:
            self.data[issnl]['issne'] = issne
        if issnp and not 'issnp' in d:
            self.data[issnl]['issnp'] = issnp
        # always return ISSN-L
        return issnl

    def load_entrez(self, path):
        print("##### Loading Entrez...")
        # JrId,JournalTitle,MedAbbr,"ISSN (Print)","ISSN (Online)",IsoAbbr,NlmId
        reader = csv.DictReader(open(path))
        skipped = 0
        count = 0
        for row in reader:
            if not (row.get('ISSN (Online)') or row.get('ISSN (Print)')):
                skipped += 1
                continue
            issnl = self.add_issn(
                issne=row.get('ISSN (Online)'),
                issnp=row.get('ISSN (Print)'),
                name=row['JournalTitle'],
            )
            count += 1
        print("Matched {}".format(count))
        print("Skipped {} for not having ISSNs".format(skipped))

    def load_road(self, path):
        print("##### Loading ROAD...")
        reader = csv.DictReader(open(path), delimiter='\t',
            fieldnames=("ISSN", "ISSN-L", "Short Title", "Title", "Publisher", "URL1", "URL2", "Region", "Lang1", "Lang2")
        )
        count = 0
        for row in reader:
            issnl = self.add_issn(
                raw_issn=row['ISSN-L'],
                name=row['Short Title'],
                publisher=row['Publisher'],
            )
            count += 1
            d = self.data[issnl]
            if row['URL1'] and not 'url' in d:
                self.data[issnl]['url'] = row['URL1']
            # TODO: region mapping: "Europe and North America"
            # TODO: lang mapping: already alpha-3
            self.data[issnl]['road'] = dict(as_of=ROAD_DATE)
        print("Matched {}".format(count))

    def load_doaj(self, path):
        print("##### Loading DOAJ...")
        #Journal title  Journal URL Alternative title   ISSN-print  ISSN-electronic Publisher   Society or institution  Platform, host or aggregator    Country of publisher    Journal article processing charges (APCs)   ... Deposit policy directory    Author holds copyright without restrictions Copyright information URL   Author holds publishing rights without restrictions Publishing rights information URL   DOAJ Seal   Tick: Accepted after March 2014 Added on Date   Subjects    ISSN-L
        reader = csv.DictReader(open(path))
        count = 0
        for row in reader:
            issnl = self.add_issn(
                issnp=row['Journal ISSN (print version)'],
                issne=row['Journal EISSN (online version)'],
                name=row['Journal title'],
                publisher=row['Publisher'],
            )
            count += 1
            d = self.data[issnl]
            doaj = dict(as_of=DOAJ_DATE)
            # TODO: work_level: bool (are work-level publications deposited with DOAJ?)
            # TODO: archiving: array, can include 'library' or 'other'

            if row['Platform, host or aggregator']:
                # TODO: mapping here?
                self.data[issnl]['platform'] = row['Platform, host or aggregator']
            if row['DOAJ Seal']:
                doaj['seal'] = {"no": False, "yes": True}[row['DOAJ Seal'].lower()]
            if row['Country of publisher']:
                # TODO: country mapping
                self.data[issnl]['country'] = row['Country of publisher']
            # TODO: Subjects
            self.data[issnl]['doaj'] = doaj
        print("Matched {}".format(count))

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
        count = 0
        for row in reader:
            #row['Journal Title'] = row.pop('\ufeffJournal Title')
            row.update(policies[row['RoMEO Record ID']])
            issnl = self.add_issn(
                issnp=row['ISSN'],
                issne=row['ESSN'],
                name=row['Journal Title'],
                publisher=row['Publisher'],
            )
            count += 1
            d = self.data[issnl]
            sherpa_romeo = dict()
            if row['RoMEO colour']:
                sherpa_romeo['color'] = row['RoMEO colour']
            if row['Open Access Publishing']:
                # TODO: boolean?
                sherpa_romeo['oa'] = row['Open Access Publishing']
            if row['Country'] and not 'country' in d:
                self.data[issnl]['country'] = row['Country'].lower()
            self.data[issnl]['sherpa_romeo'] = sherpa_romeo
        print("Matched {}".format(count))

    def load_norwegian(self, path):
        print("##### Loading Norwegian Registry...")
        #pandas.read_csv(NORWEGIAN_FILE, sep=';', encoding="ISO-8859-1")
        #NSD tidsskrift_id;Original title;International title;Present Level (2018);Print ISSN;Online ISSN;Open Access;NPI Scientific Field;NPI Academic Discipline;URL;Publishing Company;Publisher;Country of publication;Language;Level 2019;Level 2018;Level 2017;Level 2016;Level 2015;Level 2014;Level 2013;Level 2012;Level 2011;Level 2010;Level 2009;Level 2008;Level 2007;Level 2006;Level 2005;Level 2004;itar_id
        reader = csv.DictReader(open(path, encoding="ISO-8859-1"), delimiter=";")
        count = 0
        skip = 0
        for row in reader:
            issnp = row['Print ISSN']
            issne = row['Online ISSN']
            if issne and len(issne.strip()) != 9:
                issne = None
            if issnp and len(issnp.strip()) != 9:
                issnp = None
            if not (issnp or issne):
                skip += 1
                continue
            issnl = self.add_issn(
                issnp=issnp,
                issne=issne,
                name=row['International title'],
                publisher=row['Publisher'],
            )
            count += 1
            d = self.data[issnl]
            norwegian = dict(as_of=NORWEGIAN_DATE)
            norwegian['level'] = int(row['Present Level (2018)'])
            norwegian['id'] = int(row['NSD tidsskrift_id'])

            if row['Original title'] != row['International title'] and not 'original_name' in d:
                self.data[issnl]['original_name'] = row['Original title']
            if row['Country of publication'] and not 'country' in d:
                # TODO: country mapping
                self.data[issnl]['country'] = row['Country of publication']
            if row['Language'] and not 'language' in d:
                # TODO: language mapping
                self.data[issnl]['language'] = row['Language']
            self.data[issnl]['norwegian'] = norwegian
        print("Skipped {} for mangled ISSN".format(skip))
        print("Matched {}".format(count))

    def load_kbart(self, name, path):
        print("##### Loading KBART file for {}...".format(name))
        #publication_title      print_identifier        online_identifier       date_first_issue_online num_first_vol_online    num_first_issue_online  date_last_issue_online  num_last_vol_online     num_last_issue_online   title_url       first_author    title_id        embargo_info    coverage_depth  coverage_notes  publisher_name
        raw_file = open(path, 'rb').read().decode(errors='replace')
        fixed_file = ftfy.fix_text(raw_file)
        reader = csv.DictReader(fixed_file.split('\n'), delimiter='\t')
        count = 0
        skip = 0
        for row in reader:
            if not row['print_identifier'] and not row['online_identifier']:
                skip += 1
                continue
            issnl = self.add_issn(
                issnp=row['print_identifier'],
                issne=row['online_identifier'],
                name=row['publication_title'],
                publisher=row['publisher_name'],
            )
            count += 1
            d = self.data[issnl]
            if not 'kbart' in d:
                self.data[issnl]['kbart'] = dict()
            kbart = dict()
            if row['date_first_issue_online'] and row['date_last_issue_online']:
                kbart['year_span'] = [[int(row['date_first_issue_online'][:4]), int(row['date_last_issue_online'][:4])]]
            self.data[issnl]['kbart'][name] = kbart
        print("Skipped {} missing ISSN".format(skip))
        print("Matched {}".format(count))

    def load_crossref(self, path):
        print("##### Loading Crossref...")
        #"JournalTitle","JournalID","Publisher","pissn","eissn","additionalIssns","doi","(year1)[volume1]issue1,issue2,issue3(year2)[volume2]issue4,issues5"
        reader = csv.DictReader(open(path))
        count = 0
        skip = 0
        for row in reader:
            if row['pissn'] and len(row['pissn']) == 8:
                row['pissn'] = row['pissn'][:4] + '-' + row['pissn'][4:]
            if row['eissn'] and len(row['eissn']) == 8:
                row['eissn'] = row['eissn'][:4] + '-' + row['eissn'][4:]
            if not (row['pissn'] or row['eissn']):
                skip += 1
                continue
            issnl = self.add_issn(
                issnp=row['pissn'],
                issne=row['eissn'],
                name=row['JournalTitle'],
                publisher=row['Publisher'],
            )
            count += 1
            d = self.data[issnl]
            crossref = dict()
            if row['doi']:
                crossref['doi'] = row['doi']
            self.data[issnl]['crossref'] = crossref
        print("Skipped {} missing ISSN".format(skip))
        print("Matched {}".format(count))

    def load_sim(self, path):
        print("##### Loading SIM Metadata...")
        #NA Pub Cat ID,Title,Publisher,ISSN,Impact Rank,Total Cities,Journal Impact Factor,Eigenfact or Score,First Volume,Last Volume,NA Gaps,"Scholarly / Peer-\n Reviewed","Peer-\n Reviewed",Pub Type,Pub Language,Subjects
        reader = csv.DictReader(open(path))
        count = 0
        skip = 0
        for row in reader:
            if not row['ISSN'] or row['ISSN'] == "NULL":
                skip += 1
                continue
            issnl = self.add_issn(
                raw_issn=row['ISSN'][:9],
                name=row['Title'],
                publisher=row['Publisher'],
            )
            count += 1
            d = self.data[issnl]
            sim = dict()
            sim['id'] = row['NA Pub Cat ID']
            sim['first_year'] = row['First Volume']
            sim['last_year'] = row['Last Volume']
            sim['gaps'] = row['NA Gaps']
            # TODO: 'Pub Language'
            # TODO: 'Pub Type'
            self.data[issnl]['sim'] = sim
        print("Skipped {} missing ISSN".format(skip))
        print("Matched {}".format(count))

    def load_homepage_crawl(self, path):
        print("##### Loading IA Homepage Crawl Results...")
        reader = csv.DictReader(open(path), delimiter='\t',
            fieldnames=("ISSN", "first_url", "first_status", "last_status", "last_url")
        )
        count = 0
        skip = 0
        for row in reader:
            issnl = self.add_issn(
                raw_issn=row['ISSN'],
            )
            count += 1
            d = self.data[issnl]
            ia = d.get('ia', dict())
            ia['homepage_status'] = int(row['last_status'])
            if ia['homepage_status'] == 200:
                ia['homepage_url'] = row['last_url']
            else:
                ia['homepage_url'] = row['first_url']
            self.data[issnl]['ia'] = ia
        print("Skipped {} missing ISSN".format(skip))
        print("Matched {}".format(count))

if __name__=='__main__':
    munger = Munger()
    munger.run(sys.argv[1])

