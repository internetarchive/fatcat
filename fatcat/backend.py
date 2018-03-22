
import argparse
from flask import Flask, render_template, send_from_directory, request, \
    url_for, abort, g, redirect, jsonify
from sqlalchemy import create_engine, MetaData, Table

app = Flask(__name__)
app.config.from_object(__name__)

# Load default config and override config from an environment variable
app.config.update(dict(
    DATABASE_URI='sqlite://:memory:',
    SECRET_KEY='development-key',
    USERNAME='admin',
    PASSWORD='admin'
))
app.config.from_envvar('FATCAT_BACKEND_CONFIG', silent=True)

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

    #Column('work', ForeignKey('work_id.id')),
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

file_id = Table('file_id', metadata,
    Column('id', Integer, primary_key=True, autoincrement=False),
    Column('revision', ForeignKey('container_revision.id')),
    )

file_revision = Table('file_revision', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('previous', ForeignKey('file_revision.id'), nullable=True),
    Column('state', Enum(IdState)),
    Column('redirect_id', ForeignKey('file_id.id'), nullable=True),
    Column('edit_id', ForeignKey('edit.id')),
    Column('extra_json', ForeignKey('extra_json.sha1'), nullable=True),

    Column('size', Integer),
    Column('sha1', Integer),            # TODO: hash table... only or in addition?
    Column('url', Integer),             # TODO: URL table
    )

release_file= Table('release_file', metadata,
    Column('id', Integer, primary_key=True, autoincrement=True),
    Column('release_rev', ForeignKey('release_revision.id'), nullable=False),
    Column('file_id', ForeignKey('file_id.id'), nullable=False),
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

## Helpers ##################################################################

def is_fcid(s):
    return len(s) == 26 and s.isalnum()

# XXX: why isn't this running?
def test_is_fcid():

    for s in ("rzga5b9cd7efgh04iljk", "RZGA5B9CD7Efgh04iljk"):
        assert is_fcid() is True

    for s in ("rzga59cd7efgh04iljk", "rzga.b9cd7efgh04iljk", "", 
            "rzga5b9cd7efgh04iljkz"):
        assert is_fcid() is False

def release_list(id_list):
    # XXX: MOCK
    l = []
    for i in id_list:
        l.append({
            "id": i,
            "rev": "8fkj28fjhqkjdhkjkj9s",
            "previous": "0021jdfjhqkjdhkjkj9s",
            "state": "normal",
            "redirect_id": None,
            "edit_id": "932582iuhckjvssk",
            "extra_json": None,

            "container_id": "0021jdfjhqkjdhkjkj9s",
            "title": "Mocks are great",
            "license": "CC-0",
            "release_type": "publication",
            "date": "2017-11-22",
            "doi": "10.1000/953kj.sdfkj",
        })
    return l

def release_hydrate(release_id):
    e = release_list([release_id])[0]
    e['container'] = container_hydrate(d['container_id'])
    e.pop('container_id')
    e['creators'] = [creator_hydrate(c['id']) for c in e['creator_ids']]
    return e

def work_list(id_list):
    """This is the fast/light version: populates entity-specific lists (eg,
    identifiers), and any primaries, but doesn't transclude all other
    entities"""
    if len(id_list) == 0:
        return []

    l = []
    for i in id_list:
        l.append({
            "id": "rzga5b9cd7efgh04iljk",
            "rev": "8fkj28fjhqkjdhkjkj9s",
            "previous": "0021jdfjhqkjdhkjkj9s",
            "state": "normal",
            "redirect_id": None,
            "edit_id": "932582iuhckjvssk",
            "extra_json": None,

            "title": "Mocks are great",
            "contributors": [],
            "work_type": "journal-article",
            "date": None,

            "primary_release": release_list(["8fkj28fjhqkjdhkjkj9s"])[0],
        })
    return l

def work_hydrate(work_id):
    """This is the heavy/slowversion: everything from get_works(), but also
    recursively transcludes single-linked entities"""
    # XXX:
    return work_list([work_id])[0]

## API Methods ##############################################################

@app.route('/health', methods=['GET'])
def health():
    return jsonify({'ok': True})


@app.route('/v0/work/<work_id>', methods=['GET'])
def work_get(work_id):
    if not is_fcid(work_id):
        print("not fcid: {}".format(work_id))
        return abort(404)
    work = work_hydrate(work_id)
    return jsonify(work)

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
        default=app.config['DATABASE_URI'],
        help="sqlalchemy database string")
    args = parser.parse_args()

    app.config['DATABASE_URI'] = args.database_uri
    app.conn = create_engine(app.config['DATABASE_URI'], convert_unicode=True)
    metadata.create_all(bind=engine)

    # XXX:
    db_test_data()

    app.run(debug=args.debug, host=args.host, port=args.port)


if __name__ == '__main__':
    main()
