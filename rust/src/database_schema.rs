table! {
    changelog (id) {
        id -> Int8,
        editgroup_id -> Int8,
        timestamp -> Timestamp,
    }
}

table! {
    container_edit (id) {
        id -> Int8,
        ident_id -> Uuid,
        rev_id -> Nullable<Int8>,
        redirect_id -> Nullable<Uuid>,
        editgroup_id -> Int8,
    }
}

table! {
    container_ident (id) {
        id -> Uuid,
        is_live -> Bool,
        rev_id -> Nullable<Int8>,
        redirect_id -> Nullable<Uuid>,
    }
}

table! {
    container_rev (id) {
        id -> Int8,
        name -> Text,
        publisher -> Nullable<Text>,
        issn -> Nullable<Text>,
    }
}

table! {
    creator_edit (id) {
        id -> Int8,
        ident_id -> Uuid,
        rev_id -> Nullable<Int8>,
        redirect_id -> Nullable<Uuid>,
        editgroup_id -> Int8,
    }
}

table! {
    creator_ident (id) {
        id -> Uuid,
        is_live -> Bool,
        rev_id -> Nullable<Int8>,
        redirect_id -> Nullable<Uuid>,
    }
}

table! {
    creator_rev (id) {
        id -> Int8,
        name -> Text,
        orcid -> Nullable<Text>,
    }
}

table! {
    editgroup (id) {
        id -> Int8,
        //extra_json -> Nullable<Json>,
        editor_id -> Int8,
        description -> Nullable<Text>,
    }
}

table! {
    editor (id) {
        id -> Int8,
        username -> Text,
        is_admin -> Bool,
        active_editgroup_id -> Nullable<Int8>,
    }
}

table! {
    file_edit (id) {
        id -> Int8,
        ident_id -> Uuid,
        rev_id -> Nullable<Int8>,
        redirect_id -> Nullable<Uuid>,
        editgroup_id -> Int8,
    }
}

table! {
    file_ident (id) {
        id -> Uuid,
        is_live -> Bool,
        rev_id -> Nullable<Int8>,
        redirect_id -> Nullable<Uuid>,
    }
}

table! {
    file_release (file_rev, target_release_ident_id) {
        file_rev -> Int8,
        target_release_ident_id -> Uuid,
    }
}

table! {
    file_rev (id) {
        id -> Int8,
        //extra_json -> Nullable<Json>,
        size -> Nullable<Int8>,
        sha1 -> Nullable<Text>,
        url -> Nullable<Text>,
    }
}

table! {
    release_contrib (id) {
        id -> Int8,
        release_rev -> Int8,
        creator_ident_id -> Nullable<Uuid>,
        stub -> Nullable<Text>,
        contrib_type -> Nullable<Text>,
    }
}

table! {
    release_edit (id) {
        id -> Int8,
        ident_id -> Uuid,
        rev_id -> Nullable<Int8>,
        redirect_id -> Nullable<Uuid>,
        editgroup_id -> Int8,
    }
}

table! {
    release_ident (id) {
        id -> Uuid,
        is_live -> Bool,
        rev_id -> Nullable<Int8>,
        redirect_id -> Nullable<Uuid>,
    }
}

table! {
    release_ref (id) {
        id -> Int8,
        release_rev -> Int8,
        target_release_ident_id -> Nullable<Uuid>,
        index -> Nullable<Int8>,
        stub -> Nullable<Text>,
    }
}

table! {
    release_rev (id) {
        id -> Int8,
        work_ident_id -> Uuid,
        container_ident_id -> Nullable<Uuid>,
        title -> Text,
        release_type -> Nullable<Text>,
        date -> Nullable<Text>,
        doi -> Nullable<Text>,
        volume -> Nullable<Text>,
        pages -> Nullable<Text>,
        issue -> Nullable<Text>,
    }
}

table! {
    work_edit (id) {
        id -> Int8,
        ident_id -> Uuid,
        rev_id -> Nullable<Int8>,
        redirect_id -> Nullable<Uuid>,
        editgroup_id -> Int8,
    }
}

table! {
    work_ident (id) {
        id -> Uuid,
        is_live -> Bool,
        rev_id -> Nullable<Int8>,
        redirect_id -> Nullable<Uuid>,
    }
}

table! {
    work_rev (id) {
        id -> Int8,
        work_type -> Nullable<Text>,
        primary_release_id -> Nullable<Uuid>,
    }
}

joinable!(changelog -> editgroup (editgroup_id));
joinable!(container_edit -> container_rev (rev_id));
joinable!(container_edit -> editgroup (editgroup_id));
joinable!(container_ident -> container_rev (rev_id));
joinable!(creator_edit -> creator_rev (rev_id));
joinable!(creator_edit -> editgroup (editgroup_id));
joinable!(creator_ident -> creator_rev (rev_id));
joinable!(file_edit -> editgroup (editgroup_id));
joinable!(file_edit -> file_rev (rev_id));
joinable!(file_ident -> file_rev (rev_id));
joinable!(file_release -> creator_ident (target_release_ident_id));
joinable!(file_release -> file_rev (file_rev));
joinable!(release_contrib -> creator_ident (creator_ident_id));
joinable!(release_contrib -> release_rev (release_rev));
joinable!(release_edit -> editgroup (editgroup_id));
joinable!(release_edit -> release_rev (rev_id));
joinable!(release_ident -> release_rev (rev_id));
joinable!(release_ref -> release_ident (target_release_ident_id));
joinable!(release_ref -> release_rev (release_rev));
joinable!(release_rev -> container_ident (container_ident_id));
joinable!(release_rev -> work_ident (work_ident_id));
joinable!(work_edit -> editgroup (editgroup_id));
joinable!(work_edit -> work_rev (rev_id));
joinable!(work_ident -> work_rev (rev_id));
joinable!(work_rev -> release_ident (primary_release_id));

allow_tables_to_appear_in_same_query!(
    changelog,
    container_edit,
    container_ident,
    container_rev,
    creator_edit,
    creator_ident,
    creator_rev,
    editgroup,
    editor,
    file_edit,
    file_ident,
    file_release,
    file_rev,
    release_contrib,
    release_edit,
    release_ident,
    release_ref,
    release_rev,
    work_edit,
    work_ident,
    work_rev,
);
