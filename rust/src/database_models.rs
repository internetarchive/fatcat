#![allow(proc_macro_derive_resolution_fallback)]

use crate::database_schema::*;
use crate::errors::*;
use crate::identifiers::uuid2fcid;
use chrono::Utc;
use fatcat_api_spec::models::{
    ChangelogEntry, Editgroup, EditgroupAnnotation, Editor, EntityEdit, ReleaseRef,
};
use serde_json;
use uuid::Uuid;

// Ugh. I thought the whole point was to *not* do this, but:
// https://github.com/diesel-rs/diesel/issues/1589

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EntityState {
    WorkInProgress,
    Active(Uuid),
    Redirect(Uuid, Option<Uuid>),
    Deleted,
}

impl EntityState {
    pub fn shortname(&self) -> String {
        match self {
            EntityState::WorkInProgress => "wip",
            EntityState::Active(_) => "active",
            EntityState::Redirect(_, _) => "redirect",
            EntityState::Deleted => "deleted",
        }
        .to_string()
    }
}

pub trait EntityIdentRow {
    fn state(&self) -> Result<EntityState>;
}

pub trait EntityEditRow {
    fn into_model(self) -> Result<EntityEdit>;
}

// Helper for constructing tables
macro_rules! entity_structs {
    (
        $edit_table:expr,
        $edit_struct:ident,
        $edit_new_struct:ident,
        $ident_table:expr,
        $ident_struct:ident,
        $ident_new_struct:ident
    ) => {
        #[derive(Debug, Queryable, Identifiable, Associations, AsChangeset, QueryableByName)]
        #[table_name = $edit_table]
        pub struct $edit_struct {
            pub id: Uuid,
            pub editgroup_id: Uuid,
            pub updated: chrono::NaiveDateTime,
            pub ident_id: Uuid,
            pub rev_id: Option<Uuid>,
            pub redirect_id: Option<Uuid>,
            pub prev_rev: Option<Uuid>,
            pub extra_json: Option<serde_json::Value>,
        }

        #[derive(Debug, Associations, AsChangeset, QueryableByName, Insertable)]
        #[table_name = $edit_table]
        pub struct $edit_new_struct {
            pub editgroup_id: Uuid,
            pub ident_id: Uuid,
            pub rev_id: Option<Uuid>,
            pub redirect_id: Option<Uuid>,
            pub prev_rev: Option<Uuid>,
            pub extra_json: Option<serde_json::Value>,
        }

        impl EntityEditRow for $edit_struct {
            /// Go from a row (SQL model) to an API model
            fn into_model(self) -> Result<EntityEdit> {
                Ok(EntityEdit {
                    editgroup_id: uuid2fcid(&self.editgroup_id),
                    revision: self.rev_id.map(|v| v.to_string()),
                    redirect_ident: self.redirect_id.map(|v| uuid2fcid(&v)),
                    prev_revision: self.prev_rev.map(|v| v.to_string()),
                    ident: uuid2fcid(&self.ident_id),
                    edit_id: self.id.to_string(),
                    extra: self.extra_json,
                })
            }
        }

        #[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
        #[table_name = $ident_table]
        pub struct $ident_struct {
            pub id: Uuid,
            pub is_live: bool,
            pub rev_id: Option<Uuid>,
            pub redirect_id: Option<Uuid>,
        }

        #[derive(Debug, Associations, AsChangeset, Insertable)]
        #[table_name = $ident_table]
        pub struct $ident_new_struct {
            pub is_live: bool,
            pub rev_id: Option<Uuid>,
            pub redirect_id: Option<Uuid>,
        }

        impl EntityIdentRow for $ident_struct {
            fn state(&self) -> Result<EntityState> {
                if !self.is_live {
                    return Ok(EntityState::WorkInProgress);
                }
                match (self.redirect_id, self.rev_id) {
                    (None, None) => Ok(EntityState::Deleted),
                    (Some(redir), rev) => Ok(EntityState::Redirect(redir, rev)),
                    (None, Some(rev)) => Ok(EntityState::Active(rev)),
                    //_ => bail!("Invalid EntityIdentRow state"),
                }
            }
        }
    };
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "container_rev"]
pub struct ContainerRevRow {
    pub id: Uuid,
    pub extra_json: Option<serde_json::Value>,
    pub name: String,
    pub container_type: Option<String>,
    pub publisher: Option<String>,
    pub issnl: Option<String>,
    pub wikidata_qid: Option<String>,
}

