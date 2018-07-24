use api_helpers::uuid2fcid;
use chrono;
use database_schema::*;
use errors::*;
use fatcat_api::models::{ChangelogEntry, Editgroup, EntityEdit};
use serde_json;
use uuid::Uuid;

// Ugh. I thought the whole point was to *not* do this, but:
// https://github.com/diesel-rs/diesel/issues/1589

pub enum EntityState {
    WorkInProgress,
    Active(Uuid),
    Redirect(Uuid, Uuid),
    Deleted,
}

impl EntityState {
    pub fn shortname(&self) -> String {
        match self {
            EntityState::WorkInProgress => "wip",
            EntityState::Active(_) => "active",
            EntityState::Redirect(_, _) => "redirect",
            EntityState::Deleted => "deleted",
        }.to_string()
    }
}

pub trait EntityIdentRow {
    fn state(&self) -> Result<EntityState>;
}

pub trait EntityEditRow {
    fn to_model(self) -> Result<EntityEdit>;
}

// Helper for constructing tables
macro_rules! entity_structs {
    ($edit_table:expr, $edit_struct:ident, $ident_table:expr, $ident_struct:ident) => {
        #[derive(Debug, Queryable, Identifiable, Associations, AsChangeset, QueryableByName)]
        #[table_name = $edit_table]
        pub struct $edit_struct {
            pub id: i64,
            pub editgroup_id: Uuid,
            pub updated: chrono::NaiveDateTime,
            pub ident_id: Uuid,
            pub rev_id: Option<Uuid>,
            pub redirect_id: Option<Uuid>,
            pub prev_rev: Option<Uuid>,
            pub extra_json: Option<serde_json::Value>,
        }

        impl EntityEditRow for $edit_struct {
            /// Go from a row (SQL model) to an API model
            fn to_model(self) -> Result<EntityEdit> {
                Ok(EntityEdit {
                    editgroup_id: uuid2fcid(&self.editgroup_id),
                    revision: self.rev_id.map(|v| v.to_string()),
                    redirect_ident: self.redirect_id.map(|v| uuid2fcid(&v)),
                    prev_revision: self.prev_rev.map(|v| v.to_string()),
                    ident: uuid2fcid(&self.ident_id),
                    edit_id: self.id,
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

        impl EntityIdentRow for $ident_struct {
            fn state(&self) -> Result<EntityState> {
                if !self.is_live {
                    return Ok(EntityState::WorkInProgress);
                }
                match (self.redirect_id, self.rev_id) {
                    (None, None) => Ok(EntityState::Deleted),
                    (Some(redir), Some(rev)) => Ok(EntityState::Redirect(redir, rev)),
                    (None, Some(rev)) => Ok(EntityState::Active(rev)),
                    _ => bail!("Invalid EntityIdentRow state"),
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
    pub publisher: Option<String>,
    pub issnl: Option<String>,
    pub wikidata_qid: Option<String>,
    pub abbrev: Option<String>,
    pub coden: Option<String>,
}

entity_structs!(
    "container_edit",
    ContainerEditRow,
    "container_ident",
    ContainerIdentRow
);

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "creator_rev"]
pub struct CreatorRevRow {
    pub id: Uuid,
    pub extra_json: Option<serde_json::Value>,
    pub display_name: String,
    pub given_name: Option<String>,
    pub surname: Option<String>,
    pub orcid: Option<String>,
    pub wikidata_qid: Option<String>,
}

entity_structs!(
    "creator_edit",
    CreatorEditRow,
    "creator_ident",
    CreatorIdentRow
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
    pub size: Option<i64>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub md5: Option<String>,
    pub mimetype: Option<String>,
}

entity_structs!("file_edit", FileEditRow, "file_ident", FileIdentRow);

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "release_rev"]
pub struct ReleaseRevRow {
    pub id: Uuid,
    pub extra_json: Option<serde_json::Value>,
    pub work_ident_id: Uuid,
    pub container_ident_id: Option<Uuid>,
    pub title: String,
    pub release_type: Option<String>,
    pub release_status: Option<String>,
    pub release_date: Option<chrono::NaiveDate>,
    pub doi: Option<String>,
    pub pmid: Option<String>,
    pub pmcid: Option<String>,
    pub wikidata_qid: Option<String>,
    pub isbn13: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub publisher: Option<String>,
    pub language: Option<String>,
}

entity_structs!(
    "release_edit",
    ReleaseEditRow,
    "release_ident",
    ReleaseIdentRow
);

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "work_rev"]
pub struct WorkRevRow {
    pub id: Uuid,
    pub extra_json: Option<serde_json::Value>,
}

entity_structs!("work_edit", WorkEditRow, "work_ident", WorkIdentRow);

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
    pub raw: Option<String>,
    pub role: Option<String>,
    pub index: Option<i64>,
    pub extra_json: Option<serde_json::Value>,
}

#[derive(Debug, Insertable)]
#[table_name = "release_contrib"]
pub struct ReleaseContribNewRow {
    pub release_rev: Uuid,
    pub creator_ident_id: Option<Uuid>,
    pub raw: Option<String>,
    pub role: Option<String>,
    pub index: Option<i64>,
    pub extra_json: Option<serde_json::Value>,
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "release_ref"]
pub struct ReleaseRefRow {
    pub id: i64,
    pub release_rev: Uuid,
    pub target_release_ident_id: Option<Uuid>,
    pub index: Option<i64>,
    pub key: Option<String>,
    pub extra_json: Option<serde_json::Value>,
    pub container_title: Option<String>,
    pub year: Option<i64>,
    pub title: Option<String>,
    pub locator: Option<String>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[table_name = "release_ref"]
pub struct ReleaseRefNewRow {
    pub release_rev: Uuid,
    pub target_release_ident_id: Option<Uuid>,
    pub index: Option<i64>,
    pub key: Option<String>,
    pub extra_json: Option<serde_json::Value>,
    pub container_title: Option<String>,
    pub year: Option<i64>,
    pub title: Option<String>,
    pub locator: Option<String>,
}

#[derive(Debug, Queryable, Insertable, Associations, AsChangeset)]
#[table_name = "file_release"]
pub struct FileReleaseRow {
    pub file_rev: Uuid,
    pub target_release_ident_id: Uuid,
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "editgroup"]
pub struct EditgroupRow {
    pub id: Uuid,
    pub editor_id: Uuid,
    pub created: chrono::NaiveDateTime,
    pub extra_json: Option<serde_json::Value>,
    pub description: Option<String>,
}

impl EditgroupRow {
    /// Returns an Edigroup API model *without* the entity edits actually populated. Useful for,
    /// eg, entity history queries (where we already have the entity edit we want)
    pub fn to_model_partial(self) -> Editgroup {
        Editgroup {
            id: Some(uuid2fcid(&self.id)),
            editor_id: uuid2fcid(&self.editor_id),
            description: self.description,
            extra: self.extra_json,
            edits: None,
        }
    }
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "editor"]
pub struct EditorRow {
    pub id: Uuid,
    pub username: String,
    pub is_admin: bool,
    pub registered: chrono::NaiveDateTime,
    pub active_editgroup_id: Option<Uuid>,
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "changelog"]
pub struct ChangelogRow {
    pub id: i64,
    pub editgroup_id: Uuid,
    pub timestamp: chrono::NaiveDateTime,
}

impl ChangelogRow {
    pub fn to_model(self) -> ChangelogEntry {
        ChangelogEntry {
            index: self.id,
            editgroup_id: uuid2fcid(&self.editgroup_id),
            editgroup: None,
            timestamp: chrono::DateTime::from_utc(self.timestamp, chrono::Utc),
        }
    }
}
