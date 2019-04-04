
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


### Helper Methods ##########################################################

def form_editgroup_get_or_create(api, edit_form):
    """
    This function expects a submitted, validated 
    """
    if edit_form.editgroup_id.data:
        try:
            eg = api.get_editgroup(edit_form.editgroup_id.data)
        except ApiException as ae:
            if ae.status == 404:
                edit_form.editgroup_id.errors.append("Editgroup does not exist")
                return None
            app.log.warning(ae)
            abort(ae.status)
        # TODO: check here that editgroup hasn't been merged already
    else:
        # if no editgroup, create one from description
        try:
            eg = api.create_editgroup(
                Editgroup(description=edit_form.editgroup_description.data or None))
        except ApiException as ae:
            app.log.warning(ae)
            abort(ae.status)
        # set this session editgroup_id
        session['active_editgroup_id'] = eg.editgroup_id
        flash('Started new editgroup <a href="/editgroup/{}">{}</a>' \
            .format(eg.editgroup_id, eg.editgroup_id))
    return eg

### Views ###################################################################

@app.route('/container/create', methods=['GET', 'POST'])
@login_required
def container_create():
    form = ContainerEntityForm()
    if form.is_submitted():
        if form.validate_on_submit():
            # API on behalf of user
            user_api = auth_api(session['api_token'])
            eg = form_editgroup_get_or_create(user_api, form)
            if eg:
                # no merge or anything hard to do; just create the entity
                entity = form.to_entity()
                try:
                    edit = user_api.create_container(entity, editgroup_id=eg.editgroup_id)
                except ApiException as ae:
                    app.log.warning(ae)
                    abort(ae.status)
                # redirect to new entity
                return redirect('/container/{}'.format(edit.ident))
        elif form.errors:
            app.log.info("form errors (did not validate): {}".format(form.errors))
    else:
        editgroup_id = session.get('active_editgroup_id', None)
        form.editgroup_id.data = editgroup_id
    return render_template('container_create.html', form=form)

@app.route('/container/<ident>/edit', methods=['GET', 'POST'])
@login_required
def container_edit(ident):
    # TODO: prev_rev interlock
    try:
        entity = api.get_container(ident)
    except ApiException as ae:
        abort(ae.status)
    form = ContainerEntityForm()
    if form.is_submitted():
        if form.validate_on_submit():
            # API on behalf of user
            user_api = auth_api(session['api_token'])
            eg = form_editgroup_get_or_create(user_api, form)
            if eg:
                # all the tricky logic is in the update method
                form.update_entity(entity)
                try:
                    edit = user_api.update_container(entity.ident, entity,
                        editgroup_id=eg.editgroup_id)
                except ApiException as ae:
                    app.log.warning(ae)
                    abort(ae.status)
                # redirect to entity revision
                # TODO: container_rev_view
                return redirect('/container/{}'.format(edit.ident))
        elif form.errors:
            app.log.info("form errors (did not validate): {}".format(form.errors))
    else:
        form = ContainerEntityForm.from_entity(entity)
    if not form.is_submitted():
        editgroup_id = session.get('active_editgroup_id', None)
        form.editgroup_id.data = editgroup_id
    return render_template('container_edit.html', form=form, entity=entity)

