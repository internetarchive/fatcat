
from flask import Flask
from flask.logging import create_logger
from flask_uuid import FlaskUUID
from flask_debugtoolbar import DebugToolbarExtension
from flask_login import LoginManager
from flask_wtf.csrf import CSRFProtect
from flask_misaka import Misaka
from flask_mwoauth import MWOAuth
from authlib.flask.client import OAuth
from loginpass import create_flask_blueprint, Gitlab, GitHub, ORCiD
from raven.contrib.flask import Sentry
import fatcat_openapi_client

from fatcat_web.web_config import Config


toolbar = DebugToolbarExtension()
app = Flask(__name__, static_url_path='/static')
app.config.from_object(Config)
toolbar = DebugToolbarExtension(app)
FlaskUUID(app)
app.csrf = CSRFProtect(app)
app.log = create_logger(app)

# This is the Markdown processor; setting default here
Misaka(app,
    autolink=True,
    no_intra_emphasis=True,
    strikethrough=True,
    escape=True,
)

login_manager = LoginManager()
login_manager.init_app(app)
login_manager.login_view = "/auth/login"
oauth = OAuth(app)

# Grabs sentry config from SENTRY_DSN environment variable
sentry = Sentry(app)

conf = fatcat_openapi_client.Configuration()
conf.host = Config.FATCAT_API_HOST
api = fatcat_openapi_client.DefaultApi(fatcat_openapi_client.ApiClient(conf))

# remove most jinja2 template whitespace
app.jinja_env.trim_blocks = True
app.jinja_env.lstrip_blocks = True

def auth_api(token):
    conf = fatcat_openapi_client.Configuration()
    conf.api_key["Authorization"] = token
    conf.api_key_prefix["Authorization"] = "Bearer"
    conf.host = Config.FATCAT_API_HOST
    return fatcat_openapi_client.DefaultApi(fatcat_openapi_client.ApiClient(conf))

if Config.FATCAT_API_AUTH_TOKEN:
    print("Found and using privileged token (eg, for account signup)")
    priv_api = auth_api(Config.FATCAT_API_AUTH_TOKEN)
else:
    print("No privileged token found")
    priv_api = None

# TODO: refactor integration so this doesn't always need to be defined. If
# key/secret are empty, library will not init; if init is skipped, get
# undefined errors elsewhere.
mwoauth = MWOAuth(
    consumer_key=Config.WIKIPEDIA_CLIENT_ID or "dummy",
    consumer_secret=Config.WIKIPEDIA_CLIENT_SECRET or "dummy",
    default_return_to='wp_oauth_finish_login')
mwoauth.handshaker.user_agent = "fatcat.wiki;python_web_interface"
app.register_blueprint(mwoauth.bp, url_prefix='/auth/wikipedia')

from fatcat_web import routes, editing_routes, auth, cors, forms

# TODO: blocking on ORCID support in loginpass
if Config.ORCID_CLIENT_ID:
    orcid_bp = create_flask_blueprint(ORCiD, oauth, auth.handle_oauth)
    app.register_blueprint(orcid_bp, url_prefix='/auth/orcid')

if Config.GITLAB_CLIENT_ID:
    gitlab_bp = create_flask_blueprint(Gitlab, oauth, auth.handle_oauth)
    app.register_blueprint(gitlab_bp, url_prefix='/auth/gitlab')

if Config.GITHUB_CLIENT_ID:
    github_bp = create_flask_blueprint(GitHub, oauth, auth.handle_oauth)
    app.register_blueprint(github_bp, url_prefix='/auth/github')
