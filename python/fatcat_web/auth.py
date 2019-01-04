
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify, session, flash
from fatcat_web import login_manager, api, Config
from flask_login import logout_user, login_user, UserMixin
import pymacaroons
import fatcat_client

def auth_api(token):
    conf = fatcat_client.Configuration()
    conf.api_key["Authorization"] = token
    conf.api_key_prefix["Authorization"] = "Bearer"
    conf.host = Config.FATCAT_API_HOST
    return fatcat_client.DefaultApi(fatcat_client.ApiClient(conf))

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
    editor = api.get_editor(editor_id).to_dict()
    session['api_token'] = token
    session['editor'] = editor
    login_user(load_user(editor_id))
    return redirect("/auth/account")

# This will need to login/signup via fatcatd API, then set token in session
def handle_oauth(remote, token, user_info):
    print(remote)
    if token:
        print(remote.name, token)
    if user_info:
        print(user_info)
        print(user_info.iss)
        print(user_info.prefered_username)

        # fetch api login/signup using user_info
        params = AuthOidc(remote.name, user_info.sub, user_info.iss)
        resp = api.auth_oidc(params)
        editor = resp['editor']
        api_token = resp['token']

        # write token and username to session
        session['api_token'] = api_token
        session['editor'] = editor.editor_id

        # call login_user(load_user(editor_id))
        login_user(load_user(editor_id))
        return redirect("/")

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
