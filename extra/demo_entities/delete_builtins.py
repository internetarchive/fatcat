# cp ../extra/demo_entities/delete_builtins.py .
# pipenv run python3 delete_builtins.py

import fatcat_openapi_client
from fatcat_tools import *
import os

token = os.environ['FATCAT_API_AUTH_TOKEN']
assert token
api = authenticated_api('http://localhost:9411/v0', token)

eg = api.create_editgroup(fatcat_openapi_client.Editgroup(
    description="Clear out built-in database example entities"))

container_ids = (
    "aaaaaaaaaaaaaeiraaaaaaaaae",
    "aaaaaaaaaaaaaeiraaaaaaaaai",
    "aaaaaaaaaaaaaeiraaaaaaaaam",
)
creator_ids = (
    "aaaaaaaaaaaaaircaaaaaaaaae",
    "aaaaaaaaaaaaaircaaaaaaaaai",
    "aaaaaaaaaaaaaircaaaaaaaaam",
    #"aaaaaaaaaaaaaircaaaaaaaaaq",
)
file_ids = (
    "aaaaaaaaaaaaamztaaaaaaaaae",
    "aaaaaaaaaaaaamztaaaaaaaaai",
    "aaaaaaaaaaaaamztaaaaaaaaam",
)
fileset_ids = (
    "aaaaaaaaaaaaaztgaaaaaaaaam",
    "aaaaaaaaaaaaaztgaaaaaaaaai",
    #"aaaaaaaaaaaaaztgaaaaaaaaam",
)
webcapture_ids = (
    "aaaaaaaaaaaaa53xaaaaaaaaae",
    "aaaaaaaaaaaaa53xaaaaaaaaai",
    "aaaaaaaaaaaaa53xaaaaaaaaam",
)
release_ids = (
    "aaaaaaaaaaaaarceaaaaaaaaae",
    "aaaaaaaaaaaaarceaaaaaaaaai",
    "aaaaaaaaaaaaarceaaaaaaaaam",
)
work_ids = (
    "aaaaaaaaaaaaavkvaaaaaaaaae",
    "aaaaaaaaaaaaavkvaaaaaaaaai",
    "aaaaaaaaaaaaavkvaaaaaaaaam",
)

for eid in container_ids:
    api.delete_container(eid, editgroup_id=eg.editgroup_id)
for eid in creator_ids:
    api.delete_creator(eid, editgroup_id=eg.editgroup_id)
for eid in file_ids:
    api.delete_file(eid, editgroup_id=eg.editgroup_id)
for eid in fileset_ids:
    api.delete_fileset(eid, editgroup_id=eg.editgroup_id)
for eid in webcapture_ids:
    api.delete_webcapture(eid, editgroup_id=eg.editgroup_id)
for eid in release_ids:
    api.delete_release(eid, editgroup_id=eg.editgroup_id)
for eid in work_ids:
    api.delete_work(eid, editgroup_id=eg.editgroup_id)

api.accept_editgroup(eg.editgroup_id)

# editgroup aaaaaaaaaaaabo53aaaaaaaaa4 should be un-submitted
partial_eg = api.get_editgroup('aaaaaaaaaaaabo53aaaaaaaaa4')
api.update_editgroup('aaaaaaaaaaaabo53aaaaaaaaa4', partial_eg, submit=False)

