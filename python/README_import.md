
## ORCID

Does not work:

    ./client.py import-orcid /data/orcid/partial/public_profiles_API-2.0_2017_10_json/3/0000-0001-5115-8623.json

Instead:

    cat /data/orcid/partial/public_profiles_API-2.0_2017_10_json/3/0000-0001-5115-8623.json | jq -c . | ./client.py import-orcid -

Or for many files:

    find /data/orcid/partial/public_profiles_API-2.0_2017_10_json/3 -iname '*.json' | parallel --bar jq -c . {} | rg '"person":' | ./client.py import-orcid -


for ~9k files:

    (python-B2RYrks8) bnewbold@orithena$ time parallel --pipepart -j4 -a /data/orcid/partial/public_profiles_API-2.0_2017_10_json/all.json ./client.py import-orcid -
    real    0m15.294s
    user    0m28.112s
    sys     0m2.408s

    => 636/second

    (python-B2RYrks8) bnewbold@orithena$ time ./client.py import-orcid /data/orcid/partial/public_profiles_API-2.0_2017_10_json/all.json
    real    0m47.268s
    user    0m2.616s
    sys     0m0.104s

    => 203/second