#[derive(Debug, Associations, AsChangeset, Insertable)]
#[table_name = "container_rev"]
pub struct ContainerRevNewRow {
    pub extra_json: Option<serde_json::Value>,
    pub name: String,
    pub container_type: Option<String>,
    pub publisher: Option<String>,
    pub issnl: Option<String>,
    pub wikidata_qid: Option<String>,
}

entity_structs!(
    "container_edit",
    ContainerEditRow,
    ContainerEditNewRow,
    "container_ident",
    ContainerIdentRow,
    ContainerIdentNewRow
);

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "creator_rev"]
pub struct CreatorRevRow {
    pub id: Uuid,
    pub extra_json: Option<serde_json::Value>,
    pub display_name: String,
    pub orcid: Option<String>,
    pub wikidata_qid: Option<String>,
    pub given_name: Option<String>,
    pub surname: Option<String>,
}

#[derive(Debug, Associations, AsChangeset, Insertable)]
#[table_name = "creator_rev"]
pub struct CreatorRevNewRow {
    pub extra_json: Option<serde_json::Value>,
    pub display_name: String,
    pub orcid: Option<String>,
    pub wikidata_qid: Option<String>,
    pub given_name: Option<String>,
    pub surname: Option<String>,
}

entity_structs!(
    "creator_edit",
    CreatorEditRow,
    CreatorEditNewRow,
    "creator_ident",
    CreatorIdentRow,
    CreatorIdentNewRow
);

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "file_rev_url"]
pub struct FileRevUrlRow {
    pub id: i64,
    pub file_rev: Uuid,
    pub rel: String,
    pub url: String,
}

#[derive(Debug, Queryable, Associations, AsChangeset, Insertable)]
#[table_name = "file_rev_url"]
pub struct FileRevUrlNewRow {
    pub file_rev: Uuid,
    pub rel: String,
    pub url: String,
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "file_rev"]
pub struct FileRevRow {
    pub id: Uuid,
    pub extra_json: Option<serde_json::Value>,
    pub size_bytes: Option<i64>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub md5: Option<String>,
    pub mimetype: Option<String>,
}

#[derive(Debug, Associations, AsChangeset, Insertable)]
#[table_name = "file_rev"]
pub struct FileRevNewRow {
    pub extra_json: Option<serde_json::Value>,
    pub size_bytes: Option<i64>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub md5: Option<String>,
    pub mimetype: Option<String>,
}

entity_structs!(
    "file_edit",
    FileEditRow,
    FileEditNewRow,
    "file_ident",
    FileIdentRow,
    FileIdentNewRow
);

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "fileset_rev_file"]
pub struct FilesetRevFileRow {
    pub id: i64,
    pub fileset_rev: Uuid,
    pub path_name: String,
    pub size_bytes: i64,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub extra_json: Option<serde_json::Value>,
}

#[derive(Debug, Queryable, Associations, AsChangeset, Insertable)]
#[table_name = "fileset_rev_file"]
pub struct FilesetRevFileNewRow {
    pub fileset_rev: Uuid,
    pub path_name: String,
    pub size_bytes: i64,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub extra_json: Option<serde_json::Value>,
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "fileset_rev_url"]
pub struct FilesetRevUrlRow {
    pub id: i64,
    pub fileset_rev: Uuid,
    pub rel: String,
    pub url: String,
}

#[derive(Debug, Queryable, Associations, AsChangeset, Insertable)]
#[table_name = "fileset_rev_url"]
pub struct FilesetRevUrlNewRow {
    pub fileset_rev: Uuid,
    pub rel: String,
    pub url: String,
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "fileset_rev"]
pub struct FilesetRevRow {
    pub id: Uuid,
    pub extra_json: Option<serde_json::Value>,
}

#[derive(Debug, Associations, AsChangeset, Insertable)]
#[table_name = "fileset_rev"]
pub struct FilesetRevNewRow {
    pub extra_json: Option<serde_json::Value>,
}

