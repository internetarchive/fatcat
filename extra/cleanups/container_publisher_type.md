
A bunch of MDPI journals are incorrectly listed as 'longtail'.

    fatcat-cli search container 'publisher:mdpi publisher_type:* !publisher_type:oa' --count
    # 245

Because this is 'extra' metadata, need a little python script to change the
metadata (fatcat-cli doesn't have this feature yet):

    import sys
    import json

    publisher_type = sys.argv[1].strip().lower()
    #print(publisher_type, file=sys.stderr)

    for line in sys.stdin:
        if not line.strip():
            continue
        container = json.loads(line) 
        container["extra"]["publisher_type"] = publisher_type
        print(json.dumps(container))

Run some cleanups:

    export FATCAT_AUTH_WORKER_CLEANUP=[...]
    export FATCAT_API_AUTH_TOKEN=$FATCAT_AUTH_WORKER_CLEANUP

    fatcat-cli search container 'publisher:mdpi publisher_type:* !publisher_type:oa' --entity-json --limit 50 \
        | jq 'select(.publisher_type != "oa")' -c \
        | python3 ./container_publisher_type.py oa \
        | fatcat-cli batch update container --description "Update container publisher_type"
    # editgroup_oum6mnkl2rbn3jaua4a2gdlj5q

Looks good, run the rest:

    fatcat-cli search container 'publisher:mdpi publisher_type:* !publisher_type:oa' --entity-json --limit 300 \
        | jq 'select(.publisher_type != "oa")' -c \
        | python3 ./container_publisher_type.py oa \
        | fatcat-cli batch update container --description "Update container publisher_type" --auto-accept

Some more cleanup patterns:

    fatcat-cli search container 'publisher:"Frontiers Media SA" publisher_type:* !publisher_type:oa' --count
    # 84

    fatcat-cli search container 'publisher:"Frontiers Media SA" publisher_type:* !publisher_type:oa' --entity-json --limit 300 \
        | jq 'select(.publisher_type != "oa")' -c \
        | python3 ./container_publisher_type.py oa \
        | fatcat-cli batch update container --description "Update container publisher_type" --auto-accept

    fatcat-cli search container 'publisher:"Walter de Gruyter" publisher_type:* !publisher_type:commercial !publisher_type:archive' --count
    # 47

    fatcat-cli search container 'publisher:"Walter de Gruyter" publisher_type:* !publisher_type:commercial !publisher_type:archive' --entity-json --limit 300 \
        | jq 'select(.publisher_type != "commercial")' -c \
        | python3 ./container_publisher_type.py commercial \
        | fatcat-cli batch update container --description "Update container publisher_type" --auto-accept

    fatcat-cli search container 'publisher:"springer" publisher_type:* !publisher_type:big5 !publisher_type:archive' --count
    # 56

    fatcat-cli search container 'publisher:"springer" publisher_type:* !publisher_type:big5 !publisher_type:archive' --entity-json --limit 300 \
        | jq 'select(.publisher_type != "big5")' -c \
        | python3 ./container_publisher_type.py big5 \
        | fatcat-cli batch update container --description "Update container publisher_type" --auto-accept

    fatcat-cli search container 'publisher:"elsevier" publisher_type:* !publisher_type:big5 !publisher_type:archive' --count
    # 98

    fatcat-cli search container 'publisher:"elsevier" publisher_type:* !publisher_type:big5 !publisher_type:archive' --entity-json --limit 300 \
        | jq 'select(.publisher_type != "big5")' -c \
        | python3 ./container_publisher_type.py big5 \
        | fatcat-cli batch update container --description "Update container publisher_type" --auto-accept

    fatcat-cli search container 'publisher:"wiley" publisher_type:* !publisher_type:big5 !publisher_type:archive' --count
    # 37

    fatcat-cli search container 'publisher:"wiley" publisher_type:* !publisher_type:big5 !publisher_type:archive' --entity-json --limit 300 \
        | jq 'select(.publisher_type != "big5")' -c \
        | python3 ./container_publisher_type.py big5 \
        | fatcat-cli batch update container --description "Update container publisher_type" --auto-accept

    fatcat-cli search container 'publisher:taylor publisher:francis publisher_type:* !publisher_type:big5 !publisher_type:archive' --count
    # 558

    fatcat-cli search container 'publisher:taylor publisher:francis publisher_type:* !publisher_type:big5 !publisher_type:archive' --entity-json --limit 300 \
        | jq 'select(.publisher_type != "big5")' -c \
        | python3 ./container_publisher_type.py big5 \
        | fatcat-cli batch update container --description "Update container publisher_type" --auto-accept

    fatcat-cli search container 'publisher:sage publisher_type:* !publisher_type:big5 !publisher_type:archive' --count
    # 28

    fatcat-cli search container 'publisher:sage publisher_type:* !publisher_type:big5 !publisher_type:archive' --entity-json --limit 300 \
        | jq 'select(.publisher_type != "big5")' -c \
        | python3 ./container_publisher_type.py big5 \
        | fatcat-cli batch update container --description "Update container publisher_type" --auto-accept

Overall, around a thousand containers updated. Changes to releases will not be
reflected until they are re-indexed.
