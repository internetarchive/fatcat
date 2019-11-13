
Fatcat Identifiers
=======================

AKA, `fcid`

## Public Use / Reference

When referencing identifiers in external databases, should prefix with the
entity type. Eg:

    release_hsmo6p4smrganpb3fndaj2lon4
    editgroup_qinmjr2lbvgd3mbt7mifir23fy

Or with a prefix:

    fatcat:release_hsmo6p4smrganpb3fndaj2lon4

As a usability affordance, the public web interface (though not API) should do
permanent redirects HTTP (301 or 308) to the canonical page like:

    https://fatcat.wiki/release_hsmo6p4smrganpb3fndaj2lon4
    HTTP 301 => https://fatcat.wiki/release/hsmo6p4smrganpb3fndaj2lon4

However, no intention to use identifiers in this schema in the API itself?
