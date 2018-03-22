#!/usr/bin/env python3

import os
import argparse
import requests
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify

app = Flask(__name__)
app.config.from_object(__name__)


### Views ###################################################################

@app.route('/health', methods=['GET'])
def health():
    return jsonify({'ok': True})

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


### Entry Point #############################################################

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--debug',
        action='store_true',
        help="enable debugging interface")
    parser.add_argument('--host',
        default="127.0.0.1",
        help="listen on this host/IP")
    parser.add_argument('--port',
        type=int,
        default=5050,
        help="listen on this port")
    parser.add_argument('--backend-api',
        default="localhost:6060",
        help="backend API to connect to")
    args = parser.parse_args()

    app.run(debug=args.debug, host=args.host, port=args.port)

if __name__ == '__main__':
    main()
