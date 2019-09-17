
from collections import namedtuple
import requests
import pymacaroons
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify, session, flash
from flask_login import logout_user, login_user, UserMixin
from fatcat_web import login_manager, app, api, priv_api, Config
import fatcat_openapi_client

def handle_logout():
    logout_user()
    for k in ('editor', 'api_token'):
        if k in session:
            session.pop(k)
    session.clear()

def handle_token_login(token):
    try:
        m = pymacaroons.Macaroon.deserialize(token)
    except pymacaroons.exceptions.MacaroonDeserializationException:
        # TODO: what kind of Exceptions?
        app.log.warning("auth fail: MacaroonDeserializationException")
        return abort(400)
    # extract editor_id
    editor_id = None
    for caveat in m.first_party_caveats():
        caveat = caveat.caveat_id
        if caveat.startswith(b"editor_id = "):
            editor_id = caveat[12:].decode('utf-8')
    if not editor_id:
        app.log.warning("auth fail: editor_id missing in macaroon")
        abort(400)
    # fetch editor info
    editor = api.get_editor(editor_id)
    session.permanent = True
    session['api_token'] = token
    session['editor'] = editor.to_dict()
    login_user(load_user(editor.editor_id))
    return redirect("/auth/account")

# This will need to login/signup via fatcatd API, then set token in session
def handle_oauth(remote, token, user_info):
    if user_info:
        # fetch api login/signup using user_info
        # ISS is basically the API url (though more formal in OIDC)
        # SUB is the stable internal identifier for the user (not usually the username itself)
        # TODO: should have the real sub here
        # TODO: would be nicer to pass preferred_username for account creation
        iss = remote.OAUTH_CONFIG['api_base_url']

        # we reuse 'preferred_username' for account name auto-creation (but
        # don't store it otherwise in the backend, at least currently). But i'm
        # not sure all loginpass backends will set it
        if user_info.get('preferred_username'):
            preferred_username = user_info['preferred_username']
        elif 'orcid.org' in iss:
            # as a special case, prefix ORCiD identifier so it can be used as a
            # username. If we instead used the human name, we could have
            # collisions. Not a great user experience either way.
            preferred_username = 'i' + user_info['sub'].replace('-', '')
        else:
            preferred_username = user_info['sub']

        params = fatcat_openapi_client.AuthOidc(remote.name, user_info['sub'], iss, preferred_username)
        # this call requires admin privs
        (resp, http_status, http_headers) = priv_api.auth_oidc_with_http_info(params)
        editor = resp.editor
        api_token = resp.token

        if http_status == 201:
            flash("Welcome to Fatcat! An account has been created for you with a temporary username; you may wish to change it under account settings")
            flash("You must use the same mechanism ({}) to login in the future".format(remote.name))
        else:
            flash("Welcome back!")

        # write token and username to session
        session.permanent = True
        session['api_token'] = api_token
        session['editor'] = editor.to_dict()

        # call login_user(load_user(editor_id))
        login_user(load_user(editor.editor_id))
        return redirect("/auth/account")

    # XXX: what should this actually be?
    raise Exception("didn't receive OAuth user_info")

def handle_ia_xauth(email, password):
    resp = requests.post(Config.IA_XAUTH_URI,
        params={'op': 'authenticate'},
        json={
            'version': '1',
            'email': email,
            'password': password,
            'access': Config.IA_XAUTH_CLIENT_ID,
            'secret': Config.IA_XAUTH_CLIENT_SECRET,
        })
    if resp.status_code == 401 or (not resp.json().get('success')):
        try:
            flash("Internet Archive email/password didn't match: {}".format(resp.json()['values']['reason']))
        except:
            app.log.warning("IA XAuth fail: {}".format(resp.content))
        return render_template('auth_ia_login.html', email=email), resp.status_code
    elif resp.status_code != 200:
        flash("Internet Archive login failed (internal error?)")
        app.log.warning("IA XAuth fail: {}".format(resp.content))
        return render_template('auth_ia_login.html', email=email), resp.status_code

    # Successful login; now fetch info...
    resp = requests.post(Config.IA_XAUTH_URI,
        params={'op': 'info'},
        json={
            'version': '1',
            'email': email,
            'access': Config.IA_XAUTH_CLIENT_ID,
            'secret': Config.IA_XAUTH_CLIENT_SECRET,
        })
    if resp.status_code != 200:
        flash("Internet Archive login failed (internal error?)")
        app.log.warning("IA XAuth fail: {}".format(resp.content))
        return render_template('auth_ia_login.html', email=email), resp.status_code
    ia_info = resp.json()['values']

    # and pass off "as if" we did OAuth successfully
    FakeOAuthRemote = namedtuple('FakeOAuthRemote', ['name', 'OAUTH_CONFIG'])
    remote = FakeOAuthRemote(name='archive', OAUTH_CONFIG={'api_base_url': Config.IA_XAUTH_URI})
    oauth_info = {
        'preferred_username': ia_info['itemname'],
        'iss': Config.IA_XAUTH_URI,
        'sub': ia_info['itemname'],
    }
    return handle_oauth(remote, None, oauth_info)

def handle_wmoauth(username):
    # pass off "as if" we did OAuth successfully
    FakeOAuthRemote = namedtuple('FakeOAuthRemote', ['name', 'OAUTH_CONFIG'])
    remote = FakeOAuthRemote(name='wikipedia', OAUTH_CONFIG={'api_base_url': "https://www.mediawiki.org/w"})
    oauth_info = {
        'preferred_username': username,
        'iss': "https://www.mediawiki.org/w",
        'sub': username,
    }
    return handle_oauth(remote, None, oauth_info)

@login_manager.user_loader
def load_user(editor_id):
    # looks for extra info in session, and updates the user object with that.
    # If session isn't loaded/valid, should return None
    if (not session.get('editor')) or (not session.get('api_token')):
        return None
    editor = session['editor']
    token = session['api_token']
    user = UserMixin()
    user.id = editor_id
    user.editor_id = editor_id
    user.username = editor['username']
    user.is_admin = editor['is_admin']
    user.token = token
    return user
