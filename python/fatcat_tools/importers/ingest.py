
import datetime

import fatcat_openapi_client
from .common import EntityImporter, make_rel_url


class IngestFileResultImporter(EntityImporter):

    def __init__(self, api, require_grobid=True, **kwargs):

        eg_desc = kwargs.pop('editgroup_description', None) or "Files crawled from web using sandcrawler ingest tool"
        eg_extra = kwargs.pop('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.IngestFileResultImporter')
        kwargs['do_updates'] = kwargs.get("do_updates", False)
        super().__init__(api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)
        self.use_glutton_match = False
        self.default_link_rel = kwargs.get("default_link_rel", "web")
        assert self.default_link_rel
        self.require_grobid = require_grobid
        if self.require_grobid:
            print("Requiring GROBID status == 200 (for PDFs)")
        else:
            print("NOT checking GROBID success")
        self.ingest_request_source_allowlist = [
            'fatcat-changelog',
            'fatcat-ingest-container',
            'fatcat-ingest',
            'arabesque',
            #'mag-corpus',
            #'mag',
            'unpaywall-corpus',
            'unpaywall',
            #'s2-corpus',
            #'s2',
            'doaj',
            'dblp',
        ]
        if kwargs.get('skip_source_allowlist', False):
            self.ingest_request_source_allowlist = []

    def want_file(self, row) -> bool:
        """
        File-specific part of want(). Generic across general ingest and save-paper-now.
        """

        if not row.get('file_meta'):
            self.counts['skip-file-meta'] += 1
            return False

        # type-specific filters
        if row['request'].get('ingest_type') == 'pdf':
            if self.require_grobid and row.get('grobid', {}).get('status_code') != 200:
                self.counts['skip-grobid'] += 1
                return False
            if row['file_meta'].get('mimetype') not in ("application/pdf",):
                self.counts['skip-mimetype'] += 1
                return False
        elif row['request'].get('ingest_type') == 'xml':
            if row['file_meta'].get('mimetype') not in ("application/xml",
                    "application/jats+xml", "application/tei+xml", "text/xml"):
                self.counts['skip-mimetype'] += 1
                return False
        else:
            self.counts['skip-ingest-type'] += 1
            return False

        return True

    def want_ingest(self, row) -> bool:
        """
        Sandcrawler ingest-specific part of want(). Generic across file and
        webcapture ingest.
        """
        if row.get('hit') != True:
            self.counts['skip-hit'] += 1
            return False
        source = row['request'].get('ingest_request_source')
        if not source:
            self.counts['skip-ingest_request_source'] += 1
            return False
        if self.ingest_request_source_allowlist and source not in self.ingest_request_source_allowlist:
            self.counts['skip-ingest_request_source'] += 1
            return False

        if row['request'].get('link_source') not in ('arxiv', 'pmc', 'unpaywall', 'doi', 'mag', 's2', 'doaj', 'dblp'):
            self.counts['skip-link-source'] += 1
            return False

        if source.startswith('savepapernow'):
            # never process async savepapernow requests
            self.counts['skip-savepapernow'] += 1
            return False

        return True

    def want(self, row):
        """
        Overall logic here probably needs work (TODO):

        - Direct ingests via DOI from fatcat-changelog should probably go
          through regardless of GROBID status
        - We should filter/block things like single-page PDFs here
        - public/anonymous submissions could require successful biblio-glutton
          match, or some other sanity check on the fatcat side (eg, fuzzy title
          match)
        - handle the case of release_stage not being 'published'; if pre-print,
          potentially create a new release.

        The current logic is intentionally conservative as a first step.
        """
        if not self.want_ingest(row):
            return False
        if not self.want_file(row):
            return False

        return True

    def parse_ingest_release_ident(self, row):

        request = row['request']
        fatcat = request.get('fatcat')

        release_ident = None
        if fatcat and fatcat.get('release_ident'):
            release_ident = fatcat.get('release_ident')
        elif request.get('ext_ids'):
            # if no fatcat ident, try extids
            for extid_type in ('doi', 'pmid', 'pmcid', 'arxiv', 'doaj', 'dblp'):
                extid = request['ext_ids'].get(extid_type)
                if not extid:
                    continue
                if extid_type == 'doi':
                    extid = extid.lower()
                try:
                    release = self.api.lookup_release(**{extid_type: extid})
                except fatcat_openapi_client.rest.ApiException as err:
                    if err.status == 404:
                        continue
                    elif err.status == 400:
                        self.counts['warn-extid-invalid'] += 1
                        continue
                    raise err
                # verify release_stage
                if request.get('release_stage') and release.release_stage:
                    if request['release_stage'] != release.release_stage:
                        self.counts['skip-release-stage'] += 1
                        return None
                release_ident = release.ident
                break

        if self.use_glutton_match and not release_ident and row.get('grobid'):
            # try biblio-glutton extracted hit
            if row['grobid'].get('fatcat_release'):
                release_ident = row['grobid']['fatcat_release'].split('_')[-1]
                self.counts['glutton-match'] += 1

        return release_ident

    def parse_terminal(self, row):
        terminal = row.get('terminal')
        if not terminal:
            # support old cdx-only ingest results
            cdx = row.get('cdx')
            if not cdx:
                return None
            else:
                terminal = {
                    'terminal_url': cdx['url'],
                    'terminal_dt': cdx['datetime'],
                    'terminal_status_code': cdx.get('status_code') or cdx.get('http_status'),
                }

        # work around old schema
        if not 'terminal_url' in terminal:
            terminal['terminal_url'] = terminal['url']
        if not 'terminal_dt' in terminal:
            terminal['terminal_dt'] = terminal['dt']

        # convert CDX-style digits to ISO-style timestamp
        assert len(terminal['terminal_dt']) == 14
        terminal['terminal_timestamp'] = datetime.datetime.strptime(terminal['terminal_dt'], "%Y%m%d%H%M%S").isoformat() + "Z"
        return terminal

    def parse_urls(self, row, terminal):

        request = row['request']

        default_rel = self.default_link_rel
        if request.get('link_source') == 'doi':
            default_rel = 'publisher'
        default_rel = request.get('rel', default_rel)
        url = make_rel_url(terminal['terminal_url'], default_rel)

        if not url:
            self.counts['skip-url'] += 1
            return None
        wayback = "https://web.archive.org/web/{}/{}".format(
            terminal['terminal_dt'],
            terminal['terminal_url'])
        urls = [url, ("webarchive", wayback)]

        urls = [fatcat_openapi_client.FileUrl(rel=rel, url=url) for (rel, url) in urls]
        return urls

    def parse_edit_extra(self, row):

        request = row['request']
        edit_extra = dict()

        if request.get('edit_extra'):
            edit_extra = request['edit_extra']

        if request.get('ingest_request_source'):
            edit_extra['ingest_request_source'] = request['ingest_request_source']
        if request.get('link_source') and request.get('link_source_id'):
            edit_extra['link_source'] = request['link_source']
            edit_extra['link_source_id'] = request['link_source_id']
            if edit_extra['link_source'] == 'doi':
                edit_extra['link_source_id'] = edit_extra['link_source_id'].lower()

        # GROBID metadata, for SPN requests (when there might not be 'success')
        if request.get('ingest_type') == 'pdf':
            if row.get('grobid') and row['grobid'].get('status') != 'success':
                edit_extra['grobid_status_code'] = row['grobid']['status_code']
                edit_extra['grobid_version'] = row['grobid'].get('grobid_version')

        return edit_extra

    def parse_record(self, row):

        request = row['request']
        file_meta = row['file_meta']

        # double check that want() filtered request correctly (eg, old requests)
        if request.get('ingest_type') not in ('pdf', 'xml'):
            self.counts['skip-ingest-type'] += 1
            return None
        assert (request['ingest_type'], file_meta['mimetype']) in [
            ("pdf", "application/pdf"),
            ("xml", "application/xml"),
            ("xml", "application/jats+xml"),
            ("xml", "application/tei+xml"),
            ("xml", "text/xml"),
        ]

        # identify release by fatcat ident, or extid lookup, or biblio-glutton match
        release_ident = self.parse_ingest_release_ident(row)

        if not release_ident:
            self.counts['skip-release-not-found'] += 1
            return None

        terminal = self.parse_terminal(row)
        if not terminal:
            # TODO: support archive.org hits?
            self.counts['skip-no-terminal'] += 1
            return None

        urls = self.parse_urls(row, terminal)

        fe = fatcat_openapi_client.FileEntity(
            md5=file_meta['md5hex'],
            sha1=file_meta['sha1hex'],
            sha256=file_meta['sha256hex'],
            size=file_meta['size_bytes'],
            mimetype=file_meta['mimetype'],
            release_ids=[release_ident],
            urls=urls,
        )

        edit_extra = self.parse_edit_extra(row)
        if edit_extra:
            fe.edit_extra = edit_extra
        return fe

    def try_update(self, fe):
        # lookup sha1, or create new entity
        existing = None
        try:
            existing = self.api.lookup_file(sha1=fe.sha1)
        except fatcat_openapi_client.rest.ApiException as err:
            if err.status != 404:
                raise err

        # check for existing edits-in-progress with same file hash
        for other in self._entity_queue:
            if other.sha1 == fe.sha1:
                self.counts['skip-in-queue'] += 1
                return False

        if not existing:
            return True

        # NOTE: the following checks all assume there is an existing item
        if (fe.release_ids[0] in existing.release_ids) and existing.urls:
            # TODO: could still, in theory update with the new URL?
            self.counts['exists'] += 1
            return False

        if not self.do_updates:
            self.counts['skip-update-disabled'] += 1
            return False

        # TODO: for now, never update
        self.counts['skip-update-disabled'] += 1
        return False

    def insert_batch(self, batch):
        if self.submit_mode:
            eg = self.api.create_editgroup(fatcat_openapi_client.Editgroup(
                description=self.editgroup_description,
                extra=self.editgroup_extra))
            for fe in batch:
                self.api.create_file(eg.editgroup_id, fe)
            self.api.update_editgroup(eg.editgroup_id, eg, submit=True)
        else:
            self.api.create_file_auto_batch(fatcat_openapi_client.FileAutoBatch(
                editgroup=fatcat_openapi_client.Editgroup(
                    description=self.editgroup_description,
                    extra=self.editgroup_extra),
                entity_list=batch))


