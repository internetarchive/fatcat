
import os
import subprocess

basedir = os.path.abspath(os.path.dirname(__file__))

class Config(object):
    SQLALCHEMY_DATABASE_URI = os.environ.get('DATABASE_URI') or \
        'sqlite:///' + os.path.join(basedir, 'fatcat_dev.sqlite')
    SQLALCHEMY_TRACK_MODIFICATIONS = False
    GIT_REVISION = subprocess.check_output(["git", "describe", "--always"]).strip()
    # This is, effectively, the QA/PROD flag
    FATCAT_DOMAIN = "qa.fatcat.wiki"
    ELASTIC_BACKEND = "https://search.fatcat.wiki"
    ELASTIC_INDEX = "fatcat"

    # "Event more verbose" debug options. SECRET_KEY is bogus.
    #SQLALCHEMY_ECHO = True
    #SECRET_KEY = "kuhy0284hflskjhg01284"
    #DEBUG = True
