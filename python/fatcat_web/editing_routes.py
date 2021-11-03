from typing import Any, Optional

from fatcat_openapi_client import (
    ApiClient,
    ContainerEntity,
    CreatorEntity,
    Editgroup,
    EntityEdit,
    FileEntity,
    FilesetEntity,
    ReleaseEntity,
    WebcaptureEntity,
    WorkEntity,
)
from fatcat_openapi_client.rest import ApiException
from flask import abort, flash, redirect, render_template, session
from flask_login import login_required

from fatcat_tools.transforms import entity_from_toml
from fatcat_web import AnyResponse, api, app, auth_api
from fatcat_web.entity_helpers import generic_get_editgroup_entity, generic_get_entity
from fatcat_web.forms import (
    ContainerEntityForm,
    EntityEditForm,
    EntityTomlForm,
    FileEntityForm,
    ReleaseEntityForm,
)

### Helper Methods ##########################################################


def generic_entity_create_from_toml(
    user_api: ApiClient, entity_type: str, editgroup_id: str, toml_str: str
) -> EntityEdit:
    if entity_type == "container":
        entity = entity_from_toml(toml_str, ContainerEntity)
        edit = user_api.create_container(editgroup_id, entity)
    elif entity_type == "creator":
        entity = entity_from_toml(toml_str, CreatorEntity)
        edit = user_api.create_creator(editgroup_id, entity)
    elif entity_type == "file":
        entity = entity_from_toml(toml_str, FileEntity)
        edit = user_api.create_file(editgroup_id, entity)
    elif entity_type == "fileset":
        entity = entity_from_toml(toml_str, FilesetEntity)
        edit = user_api.create_fileset(editgroup_id, entity)
    elif entity_type == "webcapture":
        entity = entity_from_toml(toml_str, WebcaptureEntity)
        edit = user_api.create_webcapture(editgroup_id, entity)
    elif entity_type == "release":
        entity = entity_from_toml(toml_str, ReleaseEntity)
        edit = user_api.create_release(editgroup_id, entity)
    elif entity_type == "work":
        entity = entity_from_toml(toml_str, WorkEntity)
        edit = user_api.create_work(editgroup_id, entity)
    else:
        raise NotImplementedError
    return edit


def generic_entity_delete_edit(
    user_api: ApiClient, entity_type: str, editgroup_id: str, edit_id: str
) -> None:
    try:
        if entity_type == "container":
            user_api.delete_container_edit(editgroup_id, edit_id)
        elif entity_type == "creator":
            user_api.delete_creator_edit(editgroup_id, edit_id)
        elif entity_type == "file":
            user_api.delete_file_edit(editgroup_id, edit_id)
        elif entity_type == "fileset":
            user_api.delete_fileset_edit(editgroup_id, edit_id)
        elif entity_type == "webcapture":
            user_api.delete_webcapture_edit(editgroup_id, edit_id)
        elif entity_type == "release":
            user_api.delete_release_edit(editgroup_id, edit_id)
        elif entity_type == "work":
            user_api.delete_work_edit(editgroup_id, edit_id)
        else:
            raise NotImplementedError
    except ApiException as ae:
        if ae.status == 404:
            pass
        else:
            raise ae


def generic_entity_delete_entity(
    user_api: ApiClient, entity_type: str, editgroup_id: str, entity_ident: str
) -> EntityEdit:
    try:
        if entity_type == "container":
            edit = user_api.delete_container(editgroup_id, entity_ident)
        elif entity_type == "creator":
            edit = user_api.delete_creator(editgroup_id, entity_ident)
        elif entity_type == "file":
            edit = user_api.delete_file(editgroup_id, entity_ident)
        elif entity_type == "fileset":
            edit = user_api.delete_fileset(editgroup_id, entity_ident)
        elif entity_type == "webcapture":
            edit = user_api.delete_webcapture(editgroup_id, entity_ident)
        elif entity_type == "release":
            edit = user_api.delete_release(editgroup_id, entity_ident)
        elif entity_type == "work":
            edit = user_api.delete_work(editgroup_id, entity_ident)
        else:
            raise NotImplementedError
    except ApiException as ae:
        raise ae
    return edit


