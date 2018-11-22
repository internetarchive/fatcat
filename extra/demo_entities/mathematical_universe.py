# IPython log file

qa_api.get_changelog()
prod_api.get_changelog()
local_api.get_changelog()
local_api.get_changelog()
math_release = prod_api.get_release("wbyajeunkrdgplwlevboku4ivu")
get_ipython().run_line_magic('logon', '')
get_ipython().run_line_magic('pinfo', '%logstart')
get_ipython().run_line_magic('logstart', 'mathematical_universe_split')
math_release = prod_api.get_release("wbyajeunkrdgplwlevboku4ivu")
original = prod_api.get_release("wbyajeunkrdgplwlevboku4ivu")
original
base = original.copy()
copyright
base = original
base.work_id = None
base.ident = None
base.revision = None
base
base.container_id = None
base.edit_extra = dict("source": "copied from old production to QA")
base.edit_extra = dict(source="copied from old production to QA")
eg = qa_api.create_editgroup()
eg = qa_api.create_editgroup(Editgroup())
get_ipython().set_next_input('admin = qa_api.get_editor');get_ipython().run_line_magic('pinfo', 'qa_api.get_editor')
admin_id = "aaaaaaaaaaaabkvkaaaaaaaaae"
eg = qa_api.create_editgroup(Editgroup(editor_id=admin_id))
api = qa_api
api = local_api
eg = api.create_editgroup(Editgroup(editor_id=admin_id))
local_conf.host = 'http://localhost:9411/v0'
local_api = fatcat_client.DefaultApi(fatcat_client.ApiClient(local_conf))
api = local_api
eg = api.create_editgroup(Editgroup(editor_id=admin_id))
get_ipython().run_line_magic('pinfo', 'api.create_release')
api.create_release(base, editgroup_id=eg.id)
api.create_release(base, editgroup=eg.id)
base.release_type = 'article-journal'
api.create_release(base, editgroup=eg.id)
api.accept_editgroup(eg.id)
base = api.get_release('tuwhuuf755edlew5sxcxi6m4zu')
base
original_file1 = prod_api.get_file('fuj2umjhmjdlvnalduifvwe3tq')
original_file1 = prod_api.get_file('fuj2umjhmjdlvnalduifvwe3tq')
original_file1 = prod_api.get_file('nkc2yjzlwndi3mhfnc2vfmcch4')
original_file1 = prod_api.get_file('fuj2umjhmjdlvnalduifvwe3tq')
original_file2 = prod_api.get_file('nkc2yjzlwndi3mhfnc2vfmcch4')
original_file3 = prod_api.get_file('3iqiuv4a5bg65ku5t2dyvtothe')
file1 = original_file1
file2 = original_file2
file3 = original_file3
file1.ident = None
file1.revision = None
file1.releases
file1.releases = [base.ident]
file2.ident = None
file2.revision = None
file2.releases = [base.ident]
file3.releases = [base.ident]
file3.revision = None
file3.ident = None
eg = api.create_editgroup(Editgroup(editor_id=admin_id))
files = [file1,file2,file3]
api.create_files(files, editgroup=eg.id)
api.create_file_batch(files, editgroup=eg.id)
files_resp = [{'edit_id': 162,
  'editgroup_id': '6gomxpalsncxtgighxkv2lnq6y',
  'extra': None,
  'ident': '44rd4nipcrhvnk3gbtxzd5epem',
  'prev_revision': None,
  'redirect_ident': None,
  'revision': '5bdd7ef0-8c41-4fb9-ab80-367e878a08f6'}, {'edit_id': 163,
  'editgroup_id': '6gomxpalsncxtgighxkv2lnq6y',
  'extra': None,
  'ident': 'usu6tkv7fjcb5ftvmblsid54rq',
  'prev_revision': None,
  'redirect_ident': None,
  'revision': 'fc4c52de-c387-43d5-8278-f2e195ed7898'}, {'edit_id': 164,
  'editgroup_id': '6gomxpalsncxtgighxkv2lnq6y',
  'extra': None,
  'ident': 'abk3phdf3bfvdl2sykk7hg5zum',
  'prev_revision': None,
  'redirect_ident': None,
  'revision': 'e1853864-f656-4292-9837-512418a5da03'}]
