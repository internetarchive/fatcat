#![allow(unused_imports, unused_qualifications, unused_extern_crates)]
extern crate chrono;
extern crate serde_json;
extern crate uuid;

use serde::ser::Serializer;

use models;
use std::collections::HashMap;
use swagger;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Changelogentries(Vec<ChangelogentriesInner>);

impl ::std::convert::From<Vec<ChangelogentriesInner>> for Changelogentries {
    fn from(x: Vec<ChangelogentriesInner>) -> Self {
        Changelogentries(x)
    }
}

impl ::std::convert::From<Changelogentries> for Vec<ChangelogentriesInner> {
    fn from(x: Changelogentries) -> Self {
        x.0
    }
}

impl ::std::iter::FromIterator<ChangelogentriesInner> for Changelogentries {
    fn from_iter<U: IntoIterator<Item = ChangelogentriesInner>>(u: U) -> Self {
        Changelogentries(Vec::<ChangelogentriesInner>::from_iter(u))
    }
}

impl ::std::iter::IntoIterator for Changelogentries {
    type Item = ChangelogentriesInner;
    type IntoIter = ::std::vec::IntoIter<ChangelogentriesInner>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> ::std::iter::IntoIterator for &'a Changelogentries {
    type Item = &'a ChangelogentriesInner;
    type IntoIter = ::std::slice::Iter<'a, ChangelogentriesInner>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a> ::std::iter::IntoIterator for &'a mut Changelogentries {
    type Item = &'a mut ChangelogentriesInner;
    type IntoIter = ::std::slice::IterMut<'a, ChangelogentriesInner>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

impl ::std::ops::Deref for Changelogentries {
    type Target = Vec<ChangelogentriesInner>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ::std::ops::DerefMut for Changelogentries {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangelogentriesInner {
    #[serde(rename = "index")]
    pub index: i64,

    #[serde(rename = "editgroup_id")]
    pub editgroup_id: i64,

    #[serde(rename = "timestamp")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ChangelogentriesInner {
    pub fn new(index: i64, editgroup_id: i64, timestamp: chrono::DateTime<chrono::Utc>) -> ChangelogentriesInner {
        ChangelogentriesInner {
            index: index,
            editgroup_id: editgroup_id,
            timestamp: timestamp,
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

    #[serde(rename = "name")]
    pub name: String,

    // Note: inline enums are not fully supported by swagger-codegen
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<i64>,

    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<i64>,
}

impl ContainerEntity {
    pub fn new(name: String) -> ContainerEntity {
        ContainerEntity {
            issn: None,
            publisher: None,
            name: name,
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            editgroup_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatorEntity {
    #[serde(rename = "orcid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orcid: Option<String>,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<i64>,

    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<i64>,

    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    // Note: inline enums are not fully supported by swagger-codegen
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

impl CreatorEntity {
    pub fn new(name: String) -> CreatorEntity {
        CreatorEntity {
            orcid: None,
            name: name,
            editgroup_id: None,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,

    #[serde(rename = "editor_id")]
    pub editor_id: i64,

    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

impl Editgroup {
    pub fn new(editor_id: i64) -> Editgroup {
        Editgroup {
            id: None,
            editor_id: editor_id,
            description: None,
            extra: None,
        }
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
    pub editgroup_id: Option<i64>,

    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<i64>,

    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    #[serde(rename = "edit_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_id: Option<i64>,
}

impl EntityEdit {
    pub fn new() -> EntityEdit {
        EntityEdit {
            editgroup_id: None,
            revision: None,
            ident: None,
            edit_id: None,
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
    #[serde(rename = "url")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(rename = "sha1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    #[serde(rename = "size")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<i64>,

    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<i64>,

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
            editgroup_id: None,
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

    #[serde(rename = "container_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,

    #[serde(rename = "work_id")]
    pub work_id: String,

    #[serde(rename = "title")]
    pub title: String,

    // Note: inline enums are not fully supported by swagger-codegen
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<i64>,

    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<i64>,
}

impl ReleaseEntity {
    pub fn new(work_id: String, title: String) -> ReleaseEntity {
        ReleaseEntity {
            issue: None,
            pages: None,
            volume: None,
            doi: None,
            release_type: None,
            container_id: None,
            work_id: work_id,
            title: title,
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            editgroup_id: None,
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

    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<i64>,

    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<i64>,

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
            editgroup_id: None,
            redirect: None,
            revision: None,
            ident: None,
            state: None,
        }
    }
}
