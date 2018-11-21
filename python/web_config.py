
"""
Default configuration for fatcat web interface (Flask application).

In production, we currently reconfigure these values using environment
variables, not by (eg) deploying a variant copy of this file.

This config is *only* for the web interface, *not* for any of the workers or
import scripts.
"""

import os
import raven
import subprocess

basedir = os.path.abspath(os.path.dirname(__file__))

class Config(object):
    GIT_REVISION = subprocess.check_output(["git", "describe", "--always"]).strip()
    # This is, effectively, the QA/PROD flag
    FATCAT_DOMAIN = os.environ.get("FATCAT_DOMAIN", default="qa.fatcat.wiki")
    # can set this to https://search.fatcat.wiki for some experimentation
    ELASTICSEARCH_BACKEND = os.environ.get("ELASTICSEARCH_BACKEND", default="http://localhost:9200")
    ELASTICSEARCH_INDEX = os.environ.get("ELASTICSEARCH_INDEX", default="fatcat")

    try:
        git_release = raven.fetch_git_sha(os.path.dirname(os.pardir))
    except Exception as e:
        print("WARNING: couldn't set sentry git release automatically: " + str(e))
        git_release = None
    SENTRY_CONFIG = {
        #'include_paths': ['fatcat_web', 'fatcat_client', 'fatcat_tools'],
        'release': git_release,
        'fatcat_domain': FATCAT_DOMAIN,
    }

    # "Event more verbose" debug options. SECRET_KEY is bogus.
    #SQLALCHEMY_ECHO = True
    #SECRET_KEY = "kuhy0284hflskjhg01284"
    #DEBUG = True
