
from sqlalchemy import Table, Column, Integer, String, MetaData, ForeignKey

metadata = MetaData()

# TODO: http://docs.sqlalchemy.org/en/latest/orm/extensions/declarative/mixins.html

work_id = Table('work_id', metadata,
    Column('id', Integer, primary_key=True, autoincrement=False),
    Column('revision', ForeignKey('work_revision.id')),
    )

work_revision = Table('work_revision', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('previous', ForeignKey('work_revision.id'), nullable=True),
    Column('state', enum('normal', 'redirect', 'removed')),
    Column('redirect_id', ForeignKey('work_id.id'), nullable=True)),
    Column('edit_id'), ForeignKey('edit.id')),

    Column('title', String),
    Column('work_type', ForeignKey()),
    Column('date', String),
    Column('journal', String),
    Column('doi', String),
    )

creator_id = Table('creator_id', metadata,
    Column('id', Integer, primary_key=True, autoincrement=False),
    Column('revision', ForeignKey('creator_revision.id')),
    )

creator_revision = Table('creator_revision', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('previous', ForeignKey('creator_revision.id'), optional=True),
    Column('state', enum('normal', 'redirect', 'removed')),
    Column('redirect_id', ForeignKey('creator_id.id'), optional=True)),
    Column('edit_id'), ForeignKey('edit.id')),

    Column('name', String),
    Column('sortname', String),
    )

authorship = Table('authorship', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('work_rev', ForeignKey('work_revision.id'), nullable=False),
    Column('creator_id', ForeignKey('creator_id.id'), nullable=False),
    )

edit = Table('edit', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('edit_group', ForeignKey('edit_group.id')),
    Column('editor', ForeignKey('editor.id')),
    )

edit_group = Table('edit_group', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    )

editor = Table('editor', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('username', String),
    )

changelog = Table('changelog', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('edit_id', ForeignKey('edit.id')),
    Column('timestamp', Integer),
    )