class SavePaperNowFileImporter(IngestFileResultImporter):
    """
    This worker ingests from the same feed as IngestFileResultImporter, but
    only imports files from anonymous save-paper-now requests, and "submits"
    them for further human review (as opposed to accepting by default).
    """

    def __init__(self, api, submit_mode=True, **kwargs):

        eg_desc = kwargs.pop('editgroup_description', None) or "Files crawled after a public 'Save Paper Now' request"
        eg_extra = kwargs.pop('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.SavePaperNowFileImporter')
        kwargs['submit_mode'] = submit_mode
        kwargs['require_grobid'] = False
        kwargs['do_updates'] = False
        super().__init__(api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)

    def want(self, row):

        source = row['request'].get('ingest_request_source')
        if not source:
            self.counts['skip-ingest_request_source'] += 1
            return False
        if not source.startswith('savepapernow'):
            self.counts['skip-not-savepapernow'] += 1
            return False

        if row.get('hit') != True:
            self.counts['skip-hit'] += 1
            return False

        if not self.want_file(row):
            return False

        return True


class IngestWebResultImporter(IngestFileResultImporter):
    """
    Variant of IngestFileResultImporter for processing HTML ingest requests
    into webcapture objects.
    """

    def __init__(self, api, **kwargs):

        eg_desc = kwargs.pop('editgroup_description', None) or "Webcaptures crawled from web using sandcrawler ingest tool"
        eg_extra = kwargs.pop('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.IngestWebResultImporter')
        kwargs['do_updates'] = False
        super().__init__(api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)

    def want(self, row):

        if not self.want_ingest(row):
            return False

        # webcapture-specific filters
        if row['request'].get('ingest_type') != 'html':
            self.counts['skip-ingest-type'] += 1
            return False
        if not row.get('file_meta'):
            self.counts['skip-file-meta'] += 1
            return False
        if row['file_meta'].get('mimetype') not in ("text/html", "application/xhtml+xml"):
            self.counts['skip-mimetype'] += 1
            return False

        return True

    def parse_record(self, row):

        request = row['request']
        file_meta = row['file_meta']

        # double check that want() filtered request correctly (eg, old requests)
        if request.get('ingest_type') != "html":
            self.counts['skip-ingest-type'] += 1
            return None
        if file_meta['mimetype'] not in ("text/html", "application/xhtml+xml"):
            self.counts['skip-mimetype'] += 1
            return None

        # identify release by fatcat ident, or extid lookup
        release_ident = self.parse_ingest_release_ident(row)

        if not release_ident:
            self.counts['skip-release-not-found'] += 1
            return None

        terminal = self.parse_terminal(row)
        if not terminal:
            # TODO: support archive.org hits?
            self.counts['skip-no-terminal'] += 1
            return None

        urls = self.parse_urls(row, terminal)
        archive_urls = [u for u in urls if u.rel == 'webarchive']

        if terminal['terminal_status_code'] != 200:
            self.counts['skip-terminal-status-code'] += 1
            return None

        terminal_cdx = row['cdx']
        if 'revisit_cdx' in row:
            terminal_cdx = row['revisit_cdx']
        assert terminal_cdx['surt']
        if terminal_cdx['url'] != terminal['terminal_url']:
            self.counts['skip-terminal-url-mismatch'] += 1
            return None

        wc_cdx = []
        # primary resource first
        wc_cdx.append(fatcat_openapi_client.WebcaptureCdxLine(
            surt=terminal_cdx['surt'],
            timestamp=terminal['terminal_timestamp'],
            url=terminal['terminal_url'],
            mimetype=file_meta['mimetype'],
            status_code=terminal['terminal_status_code'],
            sha1=file_meta['sha1hex'],
            sha256=file_meta['sha256hex'],
            size=file_meta['size_bytes'],
        ))

        for resource in row.get('html_resources', []):
            timestamp = resource['timestamp']
            if not "+" in timestamp and not "Z" in timestamp:
                timestamp += "Z"
            wc_cdx.append(fatcat_openapi_client.WebcaptureCdxLine(
                surt=resource['surt'],
                timestamp=timestamp,
                url=resource['url'],
                mimetype=resource.get('mimetype'),
                size=resource.get('size'),
                sha1=resource.get('sha1hex'),
                sha256=resource.get('sha256hex'),
            ))

        wc = fatcat_openapi_client.WebcaptureEntity(
            cdx=wc_cdx,
            archive_urls=archive_urls,
            original_url=terminal['terminal_url'],
            timestamp=terminal['terminal_timestamp'],
            release_ids=[release_ident],
        )

        edit_extra = self.parse_edit_extra(row)

        if edit_extra:
            wc.edit_extra = edit_extra
        return wc

    def try_update(self, wc):

        # check for existing edits-in-progress with same URL
        for other in self._entity_queue:
            if other.original_url == wc.original_url:
                self.counts['skip-in-queue'] += 1
                return False

        # lookup sha1, or create new entity (TODO: API doesn't support this yet)
        #existing = None

        # TODO: currently only allow one release per webcapture
        release = self.api.get_release(wc.release_ids[0], expand="webcaptures")
        if release.webcaptures:
            # check if this is an existing match, or just a similar hit
            for other in release.webcaptures:
                if wc.original_url == other.original_url:
                    # TODO: compare very similar timestamps of same time (different formats)
                    self.counts['exists'] += 1
                    return False
            self.counts['skip-release-has-webcapture'] += 1
            return False

        # Ok, if we got here then no existing web capture for (first) release,
        # so go ahead and insert!
        return True

    def insert_batch(self, batch):
        if self.submit_mode:
            eg = self.api.create_editgroup(fatcat_openapi_client.Editgroup(
                description=self.editgroup_description,
                extra=self.editgroup_extra))
            for fe in batch:
                self.api.create_webcapture(eg.editgroup_id, fe)
            self.api.update_editgroup(eg.editgroup_id, eg, submit=True)
        else:
            self.api.create_webcapture_auto_batch(fatcat_openapi_client.WebcaptureAutoBatch(
                editgroup=fatcat_openapi_client.Editgroup(
                    description=self.editgroup_description,
                    extra=self.editgroup_extra),
                entity_list=batch))

