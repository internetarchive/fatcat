
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify
from fatcat import app, db
from fatcat.models import *
from fatcat.sql import *


### Views ###################################################################

@app.route('/v0/work/<work_id>', methods=['GET'])
def api_work_get(work_id):
    if not work_id.isdigit():
        return abort(404)
    #work = hydrate_work(work_id)
    #return jsonify(work)
    entity = WorkIdent.query.filter(WorkIdent.id==work_id).first_or_404()
    return work_schema.jsonify(entity)

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
