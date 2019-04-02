
import os
import json
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify, session, flash, Response
from flask_login import login_required

from fatcat_client import Editgroup
from fatcat_client.rest import ApiException
from fatcat_tools.transforms import *
from fatcat_web import app, api, auth_api, priv_api
from fatcat_web.auth import handle_token_login, handle_logout, load_user, handle_ia_xauth
from fatcat_web.cors import crossdomain
from fatcat_web.search import *
from fatcat_web.forms import *


### Views ###################################################################

@app.route('/container/create', methods=['GET', 'POST'])
@login_required
def container_create():
    form = ContainerEntityForm()
    if form.is_submitted():
        if form.validate_on_submit():
            # API on behalf of user
            user_api = auth_api(session['api_token'])
            if form.editgroup_id.data:
                # TODO: error handling
                eg = user_api.get_editgroup(form.editgroup_id.data)
            else:
                # if no editgroup, create one from description
                eg = user_api.create_editgroup(
                    Editgroup(description=form.editgroup_description.data or None))
                # set this session editgroup_id
                session['active_editgroup_id'] = eg.editgroup_id
                print(eg.editgroup_id) # XXX: debug
                flash('Started new editgroup <a href="/editgroup/{}">{}</a>' \
                    .format(eg.editgroup_id, eg.editgroup_id))
            # no merge or anything hard to do; just create the entity
            entity = form.to_entity()
            edit = user_api.create_container(entity, editgroup_id=eg.editgroup_id)
            # redirect to new entity
            return redirect('/container/{}'.format(edit.ident))
        elif form.errors:
            print("user form errors: {}".format(form.errors))
            print("didn't validate...")
    if not form.is_submitted():
        editgroup_id = session.get('active_editgroup_id', None)
        form.editgroup_id.data = editgroup_id
    return render_template('container_create.html',
        form=form, editgroup_id=editgroup_id)

@login_required
@app.route('/container/<ident>/edit', methods=['GET', 'POST'])
def container_edit(ident):
    # TODO: prev_rev interlock
    # TODO: factor out editgroup active/creation stuff
    try:
        entity = api.get_container(ident)
    except ApiException as ae:
        abort(ae.status)
    form = ContainerEntityForm()
    if form.is_submitted():
        if form.validate_on_submit():
            # API on behalf of user
            user_api = auth_api(session['api_token'])
            if form.editgroup_id.data:
                # TODO: error handling
                eg = user_api.get_editgroup(form.editgroup_id.data)
            else:
                # if no editgroup, create one from description
                eg = user_api.create_editgroup(
                    Editgroup(description=form.editgroup_description.data or None))
                # set this session editgroup_id
                session['active_editgroup_id'] = eg.editgroup_id
                print(eg.editgroup_id) # XXX: debug
                flash('Started new editgroup <a href="/editgroup/{}">{}</a>' \
                    .format(eg.editgroup_id, eg.editgroup_id))
            # all the tricky logic is in the update method
            form.update_entity(entity)
            edit = user_api.update_container(entity.ident, entity,
                editgroup_id=eg.editgroup_id)
            # redirect to entity revision
            # TODO: container_rev_view
            return redirect('/container/{}'.format(edit.ident))
        elif form.errors:
            print("user form errors (didn't validate): {}".format(form.errors))
    else:
        form = ContainerEntityForm.from_entity(entity)
    if not form.is_submitted():
        editgroup_id = session.get('active_editgroup_id', None)
        form.editgroup_id.data = editgroup_id
    return render_template('container_edit.html',
        form=form, editgroup_id=editgroup_id, entity=entity)

@app.route('/creator/<ident>/edit', methods=['GET'])
def creator_edit(ident):
    try:
        entity = api.get_creator(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

@app.route('/file/<ident>/edit', methods=['GET'])
def file_edit(ident):
    try:
        entity = api.get_file(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

@app.route('/fileset/<ident>/edit', methods=['GET'])
def fileset_edit(ident):
    try:
        entity = api.get_fileset(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

@app.route('/webcapture/<ident>/edit', methods=['GET'])
def webcapture_edit(ident):
    try:
        entity = api.get_webcapture(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

@app.route('/release/create', methods=['GET', 'POST'])
@login_required
def release_create():
    form = ReleaseEntityForm()
    if form.is_submitted():
        if form.validate_on_submit():
            # API on behalf of user
            user_api = auth_api(session['api_token'])
            if form.editgroup_id.data:
                # TODO: error handling
                eg = user_api.get_editgroup(form.editgroup_id.data)
            else:
                # if no editgroup, create one from description
                eg = user_api.create_editgroup(
                    Editgroup(description=form.editgroup_description.data or None))
                # set this session editgroup_id
                session['active_editgroup_id'] = eg.editgroup_id
                flash('Started new editgroup <a href="/editgroup/{}">{}</a>' \
                    .format(eg.editgroup_id, eg.editgroup_id))
            # no merge or anything hard to do; just create the entity
            entity = form.to_entity()
            edit = user_api.create_release(entity, editgroup_id=eg.editgroup_id)
            # redirect to new release
            return redirect('/release/{}'.format(edit.ident))
        elif form.errors:
            print("user form errors: {}".format(form.errors))
            print("didn't validate...")
    elif len(form.contribs) == 0:
        form.contribs.append_entry()
    if not form.is_submitted():
        editgroup_id = session.get('active_editgroup_id', None)
        form.editgroup_id.data = editgroup_id
    return render_template('release_create.html',
        form=form, editgroup_id=editgroup_id)

@login_required
@app.route('/release/<ident>/edit', methods=['GET', 'POST'])
def release_edit(ident):
    # TODO: prev_rev interlock
    # TODO: factor out editgroup active/creation stuff
    try:
        entity = api.get_release(ident)
    except ApiException as ae:
        abort(ae.status)
    form = ReleaseEntityForm()
    if form.is_submitted():
        if form.validate_on_submit():
            # API on behalf of user
            user_api = auth_api(session['api_token'])
            if form.editgroup_id.data:
                # TODO: error handling
                eg = user_api.get_editgroup(form.editgroup_id.data)
            else:
                # if no editgroup, create one from description
                eg = user_api.create_editgroup(
                    Editgroup(description=form.editgroup_description.data or None))
                # set this session editgroup_id
                session['active_editgroup_id'] = eg.editgroup_id
                print(eg.editgroup_id) # XXX: debug
                flash('Started new editgroup <a href="/editgroup/{}">{}</a>' \
                    .format(eg.editgroup_id, eg.editgroup_id))
            # all the tricky logic is in the update method
            form.update_entity(entity)
            edit = user_api.update_release(entity.ident, entity,
                editgroup_id=eg.editgroup_id)
            # redirect to entity revision
            # TODO: release_rev_view
            return redirect('/release/{}'.format(edit.ident))
        elif form.errors:
            print("user form errors (didn't validate): {}".format(form.errors))
    else:
        form = ReleaseEntityForm.from_entity(entity)
    if not form.is_submitted():
        editgroup_id = session.get('active_editgroup_id', None)
        form.editgroup_id.data = editgroup_id
    return render_template('release_edit.html',
        form=form, editgroup_id=editgroup_id, entity=entity)

@app.route('/work/create', methods=['GET'])
def work_create_view():
    return abort(404)

@app.route('/work/<ident>/edit', methods=['GET'])
def work_edit_view(ident):
    try:
        entity = api.get_work(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')