def generic_entity_update_from_toml(
    user_api: ApiClient, entity_type: str, editgroup_id: str, existing_ident: str, toml_str: str
) -> EntityEdit:
    if entity_type == "container":
        entity = entity_from_toml(toml_str, ContainerEntity)
        edit = user_api.update_container(editgroup_id, existing_ident, entity)
    elif entity_type == "creator":
        entity = entity_from_toml(toml_str, CreatorEntity)
        edit = user_api.update_creator(editgroup_id, existing_ident, entity)
    elif entity_type == "file":
        entity = entity_from_toml(toml_str, FileEntity)
        edit = user_api.update_file(editgroup_id, existing_ident, entity)
    elif entity_type == "fileset":
        entity = entity_from_toml(toml_str, FilesetEntity)
        edit = user_api.update_fileset(editgroup_id, existing_ident, entity)
    elif entity_type == "webcapture":
        entity = entity_from_toml(toml_str, WebcaptureEntity)
        edit = user_api.update_webcapture(editgroup_id, existing_ident, entity)
    elif entity_type == "release":
        entity = entity_from_toml(toml_str, ReleaseEntity)
        edit = user_api.update_release(editgroup_id, existing_ident, entity)
    elif entity_type == "work":
        entity = entity_from_toml(toml_str, WorkEntity)
        edit = user_api.update_work(editgroup_id, existing_ident, entity)
    else:
        raise NotImplementedError
    return edit


def form_editgroup_get_or_create(api: ApiClient, edit_form: Any) -> Optional[Editgroup]:
    """
    This function expects a submitted, validated edit form
    """
    if edit_form.editgroup_id.data:
        try:
            eg = api.get_editgroup(edit_form.editgroup_id.data)
        except ApiException as ae:
            if ae.status == 404:
                edit_form.editgroup_id.errors.append("Editgroup does not exist")
                return None
            app.log.warning(ae)
            raise ae
        if eg.changelog_index:
            edit_form.editgroup_id.errors.append("Editgroup has already been accepted")
            return None
    else:
        # if no editgroup, create one from description
        try:
            eg = api.create_editgroup(
                Editgroup(description=edit_form.editgroup_description.data or None)
            )
        except ApiException as ae:
            app.log.warning(ae)
            raise ae
        # set this session editgroup_id (TODO)
    return eg


