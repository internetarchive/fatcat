
from flask import Flask
from flask_sqlalchemy import SQLAlchemy
from config import Config

app = Flask(__name__)
app.config.from_object(Config)
db = SQLAlchemy(app)

examples = {
    "work": {
        "id": "rzga5b9cd7efgh04iljk",
        "rev": "12345",
        "redirect_id": None,
        "edit_id": None,
        "extra_json": None,
        "title": "Structure and Interpretation",
        "work_type": "journal-article",
        "contributors": [
            {"name": "Alyssa P. Hacker"},
        ],
        "primary": {
            "title": "Structure and Interpretation",
            "release_type": "online",
            "date": "2000-01-01",
            "doi": "10.491/599.sdo14",
        },
        "releases": [
        ],
    },
}

from fatcat import routes, models, api, sql
