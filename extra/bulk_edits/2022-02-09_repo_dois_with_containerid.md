
Some institutional repositories register DOIs for pre-prints with the metadata
for the version of record included, including an ISSN number. This results in
the release entities getting the `container_id` of the actual journal, and show
up in preservation dashboards, etc.

## Columbia University

Here is an example search query, showing two works, both marked today as "PLoS Medicine":

    https://fatcat.wiki/release/search?q=%22Contraceptive+use+among+adolescent+and+young+women+in+North+and+South+Kivu%2C+Democratic+Republic+of+the+Congo%3A+A+cross-sectional+population-based+survey%22&generic=1

Some count queries:

    fatcat-cli search releases doi_prefix:10.7916 doi_registrar:datacite 'container_id:*' release_stage:published --count
    # 10870

    fatcat-cli search releases doi_prefix:10.7916 doi_registrar:datacite 'container_id:*' release_stage:published --entity-json -n0 \
        | rg '"Columbia University"' \
        | rg '"IsVariantFormOf"' \
        | pv -l \
        > /dev/null
    # 10.7k 0:09:39

So, most of these.

Let's update these to `release_stage=submitted` and `container_id=`.

    export FATCAT_AUTH_WORKER_CLEANUP=[...]
    export FATCAT_API_AUTH_TOKEN=$FATCAT_AUTH_WORKER_CLEANUP

    # start small
    fatcat-cli search releases doi_prefix:10.7916 doi_registrar:datacite 'container_id:*' release_stage:published --entity-json --limit 50 \
        | jq 'select(.container_id != null)' -c \
        | rg '"Columbia University"' \
        | rg '"IsVariantFormOf"' \
        | pv -l \
        | fatcat-cli batch update release release_stage=submitted container_id= --description "Remove container linkage for Columbia University repository deposits"
    # editgroup_grxwpieqvvenxfaxwojnud4lla

    # full auto
    fatcat-cli search releases doi_prefix:10.7916 doi_registrar:datacite 'container_id:*' release_stage:published --entity-json --limit 11000 \
        | jq 'select(.container_id != null)' -c \
        | rg '"Columbia University"' \
        | rg '"IsVariantFormOf"' \
        | pv -l \
        | fatcat-cli batch update release release_stage=submitted container_id= --description "Remove container linkage for Columbia University repository deposits" --auto-accept

Also created a patch for fatcat datacite importer to not link these in the future.

## "RWTH Publications"

    https://fatcat.wiki/release/search?q=%22Predicting+survival+from+colorectal+cancer+histology+slides+using+deep+learning%3A+A+retrospective+multicenter+study%22&generic=1

    doi_prefix:10.18154


    fatcat-cli search releases doi_prefix:10.18154 'container_id:*' --count
    # 11364

    fatcat-cli search releases doi_prefix:10.18154 'container_id:*' doi_registrar:datacite --count
    # 11364

    fatcat-cli search releases doi_prefix:10.18154 'container_id:*' doi_registrar:datacite affiliation:RWTH --count
    # 6257

    fatcat-cli search releases doi_prefix:10.18154 'container_id:*' doi_registrar:datacite --entity-json -n0 \
        | rg 'RWTH' \
        | rg '10.18154/rwth-' \
        | rg '"IsVariantFormOf"' \
        | pv -l \
        > /dev/null
    # many/all? at least 5k, cut off there

Ok, do updates:

    # start small
    fatcat-cli search releases doi_prefix:10.18154 'container_id:*' doi_registrar:datacite affiliation:RWTH --entity-json -n50 \
        | jq 'select(.container_id != null)' -c \
        | rg 'RWTH' \
        | rg '10.18154/rwth-' \
        | rg '"IsVariantFormOf"' \
        | fatcat-cli batch update release container_id= --description "Remove container linkage for RWTH repository deposits"
    # Got 6257 hits in 1087ms
    # editgroup_cb2vdn7npfg63muppawbhzrhjq

    # do the rest
    fatcat-cli search releases doi_prefix:10.18154 'container_id:*' doi_registrar:datacite affiliation:RWTH --entity-json -n12000 \
        | jq 'select(.container_id != null)' -c \
        | rg 'RWTH' \
        | rg '10.18154/rwth-' \
        | rg '"IsVariantFormOf"' \
        | pv -l \
        | fatcat-cli batch update release container_id= --description "Remove container linkage for RWTH repository deposits" --auto-accept
    # Got 6207 hits in 696ms
    # 6.00k 0:16:37 [6.01 /s]

