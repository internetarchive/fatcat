
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

@app.route('/v0/work/<work_id>', methods=['GET'])
def api_work_get(work_id):
    if not work_id.isdigit():
        return abort(404)
    entity = WorkIdent.query.filter(WorkIdent.id==work_id).first_or_404()
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

@app.route('/v0/file/<file_id>', methods=['GET'])
def api_file_get(file_id):
    if not file_id.isdigit():
        return abort(404)
    entity = FileIdent.query.filter(FileIdent.id==file_id).first_or_404()
    return file_schema.jsonify(entity)

@app.route('/v0/file/random', methods=['GET'])
def api_file_random():
    entity = FileIdent.query.order_by(db.func.random()).first()
    return redirect('/v0/file/{}'.format(entity.id))
