#!/usr/bin/env python3

"""
This file must be moved to the fatcat:python/ directory (aka, not in
fatcat:extra/fixups) to run. It's a "one-off", so probably will bitrot pretty
quickly. There are no tests.

Example invocation:

    zcat 2018-09-23-0405.30-dumpgrobidmetainsertable.longtail_join.filtered.tsv.gz | ./fixup_longtail_issnl_unique.py /srv/fatcat/datasets/single_domain_issnl.tsv -

See also:
- bnewbold/scratch:mellon/201904_longtail_issn.md
- aitio:/rapida/OA-JOURNAL-TESTCRAWL-TWO-2018
- https://archive.org/details/OA-JOURNAL-TESTCRAWL-TWO-2018-extra
= https://archive.org/download/ia_longtail_dumpgrobidmetainsertable_2018-09-23/2018-09-23-0405.30-dumpgrobidmetainsertable.longtail_join.filtered.tsv.gz


"""

import os, sys, argparse
import json
import sqlite3
import itertools

import fatcat_client
from fatcat_tools import authenticated_api
from fatcat_tools.importers.common import EntityImporter, clean, LinePusher
from fatcat_tools.importers.arabesque import b32_hex


class LongtailIssnlSingleDomainFixup(EntityImporter):
    """
    Fixup script for bootstrap longtail OA release entities which don't have a
    container but are confidently associated with an ISSN-L based on file
    domain.

    Expected to be a one-time fixup impacting about 600k entities (around half
    the longtail OA batch).

    Reads in a mapping of unique domain-ISSNL mappings, and then iterates over
    the original matched import batch file. For each line in the later:
    
    - checks if in-scope based on domain-ISSNL map
    - uses API to lookup file (by SHA-1) and confirm domain in URL list
    - look up releases for file and retain the longtail-oa ones (an extra flag)
    - if release is longtail-oa and no container, set the container based on
      ISSN-L (using cached lookup)
    - use EntityImporter stuff to manage update/editgroup queue
    """

    def __init__(self, api, domain_issnl_tsv_file, **kwargs):

        eg_desc = kwargs.pop('editgroup_description',
            "Fixup for longtail OA releases that can be matched to specific container by file domain / ISSN-L mapping")
        eg_extra = kwargs.pop('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.LongtailIssnlSingleDomainFixup')
        super().__init__(api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)

        self._domain_issnl_map = self.load_domain_issnl(domain_issnl_tsv_file)
        self._issnl_container_map = dict()

    def load_domain_issnl(self, tsv_file):
        print("Loading domain ISSN-L file...")
        m = dict()
        for l in tsv_file:
            l = l.strip().split('\t')
            assert len(l) == 2
            domain = l[0].lower()
            issnl = l[1]
            assert len(issnl) == 9 and issnl[4] == '-'
            m[domain] = issnl
        print("Got {} matchings.".format(len(m)))
        return m

    def want(self, raw_record):
        # do it all in parse_record()
        return True

    def parse_record(self, row):
        """
        TSV rows:
        - sha1 b32 key
        - JSON string: CDX-ish
            - surt
            - url
            - <etc>
        - mime
        - size (?)
        - JSON string: grobid metadata
        """

        # parse row
        row = row.split('\t')
        assert len(row) == 5
        sha1 = b32_hex(row[0][5:])
        cdx_dict = json.loads(row[1])
        url = cdx_dict['url']
        domain = url.split('/')[2].lower()

        # domain in scope?
        issnl = self._domain_issnl_map.get(domain)
        if not issnl:
            self.counts['skip-domain-scope'] += 1
            return None

        # lookup file
        print(sha1)
        try:
            file_entity = self.api.lookup_file(sha1=sha1)
        except fatcat_client.rest.ApiException as err:
            if err.status == 404:
                self.counts['skip-file-not-found'] += 1
                return None
            else:
                raise err

        # container ident
        # TODO: memoize these issnl/container lookups
        container_id = self.lookup_container(issnl)
        if not container_id:
            self.counts['skip-container-not-found'] += 1
            return None

        # confirm domain
        url_domain_match = False
        for furl in file_entity.urls:
            fdomain = furl.url.split('/')[2].lower()
            if domain == fdomain:
                url_domain_match = True
                break
        if not url_domain_match:
            self.counts['skip-no-domain-match'] += 1
            return None

        # fetch releases
        releases = self.api.get_file_releases(file_entity.ident)
        releases = [r for r in releases if (r.extra.get('longtail-oa') == True and r.container_id == None)]
        if not releases:
            self.counts['skip-no-releases'] += 1
            return None

        # set container_id
        for r in releases:
            r.container_id = container_id
        return releases

    def try_update(self, re_list):
        for re in re_list:
            self.api.update_release(re.ident, re, editgroup_id=self.get_editgroup_id())
            self.counts['update'] += 1
        return False

    def insert_batch(self, batch):
        raise NotImplementedError

def run_fixup(args):
    fmi = LongtailIssnlSingleDomainFixup(args.api,
        args.domain_issnl_tsv_file,
        edit_batch_size=args.batch_size)
    LinePusher(fmi, args.insertable_tsv_file).run()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--api-host-url',
        default="http://localhost:9411/v0",
        help="connect to this host/port")
    parser.add_argument('--batch-size',
        help="size of batch to send",
        default=50, type=int)
    parser.add_argument('domain_issnl_tsv_file',
        help="domain/ISSNL mapping TSV file",
        type=argparse.FileType('r'))
    parser.add_argument('insertable_tsv_file',
        help="dumpgrobidmetainsertable TSV file to work over",
        default=sys.stdin, type=argparse.FileType('r'))

    auth_var = "FATCAT_AUTH_SANDCRAWLER"

    args = parser.parse_args()

    args.api = authenticated_api(
        args.api_host_url,
        # token is an optional kwarg (can be empty string, None, etc)
        token=os.environ.get(auth_var))
    run_fixup(args)

if __name__ == '__main__':
    main()
