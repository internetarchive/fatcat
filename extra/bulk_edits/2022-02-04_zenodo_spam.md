
## Cleanup Zenodo Spam

    fatcat-cli search releases 'year:2021 title:"DOWNLOAD MP3:" doi_prefix:10.5281 !journal:* in_ia:false' --count
    # 29653

    fatcat-cli search releases 'year:2021 title:"DOWNLOAD MP3:" title:"album download" doi_prefix:10.5281 !journal:* in_ia:false' --count
    # 29043

Let's nuke 'em:

    # start small, not automatic
    fatcat-cli search releases 'year:2021 title:"DOWNLOAD MP3:" title:"album download" doi_prefix:10.5281 !journal:* in_ia:false !release_type:stub' --entity-json --limit 50 \
        | jq 'select(.release_type != "stub")' -c \
        | fatcat-cli batch update release release_type=stub withdrawn_status=spam --description "Mark Zenodo spam as such"
    # editgroup_yizvqtz24bfv3jd6vmawiqnojm

    # ok, scale it up!
    fatcat-cli search releases 'year:2021 title:"DOWNLOAD MP3:" title:"album download" doi_prefix:10.5281 !journal:* in_ia:false !release_type:stub' --entity-json --limit 30000 \
        | jq 'select(.release_type != "stub")' -c \
        | pv -l \
        | fatcat-cli batch update release release_type=stub withdrawn_status=spam --description "Mark Zenodo spam as such" --auto-accept

Another pattern:

    fatcat-cli search releases 'year:2021 title:"download album" title:"zip mp3" author:download doi_prefix:10.5281 !journal:* in_ia:false !release_type:stub' --count
    # 14376

    fatcat-cli search releases 'year:2021 title:"download album" title:"zip mp3" author:download doi_prefix:10.5281 !journal:* in_ia:false !release_type:stub' --entity-json --limit 30000 \
        | jq 'select(.release_type != "stub")' -c \
        | pv -l \
        | fatcat-cli batch update release release_type=stub withdrawn_status=spam --description "Mark Zenodo spam as such" --auto-accept

Did some manual patterns.

Another pattern; checking manually all looks like spam:

    fatcat-cli search releases 'title:"live stream free" doi_prefix:10.5281 !type:stub' --count
    # 176

    fatcat-cli search releases 'title:"live stream free" doi_prefix:10.5281 !type:stub' --entity-json --limit 200 \
        | jq 'select(.release_type != "stub")' -c \
        | pv -l \
        | fatcat-cli batch update release release_type=stub withdrawn_status=spam --description "Mark Zenodo spam as such" --auto-accept
    # done

Another large pattern:

    fatcat-cli search releases 'year:2021 title:"Full Album Download" title:mp3 author:download doi_prefix:10.5281 !journal:* in_ia:false !release_type:stub' --count
    # 14800

    fatcat-cli search releases 'year:2021 title:"Full Album Download" title:mp3 author:download doi_prefix:10.5281 !journal:* in_ia:false !release_type:stub' --entity-json --limit 15000 \
        | jq 'select(.release_type != "stub")' -c \
        | pv -l \
        | fatcat-cli batch update release release_type=stub withdrawn_status=spam --description "Mark Zenodo spam as such" --auto-accept

Small pattern:

    fatcat-cli search releases 'gomovies !release_type:stub' --entity-json --limit 50 \
        | jq 'select(.release_type != "stub")' -c \
        | pv -l \
        | fatcat-cli batch update release release_type=stub withdrawn_status=spam --description "Mark Zenodo spam as such"
    # editgroup_fzumcytmljfebldd5az643wqmi