entity_structs!(
    "fileset_edit",
    FilesetEditRow,
    FilesetEditNewRow,
    "fileset_ident",
    FilesetIdentRow,
    FilesetIdentNewRow
);
#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "webcapture_rev_cdx"]
pub struct WebcaptureRevCdxRow {
    pub id: i64,
    pub webcapture_rev: Uuid,
    pub surt: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub url: String,
    pub mimetype: Option<String>,
    pub status_code: Option<i64>,
    pub sha1: String,
    pub sha256: Option<String>,
    pub size_bytes: Option<i64>,
    pub extra_json: Option<serde_json::Value>,
}

#[derive(Debug, Queryable, Associations, AsChangeset, Insertable)]
#[table_name = "webcapture_rev_cdx"]
pub struct WebcaptureRevCdxNewRow {
    pub webcapture_rev: Uuid,
    pub surt: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub url: String,
    pub mimetype: Option<String>,
    pub status_code: Option<i64>,
    pub sha1: String,
    pub sha256: Option<String>,
    pub size_bytes: Option<i64>,
    pub extra_json: Option<serde_json::Value>,
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "webcapture_rev_url"]
pub struct WebcaptureRevUrlRow {
    pub id: i64,
    pub webcapture_rev: Uuid,
    pub rel: String,
    pub url: String,
}

#[derive(Debug, Queryable, Associations, AsChangeset, Insertable)]
#[table_name = "webcapture_rev_url"]
pub struct WebcaptureRevUrlNewRow {
    pub webcapture_rev: Uuid,
    pub rel: String,
    pub url: String,
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "webcapture_rev"]
pub struct WebcaptureRevRow {
    pub id: Uuid,
    pub extra_json: Option<serde_json::Value>,
    pub original_url: String,
    pub timestamp: chrono::NaiveDateTime,
}

#[derive(Debug, Associations, AsChangeset, Insertable)]
#[table_name = "webcapture_rev"]
pub struct WebcaptureRevNewRow {
    pub extra_json: Option<serde_json::Value>,
    pub original_url: String,
    pub timestamp: chrono::NaiveDateTime,
}

entity_structs!(
    "webcapture_edit",
    WebcaptureEditRow,
    WebcaptureEditNewRow,
    "webcapture_ident",
    WebcaptureIdentRow,
    WebcaptureIdentNewRow
);

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "release_rev"]
pub struct ReleaseRevRow {
    pub id: Uuid,
    pub extra_json: Option<serde_json::Value>,
    pub work_ident_id: Uuid,
    pub container_ident_id: Option<Uuid>,
    pub refs_blob_sha1: Option<String>,
    pub title: String,
    pub original_title: Option<String>,
    pub release_type: Option<String>,
    pub release_stage: Option<String>,
    pub release_date: Option<chrono::NaiveDate>,
    pub release_year: Option<i64>,
    pub doi: Option<String>,
    pub pmid: Option<String>,
    pub pmcid: Option<String>,
    pub wikidata_qid: Option<String>,
    pub isbn13: Option<String>,
    pub core_id: Option<String>,
    pub arxiv_id: Option<String>,
    pub jstor_id: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub license_slug: Option<String>,
    pub number: Option<String>,
    pub version: Option<String>,
    pub subtitle: Option<String>,
    pub withdrawn_state: Option<String>,
    pub withdrawn_date: Option<chrono::NaiveDate>,
    pub withdrawn_year: Option<i64>,
    pub mag_id: Option<String>,
    pub ark_id: Option<String>,
}

#[derive(Debug, Associations, AsChangeset, Insertable)]
#[table_name = "release_rev"]
pub struct ReleaseRevNewRow {
    pub extra_json: Option<serde_json::Value>,
    pub work_ident_id: Uuid,
    pub container_ident_id: Option<Uuid>,
    pub refs_blob_sha1: Option<String>,
    pub title: String,
    pub original_title: Option<String>,
    pub release_type: Option<String>,
    pub release_stage: Option<String>,
    pub release_date: Option<chrono::NaiveDate>,
    pub release_year: Option<i64>,
    pub doi: Option<String>,
    pub pmid: Option<String>,
    pub pmcid: Option<String>,
    pub wikidata_qid: Option<String>,
    pub isbn13: Option<String>,
    pub core_id: Option<String>,
    pub arxiv_id: Option<String>,
    pub jstor_id: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
    pub license_slug: Option<String>,
    pub number: Option<String>,
    pub version: Option<String>,
    pub subtitle: Option<String>,
    pub withdrawn_state: Option<String>,
    pub withdrawn_date: Option<chrono::NaiveDate>,
    pub withdrawn_year: Option<i64>,
    pub mag_id: Option<String>,
    pub ark_id: Option<String>,
}

