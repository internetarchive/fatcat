
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify, session
from fatcat import app, db
from fatcat.models import *
from fatcat.sql import *


### Helpers #################################################################

def get_or_create_editgroup(param=None):
    if param != None:
        editgroup = EditGroup.query.get_or_404(int(param))
        return editgroup
    editor = Editor.query.get_or_404(1)
    if editor.active_editgroup:
        return editor.active_editgroup

    editgroup = EditGroup(editor=editor)
    db.session.add(editgroup)
    db.session.commit()
    editor.active_editgroup = editgroup
    db.session.add(editor)
    db.session.commit()
    return editgroup

### Views ###################################################################

@app.route('/v0/work/<int:ident>', methods=['GET'])
def api_work_get(ident):
    entity = WorkIdent.query.get_or_404(ident)
    return work_schema.jsonify(entity)

@app.route('/v0/work', methods=['POST'])
def api_work_create(params=None):
    # TODO: Special-case to pull out primary and create that?
    if params == None:
        params = request.get_json()
    editgroup = get_or_create_editgroup(params.get('editgroup'))
    rev = WorkRev(
        title=params.get('title', None),
        work_type=params.get('work_type', None),
    )
    ident = WorkIdent(is_live=False, rev=rev)
    edit = WorkEdit(editgroup=editgroup, ident=ident, rev=rev)
    if params.get('extra', None):
        rev.extra_json = json.dumps(params['extra'], indent=False).encode('utf-8')
    db.session.add_all([edit, ident, rev])
    db.session.commit()
    return work_schema.jsonify(ident)

@app.route('/v0/work/random', methods=['GET'])
def api_work_random():
    entity = WorkIdent.query.order_by(db.func.random()).first()
    return redirect('/v0/work/{}'.format(entity.id))


@app.route('/v0/release/<int:ident>', methods=['GET'])
def api_release_get(ident):
    entity = ReleaseIdent.query.get_or_404(ident)
    return release_schema.jsonify(entity)

@app.route('/v0/release', methods=['POST'])
def api_release_create(params=None):
    if params == None:
        params = request.get_json()
    editgroup = get_or_create_editgroup(params.get('editgroup'))
    creators = params.get('creators', [])
    creators = [CreatorIdent.query.get_or_404(c) for c in creators]
    targets = [ref['target'] for ref in params.get('refs', []) if ref.get('target') != None]
    targets = [ReleaseIdent.query.get_or_404(t) for t in targets]
    work = params.get('work')
    if work:
        work = WorkIdent.query.get_or_404(work)
    container = params.get('container')
    if container:
        container = ContainerIdent.query.get_or_404(container)
    rev = ReleaseRev(
        title=params.get('title', None),
        release_type=params.get('release_type', None),
        work=work,
        container=container,
        doi=params.get('doi', None),
    )
    contribs = [ReleaseContrib(release=rev, creator=c) for c in creators]
    rev.creators = contribs
    db.session.add_all(contribs)
    refs = [ReleaseRef(release=rev, target=t) for t in targets]
    rev.refs = refs
    db.session.add_all(refs)
    ident = ReleaseIdent(is_live=False, rev=rev)
    edit = ReleaseEdit(editgroup=editgroup, ident=ident, rev=rev)
    if params.get('extra', None):
        rev.extra_json = json.dumps(params['extra'], indent=False).encode('utf-8')
    db.session.add_all([edit, ident, rev])
    db.session.commit()
    return release_schema.jsonify(ident)

@app.route('/v0/release/<int:ident>/changelog', methods=['GET'])
def api_release_changelog(ident):
    entries = ChangelogEntry.query\
        .join(ReleaseEdit.editgroup)\
        .filter(ReleaseEdit.ident_id==ident)\
        .all()
    return changelogentry_schema.jsonify(entries, many=True)

@app.route('/v0/release/random', methods=['GET'])
def api_release_random():
    entity = ReleaseIdent.query.order_by(db.func.random()).first()
    return redirect('/v0/release/{}'.format(entity.id))

@app.route('/v0/release/lookup', methods=['GET'])
def api_release_lookup():
    params = request.get_json()
    doi = params['doi'].strip().lower()
    # TODO: proper regex
    if not (doi.startswith("10.") and len(doi.split('/')) == 2):
        abort(400)
    entity = ReleaseIdent.query\
        .join(ReleaseIdent.rev)\
        .filter(ReleaseRev.doi==doi)\
        .first_or_404()
    return release_schema.jsonify(entity)


@app.route('/v0/creator/<int:ident>', methods=['GET'])
def api_creator_get(ident):
    entity = CreatorIdent.query.get_or_404(ident)
    return creator_schema.jsonify(entity)

@app.route('/v0/creator', methods=['POST'])
def api_creator_create(params=None):
    if params == None:
        params = request.get_json()
    editgroup = get_or_create_editgroup(params.get('editgroup'))
    rev = CreatorRev(
        name=params.get('name', None),
        orcid=params.get('orcid', None),
    )
    ident = CreatorIdent(is_live=False, rev=rev)
    edit = CreatorEdit(editgroup=editgroup, ident=ident, rev=rev)
    if params.get('extra', None):
        rev.extra_json = json.dumps(params['extra'], indent=False).encode('utf-8')
    db.session.add_all([edit, ident, rev])
    db.session.commit()
    return creator_schema.jsonify(ident)

