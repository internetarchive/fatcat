table! {
    changelog (id) {
        id -> Int8,
        editgroup_id -> Int8,
        timestamp -> Nullable<Timestamp>,
    }
}

table! {
    creator_edit (id) {
        id -> Int8,
        extra_json -> Nullable<Json>,
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
        extra_json -> Nullable<Json>,
        name -> Nullable<Text>,
        orcid -> Nullable<Text>,
    }
}

table! {
    editgroup (id) {
        id -> Int8,
        extra_json -> Nullable<Json>,
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

joinable!(changelog -> editgroup (editgroup_id));
joinable!(creator_edit -> creator_rev (rev_id));
joinable!(creator_edit -> editgroup (editgroup_id));
joinable!(creator_ident -> creator_rev (rev_id));

allow_tables_to_appear_in_same_query!(
    changelog,
    creator_edit,
    creator_ident,
    creator_rev,
    editgroup,
    editor,
);
