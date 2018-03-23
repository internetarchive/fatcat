
import argparse
from fatcat import app, db

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
    db.create_all()
    app.run(debug=args.debug, host=args.host, port=args.port)

if __name__ == '__main__':
    main()
