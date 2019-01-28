
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
    GIT_REVISION = subprocess.check_output(["git", "describe", "--always"]).strip().decode('utf-8')

    # This is, effectively, the QA/PROD flag
    FATCAT_DOMAIN = os.environ.get("FATCAT_DOMAIN", default="dev.fatcat.wiki")
    FATCAT_API_AUTH_TOKEN = os.environ.get("FATCAT_API_AUTH_TOKEN", default=None)
    FATCAT_API_HOST = os.environ.get("FATCAT_API_HOST", default="https://{}/v0".format(FATCAT_DOMAIN))

    # can set this to https://search.fatcat.wiki for some experimentation
    ELASTICSEARCH_BACKEND = os.environ.get("ELASTICSEARCH_BACKEND", default="http://localhost:9200")
    ELASTICSEARCH_RELEASE_INDEX = os.environ.get("ELASTICSEARCH_RELEASE_INDEX", default="fatcat_release")
    ELASTICSEARCH_CONTAINER_INDEX = os.environ.get("ELASTICSEARCH_CONTAINER_INDEX", default="fatcat_container")

    # for flask things, like session cookies
    FLASK_SECRET_KEY = os.environ.get("FLASK_SECRET_KEY", default=None)
    SECRET_KEY = FLASK_SECRET_KEY

    GITLAB_CLIENT_ID = os.environ.get("GITLAB_CLIENT_ID", default=None)
    GITLAB_CLIENT_SECRET = os.environ.get("GITLAB_CLIENT_SECRET", default=None)

    IA_XAUTH_URI = "https://archive.org/services/xauthn/"
    IA_XAUTH_CLIENT_ID = os.environ.get("IA_XAUTH_CLIENT_ID", default=None)
    IA_XAUTH_CLIENT_SECRET = os.environ.get("IA_XAUTH_CLIENT_SECRET", default=None)

    # protect cookies (which include API tokens)
    if FATCAT_DOMAIN != "dev.fatcat.wiki":
        SESSION_COOKIE_HTTPONLY = True
        SESSION_COOKIE_SECURE = True
        SESSION_COOKIE_SAMESITE = 'Lax'
        PERMANENT_SESSION_LIFETIME = 2678400 # 31 days, in seconds

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
