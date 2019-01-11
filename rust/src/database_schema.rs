table! {
    abstracts (sha1) {
        sha1 -> Text,
        content -> Text,
    }
}

table! {
    auth_oidc (id) {
        id -> Int8,
        created -> Timestamptz,
        editor_id -> Uuid,
        provider -> Text,
        oidc_iss -> Text,
        oidc_sub -> Text,
    }
}

table! {
    changelog (id) {
        id -> Int8,
        editgroup_id -> Uuid,
        timestamp -> Timestamptz,
    }
}

table! {
    container_edit (id) {
        id -> Uuid,
        editgroup_id -> Uuid,
        updated -> Timestamptz,
        ident_id -> Uuid,
        rev_id -> Nullable<Uuid>,
        redirect_id -> Nullable<Uuid>,
        prev_rev -> Nullable<Uuid>,
        extra_json -> Nullable<Jsonb>,
    }
}

table! {
    container_ident (id) {
        id -> Uuid,
        is_live -> Bool,
        rev_id -> Nullable<Uuid>,
        redirect_id -> Nullable<Uuid>,
    }
}

table! {
    container_rev (id) {
        id -> Uuid,
        extra_json -> Nullable<Jsonb>,
        name -> Text,
        publisher -> Nullable<Text>,
        issnl -> Nullable<Text>,
        wikidata_qid -> Nullable<Text>,
        abbrev -> Nullable<Text>,
        coden -> Nullable<Text>,
    }
}

table! {
    creator_edit (id) {
        id -> Uuid,
        editgroup_id -> Uuid,
        updated -> Timestamptz,
        ident_id -> Uuid,
        rev_id -> Nullable<Uuid>,
        redirect_id -> Nullable<Uuid>,
        prev_rev -> Nullable<Uuid>,
        extra_json -> Nullable<Jsonb>,
    }
}

table! {
    creator_ident (id) {
        id -> Uuid,
        is_live -> Bool,
        rev_id -> Nullable<Uuid>,
        redirect_id -> Nullable<Uuid>,
    }
}

table! {
    creator_rev (id) {
        id -> Uuid,
        extra_json -> Nullable<Jsonb>,
        display_name -> Text,
        given_name -> Nullable<Text>,
        surname -> Nullable<Text>,
        orcid -> Nullable<Text>,
        wikidata_qid -> Nullable<Text>,
    }
}

table! {
    editgroup (id) {
        id -> Uuid,
        editor_id -> Uuid,
        created -> Timestamptz,
        submitted -> Nullable<Timestamptz>,
        is_accepted -> Bool,
        description -> Nullable<Text>,
        extra_json -> Nullable<Jsonb>,
    }
}

table! {
    editgroup_annotation (id) {
        id -> Uuid,
        editgroup_id -> Uuid,
        editor_id -> Uuid,
        created -> Timestamptz,
        comment_markdown -> Nullable<Text>,
        extra_json -> Nullable<Jsonb>,
    }
}

table! {
    editor (id) {
        id -> Uuid,
        username -> Text,
        is_superuser -> Bool,
        is_admin -> Bool,
        is_bot -> Bool,
        is_active -> Bool,
        registered -> Timestamptz,
        auth_epoch -> Timestamptz,
        wrangler_id -> Nullable<Uuid>,
    }
}

table! {
    file_edit (id) {
        id -> Uuid,
        editgroup_id -> Uuid,
        updated -> Timestamptz,
        ident_id -> Uuid,
        rev_id -> Nullable<Uuid>,
        redirect_id -> Nullable<Uuid>,
        prev_rev -> Nullable<Uuid>,
        extra_json -> Nullable<Jsonb>,
    }
}

table! {
    file_ident (id) {
        id -> Uuid,
        is_live -> Bool,
        rev_id -> Nullable<Uuid>,
        redirect_id -> Nullable<Uuid>,
    }
}

table! {
    file_rev (id) {
        id -> Uuid,
        extra_json -> Nullable<Jsonb>,
        size_bytes -> Nullable<Int8>,
        sha1 -> Nullable<Text>,
        sha256 -> Nullable<Text>,
        md5 -> Nullable<Text>,
        mimetype -> Nullable<Text>,
    }
}

table! {
    file_rev_release (file_rev, target_release_ident_id) {
        file_rev -> Uuid,
        target_release_ident_id -> Uuid,
    }
}

table! {
    file_rev_url (id) {
        id -> Int8,
        file_rev -> Uuid,
        rel -> Text,
        url -> Text,
    }
}

