"""
Default configuration for fatcat web interface (Flask application).

In production, we currently reconfigure these values using environment
variables, not by (eg) deploying a variant copy of this file.

This config is *only* for the web interface, *not* for any of the workers or
import scripts.
"""

import os
import subprocess
from typing import Union

basedir = os.path.abspath(os.path.dirname(__file__))


def bool_str(raw: Union[str, bool, None]) -> bool:
    """
    Helper for parsing environment variables
    """
    if not raw:
        return False
    if raw is True:
        return True

    if isinstance(raw, str):
        raw_str = raw.strip()
        if raw_str.lower() in ["0", "f", "false", "no", "n"]:
            return False
        if raw_str.lower() in ["1", "t", "true", "yes", "y"]:
            return True
    raise ValueError("Unparsable boolean value: {raw}")


def test_bool_str() -> None:
    assert bool_str(True) is True
    assert bool_str(None) is False
    assert bool_str(False) is False
    assert bool_str("") is False
    assert bool_str("0") is False
    assert bool_str("True") is True
    assert bool_str("FALSE") is False


def fetch_git_sha():
    """
    Get short commit id, runnable anywhere within a git repository.
    """
    return (
        subprocess.check_output(["git", "rev-parse", "--short", "HEAD"]).decode("ascii").strip()
    )


class Config(object):
    GIT_REVISION = (
        subprocess.check_output(["git", "describe", "--tags", "--long", "--always"])
        .strip()
        .decode("utf-8")
    )

    # This is, effectively, the QA/PROD flag
    FATCAT_DOMAIN = os.environ.get("FATCAT_DOMAIN", default="dev.fatcat.wiki")
    FATCAT_API_AUTH_TOKEN = os.environ.get("FATCAT_API_AUTH_TOKEN", default=None)
    FATCAT_API_HOST = os.environ.get(
        "FATCAT_API_HOST", default=f"https://api.{FATCAT_DOMAIN}/v0"
    )
    public_host_default = f"https://api.{FATCAT_DOMAIN}/v0"
    if FATCAT_DOMAIN == "dev.fatcat.wiki":
        public_host_default = FATCAT_API_HOST
    FATCAT_PUBLIC_API_HOST = os.environ.get(
        "FATCAT_PUBLIC_API_HOST", default=public_host_default
    )

    # can set this to https://search.fatcat.wiki for some experimentation
    ELASTICSEARCH_BACKEND = os.environ.get(
        "ELASTICSEARCH_BACKEND", default="http://localhost:9200"
    )
    ELASTICSEARCH_RELEASE_INDEX = os.environ.get(
        "ELASTICSEARCH_RELEASE_INDEX", default="fatcat_release"
    )
    ELASTICSEARCH_CONTAINER_INDEX = os.environ.get(
        "ELASTICSEARCH_CONTAINER_INDEX", default="fatcat_container"
    )

    # for save-paper-now. set to None if not configured, so we don't display forms/links
    KAFKA_PIXY_ENDPOINT = os.environ.get("KAFKA_PIXY_ENDPOINT", default=None) or None
    KAFKA_SAVEPAPERNOW_TOPIC = os.environ.get(
        "KAFKA_SAVEPAPERNOW_TOPIC", default="sandcrawler-dev.ingest-file-requests-priority"
    )

    # for flask things, like session cookies
    FLASK_SECRET_KEY = os.environ.get("FLASK_SECRET_KEY", default=None)
    SECRET_KEY = FLASK_SECRET_KEY

    ORCID_CLIENT_ID = os.environ.get("ORCID_CLIENT_ID", default=None)
    ORCID_CLIENT_SECRET = os.environ.get("ORCID_CLIENT_SECRET", default=None)

    WIKIPEDIA_CLIENT_ID = os.environ.get("WIKIPEDIA_CLIENT_ID", default=None)
    WIKIPEDIA_CLIENT_SECRET = os.environ.get("WIKIPEDIA_CLIENT_SECRET", default=None)

    GITLAB_CLIENT_ID = os.environ.get("GITLAB_CLIENT_ID", default=None)
    GITLAB_CLIENT_SECRET = os.environ.get("GITLAB_CLIENT_SECRET", default=None)

    GITHUB_CLIENT_ID = os.environ.get("GITHUB_CLIENT_ID", default=None)
    GITHUB_CLIENT_SECRET = os.environ.get("GITHUB_CLIENT_SECRET", default=None)

    IA_XAUTH_URI = "https://archive.org/services/xauthn/"
    IA_XAUTH_CLIENT_ID = os.environ.get("IA_XAUTH_CLIENT_ID", default=None)
    IA_XAUTH_CLIENT_SECRET = os.environ.get("IA_XAUTH_CLIENT_SECRET", default=None)

    # analytics; used in production
    ENABLE_GOATCOUNTER = bool_str(os.environ.get("ENABLE_GOATCOUNTER", default=False))
    GOATCOUNTER_ENDPOINT = os.environ.get(
        "GOATCOUNTER_ENDPOINT", default="https://goatcounter.fatcat.wiki/count"
    )
    GOATCOUNTER_SCRIPT_URL = os.environ.get(
        "GOATCOUNTER_SCRIPT_URL", default="https://goatcounter.fatcat.wiki/count.js"
    )

    # controls granularity of "shadow_only" preservation category
    FATCAT_MERGE_SHADOW_PRESERVATION = bool_str(
        os.environ.get("FATCAT_MERGE_SHADOW_PRESERVATION", default=False)
    )

    # creates a site-wide alert message
    FATCAT_ALERT_MESSAGE = os.environ.get("FATCAT_ALERT_MESSAGE", default=None) or None

    # CSRF on by default, but only for WTF forms (not, eg, search, lookups, GET
    # forms)
    WTF_CSRF_CHECK_DEFAULT = False
    WTF_CSRF_TIME_LIMIT = None

    # for login redirects
    USE_SESSION_FOR_NEXT = True

    if FATCAT_DOMAIN == "dev.fatcat.wiki":
        # "Even more verbose" debug options
        # SQLALCHEMY_ECHO = True
        # DEBUG = True
        pass
    else:
        # protect cookies (which include API tokens)
        SESSION_COOKIE_HTTPONLY = True
        SESSION_COOKIE_SECURE = True
        SESSION_COOKIE_SAMESITE = "Lax"
        PERMANENT_SESSION_LIFETIME = 2678400  # 31 days, in seconds

    try:
        GIT_RELEASE = fetch_git_sha()
    except Exception as e:
        print("WARNING: couldn't set sentry git release automatically: " + str(e))
        GIT_RELEASE = None

    SENTRY_CONFIG = {
        #'include_paths': ['fatcat_web', 'fatcat_openapi_client', 'fatcat_tools'],
        "enable-threads": True,  # for uWSGI
        "release": GIT_RELEASE,
        "tags": {
            "fatcat_domain": FATCAT_DOMAIN,
        },
    }