class SavePaperNowWebImporter(IngestWebResultImporter):
    """
    Like SavePaperNowFileImporter, but for webcapture (HTML) ingest.
    """

    def __init__(self, api, submit_mode=True, **kwargs):

        eg_desc = kwargs.pop('editgroup_description', None) or "Webcaptures crawled after a public 'Save Paper Now' request"
        eg_extra = kwargs.pop('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.SavePaperNowWebImporter')
        kwargs['submit_mode'] = submit_mode
        kwargs['do_updates'] = False
        super().__init__(api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)

    def want(self, row):
        """
        Relatively custom want() here, a synthesis of other filters.

        We do currently allow unknown-scope through for this specific code
        path, which means allowing hit=false.
        """

        source = row['request'].get('ingest_request_source')
        if not source:
            self.counts['skip-ingest_request_source'] += 1
            return False
        if not source.startswith('savepapernow'):
            self.counts['skip-not-savepapernow'] += 1
            return False

        # webcapture-specific filters
        if row['request'].get('ingest_type') != 'html':
            self.counts['skip-ingest-type'] += 1
            return False
        if not row.get('file_meta'):
            self.counts['skip-file-meta'] += 1
            return False
        if row['file_meta'].get('mimetype') not in ("text/html", "application/xhtml+xml"):
            self.counts['skip-mimetype'] += 1
            return False

        if row.get('status') not in ['success', 'unknown-scope']:
            self.counts['skip-hit'] += 1
            return False

        return True


