"""
Flask endpoints for reference (citation) endpoints. Eg, listing references
"inbound" and "outbound" from a specific release or work.
"""

from typing import Optional

from flask import render_template, abort, redirect, request
from fatcat_openapi_client import *
from fatcat_openapi_client.rest import ApiException

from fatcat_tools.references import enrich_inbound_refs_fatcat, enrich_outbound_refs_fatcat, get_inbound_refs, get_outbound_refs
from fatcat_web import app, api, auth_api
from fatcat_web.forms import *
from fatcat_web.entity_helpers import *


@app.route('/release/<string(length=26):ident>/refs/inbound', methods=['GET'])
def release_view_refs_inbound(ident):

    # lookup release ident, ensure it exists
    try:
        release = api.get_release(ident)
    except ApiException as ae:
        abort(ae.status)

    offset = request.args.get('offset', '0')
    offset = max(0, int(offset)) if offset.isnumeric() else 0

    hits = get_inbound_refs(release_ident=ident, es_client=app.es_client, offset=offset, limit=30)
    enriched_refs = enrich_inbound_refs_fatcat(hits.result_refs, fatcat_api_client=api, expand="container,files,webcaptures")

    return render_template('release_view_fuzzy_refs.html', direction="inbound", entity=release, hits=hits, enriched_refs=enriched_refs), 200

@app.route('/release/<string(length=26):ident>/refs/outbound', methods=['GET'])
def release_view_refs_outbound(ident):

    # lookup release ident, ensure it exists
    try:
        release = api.get_release(ident)
    except ApiException as ae:
        abort(ae.status)

    offset = request.args.get('offset', '0')
    offset = max(0, int(offset)) if offset.isnumeric() else 0

    hits = get_outbound_refs(release_ident=ident, es_client=app.es_client, offset=offset, limit=30)
    enriched_refs = enrich_outbound_refs_fatcat(hits.result_refs, fatcat_api_client=api, expand="container,files,webcaptures")

    return render_template('release_view_fuzzy_refs.html', direction="outbound", entity=release, hits=hits, enriched_refs=enriched_refs), 200
