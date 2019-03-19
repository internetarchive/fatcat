#!/usr/bin/env python3
"""
This file started life as an ipython log file

Ugh, it needs to be copied to ../../python (with ./files/) to actually run. Derp.
"""

import os
import datetime
import fatcat_client
from fatcat_client.rest import ApiException
from fatcat_client import *
from fatcat_tools import *

import datetime

# setup env
#admin_id = "aaaaaaaaaaaabkvkaaaaaaaaae"
#local_conf = fatcat_client.Configuration()
#local_conf.host = 'http://localhost:9411/v0'
#local_api = fatcat_client.DefaultApi(fatcat_client.ApiClient(local_conf))
#api = local_api
api = authenticated_api(
    'http://localhost:9411/v0',
    # token is an optional kwarg (can be empty string, None, etc)
    token=os.environ.get("FATCAT_API_AUTH_TOKEN"))

def upsert_release(doi, fname):
    """
    Tries to fetch a release by doi from the API. If it can't be find, is
    created from the referenced local JSON file (stripping a bunch of fields if
    necessary).

    Either way, returns ident
    """
    try:
        r = api.lookup_release(doi=doi)
        print("release already existed: {}".format(doi))
        return r.ident
    except ApiException:
        pass
    r = entity_from_json(open(fname, 'r').read(), ReleaseEntity)
    r.ident = None
    r.release_rev = None
    r.container_id = None
    r.container = None
    r.work_id = None
    if r.release_type == 'journal-article':
        r.release_type = 'article-journal'
    r.edit_extra = dict(source="copied from old production as a demo")
    files = [f for f in r.files]

    print("bootstrapping release: {}".format(doi))
    eg = api.create_editgroup(Editgroup())
    resp = api.create_release(r, editgroup_id=eg.editgroup_id)

    for f in files:
        f.ident = None
        f.revision = None
        f.release_ids = [resp.ident]
    files_resp = api.create_file_batch(files, editgroup_id=eg.editgroup_id)

    api.accept_editgroup(eg.editgroup_id)
    return resp.ident

def upsert_container(issnl, fname):
    """
    Same as upsert_release, but for containers (by issnl)
    """
    try:
        c = api.lookup_container(issnl=issnl)
        print("container already existed: {}".format(issnl))
        return c.ident
    except ApiException:
        pass
    c = entity_from_json(open(fname, 'r').read(), ContainerEntity)
    c.ident = None
    c.release_rev = None
    c.container_id = None
    c.edit_extra = dict(source="copied from old production as a demo")

    print("bootstrapping container: {}".format(issnl))
    eg = api.create_editgroup(Editgroup())
    resp = api.create_container(c, editgroup_id=eg.editgroup_id)
    api.accept_editgroup(eg.editgroup_id)
    return resp.ident

def math_universe():
    #c_ident = upsert_container("0015-9018", "files/foundations_of_physics.json")
    ident = upsert_release("10.1007/s10701-007-9186-9", "files/math_universe.json")

    base = api.get_release(ident, expand="files")
    #base.container_id = c_ident

    files = [api.get_file(f.ident) for f in base.files]

    if len(files) <= 1:
        print("already updated math_universe, skipping")
        return

    # create the pre-print version
    preprint = api.get_release(base.ident)
    preprint.doi = None
    preprint.ident = None
    preprint.revision = None
    preprint.doi = None
    preprint.container = None
    preprint.publisher = None
    preprint.volume = None
    preprint.pages = None
    preprint.wikidata_qid = None
    preprint.arxiv_id = "0704.0646v2"
    preprint.container_id = None
    preprint.extra = dict(arxiv=dict(section='gr-qc'))
    preprint.release_date = datetime.date(2007, 10, 8)
    preprint.release_status = 'submitted'
    preprint.work_id = base.work_id

    eg = api.create_editgroup(Editgroup(description="math universe pre-print and file assignment"))
    resp = api.create_release(preprint, editgroup_id=eg.editgroup_id)
    files[1].release_ids = [resp.ident]
    files[2].release_ids = [resp.ident]
    files[3].release_ids = []
    files[4].release_ids = []
    api.update_file(files[1].ident, files[1], editgroup_id=eg.editgroup_id)
    api.update_file(files[2].ident, files[2], editgroup_id=eg.editgroup_id)
    api.update_file(files[3].ident, files[3], editgroup_id=eg.editgroup_id)
    api.update_file(files[4].ident, files[4], editgroup_id=eg.editgroup_id)
    api.update_release(base.ident, base, editgroup_id=eg.editgroup_id)
    api.accept_editgroup(eg.editgroup_id)
    # ok, that seems to have split it well enough

