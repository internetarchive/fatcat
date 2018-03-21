
import argparse
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify
from sqlalchemy import create_engine, MetaData, Table

app = Flask(__name__)
app.config.from_pyfile('config.py')

metadata = MetaData()


## SQL Schema ###############################################################

import enum
from sqlalchemy import Table, Column, Integer, String, MetaData, ForeignKey, \
    Enum

# TODO: http://docs.sqlalchemy.org/en/latest/orm/extensions/declarative/mixins.html

class IdState(enum.Enum):
    normal = 1
    redirect = 2
    removed = 3

work_id = Table('work_id', metadata,
    Column('id', Integer, primary_key=True, autoincrement=False),
    Column('revision', ForeignKey('work_revision.id')),
    )

work_revision = Table('work_revision', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('previous', ForeignKey('work_revision.id'), nullable=True),
    Column('state', Enum(IdState)),
    Column('redirect_id', ForeignKey('work_id.id'), nullable=True),
    Column('edit_id', ForeignKey('edit.id')),
    Column('extra_json', ForeignKey('extra_json.sha1'), nullable=True),

    Column('title', String),
    Column('work_type', String),
    Column('date', String),
    )

release_id = Table('release_id', metadata,
    Column('id', Integer, primary_key=True, autoincrement=False),
    Column('revision', ForeignKey('release_revision.id')),
    )

release_revision = Table('release_revision', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('previous', ForeignKey('release_revision.id'), nullable=True),
    Column('state', Enum(IdState)),
    Column('redirect_id', ForeignKey('release_id.id'), nullable=True),
    Column('edit_id', ForeignKey('edit.id')),
    Column('extra_json', ForeignKey('extra_json.sha1'), nullable=True),

    Column('work', ForeignKey('work_id.id')),
    Column('container', ForeignKey('container_id.id')),
    Column('title', String),
    Column('license', String),          # TODO: oa status foreign key
    Column('release_type', String),     # TODO: foreign key
    Column('date', String),             # TODO: datetime
    Column('doi', String),              # TODO: identifier table
    )

creator_id = Table('creator_id', metadata,
    Column('id', Integer, primary_key=True, autoincrement=False),
    Column('revision', ForeignKey('creator_revision.id')),
    )

creator_revision = Table('creator_revision', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('previous', ForeignKey('creator_revision.id'), nullable=True),
    Column('state', Enum(IdState)),
    Column('redirect_id', ForeignKey('creator_id.id'), nullable=True),
    Column('edit_id', ForeignKey('edit.id')),
    Column('extra_json', ForeignKey('extra_json.sha1'), nullable=True),

    Column('name', String),
    Column('sortname', String),
    Column('orcid', String),            # TODO: identifier table
    )

work_contrib = Table('work_contrib', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('work_rev', ForeignKey('work_revision.id'), nullable=False),
    Column('creator_id', ForeignKey('creator_id.id'), nullable=False),
    Column('stub', String, nullable=False),
    )

release_contrib = Table('release_contrib', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('release_rev', ForeignKey('release_revision.id'), nullable=False),
    Column('creator_id', ForeignKey('creator_id.id'), nullable=False),
    )

container_id = Table('container_id', metadata,
    Column('id', Integer, primary_key=True, autoincrement=False),
    Column('revision', ForeignKey('container_revision.id')),
    )

container_revision = Table('container_revision', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('previous', ForeignKey('container_revision.id'), nullable=True),
    Column('state', Enum(IdState)),
    Column('redirect_id', ForeignKey('container_id.id'), nullable=True),
    Column('edit_id', ForeignKey('edit.id')),
    Column('extra_json', ForeignKey('extra_json.sha1'), nullable=True),

    Column('name', String),
    Column('container', ForeignKey('container_id.id')),
    Column('publisher', String),        # TODO: foreign key
    Column('sortname', String),
    Column('issn', String),             # TODO: identifier table
    )

edit = Table('edit', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('edit_group', ForeignKey('edit_group.id')),
    Column('editor', ForeignKey('editor.id')),
    Column('description', String),
    )

edit_group = Table('edit_group', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('editor', ForeignKey('editor.id')),
    Column('description', String),
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

extra_json = Table('extra_json', metadata,
    Column('sha1', String, primary_key=True, autoincrement=True),
    Column('json', String),
    )


## API Methods ##############################################################

@app.route('/health', methods=['GET'])
def health():
    return jsonify({'ok': True})

## Entry Point ##############################################################

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--debug',
        action='store_true',
        help="enable debugging interface")
    parser.add_argument('--host',
        default="127.0.0.1",
        help="listen on this host/IP")
    parser.add_argument('--port',
        type=int,
        default=8040,
        help="listen on this port")
    parser.add_argument('--database-uri',
        default="sqlite:///test.sqlite",
        help="sqlalchemy database string")
    args = parser.parse_args()

    app.config['DATABASE_URI'] = args.database_uri
    engine = create_engine(app.config['DATABASE_URI'], convert_unicode=True)
    metadata.create_all(bind=engine)
    app.run(debug=args.debug, host=args.host, port=args.port)


if __name__ == '__main__':
    main()
