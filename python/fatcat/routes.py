
import os
import json
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify, session
from fatcat import app, api
from fatcat_client.rest import ApiException
from fatcat.search import do_search


### Views ###################################################################

@app.route('/container/<uuid:ident>', methods=['GET'])
def container_view(ident):
    try:
        entity = api.get_container(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('container_view.html', container=entity)

@app.route('/container/<uuid:ident>/history', methods=['GET'])
def container_history(ident):
    try:
        entity = api.get_container(str(ident))
        history = api.get_container_history(str(ident))
    except ApiException as ae:
        abort(ae.status)
    print(history)
    return render_template('entity_history.html',
        page_title=entity.name,
        entity_type="container",
        entity=entity,
        history=history)

@app.route('/container/<uuid:ident>/edit', methods=['GET'])
def container_edit_view(ident):
    try:
        entity = api.get_container(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

#@app.route('/container/<uuid:ident>/edit', methods=['POST'])
#def container_edit(ident):
#    raise NotImplemented()
#    params = dict()
#    for k in request.form:
#        if k.startswith('container_'):
#            params[k[10:]] = request.form[k]
#    edit = api.update_container(params=params)
#    return redirect("/container/{}".format(edit.ident))
#    # else:
#    #return render_template('container_edit.html')

@app.route('/container/create', methods=['GET'])
def container_create_view():
    return render_template('container_create.html')

@app.route('/container/create', methods=['POST'])
def container_create():
    params = dict()
    for k in request.form:
        if k.startswith('container_'):
            params[k[10:]] = request.form[k]
    edit = api.create_container(params=params)
    return redirect("/container/{}".format(edit.ident))

@app.route('/container/lookup', methods=['GET'])
def container_lookup():
    issnl = request.args.get('issnl')
    if issnl is None:
        abort(400)
    try:
        resp = api.lookup_container(issnl)
    except ApiException as ae:
        abort(ae.status)
    return redirect('/container/{}'.format(resp.ident))

@app.route('/creator/<uuid:ident>', methods=['GET'])
def creator_view(ident):
    try:
        entity = api.get_creator(str(ident))
        releases = api.get_creator_releases(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('creator_view.html', creator=entity, releases=releases)

@app.route('/creator/<uuid:ident>/history', methods=['GET'])
def creator_history(ident):
    try:
        entity = api.get_creator(str(ident))
        history = api.get_creator_history(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_history.html',
        page_title=entity.display_name,
        entity_type="creator",
        entity=entity,
        history=history)

@app.route('/creator/<uuid:ident>/edit', methods=['GET'])
def creator_edit_view(ident):
    try:
        entity = api.get_creator(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

@app.route('/creator/lookup', methods=['GET'])
def creator_lookup():
    orcid = request.args.get('orcid')
    if orcid is None:
        abort(400)
    try:
        resp = api.lookup_creator(orcid)
    except ApiException as ae:
        abort(ae.status)
    return redirect('/creator/{}'.format(resp.ident))

@app.route('/file/<uuid:ident>', methods=['GET'])
def file_view(ident):
    try:
        entity = api.get_file(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('file_view.html', file=entity)

@app.route('/file/<uuid:ident>/history', methods=['GET'])
def file_history(ident):
    try:
        entity = api.get_file(str(ident))
        history = api.get_file_history(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_history.html',
        page_title=None,
        entity_type="file",
        entity=entity,
        history=history)

@app.route('/file/<uuid:ident>/edit', methods=['GET'])
def file_edit_view(ident):
    try:
        entity = api.get_file(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

@app.route('/file/lookup', methods=['GET'])
def file_lookup():
    sha1 = request.args.get('sha1')
    if sha1 is None:
        abort(400)
    try:
        resp = api.lookup_file(sha1)
    except ApiException as ae:
        abort(ae.status)
    return redirect('/file/{}'.format(resp.ident))

@app.route('/release/<uuid:ident>', methods=['GET'])
def release_view(ident):
    try:
        entity = api.get_release(str(ident))
        files = api.get_release_files(str(ident))
        container = None
        if entity.container_id is not None:
            container = api.get_container(entity.container_id)
    except ApiException as ae:
        abort(ae.status)
    authors = [c for c in entity.contribs if c.role in ('author', None)]
    authors = sorted(authors, key=lambda c: c.index)
    return render_template('release_view.html', release=entity,
        authors=authors, files=files, container=container)

@app.route('/release/lookup', methods=['GET'])
def release_lookup():
    doi = request.args.get('doi')
    if doi is None:
        abort(400)
    try:
        resp = api.lookup_release(doi)
    except ApiException as ae:
        abort(ae.status)
    return redirect('/release/{}'.format(resp.ident))

@app.route('/release/create', methods=['GET'])
def release_create_view():
    return render_template('release_create.html')

@app.route('/release/create', methods=['POST'])
def release_create():
    params = dict()
    for k in request.form:
        if k.startswith('release_'):
            params[k[10:]] = request.form[k]
    edit = api.create_release(params=params)
    return redirect("/release/{}".format(edit.ident))

@app.route('/release/<uuid:ident>/history', methods=['GET'])
def release_history(ident):
    try:
        entity = api.get_release(str(ident))
        history = api.get_release_history(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_history.html',
        page_title=entity.title,
        entity_type="release",
        entity=entity,
        history=history)

@app.route('/release/<uuid:ident>/edit', methods=['GET'])
def release_edit_view(ident):
    try:
        entity = api.get_release(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

@app.route('/work/<uuid:ident>', methods=['GET'])
def work_view(ident):
    try:
        entity = api.get_work(str(ident))
        releases = api.get_work_releases(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('work_view.html', work=entity, releases=releases)

@app.route('/work/<uuid:ident>/history', methods=['GET'])
def work_history(ident):
    try:
        entity = api.get_work(str(ident))
        history = api.get_work_history(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_history.html',
        page_title=None,
        entity_type="work",
        entity=entity,
        history=history)

@app.route('/work/<uuid:ident>/edit', methods=['GET'])
def work_edit_view(ident):
    try:
        entity = api.get_work(str(ident))
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

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

@app.route('/changelog', methods=['GET'])
def changelog_view():
    try:
        entries = api.get_changelog(limit=request.args.get('limit'))
    except ApiException as ae:
        abort(ae.status)
    return render_template('changelog.html', entries=entries)

@app.route('/changelog/<int:index>', methods=['GET'])
def changelog_entry_view(index):
    try:
        entry = api.get_changelog_entry(int(index))
    except ApiException as ae:
        abort(ae.status)
    return render_template('changelog_view.html', entry=entry, editgroup=entry.editgroup)

@app.route('/stats', methods=['GET'])
def stats_view():
    stats = api.get_stats()
    return render_template('stats.html', stats=stats.extra)

### Search ##################################################################

@app.route('/release/search', methods=['GET', 'POST'])
def search():

    limit = 20
    query = request.args.get('q')

    # Convert raw DOIs to DOI queries
    if query is not None:
        oldquery = query.split()
        for word in oldquery:
            if word.startswith("10.") and word.count("/") >= 1:
                query = query.replace(word, 'doi:"{}"'.format(word))

    # Convert "author:" query to "authors:"
    if query is not None:
        query = query.replace("author:", "authors:")

    if 'q' in request.args.keys():
        # always do files for HTML
        found = do_search(query, limit=limit)
        return render_template('release_search.html', found=found)
    else:
        return render_template('release_search.html')


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

@app.route('/static/fatcat.jpg', methods=['GET'])
def fatcat_photo():
    return send_from_directory(os.path.join(app.root_path, 'static'),
                               'fatcat.jpg',
                               mimetype='image/jpeg')

@app.route('/health', methods=['GET'])
def health():
    return jsonify({'ok': True})