entity_structs!(
    "release_edit",
    ReleaseEditRow,
    ReleaseEditNewRow,
    "release_ident",
    ReleaseIdentRow,
    ReleaseIdentNewRow
);

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "work_rev"]
pub struct WorkRevRow {
    pub id: Uuid,
    pub extra_json: Option<serde_json::Value>,
}

#[derive(Debug, Associations, AsChangeset, Insertable)]
#[table_name = "work_rev"]
pub struct WorkRevNewRow {
    pub extra_json: Option<serde_json::Value>,
}

entity_structs!(
    "work_edit",
    WorkEditRow,
    WorkEditNewRow,
    "work_ident",
    WorkIdentRow,
    WorkIdentNewRow
);

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "release_rev_abstract"]
pub struct ReleaseRevAbstractRow {
    pub id: i64,
    pub release_rev: Uuid,
    pub abstract_sha1: String,
    pub mimetype: Option<String>,
    pub lang: Option<String>,
}

#[derive(Debug, Queryable, Associations, AsChangeset, Insertable)]
#[table_name = "release_rev_abstract"]
pub struct ReleaseRevAbstractNewRow {
    pub release_rev: Uuid,
    pub abstract_sha1: String,
    pub mimetype: Option<String>,
    pub lang: Option<String>,
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "release_contrib"]
pub struct ReleaseContribRow {
    pub id: i64,
    pub release_rev: Uuid,
    pub creator_ident_id: Option<Uuid>,
    pub raw_name: Option<String>,
    pub role: Option<String>,
    pub raw_affiliation: Option<String>,
    pub index_val: Option<i32>,
    pub extra_json: Option<serde_json::Value>,
    pub given_name: Option<String>,
    pub surname: Option<String>,
}

#[derive(Debug, Insertable)]
#[table_name = "release_contrib"]
pub struct ReleaseContribNewRow {
    pub release_rev: Uuid,
    pub creator_ident_id: Option<Uuid>,
    pub raw_name: Option<String>,
    pub role: Option<String>,
    pub raw_affiliation: Option<String>,
    pub index_val: Option<i32>,
    pub extra_json: Option<serde_json::Value>,
    pub given_name: Option<String>,
    pub surname: Option<String>,
}

#[derive(Debug, Queryable, Insertable, Associations, AsChangeset)]
#[table_name = "release_ref"]
pub struct ReleaseRefRow {
    pub release_rev: Uuid,
    pub index_val: i32,
    pub target_release_ident_id: Uuid,
}

#[derive(Debug, Queryable, Insertable, Associations, AsChangeset)]
#[table_name = "refs_blob"]
pub struct RefsBlobRow {
    pub sha1: String,
    pub refs_json: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// This model is a stable representation of what goes in a RefsBlobRow `refs_json` field (an array
/// of this model). We could rely on the `ReleaseRef` API spec model directly, but that would lock
/// the database contents to the API spec rigidly; by defining this struct independently, we can
/// migrate the schemas. To start, this is a direct copy of the `ReleaseRef` model.
pub struct RefsBlobJson {
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

impl RefsBlobJson {
    pub fn into_model(self) -> ReleaseRef {
        ReleaseRef {
            index: self.index,
            target_release_id: self.target_release_id,
            extra: self.extra,
            key: self.key,
            year: self.year,
            container_name: self.container_name,
            title: self.title,
            locator: self.locator,
        }
    }

    pub fn to_model(&self) -> ReleaseRef {
        ReleaseRef {
            index: self.index,
            target_release_id: self.target_release_id.clone(),
            extra: self.extra.clone(),
            key: self.key.clone(),
            year: self.year,
            container_name: self.container_name.clone(),
            title: self.title.clone(),
            locator: self.locator.clone(),
        }
    }

