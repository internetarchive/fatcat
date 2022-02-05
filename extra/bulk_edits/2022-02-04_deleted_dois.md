
## Wild Volume/Issue Numbers

    fatcat-cli search release --count 'volume:99999 author:xxxxxxxxxx doi:* !release_type:stub'
    # 37

A number of these have duplicated PMID/PMCID

Should update with:

    release_type:stub release_stage: pmid: pmcid: wikidata_qid: volume: issue: pages:

    export FATCAT_AUTH_WORKER_CLEANUP=[...]
    export FATCAT_API_AUTH_TOKEN=$FATCAT_AUTH_WORKER_CLEANUP
    fatcat-cli search releases 'volume:99999 author:xxxxxxxxxx doi:* !release_type:stub' --entity-json --limit 50 \
        | jq 'select(.release_type != "stub")' -c \
        | pv -l \
        | fatcat-cli batch update release release_type=stub volume= issue= container_id= pages= pmid= pmcid= wikidata_qid= --description "Cleanup of de-registered/stub Crossref DOIs"
    # editgroup_exitdv37d5h5zlhmnc6bkwpz6a

This small batch seems to just be partial/bad metadata, but real releases:

    fatcat-cli search release --count 'volume:9999 issue:9999 container_id:4ozjmpq3dvd2xjdnavdvvq3bam'
    # 7

    fatcat-cli search releases 'volume:9999 issue:9999 container_id:4ozjmpq3dvd2xjdnavdvvq3bam' --entity-json --limit 50 \
        | jq 'select(.volume == "9999")' -c \
        | pv -l \
        | fatcat-cli batch update release release_type=stub volume= issue= container_id= pages= pmid= pmcid= wikidata_qid= --description "Cleanup of bad volume/issue numbers"
    # editgroup_bba357rix5g4znbyyz5pu4tjki

Oops, that was too agressive, not merging.

    fatcat-cli search releases 'volume:9999 issue:9999 container_id:4ozjmpq3dvd2xjdnavdvvq3bam' --entity-json --limit 50 \
        | jq 'select(.volume == "9999")' -c \
        | pv -l \
        | fatcat-cli batch update release volume= issue= --description "Cleanup of bad volume/issue numbers"
    # editgroup_vablvgsdpvexvf55zerugkcm6q

Did some other manual cleanups.

These are just bad metadata, not stubs:

    fatcat-cli search release 'volume:999 issue:999' --count
    # 456

    # first limit 50 with no auto-merge, then ran the remainder
    fatcat-cli search releases 'volume:999 issue:999' --entity-json --limit 50 \
        | jq 'select(.volume == "999")' -c \
        | pv -l \
        | fatcat-cli batch update release volume= issue= --description "Cleanup of bad volume/issue numbers"
    # editgroup_xsmvljqware4reixxw7xhuywqq

    # ok, now auto for the rest
    fatcat-cli search releases 'volume:999 issue:999' --entity-json --limit 500 \
        | jq 'select(.volume == "999")' -c \
        | pv -l \
        | fatcat-cli batch update release volume= issue= --description "Cleanup of bad volume/issue numbers" --auto-accept

## "CrossRef Listing Of Deleted DOIs"

42 releases have the same container, which was misnamed: `container_5hsepvqrxrakvcg4to77yuhbdi`

Updated that container manually.

    fatcat-cli search releases 'publisher:"Test accounts" journal:"CrossRef Listing of Deleted DOIs" doi:* !release_type:stub' --count
    # 52773

    # start small
    fatcat-cli search releases 'publisher:"Test accounts" journal:"CrossRef Listing of Deleted DOIs" doi:* !release_type:stub' --entity-json --limit 50 \
        | jq 'select(.release_type != "stub")' -c \
        | pv -l \
        | fatcat-cli batch update release release_type=stub volume= issue= container_id= pages= pmid= pmcid= wikidata_qid= --description "Cleanup of de-registered/stub Crossref DOIs"
    # editgroup_hhdr2ptknjemrjwx7kum6a4c6y

