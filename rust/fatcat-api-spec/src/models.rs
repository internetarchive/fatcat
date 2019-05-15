#![allow(unused_imports, unused_qualifications, unused_extern_crates)]
extern crate chrono;
extern crate serde_json;
extern crate uuid;

use serde::ser::Serializer;

use models;
use std::collections::HashMap;
use swagger;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthOidc {
    #[serde(rename = "provider")]
    pub provider: String,

    #[serde(rename = "sub")]
    pub sub: String,

    #[serde(rename = "iss")]
    pub iss: String,

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
pub struct ChangelogEntry {
    #[serde(rename = "index")]
    pub index: i64,

    #[serde(rename = "editgroup_id")]
    pub editgroup_id: String,

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

    #[serde(rename = "issnl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issnl: Option<String>,

    #[serde(rename = "publisher")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    /// Eg, 'journal'
    #[serde(rename = "container_type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_type: Option<String>,

    /// Required for valid entities
    #[serde(rename = "name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<serde_json::Value>,

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
            issnl: None,
            publisher: None,
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
    #[serde(rename = "wikidata_qid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wikidata_qid: Option<String>,

    #[serde(rename = "orcid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orcid: Option<String>,

    #[serde(rename = "surname")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surname: Option<String>,

    #[serde(rename = "given_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,

    /// Required for valid entities
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

    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

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
    /// base32-encoded unique identifier
    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "editor_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_id: Option<String>,

    #[serde(rename = "editor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<models::Editor>,

    #[serde(rename = "changelog_index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changelog_index: Option<i64>,

    #[serde(rename = "submitted")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

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

    /// base32-encoded unique identifier
    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "editor_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_id: Option<String>,

    #[serde(rename = "editor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<models::Editor>,

    #[serde(rename = "created")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(rename = "comment_markdown")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_markdown: Option<String>,

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
    /// base32-encoded unique identifier
    #[serde(rename = "editor_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_id: Option<String>,

    #[serde(rename = "username")]
    pub username: String,

    #[serde(rename = "is_admin")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_admin: Option<bool>,

    #[serde(rename = "is_bot")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_bot: Option<bool>,

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
    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "edit_id")]
    pub edit_id: String,

    /// base32-encoded unique identifier
    #[serde(rename = "ident")]
    pub ident: String,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "prev_revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_revision: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "redirect_ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_ident: Option<String>,

    /// base32-encoded unique identifier
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
    #[serde(rename = "release_ids")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_ids: Option<Vec<String>>,

    #[serde(rename = "mimetype")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    #[serde(rename = "urls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<models::FileUrl>>,

    #[serde(rename = "sha256")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,

    #[serde(rename = "sha1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    #[serde(rename = "md5")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,

    #[serde(rename = "size")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<serde_json::Value>,

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
    #[serde(rename = "url")]
    pub url: String,

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

    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<serde_json::Value>,
}

