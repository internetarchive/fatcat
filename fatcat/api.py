
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify
from fatcat import app, db, examples
from fatcat.models import *


### Views ###################################################################

@app.route('/v0/work/<work_id>', methods=['GET'])
def work_get(work_id):
    if work_id == "random":
        work = examples['work']
    else:
        work = WorkId.query.filter_by(id=work_id).first_or_404()
    return jsonify(work)

