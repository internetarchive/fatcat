"""
Flask endpoints for reference (citation) endpoints. Eg, listing references
"inbound" and "outbound" from a specific release or work.
"""

import json

from flask import Response, jsonify, render_template, request
from fuzzycat.grobid_unstructured import (
    grobid_api_process_citation,
    grobid_ref_to_release,
    transform_grobid_ref_xml,
)
from fuzzycat.simple import close_fuzzy_biblio_matches, close_fuzzy_release_matches

from fatcat_tools.references import (
    RefHits,
    enrich_inbound_refs,
    enrich_outbound_refs,
    get_inbound_refs,
    get_outbound_refs,
)
from fatcat_tools.transforms.access import release_access_options
from fatcat_tools.transforms.entities import entity_to_dict
from fatcat_web import api, app
from fatcat_web.cors import crossdomain
from fatcat_web.entity_helpers import generic_get_entity
from fatcat_web.forms import ReferenceMatchForm


def _refs_web(direction, release_ident=None, work_ident=None, openlibrary_id=None, wikipedia_article=None) -> RefHits:
    offset = request.args.get('offset', '0')
    offset = max(0, int(offset)) if offset.isnumeric() else 0
    limit = request.args.get('limit', '30')
    limit = min(max(0, int(limit)), 100) if limit.isnumeric() else 30
    if direction == "in":
        hits = get_inbound_refs(
            release_ident=release_ident,
            work_ident=work_ident,
            openlibrary_work=openlibrary_id,
            es_client=app.es_client,
            offset=offset,
            limit=limit,
        )
        hits.result_refs = enrich_inbound_refs(
            hits.result_refs,
            fatcat_api_client=api,
            expand="container,files,webcaptures",
        )
    elif direction == "out":
        hits = get_outbound_refs(
            release_ident=release_ident,
            wikipedia_article=wikipedia_article,
            work_ident=work_ident,
            es_client=app.es_client,
            offset=offset,
            limit=limit,
        )
        hits.result_refs = enrich_outbound_refs(
            hits.result_refs,
            fatcat_api_client=api,
            expand="container,files,webcaptures",
        )
    else:
        raise ValueError()
    return hits


@app.route('/release/<string(length=26):ident>/refs-in', methods=['GET'])
def release_view_refs_inbound(ident):
    if request.accept_mimetypes.best == "application/json":
        return release_view_refs_inbound_json(ident)

    release = generic_get_entity("release", ident)
    hits = _refs_web("in", release_ident=ident)
    return render_template('release_view_fuzzy_refs.html', direction="in", entity=release, hits=hits), 200


@app.route('/release/<string(length=26):ident>/refs-out', methods=['GET'])
def release_view_refs_outbound(ident):
    if request.accept_mimetypes.best == "application/json":
        return release_view_refs_outbound_json(ident)

    release = generic_get_entity("release", ident)
    hits = _refs_web("out", release_ident=ident)
    return render_template('release_view_fuzzy_refs.html', direction="out", entity=release, hits=hits), 200

@app.route('/openlibrary/OL<int:id_num>W/refs-in', methods=['GET'])
def openlibrary_view_refs_inbound(id_num):
    if request.accept_mimetypes.best == "application/json":
        return openlibrary_view_refs_inbound_json(id_num)

    openlibrary_id = f"OL{id_num}W"
    hits = _refs_web("in", openlibrary_id=openlibrary_id)
    return render_template('openlibrary_view_fuzzy_refs.html', openlibrary_id=openlibrary_id, direction="in", hits=hits), 200

@app.route('/wikipedia/<string(length=2):wiki_lang>:<string:wiki_article>/refs-out', methods=['GET'])
def wikipedia_view_refs_outbound(wiki_lang: str, wiki_article: str):
    if request.accept_mimetypes.best == "application/json":
        return wikipedia_view_refs_outbound_json(wiki_lang, wiki_article)

    wiki_url = f"https://{wiki_lang}.wikipedia.org/wiki/{wiki_article}"
    wiki_article = wiki_article.replace('_', ' ')
    wikipedia_article = wiki_lang + ":" + wiki_article
    hits = _refs_web("out", wikipedia_article=wikipedia_article)
    return render_template('wikipedia_view_fuzzy_refs.html', wiki_article=wiki_article, wiki_lang=wiki_lang, wiki_url=wiki_url, direction="out", hits=hits), 200

