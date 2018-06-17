
import os
import json
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify, session
from fatcat import app, api
from fatcat_client.rest import ApiException


### Views ###################################################################


@app.route('/container/<uuid:ident>', methods=['GET'])
def container_view(ident):
    try:
        entity = api.get_container(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('container_view.html', container=entity)

@app.route('/container/random', methods=['GET'])
def container_random():
    """Not actually random, just a dummy example"""
    return redirect("/container/00000000-0000-0000-1111-000000000002")

@app.route('/container/create', methods=['GET'])
def container_create_view():
    return render_template('container_add.html')

@app.route('/container/create', methods=['POST'])
def container_create():
    params = dict()
    for k in request.form:
        if k.startswith('container_'):
            params[k[10:]] = request.form[k]
    edit = api.create_container(params=params)
    return redirect("/container/{}".format(edit.ident))

@app.route('/creator/<uuid:ident>', methods=['GET'])
def creator_view(ident):
    try:
        entity = api.get_creator(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('creator_view.html', creator=entity)

@app.route('/creator/random', methods=['GET'])
def creator_random():
    """Not actually random, just a dummy example"""
    return redirect("/creator/00000000-0000-0000-2222-000000000002")

@app.route('/file/<uuid:ident>', methods=['GET'])
def file_view(ident):
    try:
        entity = api.get_file(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('file_view.html', file=entity)

@app.route('/file/random', methods=['GET'])
def file_random():
    """Not actually random, just a dummy example"""
    return redirect("/file/00000000-0000-0000-3333-000000000002")

@app.route('/release/<uuid:ident>', methods=['GET'])
def release_view(ident):
    try:
        entity = api.get_release(str(ident))
    except ApiException as ae:
        abort(ae.status)
    authors = [c for c in entity.contribs if c.role in ('author', None)]
    authors = sorted(authors, key=lambda c: c.index)
    return render_template('release_view.html', release=entity, authors=authors)

@app.route('/release/random', methods=['GET'])
def release_random():
    """Not actually random, just a dummy example"""
    return redirect("/release/00000000-0000-0000-4444-000000000002")

#@app.route('/release/<uuid:ident>/changelog', methods=['GET'])
#def release_changelog(ident):
#    try:
#        entity = api.get_release(str(ident))
#    except ApiException as ae:
#        abort(ae.status)
#    try:
#        entries = api.release_changelog(str(ident))
#    except ApiException as ae:
#        abort(ae.status)
#    return render_template('release_changelog.html', release=entity,
#        changelog_entries=entries)

@app.route('/work/<uuid:ident>', methods=['GET'])
def work_view(ident):
    try:
        entity = api.get_work(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('work_view.html', work=entity)

@app.route('/work/random', methods=['GET'])
def work_random():
    """Not actually random, just a dummy example"""
    return redirect("/work/00000000-0000-0000-5555-000000000002")

@app.route('/work/create', methods=['GET'])
def work_create():
    return render_template('work_add.html')

@app.route('/editgroup/<int:ident>', methods=['GET'])
def editgroup_view(ident):
    try:
        entity = api.get_editgroup(str(ident))
    except ApiException as ae:
        print(ae.body)
        abort(ae.status)
    return render_template('editgroup_view.html', editgroup=entity)

@app.route('/editgroup/current', methods=['GET'])
def editgroup_current():
    raise NotImplemented()
    #eg = api.get_or_create_editgroup()
    #return redirect('/editgroup/{}'.format(eg.id))

@app.route('/editor/<username>', methods=['GET'])
def editor_view(username):
    entity = api.get_editor(username)
    return render_template('editor_view.html', editor=entity)

@app.route('/editor/<username>/changelog', methods=['GET'])
def editor_changelog(username):
    editor = api.get_editor(username)
    changelog_entries = api.get_editor_changelog(username)
    return render_template('editor_changelog.html', editor=editor,
        changelog_entries=changelog_entries)


### Static Routes ###########################################################

@app.errorhandler(404)
def page_not_found(e):
    return render_template('404.html'), 404

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
