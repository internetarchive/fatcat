
from flask import abort
from fatcat_openapi_client.rest import ApiException
from fatcat_tools.transforms import *
from fatcat_web import app, api
from fatcat_web.search import get_elastic_container_stats, get_elastic_container_random_releases
from fatcat_web.hacks import strip_extlink_xml, wayback_suffix

def enrich_container_entity(entity):
    if entity.state in ('redirect', 'deleted'):
        return entity
    if entity.state == "active":
        entity._es = container_to_elasticsearch(entity, force_bool=False)
    entity._stats = None
    try:
        entity._stats = get_elastic_container_stats(entity.ident, issnl=entity.issnl)
        #if entity._stats['total'] > 0:
        entity._random_releases = get_elastic_container_random_releases(entity.ident)
    except Exception as e:
        app.log.error(e)
        pass
    return entity

def enrich_creator_entity(entity):
    if entity.state in ('redirect', 'deleted'):
        return entity
    entity._releases = None
    if entity.state in ('active', 'wip'):
        entity._releases = api.get_creator_releases(entity.ident)
    return entity

def enrich_file_entity(entity):
    return entity

def enrich_fileset_entity(entity):
    if entity.state in ('redirect', 'deleted'):
        return entity
    entity._total_size = None
    if entity.manifest != None:
        entity._total_size = sum([f.size for f in entity.manifest]) or 0
    return entity

def enrich_webcapture_entity(entity):
    if entity.state in ('redirect', 'deleted'):
        return entity
    entity._wayback_suffix = wayback_suffix(entity)
    return entity

def enrich_release_entity(entity):
    if entity.state in ('redirect', 'deleted'):
        return entity
    if entity.state == "active":
        entity._es = release_to_elasticsearch(entity, force_bool=False)
    if entity.container and entity.container.state == "active":
        entity.container._es = container_to_elasticsearch(entity.container, force_bool=False)
    if entity.files:
        # remove shadows-only files with no URLs
        entity.files = [f for f in entity.files
            if not (f.extra and f.extra.get('shadows') and not f.urls)]
    if entity.filesets:
        for fs in entity.filesets:
            fs._total_size = sum([f.size for f in fs.manifest])
    if entity.webcaptures:
        for wc in entity.webcaptures:
            wc._wayback_suffix = wayback_suffix(wc)
    for ref in entity.refs:
        # this is a UI hack to get rid of XML crud in unstructured refs like:
        # LOCKSS (2014) Available: <ext-link
        # xmlns:xlink="http://www.w3.org/1999/xlink" ext-link-type="uri"
        # xlink:href="http://lockss.org/"
        # xlink:type="simple">http://lockss.org/</ext-link>. Accessed: 2014
        # November 1.
        if ref.extra and ref.extra.get('unstructured'):
            ref.extra['unstructured'] = strip_extlink_xml(ref.extra['unstructured'])
    # for backwards compatability, copy extra['subtitle'] to subtitle
    if not entity.subtitle and entity.extra and entity.extra.get('subtitle'):
        if isinstance(entity.extra['subtitle'], str):
            entity.subtitle = entity.extra['subtitle']
    # author list to display; ensure it's sorted by index (any othors with
    # index=None go to end of list)
    authors = [c for c in entity.contribs if c.role in ('author', None)]
    entity._authors = sorted(authors, key=lambda c: (c.index == None and 99999999) or c.index)
    if entity.abstracts:
        # hack to show plain text instead of latex abstracts
        if 'latex' in entity.abstracts[0].mimetype:
            entity.abstracts.reverse()
        # hack to (partially) clean up common JATS abstract display case
        if entity.abstracts[0].mimetype == 'application/xml+jats':
            for tag in ('p', 'jats', 'jats:p'):
                entity.abstracts[0].content = entity.abstracts[0].content.replace('<{}>'.format(tag), '')
                entity.abstracts[0].content = entity.abstracts[0].content.replace('</{}>'.format(tag), '')
                # ugh, double encoding happens
                entity.abstracts[0].content = entity.abstracts[0].content.replace('&lt;/{}&gt;'.format(tag), '')
                entity.abstracts[0].content = entity.abstracts[0].content.replace('&lt;{}&gt;'.format(tag), '')
    return entity

def enrich_work_entity(entity):
    if entity.state in ('redirect', 'deleted'):
        return entity
    entity._releases = None
    if entity.state in ('active', 'wip'):
        entity._releases = api.get_work_releases(entity.ident)
    return entity

def generic_get_entity(entity_type, ident):
    try:
        if entity_type == 'container':
            return enrich_container_entity(api.get_container(ident))
        elif entity_type == 'creator':
            return enrich_creator_entity(api.get_creator(ident))
        elif entity_type == 'file':
            return enrich_file_entity(api.get_file(ident, expand="releases"))
        elif entity_type == 'fileset':
            return enrich_fileset_entity(api.get_fileset(ident, expand="releases"))
        elif entity_type == 'webcapture':
            return enrich_webcapture_entity(api.get_webcapture(ident, expand="releases"))
        elif entity_type == 'release':
            return enrich_release_entity(api.get_release(ident, expand="container,files,filesets,webcaptures"))
        elif entity_type == 'work':
            return enrich_work_entity(api.get_work(ident))
        else:
            raise NotImplementedError
    except ApiException as ae:
        abort(ae.status)

def generic_get_entity_revision(entity_type, revision_id):
    try:
        if entity_type == 'container':
            return enrich_container_entity(api.get_container_revision(revision_id))
        elif entity_type == 'creator':
            return enrich_creator_entity(api.get_creator_revision(revision_id))
        elif entity_type == 'file':
            return enrich_file_entity(api.get_file_revision(revision_id, expand="releases"))
        elif entity_type == 'fileset':
            return enrich_fileset_entity(api.get_fileset_revision(revision_id, expand="releases"))
        elif entity_type == 'webcapture':
            return enrich_webcapture_entity(api.get_webcapture_revision(revision_id, expand="releases"))
        elif entity_type == 'release':
            return enrich_release_entity(api.get_release_revision(revision_id, expand="container"))
        elif entity_type == 'work':
            return enrich_work_entity(api.get_work_revision(revision_id))
        else:
            raise NotImplementedError
    except ApiException as ae:
        abort(ae.status)

def generic_get_editgroup_entity(editgroup, entity_type, ident):
    if entity_type == 'container':
        edits = editgroup.edits.containers
    elif entity_type == 'creator':
        edits = editgroup.edits.creators
    elif entity_type == 'file':
        edits = editgroup.edits.files
    elif entity_type == 'fileset':
        edits = editgroup.edits.filesets
    elif entity_type == 'webcapture':
        edits = editgroup.edits.webcaptures
    elif entity_type == 'release':
        edits = editgroup.edits.releases
    elif entity_type == 'work':
        edits = editgroup.edits.works
    else:
        raise NotImplementedError
    revision_id = None
    for e in edits:
        if e.ident == ident:
            revision_id = e.revision
            edit = e
            break
    if not revision_id:
        # couldn't find relevent edit in this editgroup
        abort(404)

    entity = generic_get_entity_revision(entity_type, revision_id)
    entity.ident = ident
    return entity, edit
