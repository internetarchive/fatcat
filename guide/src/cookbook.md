# Cookbook

These quickstart examples gloss over a lot of details in the API. The canonical
API documentation (generated from the OpenAPI specification) is available at
<https://api.fatcat.wiki/redoc>.

The first two simple cookbook examples here include full headers. Later
examples only show python client library code snippets.

### Lookup Fulltext URLs by DOI

Often you have a DOI or other paper identifier and want to find open copies of
the paper to read. In fatcat terms, you want to lookup a release by external
identifier, then sort through any associated file entities to find the best
files and URLs to download. Note that the [Unpaywall](https://unpaywall.org)
API is custom designed for this task and you should look in to using that
instead.

This is read-only task and requires no authentication. The simple summary is
to:

1. GET the release lookup endpoint with the external identifier as a query
   parameter. Also set the `hide` parameter to elide unused fields, and the
   `expand` parameter to `files` to include related files in a single request.
2. If you get a hit (HTTP 200), sort through the `files` field (an array) and
   for each file the `urls` field (also an array) to select the best URL(s).

The URL to use would look like
<https://api.fatcat.wiki/v0/release/lookup?doi=10.1088/0264-9381/19/7/380&expand=files&hide=abstracts,refs>
in a browser. The query parameters should be URL encoded (eg, the DOI `/`
characters replaced with  with `%20`), but almost all HTTP tools and libraries
will do this automatically.

The raw HTTP request would look like:

    GET /v0/release/lookup?doi=10.1088%2F0264-9381%2F19%2F7%2F380&expand=files&hide=abstracts%2Crefs HTTP/1.1
    Accept: */*
    Accept-Encoding: gzip, deflate
    Connection: keep-alive
    Host: api.fatcat.wiki
    User-Agent: HTTPie/0.9.8

And the response (with some headers removed and JSON body paraphrased):

    HTTP/1.1 200 OK
    Connection: keep-alive
    Content-Length: 1996
    Content-Type: application/json
    Date: Tue, 17 Sep 2019 22:47:54 GMT
    X-Frame-Options: SAMEORIGIN
    X-Span-ID: caa70cff-967d-4429-96c6-71909738ab4c

    {
        "ident": "3j36alui7fcwncbc4xdaklywb4", 
        "title": "LIGO sensing system performance", 
        "publisher": "IOP Publishing", 
        "release_date": "2002-03-19", 
        "release_stage": "published", 
        "release_type": "article-journal", 
        "release_year": 2002, 
        "revision": "2e36dfbe-9a4b-4917-95bb-f02b04f6b5d0", 
        "state": "active", 
        "work_id": "ejllv7xq4rgrrffpsf3prqurwq"
        "container_id": "j5iizqxt2rainmxg6nfmpg2ds4", 
        "contribs": [],
        "ext_ids": {
            "doi": "10.1088/0264-9381/19/7/380"
        }, 
        "files": [
            {
                "ident": "vmfyqb77r5gs3pkoekzfcjgsb4", 
                "mimetype": "application/pdf", 
                "release_ids": [
                    "3j36alui7fcwncbc4xdaklywb4"
                ], 
                "revision": "66639928-d9e2-45e2-a883-36616d5b0a67", 
                "sha1": "54244fe8d35bff2db2a3ff946e60c194f68821ae", 
                "state": "active", 
                "urls": [
                    {
                        "rel": "web", 
                        "url": "http://www.gravity.uwa.edu.au/amaldi/papers/Landry.pdf"
                    }, 
                    {
                        "rel": "webarchive", 
                        "url": "https://web.archive.org/web/20081011163648/http://www.gravity.uwa.edu.au/amaldi/papers/Landry.pdf"
                    }
                ]
            }, 
            {
                "ident": "3ta26geysncdxlgswjoaiqlbyu", 
                "mimetype": "application/pdf", 
                "release_ids": [
                    "3j36alui7fcwncbc4xdaklywb4"
                ], 
                "revision": "5c7a8cb0-4710-415a-93d5-d7cb6c42dfd1", 
                "sha1": "954c0fb370af7f72a0cb47505b8793e8e5e23136", 
                "state": "active", 
                "urls": [
                    {
                        "rel": "webarchive", 
                        "url": "https://web.archive.org/web/20050624182645/http://www.gravity.uwa.edu.au/amaldi/papers/Landry.pdf"
                    }, 
                    {
                        "rel": "webarchive", 
                        "url": "https://web.archive.org/web/20091024040004/http://www.gravity.uwa.edu.au/amaldi/papers/Landry.pdf"
                    }, 
                    {
                        "rel": "web", 
                        "url": "http://www.gravity.uwa.edu.au/amaldi/papers/Landry.pdf"
                    }
                ]
            }
        ], 
    }

An `httpie` and `jq` one-liner to grap the first URL would be:

    http https://api.fatcat.wiki/v0/release/lookup doi==10.1088/0264-9381/19/7/380 expand==files hide==abstracts,refs | jq '.files[0].urls[0].url' -r

Using the python client library (`fatcat-openapi-client`), you might do
something like:

```python
import fatcat_openapi_client
from fatcat_openapi_client.rest import ApiException

conf = fatcat_openapi_client.Configuration()
conf.host = "https://api.fatcat.wiki/v0"
api = fatcat_openapi_client.DefaultApi(fatcat_openapi_client.ApiClient(conf))
doi = "10.1088/0264-9381/19/7/380"

try:
    r = api.lookup_release(doi=doi, expand="files", hide="abstracts,refs")
except ApiException as ae:
    if ae.status == 404:
        print("DOI not found!")
        return
    else:
        raise ae

print("Fatcat release found: https://fatcat.wiki/release/{}".format(r.ident))

for f in r.files:
    if f.mimetype != 'application/pdf':
        continue
    for u in r.urls:
        if u.rel == 'webarchive' and '//web.archive.org/' in u.url:
            print("Wayback PDF URL: {}".format(u.url)
            return

print("No Wayback PDF URL found")
```

A more advanced lookup tool would check for sibling releases under the same
work and provide both alternative links ("no version of record available, but
here is the pre-print") and notify the end user about any updates or
retractions to the work as a whole.

### Creating an Entity

Let's use a container (journal) entity as a simple example of mutation of the
catalog. This assumes you already have an editor account and API token, both
obtained through the web interface.

In summary:

1. Create (POST) an editgroup
2. Create (POST) the container entity as part of editgroup
3. Submit the editgroup for review
4. (privileged) Accept the editgroup

See the [API docs](https://api.fatcat.wiki/redoc#section/Authentication) for
full details of authentication.

To create an editgroup, the raw HTTP request (to
<https://api.fatcat.wiki/v0/editgroup>) and response would look like:

    POST /v0/editgroup HTTP/1.1
    Accept: application/json, */*
    Accept-Encoding: gzip, deflate
    Authorization: Bearer AgEPZGV2LmZhdGNhdC53aWtpAhYyMDE5MDEwMS1kZXYtZHVtbXkta2V5AAImZWRpdG9yX2lkID0gYWFhYWFhYWFhYWFhYmt2a2FhYWFhYWFhYWkAAht0aW1lID4gMjAxOS0wMS0wOVQwMDo1Nzo1MloAAAYgnroNha1hSftChtxHGTnLEmM/pY8MeQS/jBSV0UNvXug=
    Connection: keep-alive
    Content-Length: 2
    Content-Type: application/json
    Host: api.fatcat.wiki
    User-Agent: HTTPie/0.9.8

    {}

    HTTP/1.1 201 Created
    Connection: keep-alive
    Content-Length: 126
    Content-Type: application/json
    Date: Tue, 17 Sep 2019 23:25:55 GMT
    X-Span-ID: cc016e0e-77ae-4ca0-b1da-b0a38e48a130

    {
        "created": "2019-09-17T23:25:55.273836Z", 
        "editgroup_id": "aqhyo2ulmzfbrewn3rv7dhl65u", 
        "editor_id": "4vmpwdwxxneitkonvgm2pk6kya"
    }

It is important to parse the response to get the `editgroup_id`. Next POST to
<https://fatcat.wiki/v0/editgroup/EDITGROUP_ID/container> (with the
`editgroup_id` substituted) and the JSON container entity as the body:


    POST /v0/editgroup/aqhyo2ulmzfbrewn3rv7dhl65u/container HTTP/1.1
    Accept: application/json, */*
    Accept-Encoding: gzip, deflate
    Authorization: Bearer AgEPZGV2LmZhdGNhdC53aWtpAhYyMDE5MDEwMS1kZXYtZHVtbXkta2V5AAImZWRpdG9yX2lkID0gYWFhYWFhYWFhYWFhYmt2a2FhYWFhYWFhYWkAAht0aW1lID4gMjAxOS0wMS0wOVQwMDo1Nzo1MloAAAYgnroNha1hSftChtxHGTnLEmM/pY8MeQS/jBSV0UNvXug=
    Connection: keep-alive
    Content-Length: 54
    Content-Type: application/json
    Host: api.fatcat.wiki
    User-Agent: HTTPie/0.9.8

    {
        "issnl": "1234-5678", 
        "name": "Journal of Something"
    }

    HTTP/1.1 201 Created
    Connection: keep-alive
    Content-Length: 181
    Content-Type: application/json
    Date: Tue, 17 Sep 2019 23:30:32 GMT
    X-Span-ID: eb2f4243-ed43-4a21-bbf0-d653590fcfe2

    {
        "edit_id": "ea203496-ecb9-45c7-ac50-3cb24cdbb58f", 
        "editgroup_id": "aqhyo2ulmzfbrewn3rv7dhl65u", 
        "ident": "g3kyxylxjbej7drf6apqpfkl6i", 
        "revision": "796429d2-44a4-4ece-a9b2-e80edcd4277a"
    }

To submit an editgroup, use the update endpoint with the `submit` query
parameter set to `true`. The body should be the editgroup object (as JSON), but
is mostly ignored:


    PUT /v0/editgroup/aqhyo2ulmzfbrewn3rv7dhl65u?submit=true HTTP/1.1
    Accept: application/json, */*
    Accept-Encoding: gzip, deflate
    Authorization: Bearer AgEPZGV2LmZhdGNhdC53aWtpAhYyMDE5MDEwMS1kZXYtZHVtbXkta2V5AAImZWRpdG9yX2lkID0gYWFhYWFhYWFhYWFhYmt2a2FhYWFhYWFhYWkAAht0aW1lID4gMjAxOS0wMS0wOVQwMDo1Nzo1MloAAAYgnroNha1hSftChtxHGTnLEmM/pY8MeQS/jBSV0UNvXug=
    Connection: keep-alive
    Content-Length: 131
    Content-Type: application/json
    Host: api.fatcat.wiki
    User-Agent: HTTPie/0.9.8

    {
        "created": "2019-09-17T23:25:55.273836Z", 
        "editgroup_id": "aqhyo2ulmzfbrewn3rv7dhl65u", 
        "editor_id": "4vmpwdwxxneitkonvgm2pk6kya"
    }

    HTTP/1.1 200 OK
    Connection: keep-alive
    Content-Length: 168
    Content-Type: application/json
    Date: Tue, 17 Sep 2019 23:37:06 GMT
    X-Span-ID: c0ac0406-83ce-4e07-a892-3f83c02ec207

    {
        "created": "2019-09-17T23:25:55.273836Z", 
        "editgroup_id": "aqhyo2ulmzfbrewn3rv7dhl65u", 
        "editor_id": "4vmpwdwxxneitkonvgm2pk6kya", 
        "submitted": "2019-09-17T23:37:06.288434Z"
    }