table! {
    fileset_edit (id) {
        id -> Uuid,
        editgroup_id -> Uuid,
        updated -> Timestamptz,
        ident_id -> Uuid,
        rev_id -> Nullable<Uuid>,
        redirect_id -> Nullable<Uuid>,
        prev_rev -> Nullable<Uuid>,
        extra_json -> Nullable<Jsonb>,
    }
}

table! {
    fileset_ident (id) {
        id -> Uuid,
        is_live -> Bool,
        rev_id -> Nullable<Uuid>,
        redirect_id -> Nullable<Uuid>,
    }
}

table! {
    fileset_rev (id) {
        id -> Uuid,
        extra_json -> Nullable<Jsonb>,
    }
}

table! {
    fileset_rev_file (id) {
        id -> Int8,
        fileset_rev -> Uuid,
        path_name -> Text,
        size_bytes -> Int8,
        md5 -> Nullable<Text>,
        sha1 -> Nullable<Text>,
        sha256 -> Nullable<Text>,
        extra_json -> Nullable<Jsonb>,
    }
}

table! {
    fileset_rev_release (fileset_rev, target_release_ident_id) {
        fileset_rev -> Uuid,
        target_release_ident_id -> Uuid,
    }
}

table! {
    fileset_rev_url (id) {
        id -> Int8,
        fileset_rev -> Uuid,
        rel -> Text,
        url -> Text,
    }
}

table! {
    release_contrib (id) {
        id -> Int8,
        release_rev -> Uuid,
        creator_ident_id -> Nullable<Uuid>,
        raw_name -> Nullable<Text>,
        role -> Nullable<Text>,
        index_val -> Nullable<Int4>,
        extra_json -> Nullable<Jsonb>,
    }
}

table! {
    release_edit (id) {
        id -> Uuid,
        editgroup_id -> Uuid,
        updated -> Timestamptz,
        ident_id -> Uuid,
        rev_id -> Nullable<Uuid>,
        redirect_id -> Nullable<Uuid>,
        prev_rev -> Nullable<Uuid>,
        extra_json -> Nullable<Jsonb>,
    }
}

table! {
    release_ident (id) {
        id -> Uuid,
        is_live -> Bool,
        rev_id -> Nullable<Uuid>,
        redirect_id -> Nullable<Uuid>,
    }
}

table! {
    release_ref (id) {
        id -> Int8,
        release_rev -> Uuid,
        target_release_ident_id -> Nullable<Uuid>,
        index_val -> Nullable<Int4>,
        key -> Nullable<Text>,
        extra_json -> Nullable<Jsonb>,
        container_name -> Nullable<Text>,
        year -> Nullable<Int4>,
        title -> Nullable<Text>,
        locator -> Nullable<Text>,
    }
}

table! {
    release_rev (id) {
        id -> Uuid,
        extra_json -> Nullable<Jsonb>,
        work_ident_id -> Uuid,
        container_ident_id -> Nullable<Uuid>,
        title -> Text,
        release_type -> Nullable<Text>,
        release_status -> Nullable<Text>,
        release_date -> Nullable<Date>,
        release_year -> Nullable<Int8>,
        doi -> Nullable<Text>,
        pmid -> Nullable<Text>,
        pmcid -> Nullable<Text>,
        wikidata_qid -> Nullable<Text>,
        isbn13 -> Nullable<Text>,
        core_id -> Nullable<Text>,
        volume -> Nullable<Text>,
        issue -> Nullable<Text>,
        pages -> Nullable<Text>,
        publisher -> Nullable<Text>,
        language -> Nullable<Text>,
    }
}

table! {
    release_rev_abstract (id) {
        id -> Int8,
        release_rev -> Uuid,
        abstract_sha1 -> Text,
        mimetype -> Nullable<Text>,
        lang -> Nullable<Text>,
    }
}

table! {
    webcapture_edit (id) {
        id -> Uuid,
        editgroup_id -> Uuid,
        updated -> Timestamptz,
        ident_id -> Uuid,
        rev_id -> Nullable<Uuid>,
        redirect_id -> Nullable<Uuid>,
        prev_rev -> Nullable<Uuid>,
        extra_json -> Nullable<Jsonb>,
    }
}

table! {
    webcapture_ident (id) {
        id -> Uuid,
        is_live -> Bool,
        rev_id -> Nullable<Uuid>,
        redirect_id -> Nullable<Uuid>,
    }
}

table! {
    webcapture_rev (id) {
        id -> Uuid,
        extra_json -> Nullable<Jsonb>,
        original_url -> Text,
        timestamp -> Timestamptz,
    }
}