def generic_entity_edit(
    editgroup_id: Optional[str],
    entity_type: str,
    existing_ident: Optional[str],
    edit_template: str,
) -> AnyResponse:
    """

    existing (entity)

    Create: existing blank, ident blank, editgroup optional
    Update: ident set

    Need to handle:
    - editgroup not set (need to create one)
    - creating entity from form
    - updating an existing ident
    - updating an existing editgroup/ident

    Views:
    - /container/create
    - /container/<ident>/edit
    - /editgroup/<editgroup_id>/container/<ident>/edit

    Helpers:
    - get_editgroup_revision(editgroup, entity_type, ident) -> None or entity

    TODO: prev_rev interlock
    """

    # fetch editgroup (if set) or 404
    editgroup = None
    if editgroup_id:
        try:
            editgroup = api.get_editgroup(editgroup_id)
        except ApiException as ae:
            raise ae

        # check that editgroup is edit-able
        if editgroup.changelog_index is not None:
            abort(400, "Editgroup already merged")

    # fetch entity (if set) or 404
    existing = None
    existing_edit = None
    if editgroup and existing_ident:
        existing, existing_edit = generic_get_editgroup_entity(
            editgroup, entity_type, existing_ident
        )
    elif existing_ident:
        existing = generic_get_entity(entity_type, existing_ident)

    # parse form (if submitted)
    status = 200
    if entity_type == "container":
        form = ContainerEntityForm()
    elif entity_type == "file":
        form = FileEntityForm()
    elif entity_type == "release":
        form = ReleaseEntityForm()
    else:
        raise NotImplementedError

    if form.is_submitted():
        if form.validate_on_submit():
            # API on behalf of user
            user_api = auth_api(session["api_token"])
            if not editgroup:
                editgroup = form_editgroup_get_or_create(user_api, form)

            if editgroup:

                if not existing_ident:  # it's a create
                    entity = form.to_entity()
                    try:
                        if entity_type == "container":
                            edit = user_api.create_container(editgroup.editgroup_id, entity)
                        elif entity_type == "file":
                            edit = user_api.create_file(editgroup.editgroup_id, entity)
                        elif entity_type == "release":
                            edit = user_api.create_release(editgroup.editgroup_id, entity)
                        else:
                            raise NotImplementedError
                    except ApiException as ae:
                        app.log.warning(ae)
                        raise ae
                    return redirect(
                        "/editgroup/{}/{}/{}".format(
                            editgroup.editgroup_id, entity_type, edit.ident
                        )
                    )
                else:  # it's an update
                    # all the tricky logic is in the update method
                    form.update_entity(existing)
                    # do we need to try to delete the current in-progress edit first?
                    # TODO: some danger of wiping database state here is
                    # "updated edit" causes, eg, a 4xx error. Better to allow
                    # this in the API itself. For now, form validation *should*
                    # catch most errors, and if not editor can hit back and try
                    # again. This means, need to allow failure of deletion.
                    if existing_edit:
                        # need to clear revision on object or this becomes just
                        # a "update pointer" edit
                        existing.revision = None
                        try:
                            generic_entity_delete_edit(
                                user_api,
                                entity_type,
                                editgroup.editgroup_id,
                                existing_edit.edit_id,
                            )
                        except ApiException as ae:
                            if ae.status == 404:
                                pass
                            else:
                                raise ae
                    try:
                        if entity_type == "container":
                            edit = user_api.update_container(
                                editgroup.editgroup_id, existing.ident, existing
                            )
                        elif entity_type == "file":
                            edit = user_api.update_file(
                                editgroup.editgroup_id, existing.ident, existing
                            )
                        elif entity_type == "release":
                            edit = user_api.update_release(
                                editgroup.editgroup_id, existing.ident, existing
                            )
                        else:
                            raise NotImplementedError
                    except ApiException as ae:
                        app.log.warning(ae)
                        raise ae
                    return redirect(
                        "/editgroup/{}/{}/{}".format(
                            editgroup.editgroup_id, entity_type, edit.ident
                        )
                    )
            else:
                status = 400
        elif form.errors:
            status = 400
            app.log.info("form errors (did not validate): {}".format(form.errors))

    else:  # form is not submitted
        if existing:
            if entity_type == "container":
                form = ContainerEntityForm.from_entity(existing)
            elif entity_type == "file":
                form = FileEntityForm.from_entity(existing)
            elif entity_type == "release":
                form = ReleaseEntityForm.from_entity(existing)
            else:
                raise NotImplementedError

    editor_editgroups = api.get_editor_editgroups(session["editor"]["editor_id"], limit=20)
    potential_editgroups = [
        e for e in editor_editgroups if e.changelog_index is None and e.submitted is None
    ]

    if not form.is_submitted():
        # default to most recent not submitted, fallback to "create new"
        form.editgroup_id.data = ""
        if potential_editgroups:
            form.editgroup_id.data = potential_editgroups[0].editgroup_id

    return (
        render_template(
            edit_template,
            form=form,
            existing_ident=existing_ident,
            editgroup=editgroup,
            potential_editgroups=potential_editgroups,
        ),
        status,
    )


