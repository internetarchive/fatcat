
from flask import Flask
from flask_debugtoolbar import DebugToolbarExtension
from config import Config

toolbar = DebugToolbarExtension()
app = Flask(__name__)
app.config.from_object(Config)
toolbar = DebugToolbarExtension(app)

from fatcat import routes
