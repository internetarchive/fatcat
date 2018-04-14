
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
    work = hydrate_work(work_id)
    return jsonify(work)

@app.route('/v0/work/random', methods=['GET'])
def api_work_random():
    work = WorkIdent.query.order_by(db.func.random()).first()
    return redirect('/v0/work/{}'.format(work.id))
