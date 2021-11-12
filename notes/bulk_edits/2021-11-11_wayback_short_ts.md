
## Production Run

At git commit `6ad9d24e4d7d901d6fc394e6e91575f6acba7ff4`.

Start small:

    export FATCAT_AUTH_WORKER_CLEANUP=[...]

    zcat /srv/fatcat/datasets/files_20211105_moreshortts.fetched.json.gz \
        | head -n100 \
        | python -m fatcat_tools.cleanups.file_short_wayback_ts -
    # Counter({'total': 100, 'update': 99, 'skip-bad-wayback-timestamp': 1, 'skip': 0, 'insert': 0, 'exists': 0})

Looks good! Run the full batch.

    zcat /srv/fatcat/datasets/files_20211105_moreshortts.fetched.json.gz \
        | pv -l \
        | parallel -j8 --linebuffer --round-robin --pipe python -m fatcat_tools.cleanups.file_short_wayback_ts -

    [...]
    bad replacement URL: partial_ts=2017 original=https://www.hydrol-earth-syst-sci.net/21/4959/2017/hess-21-4959-2017.pdf fix_url=https://web.archive.org/web/20180721004954/https://www.hydrol-earth-syst-sci.net/21/4959/2017/hess-21-4959-2017.pdf
    bad replacement URL: partial_ts=2017 original=https://www.the-cryosphere.net/11/1537/2017/tc-11-1537-2017.pdf fix_url=https://web.archive.org/web/20180719235703/https://www.the-cryosphere.net/11/1537/2017/tc-11-1537-2017.pdf
    bad replacement URL: partial_ts=2017 original=http://www.growingscience.com/msl/Vol7/msl_2017_26.pdf fix_url=https://web.archive.org/web/20180601235059/http://www.growingscience.com/msl/Vol7/msl_2017_26.pdf
    bad replacement URL: partial_ts=2017 original=https://www.hydrol-earth-syst-sci.net/21/4115/2017/hess-21-4115-2017.pdf fix_url=https://web.archive.org/web/20180719162956/https://www.hydrol-earth-syst-sci.net/21/4115/2017/hess-21-4115-2017.pdf
    bad replacement URL: partial_ts=2017 original=https://www.biogeosciences.net/14/4279/2017/bg-14-4279-2017.pdf fix_url=https://web.archive.org/web/20180720220056/https://www.biogeosciences.net/14/4279/2017/bg-14-4279-2017.pdf
    bad replacement URL: partial_ts=2017 original=https://www.biogeosciences.net/14/3669/2017/bg-14-3669-2017.pdf fix_url=https://web.archive.org/web/20180720222828/https://www.biogeosciences.net/14/3669/2017/bg-14-3669-2017.pdf
    [...]
    bad replacement URL: partial_ts=2017 original=http://www.growingscience.com/msl/Vol7/msl_2017_28.pdf fix_url=https://web.archive.org/web/20180602071632/http://www.growingscience.com/msl/Vol7/msl_2017_28.pdf
    bad replacement URL: partial_ts=2017 original=https://www.biogeosciences.net/14/4161/2017/bg-14-4161-2017.pdf fix_url=https://web.archive.org/web/20180720004438/https://www.biogeosciences.net/14/4161/2017/bg-14-4161-2017.pdf
    bad replacement URL: partial_ts=2017 original=https://core.ac.uk/download/pdf/10915563.pdf fix_url=https://web.archive.org/web/20190220174144/https://core.ac.uk/download/pdf/10915563.pdf
    bad replacement URL: partial_ts=2017 original=http://www.growingscience.com/ijiec/Vol9/IJIEC_2017_24.pdf fix_url=https://web.archive.org/web/20180602094300/http://www.growingscience.com/ijiec/Vol9/IJIEC_2017_24.pdf
    bad replacement URL: partial_ts=2017 original=https://core.ac.uk/download/pdf/36046645.pdf fix_url=https://web.archive.org/web/20190220175351/https://core.ac.uk/download/pdf/36046645.pdf
    bad replacement URL: partial_ts=2017 original=https://core.ac.uk/download/pdf/35085886.pdf fix_url=https://web.archive.org/web/20190220175410/https://core.ac.uk/download/pdf/35085886.pdf
    bad replacement URL: partial_ts=2017 original=https://www.atmos-chem-phys.net/17/10349/2017/acp-17-10349-2017.pdf fix_url=https://web.archive.org/web/20181102190649/https://www.atmos-chem-phys.net/17/10349/2017/acp-17-10349-2017.pdf
    bad replacement URL: partial_ts=2017 original=https://www.atmos-chem-phys.net/17/7775/2017/acp-17-7775-2017.pdf fix_url=https://web.archive.org/web/20181101041355/https://www.atmos-chem-phys.net/17/7775/2017/acp-17-7775-2017.pdf
    bad replacement URL: partial_ts=2017 original=http://www.veterinaryworld.org/Vol.10/March-2017/5.pdf fix_url=https://web.archive.org/web/20180721074940/http://www.veterinaryworld.org/Vol.10/March-2017/5.pdf
    bad replacement URL: partial_ts=2017 original=https://www.ann-geophys.net/35/189/2017/angeo-35-189-2017.pdf fix_url=https://web.archive.org/web/20180625214916/https://www.ann-geophys.net/35/189/2017/angeo-35-189-2017.pdf
    [...]

    # 9.96M 12:57:06 [ 213 /s]

    Counter({'total': 1272301, 'update': 1268466, 'skip-bad-wayback-timestamp': 2808, 'skip': 1026, 'skip-status': 981, 'skip-bad-replacement': 45, 'skip-bad-wayback': 1, 'insert': 0, 'exists': 0})
    Counter({'total': 1242814, 'update': 1239042, 'skip-bad-wayback-timestamp': 2734, 'skip': 1036, 'skip-status': 974, 'skip-bad-replacement': 62, 'skip-bad-wayback': 2, 'insert': 0, 'exists': 0})
    Counter({'total': 1264351, 'update': 1260695, 'skip-bad-wayback-timestamp': 2626, 'skip': 1030, 'skip-status': 977, 'skip-bad-replacement': 53, 'insert': 0, 'exists': 0})
    Counter({'total': 1244480, 'update': 1240779, 'skip-bad-wayback-timestamp': 2680, 'skip': 1020, 'skip-status': 962, 'skip-bad-replacement': 58, 'skip-bad-wayback': 1, 'insert': 0, 'exists': 0})
    Counter({'total': 1222678, 'update': 1219022, 'skip-bad-wayback-timestamp': 2698, 'skip': 956, 'skip-status': 892, 'skip-bad-replacement': 64, 'skip-bad-wayback': 2, 'insert': 0, 'exists': 0})
    Counter({'total': 1225078, 'update': 1221459, 'skip-bad-wayback-timestamp': 2597, 'skip': 1020, 'skip-status': 964, 'skip-bad-replacement': 56, 'skip-bad-wayback': 2, 'insert': 0, 'exists': 0})
    Counter({'total': 1283843, 'update': 1280014, 'skip-bad-wayback-timestamp': 2670, 'skip': 1059, 'skip-status': 997, 'skip-revision-changed': 99, 'skip-bad-replacement': 62, 'skip-bad-wayback': 1, 'insert': 0, 'exists': 0})
    Counter({'total': 1203309, 'update': 1199782, 'skip-bad-wayback-timestamp': 2556, 'skip': 971, 'skip-status': 923, 'skip-bad-replacement': 48, 'insert': 0, 'exists': 0})

On the order of 99.7% were updated/fixed, over 9.5 million file entities, taking almost 13 hours.
