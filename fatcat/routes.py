
import os
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify
from fatcat import app, db


### Views ###################################################################

@app.route('/work/create', methods=['GET'])
def work_create():
    return render_template('work_add.html')

@app.route('/work/random', methods=['GET'])
def work_random():
    work = {
        "title": "Structure and Interpretation",
        "work_type": "book",
        "date": None,
        "contributors": [
            {"name": "Alyssa P. Hacker"},
        ],
        "primary": {
            "title": "Structure and Interpretation",
            "release_type": "online",
            "date": "2000-01-01",
            "doi": "10.491/599.sdo14",
        },
        "releases": [
        ]
    }
    return render_template('work_view.html', work=work, primary=work['primary'])

@app.route('/work/<work_id>/random', methods=['GET'])
def work_view(work_id):
    return render_template('work_view.html')


### Static Routes ###########################################################

@app.route('/', methods=['GET'])
def homepage():
    return render_template('home.html')

@app.route('/about', methods=['GET'])
def aboutpage():
    return render_template('about.html')

@app.route('/robots.txt', methods=['GET'])
def robots():
    return send_from_directory(os.path.join(app.root_path, 'static'),
                               'robots.txt',
                               mimetype='text/plain')

@app.route('/health', methods=['GET'])
def health():
    return jsonify({'ok': True})