api.accept_editgroup(eg)
api.accept_editgroup(eg.id)
[api.get_file(f.ident) for f in files_resp]
[api.get_file(f['ident']) for f in files_resp]
files = [api.get_file(f['ident']) for f in files_resp]
files
files[2]
files[2].urls
list(set(files[2].urls))
files[0]
base
preprint = base
preprint.ident = None
preprint.revision = None
preprint.container = None
preprint.container_id = None
preprint.status = None
preprint
preprint.extra
preprint.extra = dict(arxiv=dict(id='0704.0646v2', section='gr-qc'))
preprint.release_date = datetime.date(2007, 10, 8)
import datetime
preprint.release_date = datetime.date(2007, 10, 8)
get_ipython().run_line_magic('pinfo', 'api.create_release_batch')
preprint.discriminator
get_ipython().run_line_magic('pinfo', 'preprint.discriminator')
preprint.extra
preprint.doi
preprint.doi = None
preprint.release_status
preprint.release_status = 'preprint'
preprint.release_type
c = api.create_release_batch([preprint], autoaccept=True)
c
preprint = api.get_release(r['ident'])
preprint = api.get_release(c['ident'])
preprint = api.get_release(c[0]['ident'])
preprint = api.get_release(c[0].ident)
files
files[0]
files [1]
preprint.work_id
base.work_id
files[1].releases = [preprint.ident]
files[2].releases = [preprint.ident]
eg = api.create_editgroup(Editgroup(editor_id=admin_id))
api.update_file(files[1], editgroup=eg.id)
api.update_file(files[1].ident, files[1], editgroup=eg.id)
api.update_file(files[2].ident, files[2], editgroup=eg.id)
api.accept_editgroup(eg.id)
files = [api.get_file(i) for f.ident in files]
files = [api.get_file(f.ident) for f in files]
files
base.doi
base
base.doi
base == preprint
base
preprint.doi
base.doi
base.extra
preprint.extra
base.release_status
preprint.ident
api.get_release('do2smg2xkbbhplfegoduv4r2ve')
preprint.ident
base.ident
preprint.ident
api.get_work('yy36collbvgppo3pgfj4i3jbwu')
api.get_work('yy36collbvgppo3pgfj4i3jbwu', expand='releases')
api.get_work('yy36collbvgppo3pgfj4i3jbwu', expand='release')
get_ipython().run_line_magic('pinfo', 'api.get_work')
api.get_work_releases(preprint.work_id)
things = api.get_work_releases(preprint.work_id)
len(things)
c
c[0]
c[0]
c[0].editgroup_id
api.get_editgroup('6ywtzyh5kjb7ra2k7arxfms22a')
eg = api.create_editgroup(Editgroup(editor_id=admin_id))
api.create_release(preprint, editgroup=eg.id)
api.accept_editgroup(eg.id)
preprint = api.get_release('mish63rawzc7diflxpr7cm5jvi')
eg = api.create_editgroup(Editgroup(editor_id=admin_id))
files[2].releases = [preprint.ident]
files[1].releases = [preprint.ident]
api.update_file(files[1].ident, files[1], editgroup=eg.id)
api.update_file(files[2].ident, files[2], editgroup=eg.id)
api.accept_editgroup(eg.id)
api.accept_editgroup(eg.id)
# ok, that seems to have split it well enough
# pub,distill)/2017/momentum 20180613072149 https://distill.pub/2017/momentum/ text/html 200 4PP5AXYVD3VBB5BYYO4XK3FJXUR6Z46V 87161
# Why Momentum Really Works
# april 4, 2017
# Gabriel Goh
get_ipython().set_next_input('goh = CreatorEntity');get_ipython().run_line_magic('pinfo', 'CreatorEntity')
goh = CreatorEntity()
eg = api.create_editgroup(Editgroup(editor_id=admin_id))
# orcid: 0000-0001-5021-2683
goh = CreatorEntity(display_name="Gabriel Goh", orcid="0000-0001-5021-2683")
goh = api.create_creator(goh, editgroup=eg.id)
get_ipython().run_line_magic('pinfo', 'CreatorEntity')
get_ipython().run_line_magic('pinfo', 'ContainerEntity')
distill = ContainerEntity(name="Distill", issnl="2476-0757", publisher="Distill", wikidata_qid="Q36437840")
distill = api.create_container(distill, editgroup=eg.id)
get_ipython().run_line_magic('pinfo', 'ReleaseEntity')
momentum_works = ReleaseEntity(title="Why Momentum Really Works", container_id=distill.ident, release_date=datetime.date(2017,4,4), language='eng', release_status='published', release_type='article-journal')
momentum_works.contribs = [ReleaseContrib(creator_id=raw_name="Gabriel Goh", role='author', index=0, creator_id=goh.ident)]
momentum_works.contribs = [ReleaseContrib(creator_id=raw_name="Gabriel Goh", role='author', index=0, creator=goh.ident)]
momentum_works.contribs = [ReleaseContrib(raw_name="Gabriel Goh", role='author', index=0, creator=goh.ident)]
momentum_works.contribs = [ReleaseContrib(raw_name="Gabriel Goh", role='author', index=0, creator_id=goh.ident)]
momentum_works = api.create_release(momentum_works, editgroup=eg.id)
momentum_page = FileEntity(sha1='e3dfd05f151eea10f438c3b9756ca9bd23ecf3d5', mimetype='text/html')
get_ipython().run_line_magic('pinfo', 'FileEntity')
get_ipython().set_next_input('momentum_page.urls = [FileEntityUrls');get_ipython().run_line_magic('pinfo', 'FileEntityUrls')
momentum_page.urls = [FileEntityUrls(url="https://distill.pub/2017/momentum/", rel="publisher"), FileEntityUrls(url="http://web.archive.org/web/20180613072149/https://distill.pub/2017/momentum/", rel="webarchive")]
api.create_file(momentum_page, editgroup_id=eg.id)
api.create_file(momentum_page, editgroup=eg.id)
api.accept_editgroup(eg.id)
eg = api.create_editgroup(Editgroup(editor_id=admin_id))
momentum_page.releases = [momentum_works.ident]
r = api.update_file(momentum_page, editgroup=eg.id)
r = api.update_file(momentum_page.id, momentum_page, editgroup=eg.id)
r = api.update_file(momentum_page.ident, momentum_page, editgroup=eg.id)
r = api.update_file(momentum_page.ident, momentum_page, editgroup=eg.id)
get_ipython().run_line_magic('pinfo', 'api.update_file')
r = api.update_file(id=momentum_page.ident, entity=momentum_page, editgroup=eg.id)
# huh.
api.create_file(momentum_page, editgroup=eg.id)
eg = api.create_editgroup(Editgroup(editor_id=admin_id))
r = api.create_file(momentum_page, editgroup=eg.id)
api.accept_editgroup(eg.id)
momentum_page = api.get_file(r.ident)
momentum_page.ident
momentum_works.abstracts
momentum_works = api.get_release(momentum_works.ident)
momentum_works.abstracts
get_ipython().run_line_magic('pinfo', 'ReleaseEntityAbstracts')
momentum_works.abstracts = [ReleaseEntityAbstracts(mimetype="text/plain", lang="eng", content="We often think of Momentum as a means of dampening oscillations and speeding up the iterations, leading to faster convergence. But it has other interesting behavior. It allows a larger range of step-sizes to be used, and creates its own oscillations. What is going on?")]
eg = api.create_editgroup(Editgroup(editor_id=admin_id))
u = api.update_work(momentum_works.ident, momentum_works, editgroup=eg.id)
u = api.update_release(momentum_works.ident, momentum_works, editgroup=eg.id)
u
api.accept_editgroup(eg.id)
momentum_works.doi = '10.23915/distill.00006'
eg = api.create_editgroup(Editgroup(editor_id=admin_id))
eg = api.create_editgroup(Editgroup(editor_id=admin_id, description="adding actual DOI"))
u = api.update_release(momentum_works.ident, momentum_works, editgroup=eg.id)
api.accept_editgroup(eg.id)
# ok, good enough for that example
dlib = prod_api.lookup_release(doi='10.1045/november14-jahja')
eg = api.create_editgroup(Editgroup(editor_id=admin_id, description="enriching d-lib article"))
for a in dlib.contribs:
    api.create_creator
    
authors = [api.create_creator(CreatorEntity(raw_name=a.raw_name), editgroup=eg.id) for a in dlib.contribs]
get_ipython().run_line_magic('pinfo', 'CreatorEntity')
authors = [api.create_creator(CreatorEntity(display_name=a.raw_name), editgroup=eg.id) for a in dlib.contribs]
dlib.contribs
dlib.contribs[i].creator_id = authors[i].ident
for i in range(3):
    dlib.contribs[i].creator_id = authors[i].ident
    
r = api.create_release(dlib, editgroup=eg.id)
dlib.release_type = 'article-journal'
r = api.create_release(dlib, editgroup=eg.id)
dlib.ident = None
dlib.release_rev = None
dlib_c = prod_api.get_container(dlib.container_id)
dlib_c.revision = None
dlib.work_id = None
c = api.create_container(dlib_c, editgroup=eg.id)
c
r = api.create_release(dlib, editgroup=eg.id)
dlib.container_id = c.ident
r = api.create_release(dlib, editgroup=eg.id)
