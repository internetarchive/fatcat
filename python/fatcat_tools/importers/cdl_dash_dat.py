#!/usr/bin/env python3

import os
import sys
import json
import magic
import urllib
import hashlib
import mimetypes
import subprocess

import fatcat_client
from fatcat_client import *
from .common import clean
from .crossref import lookup_license_slug


def single_file(prefix, path):

    full = prefix + path
    size_bytes = os.stat(full).st_size

    hashes = [
        hashlib.md5(),
        hashlib.sha1(),
        hashlib.sha256(),
    ]
    with open(full, 'rb') as fp:
        while True:
            data = fp.read(2**20)
            if not data:
                break
            for h in hashes:
                h.update(data)
    mime = magic.Magic(mime=True).from_file(full)
    if mime == 'application/octet-stream':
        # magic apparently isn't that great; try using filename as well
        guess = mimetypes.guess_type(full)[0]
        if guess:
            mime = guess

    fsm = FilesetEntityManifest(
        path=path,
        size=size_bytes,
        md5=hashes[0].hexdigest(),
        sha1=hashes[1].hexdigest(),
        sha256=hashes[2].hexdigest(),
        extra=dict(mimetype=mime))
    return fsm

def make_manifest(base_dir):
    manifest = []
    for root, dirs, files in os.walk(base_dir):
        for f in files:
            manifest.append(single_file(root, f))
    return manifest


def cdl_dash_release(meta, extra=None):

    if not extra:
        extra = dict()

    assert meta['identifier']['type'] == 'DOI'
    doi = meta['identifier']['value'].lower()
    assert doi.startswith('10.')

    ark_id = None
    for extid in meta.get('alternativeIdentifiers', []):
        if extid['value'].startswith('ark:'):
            ark_id = extid['value']
    assert ark_id
    extra['ark_id'] = ark_id

    license_slug = lookup_license_slug(meta['rights']['uri'])

    abstracts = []
    for desc in meta['descriptions']:
        if desc['type'] == "abstract":
            abstracts.append(ReleaseEntityAbstracts(
                mimetype="text/html",
                content=clean(desc['value'])))
            #print(abstracts)
    if not abstracts:
        abstracts = None
    
    contribs = []
    for creator in meta['creator']:
        contribs.append(ReleaseContrib(
            # TODO: given_name=creator['given'],
            # TODO: surname=creator['family'],
            # sorry everybody
            raw_name="{} {}".format(creator['given'], creator['family']),
            raw_affiliation=creator.get('affiliation'),
            role="author", # presumably, for these datasets?
        ))

    r = ReleaseEntity(
        doi=doi,
        title=clean(meta['title'], force_xml=True),
        publisher=clean(meta['publisher']),
        release_year=int(meta['publicationYear']),
        release_type="dataset",
        license_slug=license_slug,
        contribs=contribs,
        abstracts=abstracts,
        extra=extra,
    )
    return r

def make_release_fileset(dat_path):

    if dat_path.endswith('/'):
        dat_path = dat_path[:-1]
    dat_discovery = dat_path
    extra = dict()
    assert len(dat_discovery) == 64

    with open(dat_path + "/cdl_dash_metadata.json", 'r') as fp:
        meta_dict = json.loads(fp.read())
    
    release = cdl_dash_release(meta_dict)
    ark_id = release.extra['ark_id']

    dash_version = None
    # really crude XML parse-out
    with open(dat_path + "/stash-wrapper.xml", 'r') as fp:
        for line in fp:
            line = line.strip()
            if line.startswith("<st:version_number>"):
                dash_version = int(line[19:].split('<')[0])
    assert dash_version is not None
    extra['cdl_dash'] = dict(version=dash_version)
    release.extra['cdl_dash'] = dict(version=dash_version)

    manifest = make_manifest(dat_path + "/files/")

    bundle_url = dict(
        url="https://merritt.cdlib.org/u/{}/{}".format(
            urllib.parse.quote(ark_id, safe=''),
            dash_version),
        rel="repo-bundle")
    repo_url = dict(
        url="https://merritt.cdlib.org/d/{}/{}/".format(
            urllib.parse.quote(ark_id, safe=''),
            dash_version),
        rel="repo")
    dat_url = dict(
        url="dat://{}/files/".format(dat_discovery),
        rel="dweb")
    fs = FilesetEntity(
        urls=[bundle_url, repo_url, dat_url],
        release_ids=None,
        manifest=manifest,
        extra=extra)
    return (release, fs)

def auto_cdl_dash_dat(api, dat_path, release_id=None, editgroup_id=None):

    git_rev = subprocess.check_output(
        ["git", "describe", "--always"]).strip().decode('utf-8')

    (release, fileset) = make_release_fileset(dat_path)

    if not editgroup_id:
        eg = api.create_editgroup(Editgroup(
            description="One-off import of dataset(s) from CDL/DASH repository (via IA, Dat dweb pilot project)",
            extra=dict(
                git_rev=git_rev,
                agent="fatcat_tools.auto_cdl_dash_dat")))
        editgroup_id = eg.editgroup_id

    if not release_id and release.doi:
        try:
            r = api.lookup_release(doi=release.doi)
            release_id = r.ident
        except fatcat_client.rest.ApiException:
            pass
    if not release_id:
        edit = api.create_release(release, editgroup_id=editgroup_id)
        release_id = edit.ident

    release = api.get_release(release_id, expand="filesets")
    if len(release.filesets):
        print("A fileset already exists for release {}".format(release.ident))
        return (None, None, None)

    fileset.release_ids = [release.ident]
    edit = api.create_fileset(fileset, editgroup_id=editgroup_id)
    fileset = api.get_fileset(edit.ident)
    return (editgroup_id, release, fileset)

if __name__=='__main__':
    # pass this a discovery key that has been cloned to the local directory
    print(make_release_fileset(sys.argv[1]))
