
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify, session
from fatcat_web import login_manager


# This will need to login/signup via fatcatd API, then set token in session
def handle_oauth(remote, token, user_info):
    print(remote)
    if token:
        print(remote.name, token)
    if user_info:
        # TODO: fetch api login/signup using user_info
        print(user_info)
        # TODO: write token and username to session
        # TODO: call login_user(load_user(editor_id))
        return redirect("/")
    raise some_error


@login_manager.user_loader
def load_user(editor_id):
    # NOTE: this should look for extra info in session, and update the user
    # object with that. If session isn't loaded/valid, should return None
    user = UserMixin()
    user.id = editor_id
    return user
