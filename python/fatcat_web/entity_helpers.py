
from flask import abort
from fatcat_client.rest import ApiException
from fatcat_tools.transforms import *
from fatcat_web import api


def generic_get_entity(entity_type, ident):
    try:
        if entity_type == 'container':
            entity = api.get_container(ident)
            if entity.state == "active":
                entity.es = container_to_elasticsearch(entity, force_bool=False)
            return entity
        else:
            raise NotImplementedError
    except ApiException as ae:
        abort(ae.status)

def generic_get_editgroup_entity(editgroup, entity_type, ident):
    if entity_type == 'container':
        edits = editgroup.edits.containers
    else:
        raise NotImplementedError
    for e in edits:
        if e.ident == ident:
            revision_id = e.revision
            edit = e
            break
    if not revision_id:
        # couldn't find relevent edit in this editgroup
        abort(404)
    try:
        if entity_type == 'container':
            entity = api.get_container_revision(revision_id)
        else:
            raise NotImplementedError
    except ApiException as ae:
        abort(ae.status)
    entity.ident = ident
    return entity, edit
