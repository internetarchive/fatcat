
import os
import json
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify, session, flash, Response
from flask_login import login_required
from flask_wtf.csrf import CSRFError

from fatcat_client import Editgroup, EditgroupAnnotation
from fatcat_client.rest import ApiException
from fatcat_tools.transforms import *
from fatcat_web import app, api, auth_api, priv_api, mwoauth
from fatcat_web.auth import handle_token_login, handle_logout, load_user, handle_ia_xauth, handle_wmoauth
from fatcat_web.cors import crossdomain
from fatcat_web.search import *
from fatcat_web.hacks import strip_extlink_xml, wayback_suffix


### Views ###################################################################

@app.route('/container/<ident>/history', methods=['GET'])
def container_history(ident):
    try:
        entity = api.get_container(ident)
        history = api.get_container_history(ident)
    except ApiException as ae:
        app.log.info(ae)
        abort(ae.status)
    #print(history)
    return render_template('entity_history.html',
        page_title=entity.name,
        entity_type="container",
        entity=entity,
        history=history)

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
            app.log.error(e)
    else:
        stats = None

    if entity.state == "redirect":
        return redirect('/container/{}'.format(entity.redirect))
    if entity.state == "deleted":
        return render_template('deleted_entity.html', entity=entity, entity_type="container")
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

@app.route('/creator/lookup', methods=['GET'])
def creator_lookup():
    extid = None
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
        return render_template('deleted_entity.html', entity=entity, entity_type="creator")
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

@app.route('/file/lookup', methods=['GET'])
def file_lookup():
    extid = None
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
        entity = api.get_file(ident, expand="releases")
    except ApiException as ae:
        abort(ae.status)
    if entity.state == "redirect":
        return redirect('/file/{}'.format(entity.redirect))
    elif entity.state == "deleted":
        return render_template('deleted_entity.html', entity=entity, entity_type="file")
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

@app.route('/fileset/lookup', methods=['GET'])
def fileset_lookup():
    abort(404)

@app.route('/fileset/<ident>', methods=['GET'])
def fileset_view(ident):
    try:
        entity = api.get_fileset(ident, expand="releases")
    except ApiException as ae:
        abort(ae.status)
    if entity.state == "redirect":
        return redirect('/fileset/{}'.format(entity.redirect))
    elif entity.state == "deleted":
        return render_template('deleted_entity.html', entity=entity, entity_type="fileset")
    else:
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

@app.route('/webcapture/lookup', methods=['GET'])
def webcapture_lookup():
    abort(404)

@app.route('/webcapture/<ident>', methods=['GET'])
def webcapture_view(ident):
    try:
        entity = api.get_webcapture(ident, expand="releases")
    except ApiException as ae:
        abort(ae.status)
    if entity.state == "redirect":
        return redirect('/webcapture/{}'.format(entity.redirect))
    elif entity.state == "deleted":
        return render_template('deleted_entity.html', entity=entity, entity_type="webcapture")
    entity.wayback_suffix = wayback_suffix(entity)
    #print("SUFFIX: {}".format(entity.wayback_suffix))
    return render_template('webcapture_view.html', webcapture=entity)

@app.route('/release/lookup', methods=['GET'])
def release_lookup():
    extid = None
    for key in ('doi', 'wikidata_qid', 'pmid', 'pmcid', 'isbn13', 'jstor',
                'arxiv', 'core', 'ark', 'mag'):
        if request.args.get(key):
            extid = key
            break
    if extid is None:
        abort(400)
    try:
        resp = api.lookup_release(**{extid: request.args.get(extid)})
    except ApiException as ae:
        app.log.info(ae)
        abort(ae.status)
    return redirect('/release/{}'.format(resp.ident))

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

@app.route('/release/<ident>', methods=['GET'])
def release_view(ident):
    try:
        entity = api.get_release(ident, expand="container,files,filesets,webcaptures")
    except ApiException as ae:
        abort(ae.status)
    if entity.state == "redirect":
        return redirect('/release/{}'.format(entity.redirect))
    if entity.state == "deleted":
        return render_template('deleted_entity.html', entity=entity, entity_type="release")
    if entity.container and entity.container.state == "active":
        entity.container.es = container_to_elasticsearch(entity.container, force_bool=False)
    if entity.state == "active":
        entity.es = release_to_elasticsearch(entity, force_bool=False)
    for fs in entity.filesets:
        fs.total_size = sum([f.size for f in fs.manifest])
    for wc in entity.webcaptures:
        wc.wayback_suffix = wayback_suffix(wc)
    for ref in entity.refs:
        # this is a UI hack to get rid of XML crud in unstructured refs like:
        # LOCKSS (2014) Available: <ext-link
        # xmlns:xlink="http://www.w3.org/1999/xlink" ext-link-type="uri"
        # xlink:href="http://lockss.org/"
        # xlink:type="simple">http://lockss.org/</ext-link>. Accessed: 2014
        # November 1.
        if ref.extra and ref.extra.get('unstructured'):
            ref.extra['unstructured'] = strip_extlink_xml(ref.extra['unstructured'])
    # author list to display; ensure it's sorted by index (any othors with
    # index=None go to end of list)
    authors = [c for c in entity.contribs if c.role in ('author', None)]
    authors = sorted(authors, key=lambda c: (c.index == None and 99999999) or c.index)
    # hack to show plain text instead of latex abstracts
    if entity.abstracts:
        if 'latex' in entity.abstracts[0].mimetype:
            entity.abstracts.reverse()
    return render_template('release_view.html', release=entity,
        authors=authors, container=entity.container)

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
        return render_template('deleted_entity.html', entity=entity, entity_type="work")
    return render_template('work_view.html', work=entity, releases=releases)

