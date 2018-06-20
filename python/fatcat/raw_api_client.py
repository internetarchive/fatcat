
import sys
import json
import requests


class RawFatcatApiClient:

    def __init__(self, host_url):
        self.host_url = host_url
        self.session = requests.Session()
        self._issn_map = dict()

    def get(self, path, data=None):
        headers = {"content-type": "application/json"}
        return self.session.get(self.host_url + path, json=data,
            headers=headers)

    def post(self, path, data=None):
        headers = {"content-type": "application/json"}
        return self.session.post(self.host_url + path, json=data,
            headers=headers)

    def new_editgroup(self):
        rv = self.post('/v0/editgroup', data=dict(
            editor_id=1))
        print(rv)
        print(rv.json())
        assert rv.status_code == 201
        editgroup_id = rv.json()['id']
        return editgroup_id

    def accept_editgroup(self, eg):
        rv = self.post('/v0/editgroup/{}/accept'.format(eg))
        assert rv.status_code == 200
        return rv

    def import_issn_file(self, json_file, create_containers=False, batchsize=100):
        eg = self.new_editgroup()
        i = 0
        with open(json_file, 'r') as file:
            for line in file:
                if i % batchsize == 0:
                    sys.stdout.write('\n{}: '.format(i))
                if (i+1) % 20 == 0:
                    sys.stdout.write('.')
                i = i + 1
                obj = json.loads(line)
                if not ("author" in obj and "title" in obj):
                    continue
                try:
                    self.import_crossref_dict(obj, editgroup=eg,
                        create_containers=create_containers)
                except Exception as e:
                    print("ERROR: {}".format(e))
                if i % batchsize == 0:
                    self.accept_editgroup(eg)
                    eg = self.new_editgroup()
        if i % batchsize != 0:
            self.accept_editgroup(eg)
        print("done!")

    def health(self):
        rv = self.get("/health")
        assert rv.status_code == 200
        return rv.json()