Looks good, though not really any point in removing volume/issue/pages if we
are removing `container_id`, so I won't remove that.

    fatcat-cli search releases 'publisher:"Test accounts" journal:"CrossRef Listing of Deleted DOIs" doi:* !release_type:stub' --entity-json --limit 53000 \
        | jq 'select(.release_type != "stub")' -c \
        | pv -l \
        | fatcat-cli batch update release release_type=stub container_id= pmid= pmcid= wikidata_qid= --description "Cleanup of de-registered/stub Crossref DOIs" --auto-accept


## "Test Papers"

    fatcat-cli search releases 'title:"test paper" title:ignore author:Alejandro container_id:tol7woxlqjeg5bmzadeg6qrg3e' --count
    # 38

    fatcat-cli search releases 'title:"test paper" title:ignore author:Alejandro container_id:tol7woxlqjeg5bmzadeg6qrg3e' --entity-json --limit 50 \
        | jq 'select(.release_type != "stub")' -c \
        | pv -l \
        | fatcat-cli batch update release release_type=stub --description "Mark 'testing' / 'debug' works as stubs"
    # editgroup_ikgq6flyhjds7mxe3pig3pvduu

    fatcat-cli search releases 'title:ABCDEF author:EFGH container_id:"dem5zlrvj5fozg4qkmp46jeb4a" !release_type:stub' --count
    104

    fatcat-cli search releases 'title:ABCDEF author:EFGH container_id:"dem5zlrvj5fozg4qkmp46jeb4a" !release_type:stub' --entity-json --limit 200 \
        | jq 'select(.release_type != "stub")' -c \
        | pv -l \
        | fatcat-cli batch update release release_type=stub --description "Mark 'testing' / 'debug' works as stubs"
    # editgroup_x2ki7xw35nablf3trjt43zxhpm
    # editgroup_2kinca3wgbgujiu634l6g2bxpq
    # editgroup_utkdkgvfcvbu5nr4ebiz35a5m4

    fatcat-cli search releases 'doi_prefix:10.1254 title:ABCDEF author:EFGH !type:stub' --entity-json --limit 50 \
        | jq 'select(.release_type != "stub")' -c \
        | pv -l \
        | fatcat-cli batch update release release_type=stub volume= release_stage= --description "Mark 'testing' / 'debug' works as stubs"
    # editgroup_ge26fv3gqncvde47peu67dwkn4


## Bogus DOIs (10.5555)

    fatcat-cli search releases 'doi_prefix:10.5555 pmcid:* !type:stub' --count
    133

    fatcat-cli search releases 'doi_prefix:10.5555 pmcid:* !type:stub container_id:w46j4of25bd4bjrfy4botn5ezi' --count
    119

These seem to all be bogus, never-registered DOIs. Going to remove them from the release entities.

    fatcat-cli search releases 'doi_prefix:10.5555 pmcid:* !type:stub container_id:w46j4of25bd4bjrfy4botn5ezi' --entity-json --limit 120 \
        | pv -l \
        | fatcat-cli batch update release doi= --description "Remove some non-existant DOIs from PMCID works"
    # editgroup_5xzb5d2fh5goremlrrwtlp372i
    # editgroup_rkcgwcideza4vguje2vonsyeua
    # editgroup_nfh7yg4l75fjdbjsvle2otz4ee

## PsycEXTRA

    fatcat-cli search releases 'journal:PsycEXTRA  publisher:"Test accounts" doi_registrar:crossref !type:stub' --count
    13354

Not sure what the deal is. These seem to all have been de-registered? But not
confident enough to run import. We have many of these crawled and archived.

## null/null DOIs

    fatcat-cli search releases 'title:null author:null !type:stub' --count
    16

These are not all necessarily deleted. Went through manually. Many seemed to be withdrawn, not stubs.

## Known Crossref Test Stuff

For now, not going to remove or mark these.

"The Journal Of Test Deposits": https://fatcat.wiki/container/7wqkwve2ezbtfn7gkorcuvjd3m

"Journal of Psychoceramics": https://fatcat.wiki/container/u6q4326uzjak3jx7qjmj7742ea

"Annals of Psychoceramics B": https://fatcat.wiki/container/ywwmljqajvam7gzhwpjmvahs5y
