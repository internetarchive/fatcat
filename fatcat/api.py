
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify
from fatcat import app, db
from fatcat.models import *


### Views ###################################################################

@app.route('/work/<work_id>', methods=['GET'])
def work_get():
    work = WorkId.query.filter_by(id=work_id).first_or_404()
    return jsonify(work)