impl FilesetEntity {
    pub fn new() -> FilesetEntity {
        FilesetEntity {
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
    #[serde(rename = "path")]
    pub path: String,

    #[serde(rename = "size")]
    pub size: i64,

    #[serde(rename = "md5")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,

    #[serde(rename = "sha1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    #[serde(rename = "sha256")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,

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
            extra: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilesetUrl {
    #[serde(rename = "url")]
    pub url: String,

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
    #[serde(rename = "sha1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    #[serde(rename = "content")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    #[serde(rename = "mimetype")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

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
    #[serde(rename = "index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,

    /// base32-encoded unique identifier
    #[serde(rename = "creator_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<String>,

    /// Optional; GET-only
    #[serde(rename = "creator")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<models::CreatorEntity>,

    #[serde(rename = "raw_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_name: Option<String>,

    #[serde(rename = "given_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,

    #[serde(rename = "surname")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surname: Option<String>,

    #[serde(rename = "role")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Raw affiliation string as displayed in text
    #[serde(rename = "raw_affiliation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_affiliation: Option<String>,

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

    /// Short version of license name. Eg, 'CC-BY'
    #[serde(rename = "license_slug")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_slug: Option<String>,

    /// Two-letter RFC1766/ISO639-1 language code, with extensions
    #[serde(rename = "language")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    #[serde(rename = "publisher")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    #[serde(rename = "version")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(rename = "number")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,

    #[serde(rename = "pages")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<String>,

    #[serde(rename = "issue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,

    #[serde(rename = "volume")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,

    #[serde(rename = "ext_ids")]
    pub ext_ids: models::ReleaseExtIds,

    #[serde(rename = "withdrawn_year")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub withdrawn_year: Option<i64>,

    #[serde(rename = "withdrawn_date")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub withdrawn_date: Option<chrono::NaiveDate>,

    #[serde(rename = "withdrawn_status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub withdrawn_status: Option<String>,

    #[serde(rename = "release_year")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_year: Option<i64>,

    #[serde(rename = "release_date")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<chrono::NaiveDate>,

    #[serde(rename = "release_stage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_stage: Option<String>,

    #[serde(rename = "release_type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_type: Option<String>,

    #[serde(rename = "container_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,

    /// Optional; GET-only
    #[serde(rename = "webcaptures")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webcaptures: Option<Vec<models::WebcaptureEntity>>,

    /// Optional; GET-only
    #[serde(rename = "filesets")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesets: Option<Vec<models::FilesetEntity>>,

    /// Optional; GET-only
    #[serde(rename = "files")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<models::FileEntity>>,

    /// Optional; GET-only
    #[serde(rename = "container")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<models::ContainerEntity>,

    #[serde(rename = "work_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_id: Option<String>,

    /// Title in original language (or, the language of the full text of this release)
    #[serde(rename = "original_title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_title: Option<String>,

    /// Avoid this field if possible, and merge with title; usually English
    #[serde(rename = "subtitle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,

    /// Required for valid entities. The title used in citations and for display; usually English
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

    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

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
    #[serde(rename = "doi")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,

    #[serde(rename = "wikidata_qid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wikidata_qid: Option<String>,

    #[serde(rename = "isbn13")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isbn13: Option<String>,

    #[serde(rename = "pmid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,

    #[serde(rename = "pmcid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<String>,

    #[serde(rename = "core")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub core: Option<String>,

    #[serde(rename = "arxiv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arxiv: Option<String>,

    #[serde(rename = "jstor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jstor: Option<String>,

    #[serde(rename = "ark")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ark: Option<String>,

    #[serde(rename = "mag")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mag: Option<String>,
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseRef {
    #[serde(rename = "index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,

    /// base32-encoded unique identifier
    #[serde(rename = "target_release_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_release_id: Option<String>,

    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    #[serde(rename = "key")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    #[serde(rename = "year")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i64>,

    #[serde(rename = "container_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_name: Option<String>,

    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

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
    #[serde(rename = "surt")]
    pub surt: String,

    /// UTC, 'Z'-terminated, second (or better) precision
    #[serde(rename = "timestamp")]
    pub timestamp: chrono::DateTime<chrono::Utc>,

    #[serde(rename = "url")]
    pub url: String,

    #[serde(rename = "mimetype")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    #[serde(rename = "status_code")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<i64>,

    #[serde(rename = "size")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    #[serde(rename = "sha1")]
    pub sha1: String,

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
    #[serde(rename = "release_ids")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_ids: Option<Vec<String>>,

    /// same format as CDX line timestamp (UTC, etc). Corresponds to the overall capture timestamp. Can be the earliest or average of CDX timestamps if that makes sense.
    #[serde(rename = "timestamp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(rename = "original_url")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_url: Option<String>,

    #[serde(rename = "archive_urls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_urls: Option<Vec<models::WebcaptureUrl>>,

    #[serde(rename = "cdx")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdx: Option<Vec<models::WebcaptureCdxLine>>,

    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<serde_json::Value>,

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
    #[serde(rename = "url")]
    pub url: String,

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
    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<serde_json::Value>,

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