@app.route('/reference/match', methods=['GET', 'POST'])
def reference_match():

    grobid_status = None
    grobid_dict = None

    form = ReferenceMatchForm()
    if not form.is_submitted() and request.args.get('submit_type'):
        form = ReferenceMatchForm(request.args)

    if form.is_submitted() or request.args.get('title'):
        if form.validate():
            if form.submit_type.data == 'parse':
                resp_xml = grobid_api_process_citation(form.raw_citation.data)
                if not resp_xml:
                    grobid_status = "failed"
                    return render_template('reference_match.html', form=form, grobid_status=grobid_status), 400
                grobid_dict = transform_grobid_ref_xml(resp_xml)
                if not grobid_dict:
                    grobid_status = "empty"
                    return render_template('reference_match.html', form=form, grobid_status=grobid_status), 200
                #print(grobid_dict)
                release_stub = grobid_ref_to_release(grobid_dict)
                # remove empty values from GROBID parsed dict
                grobid_dict = {k: v for k, v in grobid_dict.items() if v is not None}
                form = ReferenceMatchForm.from_grobid_parse(grobid_dict, form.raw_citation.data)
                grobid_status = "success"
                matches = close_fuzzy_release_matches(es_client=app.es_client, release=release_stub, match_limit=10) or []
            elif form.submit_type.data == 'match':
                matches = close_fuzzy_biblio_matches(es_client=app.es_client, biblio=form.data, match_limit=10) or []
            else:
                raise NotImplementedError()

            for m in matches:
                # expand releases more completely
                m.release = api.get_release(m.release.ident, expand="container,files,filesets,webcaptures", hide="abstract,refs")
                # hack in access options
                m.access_options = release_access_options(m.release)

            return render_template('reference_match.html', form=form, grobid_dict=grobid_dict, grobid_status=grobid_status, matches=matches), 200

        elif form.errors:
            return render_template('reference_match.html', form=form), 400

    return render_template('reference_match.html', form=form), 200


### Pseudo-APIs #############################################################

@app.route('/release/<string(length=26):ident>/refs-out.json', methods=['GET', 'OPTIONS'])
@crossdomain(origin='*',headers=['access-control-allow-origin','Content-Type'])
def release_view_refs_outbound_json(ident):
    hits = _refs_web("out", release_ident=ident)
    return Response(hits.json(exclude_unset=True), mimetype="application/json")


@app.route('/release/<string(length=26):ident>/refs-in.json', methods=['GET', 'OPTIONS'])
@crossdomain(origin='*',headers=['access-control-allow-origin','Content-Type'])
def release_view_refs_inbound_json(ident):
    hits = _refs_web("in", release_ident=ident)
    return Response(hits.json(exclude_unset=True), mimetype="application/json")

@app.route('/openlibrary/OL<int:id_num>W/refs-in.json', methods=['GET', 'OPTIONS'])
@crossdomain(origin='*',headers=['access-control-allow-origin','Content-Type'])
def openlibrary_view_refs_inbound_json(id_num):
    openlibrary_id = f"OL{id_num}W"
    hits = _refs_web("in", openlibrary_id=openlibrary_id)
    return Response(hits.json(exclude_unset=True), mimetype="application/json")

@app.route('/wikipedia/<string(length=2):wiki_lang>:<string:wiki_article>/refs-out.json', methods=['GET', 'OPTIONS'])
@crossdomain(origin='*',headers=['access-control-allow-origin','Content-Type'])
def wikipedia_view_refs_outbound_json(wiki_lang: str, wiki_article: str):
    wiki_article = wiki_article.replace('_', ' ')
    wikipedia_article = wiki_lang + ":" + wiki_article
    hits = _refs_web("out", wikipedia_article=wikipedia_article)
    return Response(hits.json(exclude_unset=True), mimetype="application/json")


@app.route('/reference/match.json', methods=['GET', 'OPTIONS'])
@crossdomain(origin='*',headers=['access-control-allow-origin','Content-Type'])
def reference_match_json():
    form = ReferenceMatchForm(request.args)
    if form.validate():
        if form.submit_type.data == 'match':
            matches = close_fuzzy_biblio_matches(es_client=app.es_client, biblio=form.data, match_limit=10) or []
        else:
            raise NotImplementedError()
        resp = []
        for m in matches:
            # expand releases more completely
            m.release = api.get_release(m.release.ident, expand="container,files,filesets,webcaptures", hide="abstract,refs")
            # hack in access options
            m.access_options = release_access_options(m.release)

            # and manually convert to dict (for jsonify)
            info = m.__dict__
            info['release'] = entity_to_dict(m.release)
            info['access_options'] = [o.dict() for o in m.access_options]
            resp.append(info)
        return jsonify(resp), 200
    else:
        return Response(json.dumps(dict(errors=form.errors)), mimetype="application/json", status=400)