@app.route('/creator/<ident>/edit', methods=['GET'])
@login_required
def creator_edit(ident):
    try:
        entity = api.get_creator(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html')

@app.route('/file/create', methods=['GET', 'POST'])
@login_required
def file_create():
    form = FileEntityForm()
    if form.is_submitted():
        if form.validate_on_submit():
            # API on behalf of user
            user_api = auth_api(session['api_token'])
            eg = form_editgroup_get_or_create(user_api, form)
            if eg:
                # no merge or anything hard to do; just create the entity
                entity = form.to_entity()
                try:
                    edit = user_api.create_file(entity, editgroup_id=eg.editgroup_id)
                except ApiException as ae:
                    app.log.warning(ae)
                    abort(ae.status)
                # redirect to new entity
                return redirect('/file/{}'.format(edit.ident))
        elif form.errors:
            app.log.info("form errors (did not validate): {}".format(form.errors))
    else:
        editgroup_id = session.get('active_editgroup_id', None)
        form.editgroup_id.data = editgroup_id
        form.urls.append_entry()
        form.release_ids.append_entry()
    return render_template('file_create.html',
        form=form)

@app.route('/file/<ident>/edit', methods=['GET', 'POST'])
@login_required
def file_edit(ident):
    # TODO: prev_rev interlock
    try:
        entity = api.get_file(ident)
    except ApiException as ae:
        abort(ae.status)
    form = FileEntityForm()
    if form.is_submitted():
        if form.validate_on_submit():
            # API on behalf of user
            user_api = auth_api(session['api_token'])
            eg = form_editgroup_get_or_create(user_api, form)
            if eg:
                # all the tricky logic is in the update method
                form.update_entity(entity)
                try:
                    edit = user_api.update_file(entity.ident, entity,
                        editgroup_id=eg.editgroup_id)
                except ApiException as ae:
                    app.log.warning(ae)
                    abort(ae.status)
                # redirect to entity revision
                # TODO: file_rev_view
                return redirect('/file/{}'.format(edit.ident))
        elif form.errors:
            app.log.info("form errors (did not validate): {}".format(form.errors))
    else: # not submitted
        form = FileEntityForm.from_entity(entity)
        editgroup_id = session.get('active_editgroup_id', None)
        form.editgroup_id.data = editgroup_id
    return render_template('file_edit.html', form=form, entity=entity)

@app.route('/fileset/<ident>/edit', methods=['GET'])
def fileset_edit(ident):
    try:
        entity = api.get_fileset(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html'), 404

@app.route('/webcapture/<ident>/edit', methods=['GET'])
def webcapture_edit(ident):
    try:
        entity = api.get_webcapture(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html'), 404

@app.route('/release/create', methods=['GET', 'POST'])
@login_required
def release_create():
    form = ReleaseEntityForm()
    if form.is_submitted():
        if form.validate_on_submit():
            # API on behalf of user
            user_api = auth_api(session['api_token'])
            eg = form_editgroup_get_or_create(user_api, form)
            if eg:
                # no merge or anything hard to do; just create the entity
                entity = form.to_entity()
                try:
                    edit = user_api.create_release(entity, editgroup_id=eg.editgroup_id)
                except ApiException as ae:
                    app.log.warning(ae)
                    abort(ae.status)
                # redirect to new release
                return redirect('/release/{}'.format(edit.ident))
        elif form.errors:
            app.log.info("form errors (did not validate): {}".format(form.errors))
    else: # not submitted
        form.contribs.append_entry()
        editgroup_id = session.get('active_editgroup_id', None)
        form.editgroup_id.data = editgroup_id
    return render_template('release_create.html', form=form)

@app.route('/release/<ident>/edit', methods=['GET', 'POST'])
@login_required
def release_edit(ident):
    # TODO: prev_rev interlock
    try:
        entity = api.get_release(ident)
    except ApiException as ae:
        abort(ae.status)
    form = ReleaseEntityForm()
    if form.is_submitted():
        if form.validate_on_submit():
            # API on behalf of user
            user_api = auth_api(session['api_token'])
            eg = form_editgroup_get_or_create(user_api, form)
            if eg:
                # all the tricky logic is in the update method
                form.update_entity(entity)
                try:
                    edit = user_api.update_release(entity.ident, entity,
                        editgroup_id=eg.editgroup_id)
                except ApiException as ae:
                    app.log.warning(ae)
                    abort(ae.status)
                # redirect to entity revision
                # TODO: release_rev_view
                return redirect('/release/{}'.format(edit.ident))
        elif form.errors:
            app.log.info("form errors (did not validate): {}".format(form.errors))
    else: # not submitted
        form = ReleaseEntityForm.from_entity(entity)
        editgroup_id = session.get('active_editgroup_id', None)
        form.editgroup_id.data = editgroup_id
    return render_template('release_edit.html', form=form, entity=entity)

@app.route('/work/create', methods=['GET'])
def work_create_view():
    return abort(404)

@app.route('/work/<ident>/edit', methods=['GET'])
def work_edit_view(ident):
    try:
        entity = api.get_work(ident)
    except ApiException as ae:
        abort(ae.status)
    return render_template('entity_edit.html'), 404
