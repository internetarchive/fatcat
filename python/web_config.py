
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

    # bogus values for dev/testing
    SECRET_KEY = os.environ.get("SECRET_KEY", default="mQLO6DpyR4t91G1tl/LPMvb/5QFV9vIUDZah5PapTUSmP8jVIrvCRw")

    GITLAB_CLIENT_ID = os.environ.get("GITLAB_CLIENT_ID", default="bogus")
    GITLAB_CLIENT_SECRET = os.environ.get("GITLAB_CLIENT_SECRET", default="bogus")

    try:
        GIT_RELEASE = raven.fetch_git_sha('..')
    except Exception as e:
        print("WARNING: couldn't set sentry git release automatically: " + str(e))
        GIT_RELEASE = None

    SENTRY_CONFIG = {
        #'include_paths': ['fatcat_web', 'fatcat_client', 'fatcat_tools'],
        'enable-threads': True, # for uWSGI
        'release': GIT_RELEASE,
        'tags': {
            'fatcat_domain': FATCAT_DOMAIN,
        },
    }

    # "Even more verbose" debug options
    #SQLALCHEMY_ECHO = True
    #DEBUG = True