@app.route('/v0/creator/lookup', methods=['GET'])
def api_creator_lookup():
    params = request.get_json()
    orcid = params['orcid'].strip()
    # TODO: proper regex
    if not (len(orcid) == len("0000-0002-1825-0097") and len(orcid.split('-')) == 4):
        abort(400)
    entity = CreatorIdent.query\
        .join(CreatorIdent.rev)\
        .filter(CreatorRev.orcid==orcid)\
        .first_or_404()
    return creator_schema.jsonify(entity)


@app.route('/v0/container/<int:ident>', methods=['GET'])
def api_container_get(ident):
    entity = ContainerIdent.query.get_or_404(ident)
    return container_schema.jsonify(entity)

@app.route('/v0/container', methods=['POST'])
def api_container_create(params=None):
    if params == None:
        params = request.get_json()
    editgroup = get_or_create_editgroup(params.get('editgroup'))
    rev = ContainerRev(
        name=params.get('name', None),
        publisher=params.get('publisher', None),
        issn=params.get('issn', None),
    )
    ident = ContainerIdent(is_live=False, rev=rev)
    edit = ContainerEdit(editgroup=editgroup, ident=ident, rev=rev)
    if params.get('extra', None):
        rev.extra_json = json.dumps(params['extra'], indent=False).encode('utf-8')
    db.session.add_all([edit, ident, rev])
    db.session.commit()
    return container_schema.jsonify(ident)

@app.route('/v0/container/lookup', methods=['GET'])
def api_container_lookup():
    params = request.get_json()
    issn = params['issn'].strip()
    # TODO: proper regex
    if not (len(issn) == 9 and issn[0:4].isdigit() and issn[5:7].isdigit()):
        abort(400)
    entity = ContainerIdent.query\
        .join(ContainerIdent.rev)\
        .filter(ContainerRev.issn==issn)\
        .first_or_404()
    return container_schema.jsonify(entity)


@app.route('/v0/file/<int:ident>', methods=['GET'])
def api_file_get(ident):
    entity = FileIdent.query.get_or_404(ident)
    return file_schema.jsonify(entity)

@app.route('/v0/file', methods=['POST'])
def api_file_create(params=None):
    if params == None:
        params = request.get_json()
    editgroup = get_or_create_editgroup(params.get('editgroup'))
    releases = params.get('releases', [])
    releases = [ReleaseIdent.query.get_or_404(r) for r in releases]
    rev = FileRev(
        sha1=params.get('sha1', None),
        size=params.get('size', None),
        url=params.get('url', None),
    )
    file_releases = [FileRelease(file=rev, release=r) for r in releases]
    rev.releases = file_releases
    db.session.add_all(file_releases)
    ident = FileIdent(is_live=False, rev=rev)
    edit = FileEdit(editgroup=editgroup, ident=ident, rev=rev)
    if params.get('extra', None):
        rev.extra_json = json.dumps(params['extra'], indent=False).encode('utf-8')
    db.session.add_all([edit, ident, rev])
    db.session.commit()
    return file_schema.jsonify(ident)


@app.route('/v0/editgroup/<int:ident>', methods=['GET'])
def api_editgroup_get(ident):
    entity = EditGroup.query\
        .join(EditGroup.editor)\
        .filter(EditGroup.id==ident)\
        .first_or_404()
    rv = editgroup_schema.dump(entity).data
    rv['work_edits'] = work_edit_schema.dump(
        WorkEdit.query.filter(EditGroup.id==ident).all(), many=True).data
    rv['release_edits'] = release_edit_schema.dump(
        ReleaseEdit.query.filter(EditGroup.id==ident).all(), many=True).data
    rv['creator_edits'] = creator_edit_schema.dump(
        CreatorEdit.query.filter(EditGroup.id==ident).all(), many=True).data
    rv['container_edits'] = container_edit_schema.dump(
        ContainerEdit.query.filter(EditGroup.id==ident).all(), many=True).data
    rv['file_edits'] = file_edit_schema.dump(
        FileEdit.query.filter(EditGroup.id==ident).all(), many=True).data
    return jsonify(rv)

@app.route('/v0/editgroup', methods=['POST'])
def api_editgroup_create(params=None):
    if params == None:
        params = request.get_json()
    eg = EditGroup(
        editor_id=1,
        description=params.get('description', None),
    )
    if params.get('extra', None):
        eg.extra_json = json.dumps(params['extra'], indent=False).encode('utf-8')
    db.session.add(eg)
    db.session.commit()
    return editgroup_schema.jsonify(eg)

@app.route('/v0/editgroup/<int:ident>/accept', methods=['POST'])
def api_editgroup_accept(ident):
    entity = EditGroup.query.get_or_404(ident)
    accept_editgroup(entity)
    return jsonify({'success': True})


@app.route('/v0/editor/<username>', methods=['GET'])
def api_editor_get(username):
    entity = Editor.query.filter(Editor.username==username).first_or_404()
    return editor_schema.jsonify(entity)

@app.route('/v0/editor/<username>/changelog', methods=['GET'])
def api_editor_changelog(username):
    entries = ChangelogEntry.query\
        .join(ChangelogEntry.editgroup)\
        .join(EditGroup.editor)\
        .filter(Editor.username==username)\
        .all()
    return changelogentry_schema.jsonify(entries, many=True)
