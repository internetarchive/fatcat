#!/usr/bin/env python3

import argparse
import fatcat.sql
from fatcat import app, db

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--debug',
        action='store_true',
        help="enable debugging interface (note: not for everything)")
    parser.add_argument('--host',
        default="127.0.0.1",
        help="listen on this host/IP")
    parser.add_argument('--port',
        type=int,
        default=8040,
        help="listen on this port")
    parser.add_argument('--database-uri',
        default=app.config['SQLALCHEMY_DATABASE_URI'],
        help="sqlalchemy database string")
    parser.add_argument('--init-db',
        action='store_true',
        help="create database tables and insert dummy data")
    args = parser.parse_args()

    app.config['SQLALCHEMY_DATABASE_URI'] = args.database_uri

    if args.init_db:
        db.create_all()
        fatcat.sql.populate_db()
        print("Dummy database configured: " + app.config['SQLALCHEMY_DATABASE_URI'])
        return

    app.run(debug=args.debug, host=args.host, port=args.port)

if __name__ == '__main__':
    main()
