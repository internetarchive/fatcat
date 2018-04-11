
from flask import Flask
from config import Config
from flask_sqlalchemy import SQLAlchemy

app = Flask(__name__)
app.config.from_object(Config)
db = SQLAlchemy(app)

examples = {
    "work": {
        "id": "rzga5b9cd7efgh04iljk",
        "rev": "12345",
        "previous": None,
        "state": None,
        "redirect_id": None,
        "edit_id": None,
        "extra_json": None,
        "title": "Structure and Interpretation",
        "work_type": "journal-article",
        "date": None,
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
