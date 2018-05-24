
use errors::*;
use diesel;
use diesel::prelude::*;
use database_models::*;
use database_schema::{editgroup, editor};

pub fn get_or_create_editgroup(editor_id: i64, conn: &PgConnection) -> Result<i64> {
    // check for current active
    let ed_row: EditorRow = editor::table.find(editor_id).first(conn)?;
    if let Some(current) = ed_row.active_editgroup_id {
        return Ok(current);
    }

    // need to insert and update
    conn.build_transaction().run(|| {

        let eg_row: EditgroupRow = diesel::insert_into(editgroup::table)
            .values((
                editgroup::editor_id.eq(ed_row.id),
            ))
            .get_result(conn)?;
        diesel::update(editor::table.find(ed_row.id))
            .set(editor::active_editgroup_id.eq(eg_row.id))
            .execute(conn)?;
        Ok(eg_row.id)
    })
}
