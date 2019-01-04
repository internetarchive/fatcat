
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
conf.host = "http://localhost:9411/v0"
api = fatcat_client.DefaultApi(fatcat_client.ApiClient(conf))

if Config.FATCAT_API_AUTH_TOKEN:
    print("Found and using privileged token (eg, for account signup)")
    priv_conf = fatcat_client.Configuration()
    priv_conf.api_key["Authorization"] = Config.FATCAT_API_AUTH_TOKEN
    priv_conf.api_key_prefix["Authorization"] = "Bearer"
    priv_conf.host = 'http://localhost:9411/v0'
    priv_api = fatcat_client.DefaultApi(fatcat_client.ApiClient(local_conf))
else:
    print("No privileged token found")
    priv_api = None

from fatcat_web import routes, auth

gitlab_bp = create_flask_blueprint(Gitlab, oauth, auth.handle_oauth)
app.register_blueprint(gitlab_bp, url_prefix='/auth/gitlab')
