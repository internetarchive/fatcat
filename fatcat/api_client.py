
import sys
import json
import requests


class FatCatApiClient:

    def __init__(self, host_url):
        self.host_url = host_url
        self.session = requests.Session()

    def get(self, path):
        return self.session.get(self.host_url + path)

    def post(self, path, data=None):
        headers = {"content-type": "application/json"}
        return self.session.post(self.host_url + path, json=data,
            headers=headers)

    def new_editgroup(self):
        rv = self.post('/v0/editgroup', data=dict(
            editor=1))
        assert rv.status_code == 200
        editgroup_id = rv.json()['id']
        return editgroup_id

    def accept_editgroup(self, eg):
        rv = self.post('/v0/editgroup/{}/accept'.format(eg))
        assert rv.status_code == 200
        return rv

    def import_crossref_file(self, json_file):
        eg = self.new_editgroup()
        i = 0
        with open(json_file, 'r') as file:
            for line in file:
                if i % 1000 == 0:
                    sys.stdout.write('\n{}: '.format(i))
                if (i+1) % 20 == 0:
                    sys.stdout.write('.')
                i = i + 1
                obj = json.loads(line)
                if not ("author" in obj and "title" in obj):
                    continue
                try:
                    self.import_crossref_dict(obj, editgroup=eg)
                except Exception as e:
                    print("ERROR: {}".format(e))
        if i % 1000 != 0:
            self.accept_editgroup(eg)
        print("done!")

    def import_crossref_dict(self, meta, editgroup=None, do_extra=False):

        # creators
        creators = []
        for am in meta['author']:
            c = dict(name="{} {}".format(am['given'], am['family']),
                     sortname="{}, {}".format(am['family'], am['given']),
                     orcid=None)
            creators.append(c)

        # container
        container = dict(
            issn=meta.get('ISSN', [None])[0],
            name=meta['container-title'][0],
            #container_id=None,
            #sortname=meta['short-container-title'][0])
            publisher=meta['publisher'])
        #rv = self.post('/v0/container', data=container)
        #assert rv.status_code == 200
        #container_id = rv.json()['id']

        # references
        refs = []
        for i, rm in enumerate(meta.get('reference', [])):
            ref = dict(
                doi=rm.get("DOI", None),
                index=i+1,
                # TODO: how to generate a proper stub here from k/v metadata?
                stub="| ".join(rm.values()))
            refs.append(ref)

        # work and release
        title = meta['title'][0]
        rv = self.post('/v0/work',
            data=dict(title=title, editgroup=editgroup)) #work_type="book"
        assert rv.status_code == 200
        work_id = rv.json()['id']

        if do_extra:
            extra = dict(crossref={
                'links': meta.get('link', []),
                'subject': meta.get('subject'),
                'type': meta['type'],
                'alternative-id': meta.get('alternative-id', [])})
        else:
            extra = None
        rv = self.post('/v0/release', data=dict(
            title=title,
            work=work_id,
            # XXX: creators=creators,
            # XXX: refs=refs,
            #container=container_id,
            release_type=meta['type'],
            doi=meta['DOI'],
            date=meta['created']['date-time'],
            license=meta.get('license', [dict(URL=None)])[0]['URL'] or None,
            issue=meta.get('issue', None),
            volume=meta.get('volume', None),
            pages=meta.get('page', None),
            editgroup=editgroup,
            extra=extra))
        assert rv.status_code == 200
        release_id = rv.json()['id']

    def health(self):
        rv = self.get("/health")
        assert rv.status_code == 200
        return rv.json()