def generic_entity_toml_edit(
    editgroup_id: Optional[str],
    entity_type: Any,
    existing_ident: Optional[str],
    edit_template: str,
) -> AnyResponse:
    """
    Similar to generic_entity_edit(), but for TOML editing mode.

    Handles both creation and update/edit paths.
    """

    # fetch editgroup (if set) or 404
    editgroup = None
    if editgroup_id:
        try:
            editgroup = api.get_editgroup(editgroup_id)
        except ApiException as ae:
            raise ae

        # check that editgroup is edit-able
        if editgroup.changelog_index is not None:
            flash("Editgroup already merged")
            abort(400)

    # fetch entity (if set) or 404
    existing = None
    existing_edit = None
    if editgroup and existing_ident:
        existing, existing_edit = generic_get_editgroup_entity(
            editgroup, entity_type, existing_ident
        )
    elif existing_ident:
        existing = generic_get_entity(entity_type, existing_ident)

    # parse form (if submitted)
    status = 200
    form = EntityTomlForm()

    if form.is_submitted():
        if form.validate_on_submit():
            # API on behalf of user
            user_api = auth_api(session["api_token"])
            if not editgroup:
                editgroup = form_editgroup_get_or_create(user_api, form)

            if editgroup:

                if not existing_ident:  # it's a create
                    try:
                        edit = generic_entity_create_from_toml(
                            user_api, entity_type, editgroup.editgroup_id, form.toml.data
                        )
                    except ValueError as ve:
                        form.toml.errors = [ve]
                        status = 400
                    except ApiException as ae:
                        app.log.warning(ae)
                        raise ae
                    if status == 200:
                        return redirect(
                            "/editgroup/{}/{}/{}".format(
                                editgroup.editgroup_id, entity_type, edit.ident
                            )
                        )
                else:  # it's an update
                    # TODO: some danger of wiping database state here is
                    # "updated edit" causes, eg, a 4xx error. Better to allow
                    # this in the API itself. For now, form validation *should*
                    # catch most errors, and if not editor can hit back and try
                    # again. This means, need to allow failure of deletion.
                    if existing_edit:
                        # need to clear revision on object or this becomes just
                        # a "update pointer" edit
                        existing.revision = None
                        generic_entity_delete_edit(
                            user_api, entity_type, editgroup.editgroup_id, existing_edit.edit_id
                        )
                    try:
                        edit = generic_entity_update_from_toml(
                            user_api,
                            entity_type,
                            editgroup.editgroup_id,
                            existing.ident,
                            form.toml.data,
                        )
                    except ValueError as ve:
                        form.toml.errors = [ve]
                        status = 400
                    except ApiException as ae:
                        app.log.warning(ae)
                        raise ae
                    if status == 200:
                        return redirect(
                            "/editgroup/{}/{}/{}".format(
                                editgroup.editgroup_id, entity_type, edit.ident
                            )
                        )
            else:
                status = 400
        elif form.errors:
            status = 400
            app.log.info("form errors (did not validate): {}".format(form.errors))

    else:  # form is not submitted
        if existing:
            form = EntityTomlForm.from_entity(existing)

    editor_editgroups = api.get_editor_editgroups(session["editor"]["editor_id"], limit=20)
    potential_editgroups = [
        e for e in editor_editgroups if e.changelog_index is None and e.submitted is None
    ]

    if not form.is_submitted():
        # default to most recent not submitted, fallback to "create new"
        form.editgroup_id.data = ""
        if potential_editgroups:
            form.editgroup_id.data = potential_editgroups[0].editgroup_id

    return (
        render_template(
            edit_template,
            form=form,
            entity_type=entity_type,
            existing_ident=existing_ident,
            editgroup=editgroup,
            potential_editgroups=potential_editgroups,
        ),
        status,
    )


