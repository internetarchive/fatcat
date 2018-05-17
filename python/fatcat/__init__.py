
from flask import Flask
from flask_sqlalchemy import SQLAlchemy
from flask_marshmallow import Marshmallow
from flask_debugtoolbar import DebugToolbarExtension
from config import Config

toolbar = DebugToolbarExtension()
app = Flask(__name__)
app.config.from_object(Config)
db = SQLAlchemy(app)
ma = Marshmallow(app)
toolbar = DebugToolbarExtension(app)

from fatcat import routes, models, api, sql, dummy
