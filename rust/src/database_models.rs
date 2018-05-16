
use uuid::Uuid;
//use diesel::prelude::*;

use database_schema::*;

// Ugh. I thought the whole point was to *not* do this, but:
// https://github.com/diesel-rs/diesel/issues/1589

/*
table! {
    changelog (id) {
        id -> Int8,
        editgroup_id -> Int8,
        timestamp -> Nullable<Timestamp>,
    }
}
*/

#[derive(Debug, Queryable, Identifiable, Associations)] // AsChangeset
#[table_name = "container_edit"]
pub struct ContainerEditRow {
    pub id: i64,
    pub ident_id: Uuid,
    pub rev_id: Option<i64>,
    pub redirect_id: Option<Uuid>,
    pub editgroup_id: i64,
    //pub extra_json: Option<Json>,
}

#[derive(Debug, Queryable, Identifiable, Associations)] // AsChangeset
#[table_name = "container_ident"]
pub struct ContainerIdentRow {
    pub id: Uuid,
    pub is_live: bool,
    pub rev_id: Option<i64>,
    pub redirect_id: Option<Uuid>,
}

#[derive(Debug, Queryable, Identifiable, Associations)] // AsChangeset
#[table_name = "container_rev"]
pub struct ContainerRevRow {
    pub id: i64,
    //extra_json: Option<Json>,
    pub name: Option<String>,
    pub parent_ident_id: Option<i64>,
    pub publisher: Option<String>,
    pub issn: Option<String>,
}

/*
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

table! {
    file_edit (id) {
        id -> Int8,
        extra_json -> Nullable<Json>,
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
    file_release (id) {
        id -> Int8,
        file_rev -> Int8,
        target_release_ident_id -> Uuid,
    }
}

table! {
    file_rev (id) {
        id -> Int8,
        extra_json -> Nullable<Json>,
        size -> Nullable<Int4>,
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
        extra_json -> Nullable<Json>,
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
        index -> Nullable<Int4>,
        stub -> Nullable<Text>,
    }
}

table! {
    release_rev (id) {
        id -> Int8,
        extra_json -> Nullable<Json>,
        work_ident_id -> Nullable<Uuid>,
        container_ident_id -> Nullable<Uuid>,
        title -> Nullable<Text>,
        license -> Nullable<Text>,
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
        extra_json -> Nullable<Json>,
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
        extra_json -> Nullable<Json>,
        work_type -> Nullable<Text>,
        primary_release_id -> Nullable<Uuid>,
    }
}
*/
