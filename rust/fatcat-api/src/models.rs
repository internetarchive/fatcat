#![allow(unused_imports, unused_qualifications, unused_extern_crates)]
extern crate chrono;
extern crate serde_json;
extern crate uuid;

use serde::ser::Serializer;

use models;
use std::collections::HashMap;
use swagger;

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
pub struct ContainerEntity {
    #[serde(rename = "coden")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coden: Option<String>,

    #[serde(rename = "abbrev")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abbrev: Option<String>,

    #[serde(rename = "wikidata_qid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wikidata_qid: Option<String>,

    #[serde(rename = "issnl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issnl: Option<String>,

    #[serde(rename = "publisher")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    /// base32-encoded unique identifier
    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<String>,

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
    pub fn new(name: String) -> ContainerEntity {
        ContainerEntity {
            coden: None,
            abbrev: None,
            wikidata_qid: None,
            issnl: None,
            publisher: None,
            name: name,
            extra: None,
            editgroup_id: None,
            redirect: None,
            revision: None,
            ident: None,
            state: None,
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

    #[serde(rename = "display_name")]
    pub display_name: String,

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

    /// base32-encoded unique identifier
    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<String>,

    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

impl CreatorEntity {
    pub fn new(display_name: String) -> CreatorEntity {
        CreatorEntity {
            wikidata_qid: None,
            orcid: None,
            surname: None,
            given_name: None,
            display_name: display_name,
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            editgroup_id: None,
            extra: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Editgroup {
    /// base32-encoded unique identifier
    #[serde(rename = "id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "editor_id")]
    pub editor_id: String,

    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    #[serde(rename = "edits")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edits: Option<models::EditgroupEdits>,
}

impl Editgroup {
    pub fn new(editor_id: String) -> Editgroup {
        Editgroup {
            id: None,
            editor_id: editor_id,
            description: None,
            extra: None,
            edits: None,
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
            releases: None,
            works: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Editor {
    #[serde(rename = "id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "username")]
    pub username: String,
}

impl Editor {
    pub fn new(username: String) -> Editor {
        Editor { id: None, username: username }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityEdit {
    #[serde(rename = "edit_id")]
    pub edit_id: i64,

    #[serde(rename = "ident")]
    pub ident: String,

    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    #[serde(rename = "prev_revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_revision: Option<String>,

    #[serde(rename = "redirect_ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_ident: Option<String>,

    #[serde(rename = "editgroup_id")]
    pub editgroup_id: String,

    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

impl EntityEdit {
    pub fn new(edit_id: i64, ident: String, editgroup_id: String) -> EntityEdit {
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
    #[serde(rename = "message")]
    pub message: String,
}

impl ErrorResponse {
    pub fn new(message: String) -> ErrorResponse {
        ErrorResponse { message: message }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileEntity {
    #[serde(rename = "releases")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releases: Option<Vec<String>>,

    #[serde(rename = "mimetype")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    #[serde(rename = "urls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<models::FileEntityUrls>>,

    #[serde(rename = "sha256")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,

    #[serde(rename = "md5")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,

    #[serde(rename = "sha1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    #[serde(rename = "size")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    /// base32-encoded unique identifier
    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<String>,

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
            mimetype: None,
            urls: None,
            sha256: None,
            md5: None,
            sha1: None,
            size: None,
            extra: None,
            editgroup_id: None,
            redirect: None,
            revision: None,
            ident: None,
            state: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileEntityUrls {
    #[serde(rename = "url")]
    pub url: String,

    #[serde(rename = "rel")]
    pub rel: String,
}

impl FileEntityUrls {
    pub fn new(url: String, rel: String) -> FileEntityUrls {
        FileEntityUrls { url: url, rel: rel }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseContrib {
    #[serde(rename = "index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,

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

    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    #[serde(rename = "role")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

impl ReleaseContrib {
    pub fn new() -> ReleaseContrib {
        ReleaseContrib {
            index: None,
            creator_id: None,
            creator: None,
            raw_name: None,
            extra: None,
            role: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseEntity {
    #[serde(rename = "abstracts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abstracts: Option<Vec<models::ReleaseEntityAbstracts>>,

    #[serde(rename = "refs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refs: Option<Vec<models::ReleaseRef>>,

    #[serde(rename = "contribs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contribs: Option<Vec<models::ReleaseContrib>>,

    /// Two-letter RFC1766/ISO639-1 language code, with extensions
    #[serde(rename = "language")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    #[serde(rename = "publisher")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    #[serde(rename = "pages")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<String>,

    #[serde(rename = "issue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,

    #[serde(rename = "volume")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,

    #[serde(rename = "wikidata_qid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wikidata_qid: Option<String>,

    #[serde(rename = "pmcid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<String>,

    #[serde(rename = "pmid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,

    #[serde(rename = "isbn13")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isbn13: Option<String>,

    #[serde(rename = "doi")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,

    #[serde(rename = "release_date")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(rename = "release_status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_status: Option<String>,

    #[serde(rename = "release_type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_type: Option<String>,

    #[serde(rename = "container_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,

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

    #[serde(rename = "title")]
    pub title: String,

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

    /// base32-encoded unique identifier
    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<String>,

    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

impl ReleaseEntity {
    pub fn new(title: String) -> ReleaseEntity {
        ReleaseEntity {
            abstracts: None,
            refs: None,
            contribs: None,
            language: None,
            publisher: None,
            pages: None,
            issue: None,
            volume: None,
            wikidata_qid: None,
            pmcid: None,
            pmid: None,
            isbn13: None,
            doi: None,
            release_date: None,
            release_status: None,
            release_type: None,
            container_id: None,
            files: None,
            container: None,
            work_id: None,
            title: title,
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            editgroup_id: None,
            extra: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseEntityAbstracts {
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

impl ReleaseEntityAbstracts {
    pub fn new() -> ReleaseEntityAbstracts {
        ReleaseEntityAbstracts {
            sha1: None,
            content: None,
            mimetype: None,
            lang: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseRef {
    #[serde(rename = "index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,

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

    #[serde(rename = "container_title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_title: Option<String>,

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
            container_title: None,
            title: None,
            locator: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatsResponse {
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

impl StatsResponse {
    pub fn new() -> StatsResponse {
        StatsResponse { extra: None }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Success {
    #[serde(rename = "message")]
    pub message: String,
}

impl Success {
    pub fn new(message: String) -> Success {
        Success { message: message }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkEntity {
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,

    /// base32-encoded unique identifier
    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<String>,

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
            extra: None,
            editgroup_id: None,
            redirect: None,
            revision: None,
            ident: None,
            state: None,
        }
    }
}
