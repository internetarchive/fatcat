
import os
import json
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify, session, flash
from flask_login import login_required

from fatcat_client.rest import ApiException
from fatcat_tools.transforms import *
from fatcat_web import app, api, auth_api, priv_api
from fatcat_web.auth import handle_token_login, handle_logout, load_user, handle_ia_xauth
from fatcat_web.cors import crossdomain
from fatcat_web.search import *


### Views ###################################################################

@app.route('/container/<ident>/history', methods=['GET'])
def container_history(ident):
    try:
        entity = api.get_container(ident)
        history = api.get_container_history(ident)
    except ApiException as ae:
        abort(ae.status)
    #print(history)
    return render_template('entity_history.html',
        page_title=entity.name,
        entity_type="container",
        entity=entity,
        history=history)

@app.route('/container/<ident>/edit', methods=['GET'])
def container_edit_view(ident):
    try:
        entity = api.get_container(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

#@app.route('/container/<ident>/edit', methods=['POST'])
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
@login_required
def container_create_view():
    return render_template('container_create.html')

@app.route('/container/create', methods=['POST'])
@login_required
def container_create():
    raise NotImplementedError
    params = dict()
    for k in request.form:
        if k.startswith('container_'):
            params[k[10:]] = request.form[k]
    container = None
    #edit = api.create_container(container, params=params)
    #return redirect("/container/{}".format(edit.ident))

@app.route('/container/lookup', methods=['GET'])
def container_lookup():
    extid = None
    for key in ('issnl', 'wikidata_qid'):
        if request.args.get(key):
            extid = key
            break
    if extid is None:
        abort(400)
    try:
        resp = api.lookup_container(**{extid: request.args.get(extid)})
    except ApiException as ae:
        abort(ae.status)
    return redirect('/container/{}'.format(resp.ident))

@app.route('/container/<ident>', methods=['GET'])
def container_view(ident):
    try:
        entity = api.get_container(ident)
    except ApiException as ae:
        abort(ae.status)

    if entity.issnl:
        try:
            stats = get_elastic_container_stats(entity.issnl)
        except Exception as e:
            stats = None
            print(e)
    else:
        stats = None

    if entity.state == "redirect":
        return redirect('/container/{}'.format(entity.redirect))
    if entity.state == "deleted":
        return render_template('deleted_entity.html', entity=entity)
    if entity.state == "active":
        entity.es = container_to_elasticsearch(entity, force_bool=False)
    return render_template('container_view.html',
        container=entity, container_stats=stats)

@app.route('/creator/<ident>/history', methods=['GET'])
def creator_history(ident):
    try:
        entity = api.get_creator(ident)
        history = api.get_creator_history(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_history.html',
        page_title=entity.display_name,
        entity_type="creator",
        entity=entity,
        history=history)

@app.route('/container/issnl/<issnl>/stats.json', methods=['GET', 'OPTIONS'])
@crossdomain(origin='*',headers=['access-control-allow-origin','Content-Type'])
def container_issnl_stats(issnl):
    try:
        stats = get_elastic_container_stats(issnl)
    except Exception as ae:
        print(ae)
        abort(503)
    return jsonify(stats)

@app.route('/creator/<ident>/edit', methods=['GET'])
def creator_edit_view(ident):
    try:
        entity = api.get_creator(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

@app.route('/creator/lookup', methods=['GET'])
def creator_lookup():
    for key in ('orcid', 'wikidata_qid'):
        if request.args.get(key):
            extid = key
            break
    if extid is None:
        abort(400)
    try:
        resp = api.lookup_creator(**{extid: request.args.get(extid)})
    except ApiException as ae:
        abort(ae.status)
    return redirect('/creator/{}'.format(resp.ident))

@app.route('/creator/<ident>', methods=['GET'])
def creator_view(ident):
    try:
        entity = api.get_creator(ident)
        releases = api.get_creator_releases(ident)
    except ApiException as ae:
        abort(ae.status)
    if entity.state == "redirect":
        return redirect('/creator/{}'.format(entity.redirect))
    if entity.state == "deleted":
        return render_template('deleted_entity.html', entity=entity)
    return render_template('creator_view.html', creator=entity, releases=releases)

@app.route('/file/<ident>/history', methods=['GET'])
def file_history(ident):
    try:
        entity = api.get_file(ident)
        history = api.get_file_history(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_history.html',
        page_title=None,
        entity_type="file",
        entity=entity,
        history=history)

@app.route('/file/<ident>/edit', methods=['GET'])
def file_edit_view(ident):
    try:
        entity = api.get_file(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

@app.route('/file/lookup', methods=['GET'])
def file_lookup():
    for key in ('md5', 'sha1', 'sha256'):
        if request.args.get(key):
            extid = key
            break
    if extid is None:
        abort(400)
    try:
        resp = api.lookup_file(**{extid: request.args.get(extid)})
    except ApiException as ae:
        abort(ae.status)
    return redirect('/file/{}'.format(resp.ident))

@app.route('/file/<ident>', methods=['GET'])
def file_view(ident):
    try:
        entity = api.get_file(ident)
    except ApiException as ae:
        abort(ae.status)
    if entity.state == "redirect":
        return redirect('/file/{}'.format(entity.redirect))
    elif entity.state == "deleted":
        return render_template('deleted_entity.html', entity=entity)
    else:
        try:
            entity.releases = []
            for r in entity.release_ids:
                entity.releases.append(api.get_release(r))
        except ApiException as ae:
            abort(ae.status)
    return render_template('file_view.html', file=entity)

@app.route('/fileset/<ident>/history', methods=['GET'])
def fileset_history(ident):
    try:
        entity = api.get_fileset(ident)
        history = api.get_fileset_history(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_history.html',
        page_title=None,
        entity_type="fileset",
        entity=entity,
        history=history)

@app.route('/fileset/<ident>/edit', methods=['GET'])
def fileset_edit_view(ident):
    try:
        entity = api.get_fileset(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

@app.route('/fileset/lookup', methods=['GET'])
def fileset_lookup():
    raise NotImplementedError

@app.route('/fileset/<ident>', methods=['GET'])
def fileset_view(ident):
    try:
        entity = api.get_fileset(ident)
    except ApiException as ae:
        abort(ae.status)
    if entity.state == "redirect":
        return redirect('/fileset/{}'.format(entity.redirect))
    elif entity.state == "deleted":
        return render_template('deleted_entity.html', entity=entity)
    else:
        try:
            entity.releases = []
            for r in entity.release_ids:
                entity.releases.append(api.get_release(r))
        except ApiException as ae:
            abort(ae.status)
        entity.total_size = sum([f.size for f in entity.manifest])
    return render_template('fileset_view.html', fileset=entity)

@app.route('/webcapture/<ident>/history', methods=['GET'])
def webcapture_history(ident):
    try:
        entity = api.get_webcapture(ident)
        history = api.get_webcapture_history(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_history.html',
        page_title=None,
        entity_type="webcapture",
        entity=entity,
        history=history)

@app.route('/webcapture/<ident>/edit', methods=['GET'])
def webcapture_edit_view(ident):
    try:
        entity = api.get_webcapture(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

@app.route('/webcapture/lookup', methods=['GET'])
def webcapture_lookup():
    raise NotImplementedError

@app.route('/webcapture/<ident>', methods=['GET'])
def webcapture_view(ident):
    try:
        entity = api.get_webcapture(ident)
    except ApiException as ae:
        abort(ae.status)
    if entity.state == "redirect":
        return redirect('/webcapture/{}'.format(entity.redirect))
    elif entity.state == "deleted":
        return render_template('deleted_entity.html', entity=entity)
    else:
        try:
            entity.releases = []
            for r in entity.release_ids:
                entity.releases.append(api.get_release(r))
        except ApiException as ae:
            abort(ae.status)
    return render_template('webcapture_view.html', webcapture=entity)

@app.route('/release/lookup', methods=['GET'])
def release_lookup():
    for key in ('doi', 'wikidata_qid', 'pmid', 'pmcid', 'isbn13', 'core_id'):
        if request.args.get(key):
            extid = key
            break
    if extid is None:
        abort(400)
    try:
        resp = api.lookup_release(**{extid: request.args.get(extid)})
    except ApiException as ae:
        abort(ae.status)
    return redirect('/release/{}'.format(resp.ident))

@app.route('/release/create', methods=['GET'])
@login_required
def release_create_view():
    return render_template('release_create.html')

@app.route('/release/create', methods=['POST'])
@login_required
def release_create():
    raise NotImplementedError
    params = dict()
    for k in request.form:
        if k.startswith('release_'):
            params[k[10:]] = request.form[k]
    release = None
    #edit = api.create_release(release, params=params)
    #return redirect("/release/{}".format(edit.ident))

@app.route('/release/<ident>/history', methods=['GET'])
def release_history(ident):
    try:
        entity = api.get_release(ident)
        history = api.get_release_history(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_history.html',
        page_title=entity.title,
        entity_type="release",
        entity=entity,
        history=history)

@app.route('/release/<ident>/edit', methods=['GET'])
def release_edit_view(ident):
    try:
        entity = api.get_release(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

@app.route('/release/<ident>', methods=['GET'])
def release_view(ident):
    try:
        entity = api.get_release(ident, expand="container,files,filesets,webcaptures")
        files = entity.files
        container = entity.container
    except ApiException as ae:
        abort(ae.status)
    if entity.state == "redirect":
        return redirect('/release/{}'.format(entity.redirect))
    if entity.state == "deleted":
        return render_template('deleted_entity.html', entity=entity)
    if entity.container and entity.container.state == "active":
        entity.container.es = container_to_elasticsearch(entity.container, force_bool=False)
    if entity.state == "active":
        entity.es = release_to_elasticsearch(entity, force_bool=False)
    authors = [c for c in entity.contribs if c.role in ('author', None)]
    authors = sorted(authors, key=lambda c: c.index)
    for fe in files:
        # crudely filter out exact duplicates
        kept = []
        for u in fe.urls:
            if not u in kept:
                kept.append(u)
        fe.urls = [u for u in kept if not '/web/None/' in u.url]
    return render_template('release_view.html', release=entity,
        authors=authors, files=files, container=container)

@app.route('/work/create', methods=['GET'])
def work_create_view():
    return abort(404)

@app.route('/work/<ident>/history', methods=['GET'])
def work_history(ident):
    try:
        entity = api.get_work(ident)
        history = api.get_work_history(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_history.html',
        page_title=None,
        entity_type="work",
        entity=entity,
        history=history)

@app.route('/work/<ident>/edit', methods=['GET'])
def work_edit_view(ident):
    try:
        entity = api.get_work(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

@app.route('/work/<ident>', methods=['GET'])
def work_view(ident):
    try:
        entity = api.get_work(ident)
        releases = api.get_work_releases(ident)
    except ApiException as ae:
        abort(ae.status)
    if entity.state == "redirect":
        return redirect('/work/{}'.format(entity.redirect))
    if entity.state == "deleted":
        return render_template('deleted_entity.html', entity=entity)
    return render_template('work_view.html', work=entity, releases=releases)

@app.route('/editgroup/<ident>', methods=['GET'])
def editgroup_view(ident):
    try:
        entity = api.get_editgroup(str(ident))
        entity.editor = api.get_editor(entity.editor_id)
    except ApiException as ae:
        abort(ae.status)
    return render_template('editgroup_view.html', editgroup=entity)

@app.route('/editor/<ident>', methods=['GET'])
def editor_view(ident):
    try:
        entity = api.get_editor(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('editor_view.html', editor=entity)

@app.route('/editor/<ident>/editgroups', methods=['GET'])
def editor_editgroups(ident):
    try:
        editor = api.get_editor(ident)
        editgroups = api.get_editor_editgroups(ident, limit=50)
    except ApiException as ae:
        abort(ae.status)
    return render_template('editor_editgroups.html', editor=editor,
        editgroups=editgroups)

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
        entry.editgroup.editor = api.get_editor(entry.editgroup.editor_id)
    except ApiException as ae:
        abort(ae.status)
    return render_template('changelog_view.html', entry=entry, editgroup=entry.editgroup)

### Search ##################################################################

@app.route('/release/search', methods=['GET', 'POST'])
def release_search():

    query = request.args.get('q')
    fulltext_only = bool(request.args.get('fulltext_only'))

    issnl = request.args.get('container_issnl')
    if issnl and query:
        query += ' container_issnl:"{}"'.format(issnl)

    if 'q' in request.args.keys():
        # always do files for HTML
        found = do_release_search(query, fulltext_only=fulltext_only)
        return render_template('release_search.html', found=found, query=query, fulltext_only=fulltext_only)
    else:
        return render_template('release_search.html', query=query, fulltext_only=fulltext_only)

@app.route('/container/search', methods=['GET', 'POST'])
def container_search():

    query = request.args.get('q')

    if 'q' in request.args.keys():
        # always do files for HTML
        found = do_container_search(query)
        return render_template('container_search.html', found=found, query=query)
    else:
        return render_template('container_search.html', query=query)

def get_changelog_stats():
    stats = {}
    latest_changelog = api.get_changelog(limit=1)[0]
    stats['changelog'] = {"latest": {
        "index": latest_changelog.index,
        "timestamp": latest_changelog.timestamp.isoformat(),
    }}
    return stats

@app.route('/stats.json', methods=['GET', 'OPTIONS'])
@crossdomain(origin='*',headers=['access-control-allow-origin','Content-Type'])
def stats_json():
    try:
        stats = get_elastic_entity_stats()
        stats.update(get_changelog_stats())
    except Exception as ae:
        print(ae)
        abort(503)
    return jsonify(stats)

@app.route('/stats', methods=['GET'])
def stats_page():
    try:
        stats = get_elastic_entity_stats()
        stats.update(get_changelog_stats())
    except Exception as ae:
        print(ae)
        abort(503)
    return render_template('stats.html', stats=stats)


### Auth ####################################################################

@app.route('/auth/login')
def login():
    # show the user a list of login options
    if not priv_api:
        flash("This web interface not configured with credentials to actually allow login (other than via token)")
    return render_template('auth_login.html')

@app.route('/auth/ia/login', methods=['GET', 'POST'])
def ia_xauth_login():
    if 'email' in request.form:
        # if a login attempt...
        return handle_ia_xauth(request.form.get('email'), request.form.get('password'))
    # else show form
    return render_template('auth_ia_login.html')

@app.route('/auth/token_login', methods=['GET', 'POST'])
def token_login():
    # show the user a list of login options
    if 'token' in request.args:
        return handle_token_login(request.args.get('token'))
    if 'token' in request.form:
        return handle_token_login(request.form.get('token'))
    return render_template('auth_token_login.html')

@app.route('/auth/change_username', methods=['POST'])
@login_required
def change_username():
    # show the user a list of login options
    if not 'username' in request.form:
        abort(400)
    # on behalf of user...
    user_api = auth_api(session['api_token'])
    editor = user_api.get_editor(session['editor']['editor_id'])
    editor.username = request.form['username']
    editor = user_api.update_editor(editor.editor_id, editor)
    # update our session
    session['editor'] = editor.to_dict()
    load_user(editor.editor_id)
    flash("Username updated successfully")
    return redirect('/auth/account')

@app.route('/auth/logout')
def logout():
    handle_logout()
    return render_template('auth_logout.html')

@app.route('/auth/account')
@login_required
def auth_account():
    editor = api.get_editor(session['editor']['editor_id'])
    session['editor'] = editor.to_dict()
    load_user(editor.editor_id)
    return render_template('auth_account.html')


### Static Routes ###########################################################

@app.errorhandler(404)
def page_not_found(e):
    return render_template('404.html'), 404

@app.errorhandler(401)
@app.errorhandler(403)
def page_not_authorized(e):
    return render_template('403.html'), 403

@app.errorhandler(400)
def page_bad_request(e):
    return render_template('400.html'), 400

@app.errorhandler(409)
def page_edit_conflict(e):
    return render_template('409.html'), 409

@app.errorhandler(500)
def page_server_error(e):
    return render_template('500.html'), 500

@app.errorhandler(502)
@app.errorhandler(503)
@app.errorhandler(504)
def page_server_down(e):
    return render_template('503.html'), 503

@app.route('/', methods=['GET'])
def page_home():
    return render_template('home.html')

@app.route('/about', methods=['GET'])
def page_about():
    return render_template('about.html')

@app.route('/rfc', methods=['GET'])
def page_rfc():
    return render_template('rfc.html')

@app.route('/search', methods=['GET'])
def page_search_redirect():
    return redirect("/release/search")

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

@app.route('/health', methods=['GET', 'OPTIONS'])
@crossdomain(origin='*',headers=['access-control-allow-origin','Content-Type'])
def health():
    return jsonify({'ok': True})