table! {
    webcapture_rev_cdx (id) {
        id -> Int8,
        webcapture_rev -> Uuid,
        surt -> Text,
        timestamp -> Text,
        url -> Text,
        mimetype -> Nullable<Text>,
        status_code -> Nullable<Int8>,
        sha1 -> Text,
        sha256 -> Nullable<Text>,
    }
}

table! {
    webcapture_rev_release (webcapture_rev, target_release_ident_id) {
        webcapture_rev -> Uuid,
        target_release_ident_id -> Uuid,
    }
}

table! {
    webcapture_rev_url (id) {
        id -> Int8,
        webcapture_rev -> Uuid,
        rel -> Text,
        url -> Text,
    }
}

table! {
    work_edit (id) {
        id -> Uuid,
        editgroup_id -> Uuid,
        updated -> Timestamptz,
        ident_id -> Uuid,
        rev_id -> Nullable<Uuid>,
        redirect_id -> Nullable<Uuid>,
        prev_rev -> Nullable<Uuid>,
        extra_json -> Nullable<Jsonb>,
    }
}

table! {
    work_ident (id) {
        id -> Uuid,
        is_live -> Bool,
        rev_id -> Nullable<Uuid>,
        redirect_id -> Nullable<Uuid>,
    }
}

table! {
    work_rev (id) {
        id -> Uuid,
        extra_json -> Nullable<Jsonb>,
    }
}

joinable!(auth_oidc -> editor (editor_id));
joinable!(changelog -> editgroup (editgroup_id));
joinable!(container_edit -> editgroup (editgroup_id));
joinable!(container_ident -> container_rev (rev_id));
joinable!(creator_edit -> editgroup (editgroup_id));
joinable!(creator_ident -> creator_rev (rev_id));
joinable!(editgroup -> editor (editor_id));
joinable!(editgroup_annotation -> editgroup (editgroup_id));
joinable!(editgroup_annotation -> editor (editor_id));
joinable!(file_edit -> editgroup (editgroup_id));
joinable!(file_ident -> file_rev (rev_id));
joinable!(file_rev_release -> file_rev (file_rev));
joinable!(file_rev_release -> release_ident (target_release_ident_id));
joinable!(file_rev_url -> file_rev (file_rev));
joinable!(fileset_edit -> editgroup (editgroup_id));
joinable!(fileset_ident -> fileset_rev (rev_id));
joinable!(fileset_rev_file -> fileset_rev (fileset_rev));
joinable!(fileset_rev_release -> fileset_rev (fileset_rev));
joinable!(fileset_rev_release -> release_ident (target_release_ident_id));
joinable!(fileset_rev_url -> fileset_rev (fileset_rev));
joinable!(release_contrib -> creator_ident (creator_ident_id));
joinable!(release_contrib -> release_rev (release_rev));
joinable!(release_edit -> editgroup (editgroup_id));
joinable!(release_ident -> release_rev (rev_id));
joinable!(release_ref -> release_ident (target_release_ident_id));
joinable!(release_ref -> release_rev (release_rev));
joinable!(release_rev -> container_ident (container_ident_id));
joinable!(release_rev -> work_ident (work_ident_id));
joinable!(release_rev_abstract -> abstracts (abstract_sha1));
joinable!(release_rev_abstract -> release_rev (release_rev));
joinable!(webcapture_edit -> editgroup (editgroup_id));
joinable!(webcapture_ident -> webcapture_rev (rev_id));
joinable!(webcapture_rev_cdx -> webcapture_rev (webcapture_rev));
joinable!(webcapture_rev_release -> release_ident (target_release_ident_id));
joinable!(webcapture_rev_release -> webcapture_rev (webcapture_rev));
joinable!(webcapture_rev_url -> webcapture_rev (webcapture_rev));
joinable!(work_edit -> editgroup (editgroup_id));
joinable!(work_ident -> work_rev (rev_id));

allow_tables_to_appear_in_same_query!(
    abstracts,
    auth_oidc,
    changelog,
    container_edit,
    container_ident,
    container_rev,
    creator_edit,
    creator_ident,
    creator_rev,
    editgroup,
    editgroup_annotation,
    editor,
    file_edit,
    file_ident,
    file_rev,
    file_rev_release,
    file_rev_url,
    fileset_edit,
    fileset_ident,
    fileset_rev,
    fileset_rev_file,
    fileset_rev_release,
    fileset_rev_url,
    release_contrib,
    release_edit,
    release_ident,
    release_ref,
    release_rev,
    release_rev_abstract,
    webcapture_edit,
    webcapture_ident,
    webcapture_rev,
    webcapture_rev_cdx,
    webcapture_rev_release,
    webcapture_rev_url,
    work_edit,
    work_ident,
    work_rev,
);
