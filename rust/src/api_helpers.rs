use database_models::*;
use database_schema::*;
use diesel;
use diesel::prelude::*;
use errors::*;

pub fn get_or_create_editgroup(editor_id: i64, conn: &PgConnection) -> Result<i64> {
    // check for current active
    let ed_row: EditorRow = editor::table.find(editor_id).first(conn)?;
    if let Some(current) = ed_row.active_editgroup_id {
        return Ok(current);
    }

    // need to insert and update
    conn.build_transaction().run(|| {
        let eg_row: EditgroupRow = diesel::insert_into(editgroup::table)
            .values((editgroup::editor_id.eq(ed_row.id),))
            .get_result(conn)?;
        diesel::update(editor::table.find(ed_row.id))
            .set(editor::active_editgroup_id.eq(eg_row.id))
            .execute(conn)?;
        Ok(eg_row.id)
    })
}

pub fn accept_editgroup(editgroup_id: i64, conn: &PgConnection) -> Result<ChangelogRow> {
    conn.build_transaction().run(|| {
        // check that we haven't accepted already (in changelog)
        // NB: could leave this to a UNIQUE constraint
        let count: i64 = changelog::table
            .filter(changelog::editgroup_id.eq(editgroup_id))
            .count()
            .get_result(conn)?;
        if count > 0 {
            bail!("editgroup {} has already been accepted", editgroup_id);
        }

        // for each entity type...
        //for entity in (container_edit, creator_edit, file_edit, release_edit, work_edit) {
            /*
            // This would be the clean and efficient way, but see:
            // https://github.com/diesel-rs/diesel/issues/1478
            diesel::update(container_ident::table)
                .inner_join(container_edit::table.on(
                    container_ident::id.eq(container_edit::ident_id)
                ))
                .filter(container_edit::editgroup_id.eq(editgroup_id))
                .values((
                    container_ident::is_live.eq(true),
                    container_ident::rev_id.eq(container_edit::rev_id),
                    container_ident::redirect_id.eq(container_edit::redirect_id),
                ))
                .execute()?;
            */

        // Sketchy... but fast? Only a few queries per accept.
        for entity in ["container", "creator", "file", "work", "release"].iter() {
            diesel::sql_query(format!(
                "
                    UPDATE {entity}_ident
                    SET
                        is_live = true,
                        rev_id = {entity}_edit.rev_id,
                        redirect_id = {entity}_edit.redirect_id
                    FROM {entity}_edit
                    WHERE
                        {entity}_ident.id = {entity}_edit.ident_id
                        AND {entity}_edit.editgroup_id = $1",
                entity = entity
            )).bind::<diesel::sql_types::BigInt, _>(editgroup_id)
                .execute(conn)?;
        }

        // append log/changelog row
        let entry: ChangelogRow = diesel::insert_into(changelog::table)
            .values((changelog::editgroup_id.eq(editgroup_id),))
            .get_result(conn)?;

        // update any editor's active editgroup
        let no_active: Option<i64> = None;
        diesel::update(editor::table)
            .filter(editor::active_editgroup_id.eq(editgroup_id))
            .set(editor::active_editgroup_id.eq(no_active))
            .execute(conn)?;
        Ok(entry)
    })
}
