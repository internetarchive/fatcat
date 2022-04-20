
See metadata cleanups for context. Basically a couple tens of thousands of sample/spam articles hosted on the domain isiarticles.com.

## Prod Updates

Start small:

    export FATCAT_API_HOST=https://api.fatcat.wiki
    export FATCAT_AUTH_WORKER_CLEANUP=[...]
    export FATCAT_API_AUTH_TOKEN=$FATCAT_AUTH_WORKER_CLEANUP

    fatcat-cli search file domain:isiarticles.com --entity-json -n0 \
        | rg -v '"content_scope"' \
        | rg 'isiarticles.com/' \
        | head -n50 \
        | pv -l \
        | fatcat-cli batch update file release_ids= content_scope=sample --description 'Un-link and mark isiarticles PDFs as content_scope=sample' --auto-accept
    # editgroup_ihx75kzsebgzfisgjrv67zew5e

The full batch:

    fatcat-cli search file domain:isiarticles.com --entity-json -n0 \
        | rg -v '"content_scope"' \
        | rg 'isiarticles.com/' \
        | pv -l \
        | fatcat-cli batch update file release_ids= content_scope=sample --description 'Un-link and mark isiarticles PDFs as content_scope=sample' --auto-accept

And some more with ':80' in the URL:

    fatcat-cli search file domain:isiarticles.com '!content_scope:*' --entity-json -n0 \
        | rg -v '"content_scope"' \
        | rg 'isiarticles.com:80/' \
        | pv -l \
        | fatcat-cli batch update file release_ids= content_scope=sample --description 'Un-link and mark isiarticles PDFs as content_scope=sample' --auto-accept

Verify:

    fatcat-cli search file domain:isiarticles.com '!content_scope:*' --count
    0