def generic_entity_delete(
    editgroup_id: Optional[str], entity_type: str, existing_ident: str
) -> AnyResponse:
    """
    Similar to generic_entity_edit(), but for deleting entities. This is a bit
    simpler!

    Handles both creation and update/edit paths.
    """

    # fetch editgroup (if set) or 404
    editgroup = None
    if editgroup_id:
        try:
            editgroup = api.get_editgroup(editgroup_id)
        except ApiException as ae:
            raise ae

        # check that editgroup is edit-able
        if editgroup.changelog_index is not None:
            flash("Editgroup already merged")
            abort(400)

    # fetch entity (if set) or 404
    existing_edit = None
    if editgroup and existing_ident:
        existing, existing_edit = generic_get_editgroup_entity(
            editgroup, entity_type, existing_ident
        )
    elif existing_ident:
        existing = generic_get_entity(entity_type, existing_ident)

    # parse form (if submitted)
    status = 200
    form = EntityEditForm()

    if form.is_submitted():
        if form.validate_on_submit():
            # API on behalf of user
            user_api = auth_api(session["api_token"])
            if not editgroup:
                editgroup = form_editgroup_get_or_create(user_api, form)

            if editgroup:
                # TODO: some danger of wiping database state here is
                # "updated edit" causes, eg, a 4xx error. Better to allow
                # this in the API itself. For now, form validation *should*
                # catch most errors, and if not editor can hit back and try
                # again. This means, need to allow failure of deletion.
                if existing_edit:
                    # need to clear revision on object or this becomes just
                    # a "update pointer" edit
                    existing.revision = None
                    generic_entity_delete_edit(
                        user_api, entity_type, editgroup.editgroup_id, existing_edit.edit_id
                    )
                try:
                    edit = generic_entity_delete_entity(
                        user_api, entity_type, editgroup.editgroup_id, existing.ident
                    )
                except ApiException as ae:
                    app.log.warning(ae)
                    raise ae
                if status == 200:
                    return redirect(
                        "/editgroup/{}/{}/{}".format(
                            editgroup.editgroup_id, entity_type, edit.ident
                        )
                    )
            else:
                status = 400
        elif form.errors:
            status = 400
            app.log.info("form errors (did not validate): {}".format(form.errors))

    else:  # form is not submitted
        if existing:
            form = EntityTomlForm.from_entity(existing)

    editor_editgroups = api.get_editor_editgroups(session["editor"]["editor_id"], limit=20)
    potential_editgroups = [
        e for e in editor_editgroups if e.changelog_index is None and e.submitted is None
    ]

    if not form.is_submitted():
        # default to most recent not submitted, fallback to "create new"
        form.editgroup_id.data = ""
        if potential_editgroups:
            form.editgroup_id.data = potential_editgroups[0].editgroup_id

    return (
        render_template(
            "entity_delete.html",
            form=form,
            entity_type=entity_type,
            existing_ident=existing_ident,
            editgroup=editgroup,
            potential_editgroups=potential_editgroups,
        ),
        status,
    )


def generic_edit_delete(
    editgroup_id: Optional[str], entity_type: Any, edit_id: str
) -> AnyResponse:
    # fetch editgroup (if set) or 404
    editgroup = None
    if editgroup_id:
        try:
            editgroup = api.get_editgroup(editgroup_id)
        except ApiException as ae:
            abort(ae.status)

        # check that editgroup is edit-able
        if editgroup.changelog_index is not None:
            flash("Editgroup already merged")
            abort(400)

    # API on behalf of user
    user_api = auth_api(session["api_token"])

    # do the deletion
    generic_entity_delete_edit(user_api, entity_type, editgroup.editgroup_id, edit_id)
    return redirect("/editgroup/{}".format(editgroup_id))


### Views ###################################################################


@app.route("/container/create", methods=["GET", "POST"])
@login_required
def container_create_view() -> AnyResponse:
    return generic_entity_edit(None, "container", None, "container_create.html")


@app.route("/container/<ident>/edit", methods=["GET", "POST"])
@login_required
def container_edit_view(ident: str) -> AnyResponse:
    return generic_entity_edit(None, "container", ident, "container_edit.html")


@app.route("/container/<ident>/delete", methods=["GET", "POST"])
@login_required
def container_delete_view(ident: str) -> AnyResponse:
    return generic_entity_delete(None, "container", ident)


