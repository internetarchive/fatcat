
from fatcat_openapi_client.rest import ApiException
from fatcat_openapi_client.models import FileEntity

from .common import EntityCleaner


class FileCleaner(EntityCleaner):
    """
    File fixups!
    """

    def __init__(self, api, **kwargs):

        eg_desc = kwargs.pop('editgroup_description', None) or "Automated cleanup of file entities (eg, remove bad URLs)"
        eg_extra = kwargs.pop('editgroup_extra', dict())
        eg_extra['agent'] = eg_extra.get('agent', 'fatcat_tools.FileCleaner')
        super().__init__(api,
            entity_type=FileEntity,
            editgroup_description=eg_desc,
            editgroup_extra=eg_extra,
            **kwargs)

    def clean_entity(self, entity):
        """
        TODO: mimetype is bogus like (???) => clean mimetype
        """

        # URL has ://web.archive.org/web/None/ link => delete URL
        entity.urls = [u for u in entity.urls if '://web.archive.org/web/None/' not in u.url]

        # URL has ://archive.org/ link with rel=repository => rel=archive
        for u in entity.urls:
            if '://archive.org/' in u.url and u.rel == 'repository':
                u.rel = 'archive'

        # URL has short wayback date ("2017") and another url with that as prefix => delete URL
        stub_wayback_urls = []
        full_wayback_urls = []
        for u in entity.urls:
            if '://web.archive.org/web/' in u.url:
                if len(u.url.split('/')[4]) <= 8:
                    stub_wayback_urls.append(u.url)
                else:
                    full_wayback_urls.append('/'.join(u.url.split('/')[5:]))
        for stub in stub_wayback_urls:
            target = '/'.join(stub.split('/')[5:])
            if target in full_wayback_urls:
                entity.urls = [u for u in entity.urls if u.url != stub]

        return entity

    def try_update(self, entity):

        try:
            existing = self.api.get_file(entity.ident)
        except ApiException as err:
            if err.status != 404:
                raise err
            self.counts['skip-not-found'] += 1
            return 0

        if existing.state != 'active':
            self.counts['skip-existing-inactive'] += 1
            return 0
        if existing.revision != entity.revision:
            self.counts['skip-revision'] += 1
            return 0

        self.api.update_file(self.get_editgroup_id(), entity.ident, entity)
        return 1
