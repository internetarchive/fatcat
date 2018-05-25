use chrono;
use database_schema::*;
use errors::*;
use serde_json;
use uuid::Uuid;

// Ugh. I thought the whole point was to *not* do this, but:
// https://github.com/diesel-rs/diesel/issues/1589

pub enum EntityState {
    WorkInProgress,
    Active(i64),
    Redirect(Uuid, i64),
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

// Helper for constructing tables
macro_rules! entity_structs {
    ($edit_table:expr, $edit_struct:ident, $ident_table:expr, $ident_struct:ident) => {
        #[derive(Debug, Queryable, Identifiable, Associations, AsChangeset, QueryableByName)]
        #[table_name = $edit_table]
        pub struct $edit_struct {
            pub id: i64,
            pub ident_id: Uuid,
            pub rev_id: Option<i64>,
            pub redirect_id: Option<Uuid>,
            pub editgroup_id: i64,
            //pub extra_json: Option<serde_json::Value>,
        }

        #[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
        #[table_name = $ident_table]
        pub struct $ident_struct {
            pub id: Uuid,
            pub is_live: bool,
            pub rev_id: Option<i64>,
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
    pub id: i64,
    //pub extra_json: Option<serde_json::Value>,
    pub name: String,
    pub publisher: Option<String>,
    pub issn: Option<String>,
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
    pub id: i64,
    //extra_json: Option<String>,
    pub name: String,
    pub orcid: Option<String>,
}

entity_structs!(
    "creator_edit",
    CreatorEditRow,
    "creator_ident",
    CreatorIdentRow
);

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "file_rev"]
pub struct FileRevRow {
    pub id: i64,
    //extra_json: Option<String>,
    pub size: Option<i32>,
    pub sha1: Option<String>,
    pub url: Option<String>,
}

entity_structs!("file_edit", FileEditRow, "file_ident", FileIdentRow);

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "release_rev"]
pub struct ReleaseRevRow {
    pub id: i64,
    //extra_json: Option<Json>,
    pub work_ident_id: Uuid,
    pub container_ident_id: Option<Uuid>,
    pub title: String,
    pub release_type: Option<String>,
    pub date: Option<String>,
    pub doi: Option<String>,
    pub volume: Option<String>,
    pub pages: Option<String>,
    pub issue: Option<String>,
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
    pub id: i64,
    //extra_json: Option<Json>,
    pub work_type: Option<String>,
    pub primary_release_id: Option<Uuid>,
}

entity_structs!("work_edit", WorkEditRow, "work_ident", WorkIdentRow);

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "release_contrib"]
pub struct ReleaseContribRow {
    id: i64,
    release_rev: i64,
    creator_ident_id: Option<Uuid>,
    stub: Option<String>,
    contrib_type: Option<String>,
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "release_ref"]
pub struct ReleaseRefRow {
    id: i64,
    release_rev: i64,
    target_release_ident_id: Option<Uuid>,
    index: Option<i64>,
    stub: Option<String>,
}

/*
#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "file_release"]
pub struct FileReleaseRow {
    id: i64,
    file_rev: i64,
    target_release_ident_id: Uuid,
}
*/

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "editgroup"]
pub struct EditgroupRow {
    pub id: i64,
    pub extra_json: Option<serde_json::Value>,
    pub editor_id: i64,
    pub description: Option<String>,
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "editor"]
pub struct EditorRow {
    pub id: i64,
    pub username: String,
    pub is_admin: bool,
    pub active_editgroup_id: Option<i64>,
}

#[derive(Debug, Queryable, Identifiable, Associations, AsChangeset)]
#[table_name = "changelog"]
pub struct ChangelogRow {
    pub id: i64,
    pub editgroup_id: i64,
    pub timestamp: chrono::NaiveDateTime,
}