# Next, a distill.pub work
def distill_pub():
    # pub,distill)/2017/momentum 20180613072149 https://distill.pub/2017/momentum/ text/html 200 4PP5AXYVD3VBB5BYYO4XK3FJXUR6Z46V 87161
    # Why Momentum Really Works
    # april 4, 2017
    # Gabriel Goh
    # orcid: 0000-0001-5021-2683
    #c_ident = upsert_container("2476-0757", "files/distill_pub.json")
    ident = upsert_release("10.23915/distill.00006", "files/distill_momentum.json")

    momentum_works = api.get_release(ident, expand="files")

    eg = api.create_editgroup(Editgroup(description="distill article enrichment"))
    #goh = CreatorEntity(display_name="Gabriel Goh", orcid="0000-0001-5021-2683")
    #goh = api.create_creator(goh, editgroup_id=eg.editgroup_id)
    #distill = ContainerEntity(name="Distill", issnl="2476-0757", publisher="Distill", wikidata_qid="Q36437840")
    #distill = api.create_container(distill, editgroup_id=eg.editgroup_id)
    momentum_works.abstracts = [ReleaseEntityAbstracts(mimetype="text/plain", lang="en", content="We often think of Momentum as a means of dampening oscillations and speeding up the iterations, leading to faster convergence. But it has other interesting behavior. It allows a larger range of step-sizes to be used, and creates its own oscillations. What is going on?")]
    #momentum_works.contribs = [ReleaseContrib(raw_name="Gabriel Goh", role='author', index=0, creator_id=goh.ident)]
    momentum_works = api.update_release(momentum_works.ident, momentum_works, editgroup_id=eg.editgroup_id)
    #momentum_works.container_id = c_ident
    momentum_page = FileEntity(sha1='e3dfd05f151eea10f438c3b9756ca9bd23ecf3d5', mimetype='text/html')
    momentum_page.urls = [FileEntityUrls(url="https://distill.pub/2017/momentum/", rel="publisher"), FileEntityUrls(url="http://web.archive.org/web/20180613072149/https://distill.pub/2017/momentum/", rel="webarchive")]
    momentum_page.release_ids = [momentum_works.ident]
    api.create_file(momentum_page, editgroup_id=eg.editgroup_id)
    api.accept_editgroup(eg.editgroup_id)

def dlib_example():
    # and now a d-lib exammple
    #c_ident = upsert_container("1082-9873", "files/dlib.json")
    ident = upsert_release("10.1045/november14-jahja", "files/dlib_example.json")
    dlib = api.get_release(ident)

    eg = api.create_editgroup(Editgroup(description="enriching d-lib article"))
    #authors = [api.create_creator(CreatorEntity(display_name=a.raw_name), editgroup_id=eg.editgroup_id) for a in dlib.contribs]
    #for i in range(3):
    #    dlib.contribs[i].creator_id = authors[i].ident
    #dlib.container_id = c_ident
    #r = api.update_release(dlib.ident, dlib, editgroup_id=eg.editgroup_id)
    dlib_page = FileEntity(sha1='a637f1d27d9bcb237310ed29f19c07e1c8cf0aa5', mimetype='text/html')
    dlib_page.urls = [FileEntityUrls(url="http://www.dlib.org/dlib/november14/jahja/11jahja.html", rel="publisher"), FileEntityUrls(url="http://web.archive.org/web/20180602033542/http://www.dlib.org/dlib/november14/jahja/11jahja.html", rel="webarchive")]
    dlib_page.release_ids = [dlib.ident]
    api.create_file(dlib_page, editgroup_id=eg.editgroup_id)
    api.accept_editgroup(eg.editgroup_id)

    # TODO: the actual web file here?

math_universe()
#distill_pub()
#dlib_example()