@app.route("/editgroup/<editgroup_id>/container/<ident>/edit", methods=["GET", "POST"])
@login_required
def container_editgroup_edit_view(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_edit(editgroup_id, "container", ident, "container_edit.html")


@app.route("/editgroup/<editgroup_id>/container/<ident>/delete", methods=["GET", "POST"])
@login_required
def container_editgroup_delete_view(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_delete(editgroup_id, "container", ident)


@app.route("/editgroup/<editgroup_id>/container/edit/<edit_id>/delete", methods=["POST"])
@login_required
def container_edit_delete(editgroup_id: str, edit_id: str) -> AnyResponse:
    return generic_edit_delete(editgroup_id, "container", edit_id)


@app.route("/creator/<ident>/delete", methods=["GET", "POST"])
@login_required
def creator_delete_view(ident: str) -> AnyResponse:
    return generic_entity_delete(None, "creator", ident)


@app.route("/editgroup/<editgroup_id>/creator/edit/<edit_id>/delete", methods=["POST"])
def creator_edit_delete(editgroup_id: str, edit_id: str) -> AnyResponse:
    return generic_edit_delete(editgroup_id, "creator", edit_id)


@app.route("/editgroup/<editgroup_id>/creator/<ident>/delete", methods=["GET", "POST"])
@login_required
def creator_editgroup_delete(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_delete(editgroup_id, "creator", ident)


@app.route("/file/create", methods=["GET", "POST"])
@login_required
def file_create_view() -> AnyResponse:
    return generic_entity_edit(None, "file", None, "file_create.html")


@app.route("/file/<ident>/edit", methods=["GET", "POST"])
@login_required
def file_edit_view(ident: str) -> AnyResponse:
    return generic_entity_edit(None, "file", ident, "file_edit.html")


@app.route("/file/<ident>/delete", methods=["GET", "POST"])
@login_required
def file_delete_view(ident: str) -> AnyResponse:
    return generic_entity_delete(None, "file", ident)


@app.route("/editgroup/<editgroup_id>/file/<ident>/edit", methods=["GET", "POST"])
@login_required
def file_editgroup_edit_view(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_edit(editgroup_id, "file", ident, "file_edit.html")


@app.route("/editgroup/<editgroup_id>/file/<ident>/delete", methods=["GET", "POST"])
@login_required
def file_editgroup_delete_view(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_delete(editgroup_id, "file", ident)


@app.route("/editgroup/<editgroup_id>/file/edit/<edit_id>/delete", methods=["POST"])
@login_required
def file_edit_delete(editgroup_id: str, edit_id: str) -> AnyResponse:
    return generic_edit_delete(editgroup_id, "file", edit_id)


@app.route("/fileset/<ident>/delete", methods=["GET", "POST"])
@login_required
def fileset_delete_view(ident: str) -> AnyResponse:
    return generic_entity_delete(None, "fileset", ident)


@app.route("/editgroup/<editgroup_id>/fileset/edit/<edit_id>/delete", methods=["POST"])
def fileset_edit_delete(editgroup_id: str, edit_id: str) -> AnyResponse:
    return generic_edit_delete(editgroup_id, "fileset", edit_id)


@app.route("/editgroup/<editgroup_id>/fileset/<ident>/delete", methods=["GET", "POST"])
@login_required
def fileset_editgroup_delete(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_delete(editgroup_id, "fileset", ident)


@app.route("/webcapture/<ident>/delete", methods=["GET", "POST"])
@login_required
def webcapture_delete_view(ident: str) -> AnyResponse:
    return generic_entity_delete(None, "webcapture", ident)


@app.route("/editgroup/<editgroup_id>/webcapture/edit/<edit_id>/delete", methods=["POST"])
def webcapture_edit_delete(editgroup_id: str, edit_id: str) -> AnyResponse:
    return generic_edit_delete(editgroup_id, "webcapture", edit_id)


@app.route("/editgroup/<editgroup_id>/webcapture/<ident>/delete", methods=["GET", "POST"])
@login_required
def webcapture_editgroup_delete(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_delete(editgroup_id, "webcapture", ident)


@app.route("/release/create", methods=["GET", "POST"])
@login_required
def release_create_view() -> AnyResponse:
    return generic_entity_edit(None, "release", None, "release_create.html")


@app.route("/release/<ident>/edit", methods=["GET", "POST"])
@login_required
def release_edit_view(ident: str) -> AnyResponse:
    return generic_entity_edit(None, "release", ident, "release_edit.html")


@app.route("/release/<ident>/delete", methods=["GET", "POST"])
@login_required
def release_delete_view(ident: str) -> AnyResponse:
    return generic_entity_delete(None, "release", ident)


@app.route("/editgroup/<editgroup_id>/release/<ident>/edit", methods=["GET", "POST"])
@login_required
def release_editgroup_edit(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_edit(editgroup_id, "release", ident, "release_edit.html")


@app.route("/editgroup/<editgroup_id>/release/<ident>/delete", methods=["GET", "POST"])
@login_required
def release_editgroup_delete(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_delete(editgroup_id, "release", ident)


@app.route("/editgroup/<editgroup_id>/release/edit/<edit_id>/delete", methods=["POST"])
@login_required
def release_edit_delete(editgroup_id: str, edit_id: str) -> AnyResponse:
    return generic_edit_delete(editgroup_id, "release", edit_id)


@app.route("/work/<ident>/delete", methods=["GET", "POST"])
@login_required
def work_delete_view(ident: str) -> AnyResponse:
    return generic_entity_delete(None, "work", ident)


@app.route("/editgroup/<editgroup_id>/work/edit/<edit_id>/delete", methods=["POST"])
def work_edit_delete(editgroup_id: str, edit_id: str) -> AnyResponse:
    return generic_edit_delete(editgroup_id, "work", edit_id)


@app.route("/editgroup/<editgroup_id>/work/<ident>/delete", methods=["GET", "POST"])
@login_required
def work_editgroup_delete(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_delete(editgroup_id, "work", ident)


### TOML Views ##############################################################


@app.route("/container/create/toml", methods=["GET", "POST"])
@login_required
def container_create_toml_view() -> AnyResponse:
    return generic_entity_toml_edit(None, "container", None, "entity_create_toml.html")


@app.route("/container/<ident>/edit/toml", methods=["GET", "POST"])
@login_required
def container_edit_toml_view(ident: str) -> AnyResponse:
    return generic_entity_toml_edit(None, "container", ident, "entity_edit_toml.html")


@app.route("/editgroup/<editgroup_id>/container/<ident>/edit/toml", methods=["GET", "POST"])
@login_required
def container_editgroup_edit_toml(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_toml_edit(editgroup_id, "container", ident, "entity_edit_toml.html")


@app.route("/creator/create/toml", methods=["GET", "POST"])
@login_required
def creator_create_toml_view() -> AnyResponse:
    return generic_entity_toml_edit(None, "creator", None, "entity_create_toml.html")


@app.route("/creator/<ident>/edit/toml", methods=["GET", "POST"])
@login_required
def creator_edit_toml_view(ident: str) -> AnyResponse:
    return generic_entity_toml_edit(None, "creator", ident, "entity_edit_toml.html")


@app.route("/editgroup/<editgroup_id>/creator/<ident>/edit/toml", methods=["GET", "POST"])
@login_required
def creator_editgroup_edit_toml(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_toml_edit(editgroup_id, "creator", ident, "entity_edit_toml.html")


@app.route("/file/create/toml", methods=["GET", "POST"])
@login_required
def file_create_toml_view() -> AnyResponse:
    return generic_entity_toml_edit(None, "file", None, "entity_create_toml.html")


@app.route("/file/<ident>/edit/toml", methods=["GET", "POST"])
@login_required
def file_edit_toml_view(ident: str) -> AnyResponse:
    return generic_entity_toml_edit(None, "file", ident, "entity_edit_toml.html")


@app.route("/editgroup/<editgroup_id>/file/<ident>/edit/toml", methods=["GET", "POST"])
@login_required
def file_editgroup_edit_toml(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_toml_edit(editgroup_id, "file", ident, "entity_edit_toml.html")


@app.route("/fileset/create/toml", methods=["GET", "POST"])
@login_required
def fileset_create_toml_view() -> AnyResponse:
    return generic_entity_toml_edit(None, "fileset", None, "entity_create_toml.html")


@app.route("/fileset/<ident>/edit/toml", methods=["GET", "POST"])
@login_required
def fileset_edit_toml_view(ident: str) -> AnyResponse:
    return generic_entity_toml_edit(None, "fileset", ident, "entity_edit_toml.html")


@app.route("/editgroup/<editgroup_id>/fileset/<ident>/edit/toml", methods=["GET", "POST"])
@login_required
def fileset_editgroup_edit_toml(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_toml_edit(editgroup_id, "fileset", ident, "entity_edit_toml.html")


@app.route("/webcapture/create/toml", methods=["GET", "POST"])
@login_required
def webcapture_create_toml_view() -> AnyResponse:
    return generic_entity_toml_edit(None, "webcapture", None, "entity_create_toml.html")


@app.route("/webcapture/<ident>/edit/toml", methods=["GET", "POST"])
@login_required
def webcapture_edit_toml_view(ident: str) -> AnyResponse:
    return generic_entity_toml_edit(None, "webcapture", ident, "entity_edit_toml.html")


@app.route("/editgroup/<editgroup_id>/webcapture/<ident>/edit/toml", methods=["GET", "POST"])
@login_required
def webcapture_editgroup_edit_toml(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_toml_edit(editgroup_id, "webcapture", ident, "entity_edit_toml.html")


@app.route("/release/create/toml", methods=["GET", "POST"])
@login_required
def release_create_toml_view() -> AnyResponse:
    return generic_entity_toml_edit(None, "release", None, "entity_create_toml.html")


@app.route("/release/<ident>/edit/toml", methods=["GET", "POST"])
@login_required
def release_edit_toml_view(ident: str) -> AnyResponse:
    return generic_entity_toml_edit(None, "release", ident, "entity_edit_toml.html")


@app.route("/editgroup/<editgroup_id>/release/<ident>/edit/toml", methods=["GET", "POST"])
@login_required
def release_editgroup_edit_toml(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_toml_edit(editgroup_id, "release", ident, "entity_edit_toml.html")


@app.route("/work/create/toml", methods=["GET", "POST"])
@login_required
def work_create_toml_view() -> AnyResponse:
    return generic_entity_toml_edit(None, "work", None, "entity_create_toml.html")


@app.route("/work/<ident>/edit/toml", methods=["GET", "POST"])
@login_required
def work_edit_toml_view(ident: str) -> AnyResponse:
    return generic_entity_toml_edit(None, "work", ident, "entity_edit_toml.html")


@app.route("/editgroup/<editgroup_id>/work/<ident>/edit/toml", methods=["GET", "POST"])
@login_required
def work_editgroup_edit_toml(editgroup_id: str, ident: str) -> AnyResponse:
    return generic_entity_toml_edit(editgroup_id, "work", ident, "entity_edit_toml.html")


### TOML-Only Editing Redirects ################################################


@app.route("/creator/create", methods=["GET"])
@login_required
def creator_create_view() -> AnyResponse:
    return redirect("/creator/create/toml")


@app.route("/creator/<ident>/edit", methods=["GET"])
@login_required
def creator_edit_view(ident: str) -> AnyResponse:
    return redirect(f"/creator/{ident}/edit/toml")


@app.route("/editgroup/<editgroup_id>/creator/<ident>/edit", methods=["GET", "POST"])
@login_required
def creator_editgroup_edit(editgroup_id: str, ident: str) -> AnyResponse:
    return redirect(f"/editgroup/{editgroup_id}/creator/{ident}/edit/toml")


@app.route("/fileset/create", methods=["GET"])
@login_required
def fileset_create_view() -> AnyResponse:
    return redirect("/fileset/create/toml")


@app.route("/fileset/<ident>/edit", methods=["GET"])
@login_required
def fileset_edit_view(ident: str) -> AnyResponse:
    return redirect(f"/fileset/{ident}/edit/toml")


@app.route("/editgroup/<editgroup_id>/fileset/<ident>/edit", methods=["GET", "POST"])
@login_required
def fileset_editgroup_edit(editgroup_id: str, ident: str) -> AnyResponse:
    return redirect(f"/editgroup/{editgroup_id}/fileset/{ident}/edit/toml")


@app.route("/webcapture/create", methods=["GET"])
@login_required
def webcapture_create_view() -> AnyResponse:
    return redirect("/webcapture/create/toml")


@app.route("/webcapture/<ident>/edit", methods=["GET"])
@login_required
def webcapture_edit_view(ident: str) -> AnyResponse:
    return redirect(f"/webcapture/{ident}/edit/toml")


@app.route("/editgroup/<editgroup_id>/webcapture/<ident>/edit", methods=["GET", "POST"])
@login_required
def webcapture_editgroup_edit(editgroup_id: str, ident: str) -> AnyResponse:
    return redirect(f"/editgroup/{editgroup_id}/webcapture/{ident}/edit/toml")


@app.route("/work/create", methods=["GET"])
@login_required
def work_create_view() -> AnyResponse:
    return redirect("/work/create/toml")


@app.route("/work/<ident>/edit", methods=["GET"])
@login_required
def work_edit_view(ident: str) -> AnyResponse:
    return redirect(f"/work/{ident}/edit/toml")


@app.route("/editgroup/<editgroup_id>/work/<ident>/edit", methods=["GET", "POST"])
@login_required
def work_editgroup_edit(editgroup_id: str, ident: str) -> AnyResponse:
    return redirect(f"/editgroup/{editgroup_id}/work/{ident}/edit/toml")