@app.route('/work/lookup', methods=['GET'])
def work_lookup():
    abort(404)

@app.route('/editgroup/<ident>', methods=['GET'])
def editgroup_view(ident):
    try:
        eg = api.get_editgroup(str(ident))
        eg.editor = api.get_editor(eg.editor_id)
        eg.annotations = api.get_editgroup_annotations(eg.editgroup_id, expand="editors")
    except ApiException as ae:
        abort(ae.status)
    # TODO: idomatic check for login?
    auth_to = dict(
        submit=False,
        accept=False,
        annotate=False,
    )
    if session.get('editor'):
        user = load_user(session['editor']['editor_id'])
        auth_to['annotate'] = True
        if user.is_admin or user.editor_id == eg.editor_id:
            auth_to['submit'] = True
        if user.is_admin:
            auth_to['accept'] = True
    return render_template('editgroup_view.html', editgroup=eg,
        auth_to=auth_to)

@app.route('/editgroup/<ident>/annotation', methods=['POST'])
@login_required
def editgroup_create_annotation(ident):
    if not app.testing:
        app.csrf.protect()
    comment_markdown = request.form.get('comment_markdown')
    if not comment_markdown:
        app.log.info("empty comment field")
        abort(400)
    # on behalf of user...
    user_api = auth_api(session['api_token'])
    try:
        eg = user_api.get_editgroup(str(ident))
        if eg.changelog_index:
            flash("Editgroup already accepted")
            abort(400)
        ega = EditgroupAnnotation(
            comment_markdown=comment_markdown,
            extra=None,
        )
        user_api.create_editgroup_annotation(eg.editgroup_id, ega)
    except ApiException as ae:
        app.log.info(ae)
        abort(ae.status)
    return redirect('/editgroup/{}'.format(ident))

@app.route('/editgroup/<ident>/accept', methods=['POST'])
@login_required
def editgroup_accept(ident):
    if not app.testing:
        app.csrf.protect()
    # on behalf of user...
    user_api = auth_api(session['api_token'])
    try:
        eg = user_api.get_editgroup(str(ident))
        if eg.changelog_index:
            flash("Editgroup already accepted")
            abort(400)
        user_api.accept_editgroup(str(ident))
    except ApiException as ae:
        app.log.info(ae)
        abort(ae.status)
    # clear active_editgroup_id cookie; this doesn't cover all cases
    if eg.editgroup_id == session.get('active_editgroup_id'):
        session.pop('active_editgroup_id')
    return redirect('/editgroup/{}'.format(ident))

@app.route('/editgroup/<ident>/unsubmit', methods=['POST'])
@login_required
def editgroup_unsubmit(ident):
    if not app.testing:
        app.csrf.protect()
    # on behalf of user...
    user_api = auth_api(session['api_token'])
    try:
        eg = user_api.get_editgroup(str(ident))
        if eg.changelog_index:
            flash("Editgroup already accepted")
            abort(400)
        user_api.update_editgroup(eg.editgroup_id, eg, submit=False)
    except ApiException as ae:
        app.log.info(ae)
        abort(ae.status)
    return redirect('/editgroup/{}'.format(ident))

@app.route('/editgroup/<ident>/submit', methods=['POST'])
@login_required
def editgroup_submit(ident):
    if not app.testing:
        app.csrf.protect()
    # on behalf of user...
    print("submitting...")
    user_api = auth_api(session['api_token'])
    try:
        eg = user_api.get_editgroup(str(ident))
        if eg.changelog_index:
            flash("Editgroup already accepted")
            abort(400)
        user_api.update_editgroup(eg.editgroup_id, eg, submit=True)
    except ApiException as ae:
        print(ae)
        app.log.info(ae)
        abort(ae.status)
    return redirect('/editgroup/{}'.format(ident))

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
        # cheaper than API-side expand?
        for eg in editgroups:
            eg.editor = editor
    except ApiException as ae:
        abort(ae.status)
    return render_template('editor_editgroups.html', editor=editor,
        editgroups=editgroups)

@app.route('/editor/<ident>/annotations', methods=['GET'])
def editor_annotations(ident):
    try:
        editor = api.get_editor(ident)
        annotations = api.get_editor_annotations(ident, limit=50)
    except ApiException as ae:
        abort(ae.status)
    return render_template('editor_annotations.html', editor=editor,
        annotations=annotations)

