
from flask import Flask
from flask_uuid import FlaskUUID
from flask_debugtoolbar import DebugToolbarExtension
from flask_login import LoginManager
from authlib.flask.client import OAuth
from loginpass import create_flask_blueprint, Gitlab
from raven.contrib.flask import Sentry
from web_config import Config
import fatcat_client

toolbar = DebugToolbarExtension()
app = Flask(__name__)
app.config.from_object(Config)
toolbar = DebugToolbarExtension(app)
FlaskUUID(app)

login_manager = LoginManager()
login_manager.init_app(app)
oauth = OAuth(app)

# Grabs sentry config from SENTRY_DSN environment variable
sentry = Sentry(app)

conf = fatcat_client.Configuration()
conf.host = Config.FATCAT_API_HOST
api = fatcat_client.DefaultApi(fatcat_client.ApiClient(conf))

from fatcat_web import routes, auth

if Config.FATCAT_API_AUTH_TOKEN:
    print("Found and using privileged token (eg, for account signup)")
    priv_api = auth.auth_api(Config.FATCAT_API_AUTH_TOKEN)
else:
    print("No privileged token found")
    priv_api = None

gitlab_bp = create_flask_blueprint(Gitlab, oauth, auth.handle_oauth)
app.register_blueprint(gitlab_bp, url_prefix='/auth/gitlab')
