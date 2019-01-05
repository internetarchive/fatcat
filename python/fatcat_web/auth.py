
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify, session, flash
from fatcat_web import login_manager, api, priv_api, Config
from flask_login import logout_user, login_user, UserMixin
import pymacaroons
import fatcat_client

def handle_logout():
    logout_user()
    for k in ('editor', 'token'):
        if k in session:
            session.pop(k)

def handle_token_login(token):
    try:
        m = pymacaroons.Macaroon.deserialize(token)
    except pymacaroons.exceptions.MacaroonDeserializationException:
        # TODO: what kind of Exceptions?
        return abort(400)
    # extract editor_id
    editor_id = None
    for caveat in m.first_party_caveats():
        caveat = caveat.caveat_id
        if caveat.startswith(b"editor_id = "):
            editor_id = caveat[12:].decode('utf-8')
    if not editor_id:
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
        else:
            preferred_username = user_info['sub']

        params = fatcat_client.AuthOidc(remote.name, user_info['sub'], iss, user_info['preferred_username'])
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

    raise some_error


@login_manager.user_loader
def load_user(editor_id):
    # looks for extra info in session, and updates the user object with that.
    # If session isn't loaded/valid, should return None
    if not 'editor' in session or not 'api_token' in session:
        return None
    editor = session['editor']
    token = session['api_token']
    user = UserMixin()
    user.id = editor_id
    user.editor_id = editor_id
    user.username = editor['username']
    user.token = token
    return user
