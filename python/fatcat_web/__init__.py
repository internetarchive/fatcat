
from flask import Flask
from flask_uuid import FlaskUUID
from flask_debugtoolbar import DebugToolbarExtension
from raven.contrib.flask import Sentry
from web_config import Config
import fatcat_client

toolbar = DebugToolbarExtension()
app = Flask(__name__)
app.config.from_object(Config)
toolbar = DebugToolbarExtension(app)
FlaskUUID(app)

# Grabs sentry config from SENTRY_DSN environment variable
sentry = Sentry(app)

conf = fatcat_client.Configuration()
conf.host = "http://localhost:9411/v0"
api = fatcat_client.DefaultApi(fatcat_client.ApiClient(conf))

from fatcat_web import routes