@app.route('/changelog', methods=['GET'])
def changelog_view():
    try:
        #limit = int(request.args.get('limit', 10))
        entries = api.get_changelog() # TODO: expand="editors"
    except ApiException as ae:
        abort(ae.status)
    return render_template('changelog.html', entries=entries)

@app.route('/changelog/<int:index>', methods=['GET'])
def changelog_entry_view(index):
    try:
        entry = api.get_changelog_entry(int(index))
        entry.editgroup.editor = api.get_editor(entry.editgroup.editor_id)
        entry.editgroup.annotations = \
            api.get_editgroup_annotations(entry.editgroup_id, expand="editors")
    except ApiException as ae:
        abort(ae.status)
    return render_template('changelog_view.html', entry=entry, editgroup=entry.editgroup)

@app.route('/reviewable', methods=['GET'])
def reviewable_view():
    try:
        #limit = int(request.args.get('limit', 10))
        entries = api.get_editgroups_reviewable(expand="editors")
    except ApiException as ae:
        abort(ae.status)
    return render_template('editgroup_reviewable.html', entries=entries)

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

@app.route('/stats', methods=['GET'])
def stats_page():
    try:
        stats = get_elastic_entity_stats()
        stats.update(get_changelog_stats())
    except Exception as ae:
        app.log.error(ae)
        abort(503)
    return render_template('stats.html', stats=stats)

### Pseudo-APIs #############################################################

@app.route('/stats.json', methods=['GET', 'OPTIONS'])
@crossdomain(origin='*',headers=['access-control-allow-origin','Content-Type'])
def stats_json():
    try:
        stats = get_elastic_entity_stats()
        stats.update(get_changelog_stats())
    except Exception as ae:
        app.log.error(ae)
        abort(503)
    return jsonify(stats)

@app.route('/container/issnl/<issnl>/stats.json', methods=['GET', 'OPTIONS'])
@crossdomain(origin='*',headers=['access-control-allow-origin','Content-Type'])
def container_issnl_stats(issnl):
    try:
        stats = get_elastic_container_stats(issnl)
    except Exception as ae:
        app.log.error(ae)
        abort(503)
    return jsonify(stats)

@app.route('/release/<ident>.bib', methods=['GET'])
def release_bibtex(ident):
    try:
        entity = api.get_release(ident)
    except ApiException as ae:
        abort(ae.status)
    csl = release_to_csl(entity)
    bibtex = citeproc_csl(csl, 'bibtex')
    return Response(bibtex, mimetype="text/plain")

@app.route('/release/<ident>/citeproc', methods=['GET'])
def release_citeproc(ident):
    style = request.args.get('style', 'harvard1')
    is_html = request.args.get('html', False)
    if is_html and is_html.lower() in ('yes', '1', 'true', 'y', 't'):
        is_html = True
    else:
        is_html = False

    try:
        entity = api.get_release(ident)
    except ApiException as ae:
        abort(ae.status)
    csl = release_to_csl(entity)
    cite = citeproc_csl(csl, style, is_html)
    if is_html:
        return Response(cite)
    elif style == "csl-json":
        return jsonify(json.loads(cite))
    else:
        return Response(cite, mimetype="text/plain")

@app.route('/health.json', methods=['GET', 'OPTIONS'])
@crossdomain(origin='*',headers=['access-control-allow-origin','Content-Type'])
def health_json():
    return jsonify({'ok': True})


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
    if not app.testing:
        app.csrf.protect()
    # show the user a list of login options
    if not 'username' in request.form:
        abort(400)
    # on behalf of user...
    user_api = auth_api(session['api_token'])
    try:
        editor = user_api.get_editor(session['editor']['editor_id'])
        editor.username = request.form['username']
        editor = user_api.update_editor(editor.editor_id, editor)
    except ApiException as ae:
        app.log.info(ae)
        abort(ae.status)
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
    # auth check on account page
    user_api = auth_api(session['api_token'])
    resp = user_api.auth_check()
    assert(resp.success)
    editor = user_api.get_editor(session['editor']['editor_id'])
    session['editor'] = editor.to_dict()
    load_user(editor.editor_id)
    return render_template('auth_account.html')

@app.route('/auth/wikipedia/auth')
def wp_oauth_rewrite():
    """
    This is a dirty hack to rewrite '/auth/wikipedia/auth' to '/auth/wikipedia/oauth-callback'
    """
    return redirect(b"/auth/wikipedia/oauth-callback?" + request.query_string, 307)

@app.route('/auth/wikipedia/finish-login')
def wp_oauth_finish_login():
    wp_username = mwoauth.get_current_user(cached=True)
    assert(wp_username)
    return handle_wmoauth(wp_username)


### Static Routes ###########################################################

@app.errorhandler(404)
def page_not_found(e):
    return render_template('404.html'), 404

@app.errorhandler(401)
@app.errorhandler(403)
def page_not_authorized(e):
    return render_template('403.html'), 403

@app.errorhandler(405)
def page_method_not_allowed(e):
    return render_template('405.html'), 405

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

@app.errorhandler(CSRFError)
def page_csrf_error(e):
    return render_template('csrf_error.html', reason=e.description), 400

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

