
import os
import subprocess

basedir = os.path.abspath(os.path.dirname(__file__))

class Config(object):
    SQLALCHEMY_DATABASE_URI = os.environ.get('DATABASE_URI') or \
        'sqlite:///' + os.path.join(basedir, 'fatcat_dev.sqlite')
    SQLALCHEMY_TRACK_MODIFICATIONS = False
    ELASTIC_BACKEND = "http://search.qa.fatcat.wiki:8088"
    ELASTIC_INDEX = "crossref-works"
    GIT_REVISION = subprocess.check_output(["git", "describe", "--always"]).strip()

    # "Event more verbose" debug options. SECRET_KEY is bogus.
    #SQLALCHEMY_ECHO = True
    #SECRET_KEY = "kuhy0284hflskjhg01284"
    #DEBUG = True
