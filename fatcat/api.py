
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify, session
from fatcat import app, db
from fatcat.models import *
from fatcat.sql import *


### Helpers #################################################################

def get_or_create_edit_group():
    editor = Editor.query.filter(Editor.id==1).first()
    if editor.active_edit_group:
        return editor.active_edit_group
    else:
        edit_group = EditGroup(editor=editor)
        db.session.add(edit_group)
        db.session.commit()
        editor.active_edit_group = edit_group
        db.session.add(editor)
        db.session.commit()
        return edit_group

### Views ###################################################################

@app.route('/v0/work/<int:ident>', methods=['GET'])
def api_work_get(ident):
    entity = WorkIdent.query.filter(WorkIdent.id==ident).first_or_404()
    return work_schema.jsonify(entity)

@app.route('/v0/work', methods=['POST'])
def api_work_create():
    """
    1. find or create edit_group
    2. create work_edit, work_rev, work_ident

    TODO: use marshmallow?
    """
    params = request.get_json()
    edit_group = get_or_create_edit_group()
    rev = WorkRev(
        title=params.get('title', None),
        work_type=params.get('work_type', None),
    )
    ident = WorkIdent(is_live=False, rev=rev)
    edit = WorkEdit(edit_group=edit_group, ident=ident, rev=rev)
    if params.get('extra', None):
        ser = json.dumps(params['extra'], indent=False).encode('utf-8')
        rev.extra_json = ExtraJson(json=ser, sha1=hashlib.sha1(ser).hexdigest())
    db.session.add_all([edit, ident, rev])
    db.session.commit()
    return work_schema.jsonify(ident)

@app.route('/v0/work/random', methods=['GET'])
def api_work_random():
    entity = WorkIdent.query.order_by(db.func.random()).first()
    return redirect('/v0/work/{}'.format(entity.id))


@app.route('/v0/release/<int:ident>', methods=['GET'])
def api_release_get(ident):
    entity = ReleaseIdent.query.filter(ReleaseIdent.id==ident).first_or_404()
    return release_schema.jsonify(entity)

@app.route('/v0/release', methods=['POST'])
def api_release_create():
    params = request.get_json()
    edit_group = get_or_create_edit_group()
    rev = ReleaseRev(
        title=params.get('title', None),
        release_type=params.get('release_type', None),
        creators=params.get('creators', []),
        #work=params.get('work', None),
        container=params.get('container', None),
        doi=params.get('doi', None),
    )
    ident = ReleaseIdent(is_live=False, rev=rev)
    edit = ReleaseEdit(edit_group=edit_group, ident=ident, rev=rev)
    if params.get('extra', None):
        ser = json.dumps(params['extra'], indent=False).encode('utf-8')
        rev.extra_json = ExtraJson(json=ser, sha1=hashlib.sha1(ser).hexdigest())
    db.session.add_all([edit, ident, rev])
    db.session.commit()
    return release_schema.jsonify(ident)


@app.route('/v0/creator/<int:ident>', methods=['GET'])
def api_creator_get(ident):
    entity = CreatorIdent.query.filter(CreatorIdent.id==ident).first_or_404()
    return creator_schema.jsonify(entity)

@app.route('/v0/creator', methods=['POST'])
def api_creator_create():
    params = request.get_json()
    edit_group = get_or_create_edit_group()
    rev = CreatorRev(
        name=params.get('name', None),
        orcid=params.get('orcid', None),
    )
    ident = CreatorIdent(is_live=False, rev=rev)
    edit = CreatorEdit(edit_group=edit_group, ident=ident, rev=rev)
    if params.get('extra', None):
        ser = json.dumps(params['extra'], indent=False).encode('utf-8')
        rev.extra_json = ExtraJson(json=ser, sha1=hashlib.sha1(ser).hexdigest())
    db.session.add_all([edit, ident, rev])
    db.session.commit()
    return creator_schema.jsonify(ident)


@app.route('/v0/container/<int:ident>', methods=['GET'])
def api_container_get(ident):
    entity = ContainerIdent.query.filter(ContainerIdent.id==ident).first_or_404()
    return container_schema.jsonify(entity)

@app.route('/v0/container', methods=['POST'])
def api_container_create():
    params = request.get_json()
    edit_group = get_or_create_edit_group()
    rev = ContainerRev(
        name=params.get('name', None),
        publisher=params.get('publisher', None),
    )
    ident = ContainerIdent(is_live=False, rev=rev)
    edit = ContainerEdit(edit_group=edit_group, ident=ident, rev=rev)
    if params.get('extra', None):
        ser = json.dumps(params['extra'], indent=False).encode('utf-8')
        rev.extra_json = ExtraJson(json=ser, sha1=hashlib.sha1(ser).hexdigest())
    db.session.add_all([edit, ident, rev])
    db.session.commit()
    return container_schema.jsonify(ident)


@app.route('/v0/file/<int:ident>', methods=['GET'])
def api_file_get(ident):
    entity = FileIdent.query.filter(FileIdent.id==ident).first_or_404()
    return file_schema.jsonify(entity)

@app.route('/v0/file', methods=['POST'])
def api_file_create():
    params = request.get_json()
    edit_group = get_or_create_edit_group()
    rev = FileRev(
        sha1=params.get('sha1', None),
        size=params.get('size', None),
        url=params.get('url', None),
    )
    ident = FileIdent(is_live=False, rev=rev)
    edit = FileEdit(edit_group=edit_group, ident=ident, rev=rev)
    if params.get('extra', None):
        ser = json.dumps(params['extra'], indent=False).encode('utf-8')
        rev.extra_json = ExtraJson(json=ser, sha1=hashlib.sha1(ser).hexdigest())
    db.session.add_all([edit, ident, rev])
    db.session.commit()
    return file_schema.jsonify(ident)


@app.route('/v0/editgroup/<int:ident>', methods=['GET'])
def api_edit_group_get(ident):
    entity = EditGroupIdent.query.filter(EditGroupIdent.id==ident).first_or_404()
    return edit_group_schema.jsonify(entity)

@app.route('/v0/editgroup', methods=['POST'])
def api_edit_group_create():
    params = request.get_json()
    eg = EditGroup(
        editor_id=1,
        description=params.get('description', None),
    )
    if params.get('extra', None):
        ser = json.dumps(params['extra'], indent=False).encode('utf-8')
        eg.extra_json = ExtraJson(json=ser, sha1=hashlib.sha1(ser).hexdigest())
    db.session.add(eg)
    db.session.commit()
    return edit_group_schema.jsonify(eg)

