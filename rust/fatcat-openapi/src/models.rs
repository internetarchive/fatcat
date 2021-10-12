#![allow(unused_imports, unused_qualifications, unused_extern_crates)]
extern crate chrono;
extern crate serde_json;
extern crate uuid;

use serde::ser::Serializer;

use crate::models;
use std::collections::HashMap;
use swagger;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthOidc {
    /// Fatcat-specific short name (slug) for remote service being used for authentication.
    #[serde(rename = "provider")]
    pub provider: String,

    /// `SUB` from OIDC protocol. Usually a URL.
    #[serde(rename = "sub")]
    pub sub: String,

    /// `ISS` from OIDC protocol. Usually a stable account username, number, or identifier.
    #[serde(rename = "iss")]
    pub iss: String,

    /// What it sounds like; returned by OIDC, and used as a hint when creating new editor accounts. Fatcat usernames are usually this string with the `provider` slug as a suffix, though some munging may occur.
    #[serde(rename = "preferred_username")]
    pub preferred_username: String,
}

impl AuthOidc {
    pub fn new(provider: String, sub: String, iss: String, preferred_username: String) -> AuthOidc {
        AuthOidc {
            provider: provider,
            sub: sub,
            iss: iss,
            preferred_username: preferred_username,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthOidcResult {
    #[serde(rename = "editor")]
    pub editor: models::Editor,

    #[serde(rename = "token")]
    pub token: String,
}

impl AuthOidcResult {
    pub fn new(editor: models::Editor, token: String) -> AuthOidcResult {
        AuthOidcResult { editor: editor, token: token }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthTokenResult {
    #[serde(rename = "token")]
    pub token: String,
}

impl AuthTokenResult {
    pub fn new(token: String) -> AuthTokenResult {
        AuthTokenResult { token: token }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangelogEntry {
    /// Monotonically increasing sequence number of this changelog entry.
    #[serde(rename = "index")]
    pub index: i64,

    /// Identifier of editgroup accepted/merged in this changelog entry.
    #[serde(rename = "editgroup_id")]
    pub editgroup_id: String,

    /// Date and time when the editgroup was accpeted.
    #[serde(rename = "timestamp")]
    pub timestamp: chrono::DateTime<chrono::Utc>,

    #[serde(rename = "editgroup")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup: Option<models::Editgroup>,
}

impl ChangelogEntry {
    pub fn new(index: i64, editgroup_id: String, timestamp: chrono::DateTime<chrono::Utc>) -> ChangelogEntry {
        ChangelogEntry {
            index: index,
            editgroup_id: editgroup_id,
            timestamp: timestamp,
            editgroup: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerAutoBatch {
    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "entity_list")]
    pub entity_list: Vec<models::ContainerEntity>,
}

impl ContainerAutoBatch {
    pub fn new(editgroup: models::Editgroup, entity_list: Vec<models::ContainerEntity>) -> ContainerAutoBatch {
        ContainerAutoBatch {
            editgroup: editgroup,
            entity_list: entity_list,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerEntity {
    #[serde(rename = "wikidata_qid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wikidata_qid: Option<String>,

    /// Print ISSN number (ISSN-P). Should be valid and registered with issn.org
    #[serde(rename = "issnp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issnp: Option<String>,

    /// Electronic ISSN number (ISSN-E). Should be valid and registered with issn.org
    #[serde(rename = "issne")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issne: Option<String>,

    /// Linking ISSN number (ISSN-L). Should be valid and registered with issn.org
    #[serde(rename = "issnl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issnl: Option<String>,

    /// Name of the organization or entity responsible for publication. Not the complete imprint/brand.
    #[serde(rename = "publisher")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    /// Whether the container is active, discontinued, etc
    #[serde(rename = "publication_status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_status: Option<String>,

    /// Type of container, eg 'journal' or 'proceedings'. See Guide for list of valid types.
    #[serde(rename = "container_type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_type: Option<String>,

    /// Name of the container (eg, Journal title). Required for entity creation.
    #[serde(rename = "name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).
    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<serde_json::Value>,

    /// Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    /// base32-encoded unique identifier
    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    // Note: inline enums are not fully supported by swagger-codegen
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

impl ContainerEntity {
    pub fn new() -> ContainerEntity {
        ContainerEntity {
            wikidata_qid: None,
            issnp: None,
            issne: None,
            issnl: None,
            publisher: None,
            publication_status: None,
            container_type: None,
            name: None,
            edit_extra: None,
            extra: None,
            redirect: None,
            revision: None,
            ident: None,
            state: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatorAutoBatch {
    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "entity_list")]
    pub entity_list: Vec<models::CreatorEntity>,
}

impl CreatorAutoBatch {
    pub fn new(editgroup: models::Editgroup, entity_list: Vec<models::CreatorEntity>) -> CreatorAutoBatch {
        CreatorAutoBatch {
            editgroup: editgroup,
            entity_list: entity_list,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatorEntity {
    /// Wikidata entity QID
    #[serde(rename = "wikidata_qid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wikidata_qid: Option<String>,

    /// ORCiD (https://orcid.org) identifier
    #[serde(rename = "orcid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orcid: Option<String>,

    /// In English commonly the last, or family name, but ordering is context and culture specific.
    #[serde(rename = "surname")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surname: Option<String>,

    /// In English commonly the first name, but ordering is context and culture specific.
    #[serde(rename = "given_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,

    /// Name as should be displayed in web interface or in author lists (not index/sorted). Required for valid entities.
    #[serde(rename = "display_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    // Note: inline enums are not fully supported by swagger-codegen
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    /// Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    /// Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).
    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<serde_json::Value>,
}

impl CreatorEntity {
    pub fn new() -> CreatorEntity {
        CreatorEntity {
            wikidata_qid: None,
            orcid: None,
            surname: None,
            given_name: None,
            display_name: None,
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            extra: None,
            edit_extra: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Editgroup {
    /// Fatcat identifier for this editgroup. Assigned on creation.
    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<String>,

    /// Fatcat identifer of editor that created this editgroup.
    #[serde(rename = "editor_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_id: Option<String>,

    /// Complete editor object identified by `container_id` field. Only included in GET responses.
    #[serde(rename = "editor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<models::Editor>,

    /// For accepted/merged editgroups, the changelog index that the accept occured at. WARNING: not populated in all contexts that an editgroup could be included in a response.
    #[serde(rename = "changelog_index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changelog_index: Option<i64>,

    /// Timestamp when this editgroup was first created.
    #[serde(rename = "created")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,

    /// Timestamp when this editgroup was most recently submitted for review. If withdrawn, or never submitted, will be `null`.
    #[serde(rename = "submitted")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted: Option<chrono::DateTime<chrono::Utc>>,

    /// Comment describing the changes in this editgroup. Can be updated with PUT request.
    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Free-form JSON metadata attached to this editgroup. Eg, metadata provenance, or script user-agent details. See guide for (unenforced) schema norms.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    /// Only included in GET responses, and not in all contexts. Do not include this field in PUT or POST requests.
    #[serde(rename = "annotations")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<models::EditgroupAnnotation>>,

    #[serde(rename = "edits")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edits: Option<models::EditgroupEdits>,
}

impl Editgroup {
    pub fn new() -> Editgroup {
        Editgroup {
            editgroup_id: None,
            editor_id: None,
            editor: None,
            changelog_index: None,
            created: None,
            submitted: None,
            description: None,
            extra: None,
            annotations: None,
            edits: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditgroupAnnotation {
    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "annotation_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_id: Option<String>,

    /// Editgroup that this annotation applies to. Set automatically in creations based on URL parameter.
    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<String>,

    /// Defaults to editor created the annotation via POST request.
    #[serde(rename = "editor_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_id: Option<String>,

    /// Only included in GET responses; ignored in PUT or POST requests.
    #[serde(rename = "editor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<models::Editor>,

    /// Timestamp when annotation was first created.
    #[serde(rename = "created")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(rename = "comment_markdown")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_markdown: Option<String>,

    /// Additional free-form JSON metadata that can be included as part of the annotation (or even as the primary annotation itself). See guide for details.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

impl EditgroupAnnotation {
    pub fn new() -> EditgroupAnnotation {
        EditgroupAnnotation {
            annotation_id: None,
            editgroup_id: None,
            editor_id: None,
            editor: None,
            created: None,
            comment_markdown: None,
            extra: None,
        }
    }
}

/// Only included in GET responses, and not in all contexts. Do not include this field in PUT or POST requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditgroupEdits {
    #[serde(rename = "containers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub containers: Option<Vec<models::EntityEdit>>,

    #[serde(rename = "creators")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creators: Option<Vec<models::EntityEdit>>,

    #[serde(rename = "files")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<models::EntityEdit>>,

    #[serde(rename = "filesets")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesets: Option<Vec<models::EntityEdit>>,

    #[serde(rename = "webcaptures")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webcaptures: Option<Vec<models::EntityEdit>>,

    #[serde(rename = "releases")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releases: Option<Vec<models::EntityEdit>>,

    #[serde(rename = "works")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub works: Option<Vec<models::EntityEdit>>,
}

impl EditgroupEdits {
    pub fn new() -> EditgroupEdits {
        EditgroupEdits {
            containers: None,
            creators: None,
            files: None,
            filesets: None,
            webcaptures: None,
            releases: None,
            works: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Editor {
    /// Fatcat identifier for the editor. Can not be changed.
    #[serde(rename = "editor_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_id: Option<String>,

    /// Username/handle (short slug-like string) to identify this editor. May be changed at any time by the editor; use the `editor_id` as a persistend identifer.
    #[serde(rename = "username")]
    pub username: String,

    /// Whether this editor has the `admin` role.
    #[serde(rename = "is_admin")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_admin: Option<bool>,

    /// Whether this editor is a bot (as opposed to a human making manual edits)
    #[serde(rename = "is_bot")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_bot: Option<bool>,

    /// Whether this editor's account is enabled (if not API tokens and web logins will not work).
    #[serde(rename = "is_active")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

impl Editor {
    pub fn new(username: String) -> Editor {
        Editor {
            editor_id: None,
            username: username,
            is_admin: None,
            is_bot: None,
            is_active: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityEdit {
    /// Unique UUID for this specific edit object.
    #[serde(rename = "edit_id")]
    pub edit_id: String,

    /// Fatcat identifier of the entity this edit is mutating.
    #[serde(rename = "ident")]
    pub ident: String,

    /// Entity revision that this edit will set the entity to. May be `null` in the case of deletions.
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// Revision of entity just before this edit. May be used in the future to prevent edit race conditions.
    #[serde(rename = "prev_revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_revision: Option<String>,

    /// When an edit is to merge entities (redirect one to another), this is the entity fatcat identifier for the target entity.
    #[serde(rename = "redirect_ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_ident: Option<String>,

    /// Editgroup identifier that this edit is part of.
    #[serde(rename = "editgroup_id")]
    pub editgroup_id: String,

    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

impl EntityEdit {
    pub fn new(edit_id: String, ident: String, editgroup_id: String) -> EntityEdit {
        EntityEdit {
            edit_id: edit_id,
            ident: ident,
            revision: None,
            prev_revision: None,
            redirect_ident: None,
            editgroup_id: editgroup_id,
            extra: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityHistoryEntry {
    #[serde(rename = "edit")]
    pub edit: models::EntityEdit,

    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "changelog_entry")]
    pub changelog_entry: models::ChangelogEntry,
}

impl EntityHistoryEntry {
    pub fn new(edit: models::EntityEdit, editgroup: models::Editgroup, changelog_entry: models::ChangelogEntry) -> EntityHistoryEntry {
        EntityHistoryEntry {
            edit: edit,
            editgroup: editgroup,
            changelog_entry: changelog_entry,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorResponse {
    #[serde(rename = "success")]
    pub success: bool,

    #[serde(rename = "error")]
    pub error: String,

    #[serde(rename = "message")]
    pub message: String,
}

impl ErrorResponse {
    pub fn new(success: bool, error: String, message: String) -> ErrorResponse {
        ErrorResponse {
            success: success,
            error: error,
            message: message,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileAutoBatch {
    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "entity_list")]
    pub entity_list: Vec<models::FileEntity>,
}

impl FileAutoBatch {
    pub fn new(editgroup: models::Editgroup, entity_list: Vec<models::FileEntity>) -> FileAutoBatch {
        FileAutoBatch {
            editgroup: editgroup,
            entity_list: entity_list,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileEntity {
    /// Full release entities, included in GET responses when `releases` included in `expand` parameter. Ignored if included in PUT or POST requests.
    #[serde(rename = "releases")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releases: Option<Vec<models::ReleaseEntity>>,

    /// Set of identifier of release entities this file represents a full manifestation of. Usually a single release, but some files contain content of multiple full releases (eg, an issue of a journal).
    #[serde(rename = "release_ids")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_ids: Option<Vec<String>>,

    #[serde(rename = "mimetype")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    #[serde(rename = "urls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<models::FileUrl>>,

    /// SHA-256 hash of data, in hex encoding
    #[serde(rename = "sha256")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,

    /// SHA-1 hash of data, in hex encoding
    #[serde(rename = "sha1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    /// MD5 hash of data, in hex encoding
    #[serde(rename = "md5")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,

    /// Size of file in bytes. Non-zero.
    #[serde(rename = "size")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    /// Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).
    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<serde_json::Value>,

    /// Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    /// base32-encoded unique identifier
    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    // Note: inline enums are not fully supported by swagger-codegen
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

impl FileEntity {
    pub fn new() -> FileEntity {
        FileEntity {
            releases: None,
            release_ids: None,
            mimetype: None,
            urls: None,
            sha256: None,
            sha1: None,
            md5: None,
            size: None,
            edit_extra: None,
            extra: None,
            redirect: None,
            revision: None,
            ident: None,
            state: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileUrl {
    /// URL/URI pointing directly to a machine retrievable copy of this exact file.
    #[serde(rename = "url")]
    pub url: String,

    /// Indicates type of host this URL points to. Eg, \"publisher\", \"repository\", \"webarchive\". See guide for list of acceptable values.
    #[serde(rename = "rel")]
    pub rel: String,
}

impl FileUrl {
    pub fn new(url: String, rel: String) -> FileUrl {
        FileUrl { url: url, rel: rel }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilesetAutoBatch {
    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "entity_list")]
    pub entity_list: Vec<models::FilesetEntity>,
}

impl FilesetAutoBatch {
    pub fn new(editgroup: models::Editgroup, entity_list: Vec<models::FilesetEntity>) -> FilesetAutoBatch {
        FilesetAutoBatch {
            editgroup: editgroup,
            entity_list: entity_list,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilesetEntity {
    /// Full release entities, included in GET responses when `releases` included in `expand` parameter. Ignored if included in PUT or POST requests.
    #[serde(rename = "releases")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releases: Option<Vec<models::ReleaseEntity>>,

    /// Set of identifier of release entities this fileset represents a full manifestation of. Usually a single release.
    #[serde(rename = "release_ids")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_ids: Option<Vec<String>>,

    #[serde(rename = "urls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<models::FilesetUrl>>,

    #[serde(rename = "manifest")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest: Option<Vec<models::FilesetFile>>,

    // Note: inline enums are not fully supported by swagger-codegen
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    /// Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    /// Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).
    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<serde_json::Value>,
}

impl FilesetEntity {
    pub fn new() -> FilesetEntity {
        FilesetEntity {
            releases: None,
            release_ids: None,
            urls: None,
            manifest: None,
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            extra: None,
            edit_extra: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilesetFile {
    /// Path name of file within this fileset (eg, directory)
    #[serde(rename = "path")]
    pub path: String,

    /// File size in bytes
    #[serde(rename = "size")]
    pub size: i64,

    /// MD5 hash of data, in hex encoding
    #[serde(rename = "md5")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,

    /// SHA-1 hash of data, in hex encoding
    #[serde(rename = "sha1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    /// SHA-256 hash of data, in hex encoding
    #[serde(rename = "sha256")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,

    #[serde(rename = "mimetype")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// Free-form additional metadata about this specific file in the set. Eg, `original_url`. See guide for nomative (but unenforced) schema fields.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

impl FilesetFile {
    pub fn new(path: String, size: i64) -> FilesetFile {
        FilesetFile {
            path: path,
            size: size,
            md5: None,
            sha1: None,
            sha256: None,
            mimetype: None,
            extra: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilesetUrl {
    #[serde(rename = "url")]
    pub url: String,

    /// Indicates type of host this URL points to. See guide for list of acceptable values.
    #[serde(rename = "rel")]
    pub rel: String,
}

impl FilesetUrl {
    pub fn new(url: String, rel: String) -> FilesetUrl {
        FilesetUrl { url: url, rel: rel }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseAbstract {
    /// SHA-1 hash of data, in hex encoding
    #[serde(rename = "sha1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    /// Abstract content. May be encoded, as per `mimetype` field, but only string/text content may be included.
    #[serde(rename = "content")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Mimetype of abstract contents. `text/plain` is the default if content isn't encoded.
    #[serde(rename = "mimetype")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// ISO language code of the abstract. Same semantics as release `language` field.
    #[serde(rename = "lang")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

impl ReleaseAbstract {
    pub fn new() -> ReleaseAbstract {
        ReleaseAbstract {
            sha1: None,
            content: None,
            mimetype: None,
            lang: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseAutoBatch {
    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "entity_list")]
    pub entity_list: Vec<models::ReleaseEntity>,
}

impl ReleaseAutoBatch {
    pub fn new(editgroup: models::Editgroup, entity_list: Vec<models::ReleaseEntity>) -> ReleaseAutoBatch {
        ReleaseAutoBatch {
            editgroup: editgroup,
            entity_list: entity_list,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseContrib {
    /// Internally assigned zero-indexed sequence number of contribution. Authors should come first; this encodes the order of attriubtion.
    #[serde(rename = "index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,

    /// If known, indicates the creator entity this contribution was made by.
    #[serde(rename = "creator_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<String>,

    /// Complete creator entity. Only returned in GET responses, and only if `contribs` included in the `expand` query parameter.
    #[serde(rename = "creator")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<models::CreatorEntity>,

    /// Full name of the contributor as typeset in the release.
    #[serde(rename = "raw_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_name: Option<String>,

    /// In English commonly the first name, but ordering is context and culture specific.
    #[serde(rename = "given_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,

    /// In English commonly the last, or family name, but ordering is context and culture specific.
    #[serde(rename = "surname")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surname: Option<String>,

    /// Short string (slug) indicating type of contribution (eg, \"author\", \"translator\"). See guide for list of accpeted values.
    #[serde(rename = "role")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Raw affiliation string as displayed in text
    #[serde(rename = "raw_affiliation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_affiliation: Option<String>,

    /// Additional free-form JSON metadata about this contributor/contribution. See guide for normative schema.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

impl ReleaseContrib {
    pub fn new() -> ReleaseContrib {
        ReleaseContrib {
            index: None,
            creator_id: None,
            creator: None,
            raw_name: None,
            given_name: None,
            surname: None,
            role: None,
            raw_affiliation: None,
            extra: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseEntity {
    #[serde(rename = "abstracts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abstracts: Option<Vec<models::ReleaseAbstract>>,

    #[serde(rename = "refs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refs: Option<Vec<models::ReleaseRef>>,

    #[serde(rename = "contribs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contribs: Option<Vec<models::ReleaseContrib>>,

    /// Short string (slug) name of license under which release is openly published (if applicable).
    #[serde(rename = "license_slug")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_slug: Option<String>,

    /// Primary language of the content of the full release. Two-letter RFC1766/ISO639-1 language code, with some custom extensions/additions. See guide.
    #[serde(rename = "language")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Name, usually English, of the entity or institution responsible for publication of this release. Not necessarily the imprint/brand. See guide.
    #[serde(rename = "publisher")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    /// For, eg, updated technical reports or software packages, where the version string may be the only field disambiguating between releases.
    #[serde(rename = "version")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// For, eg, technical reports, which are published in series or assigned some other institutional or container-specific identifier.
    #[serde(rename = "number")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,

    /// Either a single page number (\"first page\") or a range of pages separated by a dash (\"-\"). See guide for details.
    #[serde(rename = "pages")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<String>,

    /// Issue number of volume/container that this release was published in. Sometimes coresponds to a month number in the year, but can be any string. See guide.
    #[serde(rename = "issue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,

    /// Volume number of container that this release was published in. Often corresponds to the \"Nth\" year of publication, but can be any string. See guide.
    #[serde(rename = "volume")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,

    /// Set of external identifiers for this release.
    #[serde(rename = "ext_ids")]
    pub ext_ids: models::ReleaseExtIds,

    /// Year corresponding with `withdrawn_date` like `release_year`/`release_date`.
    #[serde(rename = "withdrawn_year")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub withdrawn_year: Option<i64>,

    /// Full date when this release was formally withdrawn (if applicable). ISO format, like `release_date`.
    #[serde(rename = "withdrawn_date")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub withdrawn_date: Option<chrono::NaiveDate>,

    /// Type of withdrawl or retraction of this release, if applicable. If release has not been withdrawn, should be `null` (aka, not set, not the string \"null\" or an empty string).
    #[serde(rename = "withdrawn_status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub withdrawn_status: Option<String>,

    /// Year when this release was formally published. Must match `release_date` if that field is set; this field exists because sometimes only the year is known.
    #[serde(rename = "release_year")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_year: Option<i64>,

    /// Full date when this release was formally published. ISO format, like `2019-03-05`. See guide for semantics.
    #[serde(rename = "release_date")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<chrono::NaiveDate>,

    /// The stage of publication of this specific release. See guide for valid values and semantics.
    #[serde(rename = "release_stage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_stage: Option<String>,

    /// \"Type\" or \"medium\" that this release is published as. See guide for valid values.
    #[serde(rename = "release_type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_type: Option<String>,

    /// Used to link this release to a container entity that the release was published as part of.
    #[serde(rename = "container_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,

    /// Complete webcapture entities identified by `webcapture_ids` field. Only included in GET responses when `webcaptures` included in `expand` parameter; ignored in PUT or POST requests.
    #[serde(rename = "webcaptures")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webcaptures: Option<Vec<models::WebcaptureEntity>>,

    /// Complete file entities identified by `filesets_ids` field. Only included in GET responses when `filesets` included in `expand` parameter; ignored in PUT or POST requests.
    #[serde(rename = "filesets")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesets: Option<Vec<models::FilesetEntity>>,

    /// Complete file entities identified by `file_ids` field. Only included in GET responses when `files` included in `expand` parameter; ignored in PUT or POST requests.
    #[serde(rename = "files")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<models::FileEntity>>,

    /// Complete container entity identified by `container_id` field. Only included in GET reponses when `container` included in `expand` parameter; ignored in PUT or POST requests.
    #[serde(rename = "container")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<models::ContainerEntity>,

    /// Identifier of work this release is part of. In creation (POST) requests, a work entity will be created automatically if this field is not set.
    #[serde(rename = "work_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_id: Option<String>,

    /// Title in original language if `title` field has been translated. See guide for details.
    #[serde(rename = "original_title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_title: Option<String>,

    /// Subtitle of release. In many cases, better to merge with title than include as separate field (unless combined title would be very long). See guide for details.
    #[serde(rename = "subtitle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,

    /// Required for valid entities. The title used in citations and for display. Sometimes the English translation of title e even if release content is not English.
    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    // Note: inline enums are not fully supported by swagger-codegen
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    /// Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    /// Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).
    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<serde_json::Value>,
}

impl ReleaseEntity {
    pub fn new(ext_ids: models::ReleaseExtIds) -> ReleaseEntity {
        ReleaseEntity {
            abstracts: None,
            refs: None,
            contribs: None,
            license_slug: None,
            language: None,
            publisher: None,
            version: None,
            number: None,
            pages: None,
            issue: None,
            volume: None,
            ext_ids: ext_ids,
            withdrawn_year: None,
            withdrawn_date: None,
            withdrawn_status: None,
            release_year: None,
            release_date: None,
            release_stage: None,
            release_type: None,
            container_id: None,
            webcaptures: None,
            filesets: None,
            files: None,
            container: None,
            work_id: None,
            original_title: None,
            subtitle: None,
            title: None,
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            extra: None,
            edit_extra: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseExtIds {
    /// Digital serde_json::Value Identifier (DOI), mostly for published papers and datasets. Should be registered and resolvable via https://doi.org/
    #[serde(rename = "doi")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,

    /// Wikidata entity QID
    #[serde(rename = "wikidata_qid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wikidata_qid: Option<String>,

    /// ISBN-13, for books. Usually not set for chapters. ISBN-10 should be converted to ISBN-13.
    #[serde(rename = "isbn13")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isbn13: Option<String>,

    /// PubMed Identifier
    #[serde(rename = "pmid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,

    /// PubMed Central Identifier
    #[serde(rename = "pmcid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<String>,

    /// CORE (https://core.ac.uk) identifier
    #[serde(rename = "core")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub core: Option<String>,

    /// arXiv (https://arxiv.org) identifier; must include version
    #[serde(rename = "arxiv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arxiv: Option<String>,

    /// JSTOR work identifier
    #[serde(rename = "jstor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jstor: Option<String>,

    /// ARK identifier
    #[serde(rename = "ark")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ark: Option<String>,

    /// Microsoft Academic Graph identifier
    #[serde(rename = "mag")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mag: Option<String>,

    /// DOAJ article-level identifier
    #[serde(rename = "doaj")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doaj: Option<String>,

    /// dblp (https://dblp.uni-trier.de/) paper identifier; eg for conference proceedings
    #[serde(rename = "dblp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dblp: Option<String>,

    /// OAI-PMH identifier; only used when an OAI-PMH record is the only authoritative metadata (eg, journal OAI-PMH feeds w/o DOIs)
    #[serde(rename = "oai")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oai: Option<String>,

    /// Handle identifier. Do not put DOIs in this field
    #[serde(rename = "hdl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hdl: Option<String>,
}

impl ReleaseExtIds {
    pub fn new() -> ReleaseExtIds {
        ReleaseExtIds {
            doi: None,
            wikidata_qid: None,
            isbn13: None,
            pmid: None,
            pmcid: None,
            core: None,
            arxiv: None,
            jstor: None,
            ark: None,
            mag: None,
            doaj: None,
            dblp: None,
            oai: None,
            hdl: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseRef {
    /// Zero-indexed sequence number of this reference in the list of references. Assigned automatically and used internally; don't confuse with `key`.
    #[serde(rename = "index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,

    /// Optional, fatcat identifier of release entity that this reference is citing.
    #[serde(rename = "target_release_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_release_id: Option<String>,

    /// Additional free-form JSON metadata about this citation. Generally follows Citation Style Language (CSL) JSON schema. See guide for details.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    /// Short string used to indicate this reference from within the release text; or numbering of references as typeset in the release itself. Optional; don't confuse with `index` field.
    #[serde(rename = "key")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Year that the cited work was published in.
    #[serde(rename = "year")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i64>,

    /// Name of the container (eg, journal) that the citation work was published as part of. May be an acronym or full name.
    #[serde(rename = "container_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_name: Option<String>,

    /// Name of the work being cited.
    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Page number or other indicator of the specific subset of a work being cited. Not to be confused with the first page (or page range) of an entire paper or chapter being cited.
    #[serde(rename = "locator")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
}

impl ReleaseRef {
    pub fn new() -> ReleaseRef {
        ReleaseRef {
            index: None,
            target_release_id: None,
            extra: None,
            key: None,
            year: None,
            container_name: None,
            title: None,
            locator: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Success {
    #[serde(rename = "success")]
    pub success: bool,

    #[serde(rename = "message")]
    pub message: String,
}

impl Success {
    pub fn new(success: bool, message: String) -> Success {
        Success { success: success, message: message }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebcaptureAutoBatch {
    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "entity_list")]
    pub entity_list: Vec<models::WebcaptureEntity>,
}

impl WebcaptureAutoBatch {
    pub fn new(editgroup: models::Editgroup, entity_list: Vec<models::WebcaptureEntity>) -> WebcaptureAutoBatch {
        WebcaptureAutoBatch {
            editgroup: editgroup,
            entity_list: entity_list,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebcaptureCdxLine {
    /// \"Sortable URL\" format. See guide for details.
    #[serde(rename = "surt")]
    pub surt: String,

    /// Date and time of capture, in ISO format. UTC, 'Z'-terminated, second (or better) precision.
    #[serde(rename = "timestamp")]
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Full URL/URI of resource captured.
    #[serde(rename = "url")]
    pub url: String,

    /// Mimetype of the resource at this URL. May be the Content-Type header, or the actually sniffed file type.
    #[serde(rename = "mimetype")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// HTTP status code. Should generally be 200, especially for the primary resource, but may be 3xx (redirect) or even error codes if embedded resources can not be fetched successfully.
    #[serde(rename = "status_code")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<i64>,

    /// Resource (file) size in bytes
    #[serde(rename = "size")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    /// SHA-1 hash of data, in hex encoding
    #[serde(rename = "sha1")]
    pub sha1: String,

    /// SHA-256 hash of data, in hex encoding
    #[serde(rename = "sha256")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

impl WebcaptureCdxLine {
    pub fn new(surt: String, timestamp: chrono::DateTime<chrono::Utc>, url: String, sha1: String) -> WebcaptureCdxLine {
        WebcaptureCdxLine {
            surt: surt,
            timestamp: timestamp,
            url: url,
            mimetype: None,
            status_code: None,
            size: None,
            sha1: sha1,
            sha256: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebcaptureEntity {
    /// Full release entities, included in GET responses when `releases` included in `expand` parameter. Ignored if included in PUT or POST requests.
    #[serde(rename = "releases")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releases: Option<Vec<models::ReleaseEntity>>,

    /// Set of identifier of release entities this fileset represents a full manifestation of. Usually a single release.
    #[serde(rename = "release_ids")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_ids: Option<Vec<String>>,

    /// Same format as CDX line timestamp (UTC, etc). Corresponds to the overall capture timestamp. Should generally be the timestamp of capture of the primary resource URL.
    #[serde(rename = "timestamp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,

    /// Base URL of the primary resource this is a capture of
    #[serde(rename = "original_url")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_url: Option<String>,

    #[serde(rename = "archive_urls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_urls: Option<Vec<models::WebcaptureUrl>>,

    #[serde(rename = "cdx")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdx: Option<Vec<models::WebcaptureCdxLine>>,

    /// Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).
    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<serde_json::Value>,

    /// Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    /// base32-encoded unique identifier
    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    // Note: inline enums are not fully supported by swagger-codegen
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

impl WebcaptureEntity {
    pub fn new() -> WebcaptureEntity {
        WebcaptureEntity {
            releases: None,
            release_ids: None,
            timestamp: None,
            original_url: None,
            archive_urls: None,
            cdx: None,
            edit_extra: None,
            extra: None,
            redirect: None,
            revision: None,
            ident: None,
            state: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebcaptureUrl {
    /// URL/URI pointing to archive of this web resource.
    #[serde(rename = "url")]
    pub url: String,

    /// Type of archive endpoint. Usually `wayback` (WBM replay of primary resource), or `warc` (direct URL to a WARC file containing all resources of the capture). See guide for full list.
    #[serde(rename = "rel")]
    pub rel: String,
}

impl WebcaptureUrl {
    pub fn new(url: String, rel: String) -> WebcaptureUrl {
        WebcaptureUrl { url: url, rel: rel }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkAutoBatch {
    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "entity_list")]
    pub entity_list: Vec<models::WorkEntity>,
}

impl WorkAutoBatch {
    pub fn new(editgroup: models::Editgroup, entity_list: Vec<models::WorkEntity>) -> WorkAutoBatch {
        WorkAutoBatch {
            editgroup: editgroup,
            entity_list: entity_list,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkEntity {
    /// Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).
    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<serde_json::Value>,

    /// Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    /// base32-encoded unique identifier
    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    // Note: inline enums are not fully supported by swagger-codegen
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

impl WorkEntity {
    pub fn new() -> WorkEntity {
        WorkEntity {
            edit_extra: None,
            extra: None,
            redirect: None,
            revision: None,
            ident: None,
            state: None,
        }
    }
}
