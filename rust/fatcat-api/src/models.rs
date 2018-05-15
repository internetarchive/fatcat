#![allow(unused_imports, unused_qualifications, unused_extern_crates)]
extern crate chrono;
extern crate uuid;

use serde::ser::Serializer;

use models;
use std::collections::HashMap;
use swagger;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Changelogentry {
    #[serde(rename = "index")]
    pub index: isize,

    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<isize>,

    #[serde(rename = "timestamp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl Changelogentry {
    pub fn new(index: isize) -> Changelogentry {
        Changelogentry {
            index: index,
            editgroup_id: None,
            timestamp: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerEntity {
    #[serde(rename = "issn")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issn: Option<String>,

    #[serde(rename = "publisher")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    #[serde(rename = "parent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,

    #[serde(rename = "name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    // Note: inline enums are not fully supported by swagger-codegen
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<isize>,

    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    #[serde(rename = "editgroup")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup: Option<isize>,
}

impl ContainerEntity {
    pub fn new() -> ContainerEntity {
        ContainerEntity {
            issn: None,
            publisher: None,
            parent: None,
            name: None,
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            editgroup: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatorEntity {
    #[serde(rename = "orcid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orcid: Option<String>,

    #[serde(rename = "name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(rename = "editgroup")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup: Option<isize>,

    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<isize>,

    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    // Note: inline enums are not fully supported by swagger-codegen
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

impl CreatorEntity {
    pub fn new() -> CreatorEntity {
        CreatorEntity {
            orcid: None,
            name: None,
            editgroup: None,
            redirect: None,
            revision: None,
            ident: None,
            state: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Editgroup {
    #[serde(rename = "id")]
    pub id: isize,

    #[serde(rename = "editor_id")]
    pub editor_id: isize,
}

impl Editgroup {
    pub fn new(id: isize, editor_id: isize) -> Editgroup {
        Editgroup { id: id, editor_id: editor_id }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Editor {
    #[serde(rename = "username")]
    pub username: String,
}

impl Editor {
    pub fn new(username: String) -> Editor {
        Editor { username: username }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityEdit {
    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<isize>,

    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<isize>,

    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    #[serde(rename = "id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<isize>,
}

impl EntityEdit {
    pub fn new() -> EntityEdit {
        EntityEdit {
            editgroup_id: None,
            revision: None,
            ident: None,
            id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Error {
    #[serde(rename = "message")]
    pub message: String,
}

impl Error {
    pub fn new(message: String) -> Error {
        Error { message: message }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileEntity {
    #[serde(rename = "url")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(rename = "sha1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    #[serde(rename = "size")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<isize>,

    #[serde(rename = "editgroup")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup: Option<isize>,

    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<isize>,

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
            url: None,
            sha1: None,
            size: None,
            editgroup: None,
            redirect: None,
            revision: None,
            ident: None,
            state: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseEntity {
    #[serde(rename = "issue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,

    #[serde(rename = "pages")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<String>,

    #[serde(rename = "volume")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,

    #[serde(rename = "doi")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,

    #[serde(rename = "release_type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_type: Option<String>,

    #[serde(rename = "license")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    #[serde(rename = "container")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,

    #[serde(rename = "work")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work: Option<String>,

    // Note: inline enums are not fully supported by swagger-codegen
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<isize>,

    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    #[serde(rename = "editgroup")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup: Option<isize>,
}

impl ReleaseEntity {
    pub fn new() -> ReleaseEntity {
        ReleaseEntity {
            issue: None,
            pages: None,
            volume: None,
            doi: None,
            release_type: None,
            license: None,
            container: None,
            work: None,
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            editgroup: None,
        }
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
    #[serde(rename = "work_type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_type: Option<String>,

    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "editgroup")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup: Option<isize>,

    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<isize>,

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
            work_type: None,
            title: None,
            editgroup: None,
            redirect: None,
            revision: None,
            ident: None,
            state: None,
        }
    }
}