class IngestFilesetResultImporter(IngestFileResultImporter):
    """
    Variant of IngestFileResultImporter for processing, eg, dataset ingest
    results into fileset objects.
    """

    def __init__(self, api, **kwargs):

        eg_desc = kwargs.pop('editgroup_description', None) or "Filesets crawled from web using sandcrawler ingest tool"
        eg_extra = kwargs.pop('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.IngestFilesetResultImporter')
        kwargs['do_updates'] = False
        super().__init__(api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)
        self.max_file_count = 300

    def want_fileset(self, row):

        if not row.get('manifest') or len(row.get('manifest')) == 0:
            self.counts['skip-empty-manifest'] += 1
            return False

        if len(row.get('manifest')) > self.max_file_count:
            self.counts['skip-too-many-files'] += 1
            return False

        return True

    def want(self, row):

        if not self.want_ingest(row):
            return False

        # fileset-specific filters
        if row['request'].get('ingest_type') not in ['dataset',]:
            self.counts['skip-ingest-type'] += 1
            return False

        if not self.want_fileset(row):
            return False

        return True

    def parse_fileset_urls(self, row):
        # XXX: create URLs and rel for dataset ingest
        if not row.get('strategy'):
            return []
        if row['strategy'].startswith('archiveorg') and row.get('archiveorg_item_name'):
            return [
                fatcat_openapi_client.FilesetUrl(
                    url=f"https://archive.org/download/{row['archiveorg_item_name']}",
                    rel="archive",
                )
            ]
        elif row['strategy'].startswith('web') and row.get('web_base_url'):
            return [
                fatcat_openapi_client.FilesetUrl(
                    url=f"https://web.archive.org/web/{row['web_base_url_dt']}/{row['web_base_url']}",
                    rel="webarchive",
                )
            ]
        elif row['strategy'] == 'web-file-bundle' and row.get('web_bundle_url'):
            return [
                fatcat_openapi_client.FilesetUrl(
                    url=f"https://web.archive.org/web/{row['web_bundle_url_dt']}/{row['web_bundle_url']}",
                    rel="webarchive",
                )
            ]
        else:
            return []

    def parse_record(self, row):

        request = row['request']

        # double check that want() filtered request correctly
        if request.get('ingest_type') not in ["dataset",]:
            self.counts['skip-ingest-type'] += 1
            return None

        # identify release by fatcat ident, or extid lookup
        release_ident = self.parse_ingest_release_ident(row)

        if not release_ident:
            self.counts['skip-release-not-found'] += 1
            return None

        entity_extra = dict()
        edit_extra = self.parse_edit_extra(row)
        edit_extra['ingest_strategy'] = row['ingest_strategy']
        if row.get('platform'):
            edit_extra['platform'] = row['platform']
        if row.get('platform_id'):
            edit_extra['platform_id'] = row['platform_id']

        entity_urls = self.parse_fileset_urls(row)
        if not entity_urls:
            self.counts['skip-no-access-url'] += 1
            return None

        assert row['file_count'] == len(row['manifest'])
        if row['file_count'] > self.max_file_count:
            self.counts['skip-too-many-manifest-files'] += 1
            return None

        manifest = []
        for ingest_file in row['manifest']:
            fsf = fatcat_openapi_client.FilesetFile(
                path=ingest_file['path'],
                size=ingest_file['size'],
                md5=ingest_file['md5'],
                sha1=ingest_file['sha1'],
                sha256=ingest_file.get('sha256'),
                extra=dict(
                    mimetype=ingest_file['mimetype'],
                ),
            )
            if not (fsf.md5 and fsf.sha1 and fsf.path and fsf.size):
                self.counts['skip-partial-file-info'] += 1
                return None
            if ingest_file.get('platform_url'):
                # XXX: should we include this?
                fsf.extra['original_url'] = ingest_file['platform_url']
            if ingest_file.get('terminal_url') and ingest_file.get('terminal_dt'):
                fsf.extra['wayback_url'] = f"https://web.archive.org/web/{ingest_file['terminal_dt']}/{ingest_file['terminal_url']}"
            manifest.append(fsf)

        fe = fatcat_openapi_client.FilesetEntity(
            manifest=manifest,
            urls=entity_urls,
            release_ids=[release_ident],
        )

        if entity_extra:
            fe.extra = entity_extra
        if edit_extra:
            fe.edit_extra = edit_extra
        return fe

    def try_update(self, wc):

        # check for existing edits-in-progress with same URL
        for other in self._entity_queue:
            # XXX: how to duplicate check?
            if other.original_url == wc.original_url:
                self.counts['skip-in-queue'] += 1
                return False

        # lookup sha1, or create new entity (TODO: API doesn't support this yet)
        #existing = None

        # NOTE: in lieu of existing checks (by lookup), only allow one fileset per release
        release = self.api.get_release(wc.release_ids[0], expand="filesets")
        if release.filesets:
            # XXX: how to duplicate check filesets?
            # check if this is an existing match, or just a similar hit
            for other in release.filesets:
                if wc.original_url == other.original_url:
                    # TODO: compare very similar timestamps of same time (different formats)
                    self.counts['exists'] += 1
                    return False
            self.counts['skip-release-has-fileset'] += 1
            return False

        return True

    def insert_batch(self, batch):
        if self.submit_mode:
            eg = self.api.create_editgroup(fatcat_openapi_client.Editgroup(
                description=self.editgroup_description,
                extra=self.editgroup_extra))
            for fe in batch:
                self.api.create_fileset(eg.editgroup_id, fe)
            self.api.update_editgroup(eg.editgroup_id, eg, submit=True)
        else:
            self.api.create_fileset_auto_batch(fatcat_openapi_client.FilesetAutoBatch(
                editgroup=fatcat_openapi_client.Editgroup(
                    description=self.editgroup_description,
                    extra=self.editgroup_extra),
                entity_list=batch))


class SavePaperNowFilesetImporter(IngestFilesetResultImporter):
    """
    Like SavePaperNowFileImporter, but for fileset/dataset ingest.
    """

    def __init__(self, api, submit_mode=True, **kwargs):

        eg_desc = kwargs.pop('editgroup_description', None) or "Fileset crawled after a public 'Save Paper Now' request"
        eg_extra = kwargs.pop('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.SavePaperNowFilesetImporter')
        kwargs['submit_mode'] = submit_mode
        kwargs['do_updates'] = False
        super().__init__(api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)

    def want(self, row):

        source = row['request'].get('ingest_request_source')
        if not source:
            self.counts['skip-ingest_request_source'] += 1
            return False
        if not source.startswith('savepapernow'):
            self.counts['skip-not-savepapernow'] += 1
            return False

        if row.get('hit') != True:
            self.counts['skip-hit'] += 1
            return False

        if not self.want_fileset(row):
            return False

        return True