After that process, there were still many mis-matched DOIs, so relaxing
constraints. This repository *does* contain a bunch of publications from RWTH
itself (books, conference series, etc), so don't want to update everything.

    fatcat-cli search releases doi_prefix:10.18154 'container_id:*' doi_registrar:datacite '!journal:RWTH' '!container_id:m2cho7mmmbgxzdpfz7cmjgegbu' --count
    # 3946

    fatcat-cli search releases doi_prefix:10.18154 'container_id:*' doi_registrar:datacite '!journal:RWTH' '!container_id:m2cho7mmmbgxzdpfz7cmjgegbu' --entity-json -n6000 \
        | jq 'select(.container_id != null)' -c \
        | rg 'RWTH' \
        | rg '10.18154/rwth-20' \
        | rg '"IsVariantFormOf"' \
        | pv -l \
        | fatcat-cli batch update release container_id= --description "Remove container linkage for RWTH repository deposits" --auto-accept
    # Got 3946 hits in 77ms

Specifically, some more PLOS ones:

    fatcat-cli search releases doi_prefix:10.18154 'container_id:*' doi_registrar:datacite '!journal:RWTH' '!container_id:m2cho7mmmbgxzdpfz7cmjgegbu' journal:plos --count
    # 338

    fatcat-cli search releases doi_prefix:10.18154 'container_id:*' doi_registrar:datacite '!journal:RWTH' '!container_id:m2cho7mmmbgxzdpfz7cmjgegbu' 'journal:plos' --entity-json -n500 \
        | jq 'select(.container_id != null)' -c \
        | rg '10.18154/rwth-' \
        | pv -l \
        | fatcat-cli batch update release container_id= --description "Remove container linkage for RWTH repository deposits" --auto-accept
    # Got 338 hits in 33ms

## DESY Pre-Print Server (PUBDB)

    https://fatcat.wiki/release/search?q=%22viral+phosphatase+adaptor+that+promotes+herpes+simplex+virus+replication+and+spread%22+type%3Aarticle-journal+%21title%3Acorrection

    fatcat-cli search releases doi_prefix:10.3204 'container_id:*' doi_registrar:datacite publisher:DESY --count
    # 313

    fatcat-cli search releases doi_prefix:10.3204 'container_id:*' doi_registrar:datacite --count
    # 6679

    fatcat-cli search releases doi_prefix:10.3204 'container_id:*' doi_registrar:datacite --entity-json -n7000 \
        | jq 'select(.container_id != null)' -c \
        | rg '10.3204/(pubdb|phppubdb)-' \
        | rg '"IsVariantFormOf"' \
        | pv -l \
        > /dev/null
    # at least hundreds

    # start small
    fatcat-cli search releases doi_prefix:10.3204 'container_id:*' doi_registrar:datacite --entity-json -n50 \
        | jq 'select(.container_id != null)' -c \
        | rg '10.3204/(pubdb|phppubdb)-' \
        | rg '"IsVariantFormOf"' \
        | fatcat-cli batch update release container_id= --description "Remove container linkage for DESY repository deposits"
    # Got 6679 hits in 368ms
    # editgroup_vhcxvqjyinhxfplkoqjtprnxj4

    fatcat-cli search releases doi_prefix:10.3204 'container_id:*' doi_registrar:datacite --entity-json -n7000 \
        | jq 'select(.container_id != null)' -c \
        | rg '10.3204/(pubdb|phppubdb)-' \
        | rg '"IsVariantFormOf"' \
        | fatcat-cli batch update release container_id= --description "Remove container linkage for DESY repository deposits" --auto-accept


## Kluedo: Publication Server of University of Kaiserslautern

doi:10.26204/kluedo/6163

    fatcat-cli search releases doi_prefix:10.26204 'container_id:*' --count
    # 7

Whew, an easy one!

    fatcat-cli search releases doi_prefix:10.26204 'container_id:*' --entity-json -n50 \
        | jq 'select(.container_id != null)' -c \
        | rg '10.26204/kluedo/' \
        | fatcat-cli batch update release release_stage=submitted container_id= --description "Remove container linkage for 'Kluedo' repository deposits"
    # Got 7 hits in 20ms
    # editgroup_tmyyg4yl7vbg7mveyfcdxhptfu

## Universitat Bayreuth

    doi:10.15495/epub_ubt_00005577 

    fatcat-cli search releases doi_prefix:10.15495 'container_id:*' --count
    # 554

Great, also not very large.

    # start small
    fatcat-cli search releases doi_prefix:10.15495 'container_id:*' --entity-json -n50 \
        | jq 'select(.container_id != null)' -c \
        | rg '10.15495/epub_ubt_' \
        | fatcat-cli batch update release container_id= --description "Remove container linkage for University of Bayreuth  repository deposits"
    # 554
    # editgroup_6oubgez7jrfabprdckijvijsa4

    fatcat-cli search releases doi_prefix:10.15495 'container_id:*' --entity-json -n600 \
        | jq 'select(.container_id != null)' -c \
        | rg '10.15495/epub_ubt_' \
        | fatcat-cli batch update release container_id= --description "Remove container linkage for University of Bayreuth  repository deposits" --auto-accept
    # did a variant with `publisher:Bayreuth`, which only matched a single release
    # Got 503 hits in 310ms

Could also have filtered on publisher "University of Bayreuth", in the post-fetch part.
