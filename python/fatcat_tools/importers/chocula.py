
import fatcat_openapi_client
from .common import EntityImporter, clean


class ChoculaImporter(EntityImporter):
    """
    Creates or updates container entities based on output of "chocula" script,
    which munges/processes journal metadata from several sources, including
    fatcat itself.

    See guide for details on the many 'extra' fields used here.
    """

    def __init__(self, api, **kwargs):

        eg_desc = kwargs.get('editgroup_description',
            "Automated import of container-level metadata from Chocula tool.")
        eg_extra = kwargs.get('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.ChoculaImporter')
        super().__init__(api,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)

    def want(self, raw_record):
        if not raw_record.get('ident') and not raw_record.get('_known_issnl'):
            self.counts['skip-unknown-new-issnl'] += 1
            return False
        if raw_record.get('issnl') and raw_record.get('name'):
            return True
        return False

    def parse_record(self, row):
        """
        row is a python dict (parsed from JSON).

        returns a ContainerEntity (or None if invalid or couldn't parse)
        """

        name = clean(row.get('name'))
        if not name:
            # Name is required (by schema)
            return None

        name = name.strip()

        if name.endswith(',  Proceedings of the'):
            name = "Proceedings of the " + name.split(',')[0]

        if name.endswith('.'):
            name = name[:-1]

        extra = dict()
        for k in ('urls', 'webarchive_urls', 'issne', 'issnp', 'country',
                  'sherpa_romeo', 'ezb', 'szczepanski', 'doaj', 'languages'):
            if row['extra'].get(k):
                extra[k] = row['extra'][k]

        container_type = None
        if 'proceedings' in name.lower():
            container_type = 'proceedings'
        elif 'journal ' in name.lower():
            container_type = 'journal'

        ce = fatcat_openapi_client.ContainerEntity(
            issnl=row['issnl'],
            ident=row['ident'],
            name=name,
            container_type=container_type,
            publisher=clean(row.get('publisher')),
            wikidata_qid=row.get('wikidata_qid'),
            extra=extra)
        return ce

    def try_update(self, ce):

        existing = None
        if ce.ident:
            try:
                existing = self.api.get_container(ce.ident)
            except fatcat_openapi_client.rest.ApiException as err:
                if err.status != 404:
                    raise err
                self.counts['exists'] += 1
                self.counts['exists-not-found'] += 1
                return False
            if existing.state != 'active':
                self.counts['exists'] += 1
                self.counts['exists-inactive'] += 1
                return False

        if not existing:
            # check if existing by ISSN-L
            try:
                existing = self.api.lookup_container(issnl=ce.issnl)
            except fatcat_openapi_client.rest.ApiException as err:
                if err.status != 404:
                    raise err
            if existing:
                self.counts['exists'] += 1
                self.counts['exists-by-issnl'] += 1
                return False
            # doesn't exist, always create
            return True

        # decide whether to update
        do_update = False
        if not self.do_updates:
            self.counts['exists'] += 1
            return False
        if not existing.extra:
            existing.extra = dict()
        if set(ce.extra.get('urls', [])) != set(existing.extra.get('urls', [])):
            do_update = True
        if set(ce.extra.get('webarchive_urls', [])) != set(existing.extra.get('webarchive_urls', [])):
            do_update = True
        for k in ('ezb', 'szczepanski', 'doaj'):
            if ce.extra.get(k) and not existing.extra.get(k):
                do_update = True
        if ce.publisher and not existing.publisher:
            do_update = True
        if ce.wikidata_qid and not existing.wikidata_qid:
            do_update = True

        if do_update:
            existing.wikidata_qid = ce.wikidata_qid
            existing.publisher = ce.publisher
            existing.container_type = existing.container_type or ce.container_type
            for k in ('urls', 'webarchive_urls'):
                # update, which might clobber, but won't remove
                if ce.extra.get(k):
                    existing.extra[k] = ce.extra.get(k, [])
                # note: in some cases we might *want* to clobber existing (if
                # all URLs found to be bad), but being conservative for now so
                # we don't clobber human edits
            for k in ('issne', 'issnp', 'country', 'sherpa_romeo', 'ezb',
                      'szczepanski', 'doaj'):
                # update/overwrite, but don't remove any existing value
                if ce.extra.get(k):
                    existing.extra[k] = ce.extra[k]
            if ce.extra.get('languages'):
                if not existing.extra.get('languages'):
                    existing.extra['languages'] = ce.extra['languages']
                elif not ce.extra['languages'][0] in existing.extra['languages']:
                    existing.extra['languages'].append(ce.extra['languages'][0])

            self.api.update_container(self.get_editgroup_id(), existing.ident, existing)
            self.counts['update'] += 1
            return False
        else:
            self.counts['exists'] += 1
            self.counts['exists-skip-update'] += 1
            return False

        # if we got this far, it's a bug
        raise NotImplementedError

    def insert_batch(self, batch):
        self.api.create_container_auto_batch(fatcat_openapi_client.ContainerAutoBatch(
            editgroup=fatcat_openapi_client.Editgroup(
                description=self.editgroup_description,
                extra=self.editgroup_extra),
            entity_list=batch))