    pub fn from_model(model: &ReleaseRef) -> RefsBlobJson {
        RefsBlobJson {
            index: model.index,
            target_release_id: model.target_release_id.clone(),
            extra: model.extra.clone(),
            key: model.key.clone(),
            year: model.year,
            container_name: model.container_name.clone(),
            title: model.title.clone(),
            locator: model.locator.clone(),
        }
    }
}

#[derive(Debug, Queryable, Insertable, Associations, AsChangeset)]
#[table_name = "file_rev_release"]
pub struct FileRevReleaseRow {
    pub file_rev: Uuid,
    pub target_release_ident_id: Uuid,
}

#[derive(Debug, Queryable, Insertable, Associations, AsChangeset)]
#[table_name = "fileset_rev_release"]
pub struct FilesetRevReleaseRow {
    pub fileset_rev: Uuid,
    pub target_release_ident_id: Uuid,
}

#[derive(Debug, Queryable, Insertable, Associations, AsChangeset)]
#[table_name = "webcapture_rev_release"]
pub struct WebcaptureRevReleaseRow {
    pub webcapture_rev: Uuid,
    pub target_release_ident_id: Uuid,
}

#[derive(Debug, Queryable, Insertable, Associations, AsChangeset)]
#[table_name = "abstracts"]
pub struct AbstractsRow {
    pub sha1: String,
    pub content: String,
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "editgroup"]
pub struct EditgroupRow {
    pub id: Uuid,
    pub editor_id: Uuid,
    pub created: chrono::NaiveDateTime,
    pub submitted: Option<chrono::NaiveDateTime>,
    pub is_accepted: bool,
    pub description: Option<String>,
    pub extra_json: Option<serde_json::Value>,
}

impl EditgroupRow {
    /// Returns an Editgroup API model *without* the entity edits actually populated. Useful for,
    /// eg, entity history queries (where we already have the entity edit we want)
    pub fn into_model_partial(self, changelog_index: Option<i64>) -> Editgroup {
        Editgroup {
            editgroup_id: Some(uuid2fcid(&self.id)),
            editor_id: Some(uuid2fcid(&self.editor_id)),
            editor: None,
            changelog_index: changelog_index,
            submitted: self
                .submitted
                .map(|v| chrono::DateTime::from_utc(v, chrono::Utc)),
            description: self.description,
            extra: self.extra_json,
            annotations: None,
            edits: None,
        }
    }
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "editgroup_annotation"]
pub struct EditgroupAnnotationRow {
    pub id: Uuid,
    pub editgroup_id: Uuid,
    pub editor_id: Uuid,
    pub created: chrono::NaiveDateTime,
    pub comment_markdown: Option<String>,
    pub extra_json: Option<serde_json::Value>,
}

impl EditgroupAnnotationRow {
    pub fn into_model(self) -> EditgroupAnnotation {
        EditgroupAnnotation {
            annotation_id: Some(self.id.to_string()),
            editgroup_id: Some(uuid2fcid(&self.editgroup_id)),
            editor_id: Some(uuid2fcid(&self.editor_id)),
            editor: None,
            created: Some(chrono::DateTime::from_utc(self.created, chrono::Utc)),
            comment_markdown: self.comment_markdown,
            extra: self.extra_json,
        }
    }
}

#[derive(Debug, Clone, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "editor"]
pub struct EditorRow {
    pub id: Uuid,
    pub username: String,
    pub is_superuser: bool,
    pub is_admin: bool,
    pub is_bot: bool,
    pub is_active: bool,
    pub registered: chrono::NaiveDateTime,
    pub auth_epoch: chrono::NaiveDateTime,
    pub wrangler_id: Option<Uuid>,
}

impl EditorRow {
    pub fn into_model(self) -> Editor {
        Editor {
            editor_id: Some(uuid2fcid(&self.id)),
            username: self.username,
            is_admin: Some(self.is_admin),
            is_bot: Some(self.is_bot),
            is_active: Some(self.is_active),
        }
    }
}

#[derive(Debug, Clone, Queryable, Associations, AsChangeset)]
#[table_name = "auth_oidc"]
pub struct AuthOidcRow {
    pub id: i64,
    pub created: chrono::NaiveDateTime,
    pub editor_id: Uuid,
    pub provider: String,
    pub oidc_iss: String,
    pub oidc_sub: String,
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "changelog"]
pub struct ChangelogRow {
    pub id: i64,
    pub editgroup_id: Uuid,
    pub timestamp: chrono::NaiveDateTime,
}

impl ChangelogRow {
    pub fn into_model(self) -> ChangelogEntry {
        ChangelogEntry {
            index: self.id,
            editgroup_id: uuid2fcid(&self.editgroup_id),
            editgroup: None,
            timestamp: chrono::DateTime::from_utc(self.timestamp, chrono::Utc),
        }
    }
}