Lastly, if your editor account as the `admin` role, you can "accept" the
editgroup using the accept endpoint:


    POST /v0/editgroup/aqhyo2ulmzfbrewn3rv7dhl65u/accept HTTP/1.1
    Accept: */*
    Accept-Encoding: gzip, deflate
    Authorization: Bearer AgEPZGV2LmZhdGNhdC53aWtpAhYyMDE5MDEwMS1kZXYtZHVtbXkta2V5AAImZWRpdG9yX2lkID0gYWFhYWFhYWFhYWFhYmt2a2FhYWFhYWFhYWkAAht0aW1lID4gMjAxOS0wMS0wOVQwMDo1Nzo1MloAAAYgnroNha1hSftChtxHGTnLEmM/pY8MeQS/jBSV0UNvXug=
    Connection: keep-alive
    Content-Length: 0
    Host: api.fatcat.wiki
    User-Agent: HTTPie/0.9.8



    HTTP/1.1 200 OK
    Connection: keep-alive
    Content-Length: 36
    Content-Type: application/json
    Date: Tue, 17 Sep 2019 23:40:21 GMT
    X-Span-ID: cb4d66f0-9e67-4908-8dff-97489cc87ca2

    {
        "message": "horray!", 
        "success": true
    }

This whole exchange is, of course, must faster with the python library:

```python
import fatcat_openapi_client
from fatcat_openapi_client.rest import ApiException

conf = fatcat_openapi_client.Configuration()
conf.host = "https://api.fatcat.wiki/v0"
conf.api_key_prefix["Authorization"] = "Bearer"
conf.api_key["Authorization"] = "AgEPZGV2LmZhdGNhdC53aWtpAhYyMDE5MDEwMS1kZXYtZHVtbXkta2V5AAImZWRpdG9yX2lkID0gYWFhYWFhYWFhYWFhYmt2a2FhYWFhYWFhYWkAAht0aW1lID4gMjAxOS0wMS0wOVQwMDo1Nzo1MloAAAYgnroNha1hSftChtxHGTnLEmM/pY8MeQS/jBSV0UNvXug="
api = fatcat_openapi_client.DefaultApi(fatcat_openapi_client.ApiClient(conf))

c = fatcat_openapi_client.ContainerEntity(
    name="Test Journal",
    issnl="1234-5678",
)
editgroup = api.create_editgroup(description="my test editgroup")
c_edit = api.create_container(editgroup.editgroup_id, c)
api.update_editgroup(editgroup.editgroup_id, submit=True)

# only if you have permissions
api.accept_editgroup(editgroup.editgroup_id)
```

### Updating an Existing Entity

It is important to ensure that edits/updates are *idempotent*, in this case
meaning that if you ran the same script twice in quick succession, no mutation
or update would occur the second time. This is usually achieved by always
fetching entities just before an edit and checking that updates are actually
necessary.

The basic process is to:

1. Fetch (GET) or Lookup (GET) the existing entity. Check that edit is actually necessary!
2. Create (POST) a new editgroup
3. Update (PUT) the entity
4. Submit (PUT) the editgroup for review

Python example code:

```python
import fatcat_openapi_client
from fatcat_openapi_client.rest import ApiException

conf = fatcat_openapi_client.Configuration()
conf.host = "https://api.fatcat.wiki/v0"
conf.api_key_prefix["Authorization"] = "Bearer"
conf.api_key["Authorization"] = "AgEPZGV2LmZhdGNhdC53aWtpAhYyMDE5MDEwMS1kZXYtZHVtbXkta2V5AAImZWRpdG9yX2lkID0gYWFhYWFhYWFhYWFhYmt2a2FhYWFhYWFhYWkAAht0aW1lID4gMjAxOS0wMS0wOVQwMDo1Nzo1MloAAAYgnroNha1hSftChtxHGTnLEmM/pY8MeQS/jBSV0UNvXug="
api = fatcat_openapi_client.DefaultApi(fatcat_openapi_client.ApiClient(conf))

new_name = "Classical and Quantum Gravity"
c = api.get_container('j5iizqxt2rainmxg6nfmpg2ds4')
if c.name == new_name:
    print("Already updated!")
    return

c.name = new_name

editgroup = api.create_editgroup(description="my test container editgroup")
c_edit = api.update_container(editgroup.editgroup_id, c.ident, c)
api.update_editgroup(editgroup.editgroup_id, submit=True)
```

### Merging Duplicate Entities

Like other mutations, be careful that any merge oprations do not clobber the
catalog if run multiple times.

Summary:

1. Fetch (GET) both entities. Ensure that merging is still required.
2. Decide which will be the "primary" entity (the other will redirect to it)
3. Create (POST) a new editgroup
4. Update (PUT) the "primary" entity with any updated metadata merged from the
   other entity (optional), and the editgroup id set
5. Update (PUT) the "other" entity with the redirect flag set to the primary's
   identifier.
6. Submit (PUT) the editgroup for review
7. Somebody (human or bot) with admin privileges will Accept (POST) the editgroup.

Python example code:

```python
import fatcat_openapi_client
from fatcat_openapi_client.rest import ApiException

conf = fatcat_openapi_client.Configuration()
conf.host = "https://api.fatcat.wiki/v0"
conf.api_key_prefix["Authorization"] = "Bearer"
conf.api_key["Authorization"] = "AgEPZGV2LmZhdGNhdC53aWtpAhYyMDE5MDEwMS1kZXYtZHVtbXkta2V5AAImZWRpdG9yX2lkID0gYWFhYWFhYWFhYWFhYmt2a2FhYWFhYWFhYWkAAht0aW1lID4gMjAxOS0wMS0wOVQwMDo1Nzo1MloAAAYgnroNha1hSftChtxHGTnLEmM/pY8MeQS/jBSV0UNvXug="
api = fatcat_openapi_client.DefaultApi(fatcat_openapi_client.ApiClient(conf))

left = api.get_creator('iimvc523xbhqlav6j3sbthuehu')
right = api.get_creator('lav6j3sbthuehuiimvc523xbhq')

# check that merge/redirect hasn't happened yet
assert left.state == 'active' and right.state == 'active'
assert left.redirect = None and right.redirect_id = None
assert left.revision != right.revision

# decide to merge "right" into "left"
if not left.orcid:
    left.orcid = right.orcid
if not left.surname:
    left.surname = right.surname

editgroup = api.create_editgroup(description="my test creator merge editgroup")
left_edit = api.update_creator(editgroup.editgroup_id, left.ident, left)
right_edit = api.update_creator(eidtgroup.editgroup_id, right.ident,
    fatcat_openapi_client.CreatorEntity(redirect=left.ident))
api.update_editgroup(editgroup.editgroup_id, submit=True)
```

### Batch Create Entities

When importing large numbers (thousands) of entities, it can be faster to use
the batch create operations instead of individual editgroup and entity
creation. Using the batch endpoints requires care because the potential to
pollute the catalog with bad entities (and the effort required to clean up) can
be much larger.

These methods always require the `admin` role, because they are the equivalent
of creation and editgroup accept.

It is not currently possible to do batch updates or deletes in a single
request.

The basic process is:

1. Confirm that input entities should be created (eg, using identifier
   lookups), and bundle into groups of 50-100 entities.
2. Batch create (POST) a set of entities, with editgroup metadata included
   along with list of entities (all of a single type). Entire batch is inserted
   in a single request.


```python
import fatcat_openapi_client
from fatcat_openapi_client.rest import ApiException

conf = fatcat_openapi_client.Configuration()
conf.host = "https://api.fatcat.wiki/v0"
conf.api_key_prefix["Authorization"] = "Bearer"
conf.api_key["Authorization"] = "AgEPZGV2LmZhdGNhdC53aWtpAhYyMDE5MDEwMS1kZXYtZHVtbXkta2V5AAImZWRpdG9yX2lkID0gYWFhYWFhYWFhYWFhYmt2a2FhYWFhYWFhYWkAAht0aW1lID4gMjAxOS0wMS0wOVQwMDo1Nzo1MloAAAYgnroNha1hSftChtxHGTnLEmM/pY8MeQS/jBSV0UNvXug="
api = fatcat_openapi_client.DefaultApi(fatcat_openapi_client.ApiClient(conf))

releases = [
    ReleaseEntity(
      ext_ids=ReleaseExtIds(doi="10.123/456"),
      title="Dummy Release",
    ),
    ReleaseEntity(
      ext_ids=ReleaseExtIds(doi="10.123/789"),
      title="Another Dummy Release",
    ),
    # ... more releases here ...
]

# check that releases don't exist already; this could be a filter
for r in releases:
    existing = None
    try:
        existing = api.lookup_release(doi=r.ext_ids.doi)
    except ApiException as ae
        assert ae.status == 404
    assert existing is None

# ensure our batch size isn't too large
assert len(releases) <= 100

editgroup = api.create_release_auto_batch(
    fatcat_openapi_client.ReleaseAutoBatch(
        editgroup=fatcat_openapi_client.Editgroup(
            description="my test batch",
        ),
        entity_list=releases,
    )
)
```

### Import New Files Linked to Releases

Let's say you knew of many open access PDFs, including their SHA-1, size, and a URL:

    10.123/456  7043946a7afe0ee32c9d4c22a9b3fc2ba6d34b42    7238    https://archive.org/download/open_access_files/456.pdf
    10.123/789  350a8d5c6fac151ec2c81d4df5d58d14aeefc72f    1277    https://archive.org/download/open_access_files/789.pdf
    10.123/900  9d9a9868a661b13c32fd38021addadb7b4a31122     166    https://archive.org/download/open_access_files/900.pdf
    [...]

The process for adding these could be something like:

1. For each row, check if file with SHA-1 exists; if so, skip
2. For each row, lookup the release by DOI; if it doesn't exist, skip
3. Transform into File entities
3. Group entities into batches
4. Submit batches to API

There are multiple ways to structure code to do this. You may want to look at
the `importer` class under `python/fatcat_tools/importers/common.py`, and other
existing import scripts in that directory for a framework to structure this
type of import.

Here is a simpler example using only the python library:

```python

# TODO: actually test this code

import sys
import fatcat_openapi_client
from fatcat_openapi_client import *
from fatcat_openapi_client.rest import ApiException

conf = fatcat_openapi_client.Configuration()
conf.host = "https://api.fatcat.wiki/v0"
conf.api_key_prefix["Authorization"] = "Bearer"
conf.api_key["Authorization"] = "AgEPZGV2LmZhdGNhdC53aWtpAhYyMDE5MDEwMS1kZXYtZHVtbXkta2V5AAImZWRpdG9yX2lkID0gYWFhYWFhYWFhYWFhYmt2a2FhYWFhYWFhYWkAAht0aW1lID4gMjAxOS0wMS0wOVQwMDo1Nzo1MloAAAYgnroNha1hSftChtxHGTnLEmM/pY8MeQS/jBSV0UNvXug="
api = fatcat_openapi_client.DefaultApi(fatcat_openapi_client.ApiClient(conf))

HAVE_ADMIN = False

def try_row(fields):
    # remove any extra whitespace
    fields = [f.strip() for f in fields]
    doi = fields[0]
    sha1 = fields[1]
    size = int(fields[2])
    url = fields[3]

    # check for existing file
    try:
        existing = api.lookup_file(sha1=sha1)
        print("File with SHA-1 exists: {}".format(sha1))
        return None
    except ApiException as ae:
        if ae.status != 404:
            raise ae

    # lookup release by DOI
    try:
        release = api.lookup_release(doi=doi)
    except ApiException as ae:
        if ae.status == 404:
            print("No existing release for DOI: {}".format(doi))
            return None
        else:
            raise ae

    fe = FileEntity(
        release_ids=[release.ident],
        sha1=sha1,
        size=size,
        urls=[FileUrl(rel="archive", url=url)],
    )
    return fe

def run(input_file):
    file_entities = []
    for line in input_file:
        fe = try_row(line.split('\t'))
        if fe:
            file_entities.append(fe)
    if not file_entities:
        print("Tried all lines, nothing to do!")

    # TODO: iterate over fixed-size batches
    first_batch = file_entities[:100]

    # easy way: create as a batch if you have permission
    if HAVE_ADMIN:
        editgroup = api.create_release_auto_batch(
            fatcat_openapi_client.ReleaseAutoBatch(
                editgroup=fatcat_openapi_client.Editgroup(
                    description="my test batch",
                ),
                entity_list=releases,
            )
        )
        return

    # longer way: create one-at-a-time
    editgroup = api.create_editgroup(Editgroup(
        description="batch import of files-by-DOI. Data from XYZ",
        extra={
            # put the name of your script/project here
            'agent': 'tutorial_example_script',
        },
    ))

    for fe in first_batch:
        edit = api.create_file(editgroup.editgroup_id, fe)

    # submit for review
    api.update_editgroup(editgroup.editgroup_id, editgroup, submit=True)
    print("Submitted editgroup: https://fatcat.wiki/editgroup/{}".format(editgroup.editgroup_id))

    print("Done!")

if __name__=='__main__':
    if len(sys.argv) != 2:
        print("Pass input TSV file as argument")
        sys.exit(-1)
    with open(sys.argv[1], 'r') as input_file:
        run(input_file)

# ensure our batch size isn't too large
assert len(releases) <= 100

editgroup = api.create_release_auto_batch(
    fatcat_openapi_client.ReleaseAutoBatch(
        editgroup=fatcat_openapi_client.Editgroup(
            description="my test batch",
        ),
        entity_list=releases,
    )
)
```

