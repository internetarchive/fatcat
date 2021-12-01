
See `container_issnl_dedupe` cleanup notes for background and dataset prep.

This bulk edit updated over 2,000 container entities (redirects), and following
a couple additional manual redirects, there are no duplicate containers in the
index.

## Prod Run

    git log | head -n1
    # commit 929baa24c6eec96e303286f258f2be0949303266

    export FATCAT_AUTH_API_TOKEN=[...]

Start small, with the random sample, no releases or human edits allowed:

    cat /srv/fatcat/datasets/container_issnl_dupes.sample.json \
        | python -m fatcat_tools.mergers.containers --editgroup-description-override "Automated merging of duplicate container entities with the same ISSN-L" merge-containers -
    # Counter({'updated-entities': 307, 'skip-container-release-count': 251, 'lines': 100, 'merged': 99, 'skip': 1, 'skip-not-active-entity': 1, 'updated-total': 0})

Continue with the full batch with the 100 cut-off (original plan):

    cat /srv/fatcat/datasets/container_issnl_dupes.json \
        | python -m fatcat_tools.mergers.containers --editgroup-description-override "Automated merging of duplicate container entities with the same ISSN-L" merge-containers - --max-container-releases 100
    # Counter({'updated-entities': 2025, 'lines': 835, 'merged': 732, 'skip-container-release-count': 112, 'skip': 103, 'skip-not-active-entity': 101, 'skip-human-edited': 2, 'updated-total': 0})

Full output log:

    skipping container_v2zsod52jnhfbb5tk3bujgvhn4: release count 189
    skipping container_sk4nevyiwfgk3hwdy6h3rgk5ou: release count 212
    skipping container_3sdew7wvcvdi3ancgeemu2qhtu: release count 175
    skipping container_uffot534onejpohj3l4m36pyu4: release count 280
    skipping container_zd7uysnu5nejxjmvmieehodd3a: release count 150
    skipping container_lbskm3w74nhkvobfy3kefaqcxq: release count 388
    skipping container_yf76l2b5d5hu3hlr45e5qm3tai: release count 271
    skipping container_ykoitf76dbe5lbhkgozjay3krq: release count 405
    skipping container_z2xwk5y4oredlg7oaxcsd266ie: release count 203
    skipping container_2cxnpvvjkvfdxcz2nb3qkjucei: release count 464
    skipping container_5yq3233lrfeclbiqa54ed5il6y: release count 138
    skipping container_asoaslvd5nghlabgerd3rt7ffq: release count 398
    skipping container_nvvtdqsmdvfilfuwsg44dl4gdy: release count 115
    skipping container_gxkfhz3ywzglngcj46m2mfqeri: release count 333
    skipping container_twvb6baapfaotnhhe6xynd7n6q: release count 249
    skipping container_gufxbxnddfdpjag36erclh476i: release count 127
    skipping container_qthwmftdgjc2fgrzx6w7py2mqe: release count 138
    skipping container_dbpyvcd245ednlpc5f4eo33u4u: release count 1097
    skipping container_c3h2ajl72fhf7bwzgsntpipd6u: release count 264
    skipping container_vin6xe6ojndyzfl23rkzbldxym: release count 218
    skipping container_eef7zpisarfcbdmgp3pyixchom: release count 175
    skipping container_ls56ijz6dre7dfu5sbg32pi7pq: release count 456
    skipping container_rftmo4qx6zfpxbuaellrlypqza: release count 326
    skipping container_uhuemivuxffbdhdqae75bru7ie: release count 235
    skipping container_4tyenz4zibemzpmodpefemlbia: release count 117
    skipping container_zznalfkafzdo5p32z3qe5hflkm: release count 193
    skipping container_3kio7nj2enbodc5mcfefalbbti: release count 181
    skipping container_gxk755hay5bwvbllydcetcupqq: release count 119
    skipping container_eixvqbgdprej3bm4bqouiqzp3e: release count 247
    skipping container_rmmzgzkswjhitkwrv6zxgmji5i: release count 157
    skipping container_pkq6njkgvrekham3qipbjd6czu: release count 140
    skipping container_hkpd4a467bepho76ifji6jg4ue: release count 258
    skipping container_dfxmy3gx5fe4ff6nr2tiemiyyi: release count 101
    skipping container_eqqja4xhfzatrou3dkpjryrmaq: release count 114
    skipping container_sku6jjds5ngwrhqwlxfq2jvgzu: release count 1135
    skipping container_qyh3ncxjqzfmlajauwcsqqlb5q: release count 265
    skipping container_syh4k3q3yjf7hcva3ficd2g4dy: release count 262
    skipping container_ptmjhxoyfffjzbwkbe65fle4qm: release count 201
    skipping container_3tzyaivyerexflyiy463i5q4x4: release count 466
    skipping container_bbwsceba7rh2fgiyhbzspyykxe: release count 110
    skipping container_e7yk3xwws5dnvk7osdoxpulv6m: release count 200
    skipping container_hmyaeocgsze35p5vo5u6q4o7jm: release count 119
    skipping container_ireoyurf5fcgfeem22ng7liel4: release count 313
    skipping container_dkvv2nptnza3pmbiojxo2syom4: release count 274
    skipping container_hyzwgzfumvaj7pqkrh65e5ev6q: release count 261
    skipping container_kyfdgd22k5a53nm2jfjceklbly: release count 125
    skipping container_avfetikkhnb53asxdkowshucce: release count 239
    skipping container_7wrb5pc3tzcjfho3t74ab7ijjq: release count 164
    skipping container_27wax7sd4vbkrfl3mbe227ui5u: release count 231
    skipping container_4mwjbsjlq5c7la7dvmqbegtslq: release count 122
    skipping container_udtqmaqkcvhbnjo3vhfpbubaie: release count 106
    skipping container_y7hcehu5izdbpi65f4pxm6cqla: release count 170
    skipping container_ddgokru3vvh3nglqkfzx5orlfe: release count 247
    skipping container_qhtesw5bb5fhvpw7gymydznhne: release count 215
    skipping container_62jks7u4vjbrpl4sycma7yzhru: release count 314
    skipping container_knhzl5oq6resrghowrxocj52mq: release count 123
    skipping container_yfgztqw72vhabfohcrr77v6b3a: release count 167
    skipping container_i7v5yf24erezljqlu5dsvctulm: release count 174
    skipping container_75jngrgwnbb4tn22c5qk3fxo4i: release count 198
    skipping container_3zgq2ml4jjc5jh5w4mrx44qda4: release count 162
    skipping container_xbdlayhr2zcajfsmgj7niiylqm: release count 120
    skipping container_cpjsvpcsgrfy5ms5czaj4fcgbm: release count 228
    skipping container_o44k5mmoxzdtnozysjun66slju: release count 179
    skipping container_qph344zobjev7kmke2ozk33mwi: release count 241
    skipping container_xkii4g5qnrdnleyhhwvd4uoz2i: release count 108
    skipping container_vrdwdxnidbdiracilr3gr5v3cm: release count 204
    skipping container_4gw5lyn3xna2jajzbxot6fifdq: release count 101
    skipping container_t7sx4zbwyzhbjbzsvrazlwtbvy: release count 199
    skipping container_xxnbdrcamrgfbhydxv6v3jy2fy: release count 101
    skipping container_usm5yyjmcfgmjcu4lcy3f6sade: release count 115
    skipping container_fqmqy4ws5fcf7axos7esy2fcce: release count 191
    skipping container_wadmkiqbt5hhjfpm6lbn47agyu: release count 114
    skipping container_ujc3wxewfraizaffzouy2mxaau: release count 136
    skipping container_um6qxikn3nhlpoiovdrww5gp4q: release count 144
    skipping container_bykun5gbb5cgnfo25j5jzm5iai: release count 320
    skipping container_amyo2w2zgjafxbxvmlb5hfndoy: release count 109
    skipping container_mfbbi2zyhrhvvjh45mvd4fz5qy: release count 119
    skipping container_jftmuuvc2fa6xcu77zjsyrs5b4: release count 251
    skipping container_dqrilnh3ord7fachdd4vaktvpe: release count 143
    skipping container_oj4ahzx3mvcbrggtpa6sqnjtny: release count 160
    skipping container_p452ltmuhbdnlaz6donkkr5une: release count 166
    skipping container_dmsweozizzgqza4sd5byomyqve: release count 123
    skipping container_xckj2u5pdrevxei43pgqe4c4sq: release count 119
    skipping container_5qpk5qalazcmjooeruos7zck5i: release count 194
    skipping container_eaqcxaog3jgjhjmpf5kdxrmmcm: release count 149
    skipping container_33tn3bydy5dvtkeoe5falrto6q: release count 115
    skipping container_i4t74quqcjdgplmeyga5payqhm: release count 118
    skipping container_2olekyosjzer5ds3s7gc3zb624: release count 200
    skipping container_yh4uvtwy7fa27ibgddi3qmewg4: release count 199
    skipping container_qjidb4al4japfcbxce2dw627ni: release count 545
    skipping container_y2cy2ajbwzbo3d6lwyseqig2ce: release count 191
    skipping container_abehsncrarf4ffttto4toaayay: release count 440
    skipping container_elmhdn55tbfgvod2oic6xlwr3e: release count 159
    skipping container_ylzwxwnwpfdf3br2xgwobkqjwa: release count 239
    skipping container_g3drm2f6mjazxc7vckimcycn5y: release count 116
    skipping container_33f3tq7re5abtafbxdbzd6jpxi: release count 210
    skipping container_jtx5kwkiwbcezbqk7gwhor53wm: release count 265
    skipping container_rrikfc42xrg6vl7lnhlf26mtxm: release count 150
    skipping container_pzqxtxvco5fhlj7k23bqw374yu: release count 106
    skipping container_xde4qg5sh5djzdnawhou5dhb4u: release count 366
    skipping container_omshcvxszffgdlspsehxlyksse: release count 165
    skipping container_ekjk6clxtfgmbgvkkgixeaadfy: release count 224
    skipping container_tyindzesqbg27nblbfwa6hptku: release count 190
    skipping container_ho6rwlwmsnepri2amemptyvnhm: release count 134
    skipping container_uqezfmzrnre5zdtspj6wphlvbm: release count 189
    skipping container_6xslqgviybh7lkl2edk5ghtvum: release count 141
    skipping container_3wgusadkfjhf3ejpn2nnn3v5ka: release count 307
    skipping container_cbiclofvcbhmjcxubvkg5vyhxi: release count 1368
    skipping container_ahvaawe3fvd3zb4mfiaado7wyy: human edited
    skipping container_ejgmmh6ozvf7hczdpok2ab3fni: release count 102
    skipping container_s3h3himubfaftbhcpfzgc354zq: release count 174
    skipping container_wzabvii33zakjbf24dspb2tst4: release count 122
    skipping container_kly2opavcjejxesqru5tib455q: release count 159
    skipping container_zw7uyavzgzdtleeliwh5cnp23q: human edited

Check progress (after partial ident dump and `fatcat_export`, just for containers):

    zcat container_export.json.gz \
        | jq '[.issnl, .ident] | @tsv' -r \
        | sort -S 4G \
        | uniq -D -w 9 \
        > issnl_ident.dupes.tsv

    wc -l issnl_ident.dupes.tsv
    # 4

    cut -f1 issnl_ident.dupes.tsv | uniq | wc -l
    # 2

Huh, this doesn't seem right. What happened to all the `skip-container-release-count`, etc?

Ah, the `skip-container-release-count` condition 1) can include the primary
release and 2) is only printed, never does anything (!).

Well, yikes, but no indication anything has gone really wrong.

Let's manually look at the human-edited cases:

    # 2709-2143
    # zw7uyavzgzdtleeliwh5cnp23q is better primary
    {
      "duplicate_ids": [
        "osev5sprhfdwdkoyypihlspsme",
        "zw7uyavzgzdtleeliwh5cnp23q"
      ],
      "entity_type": "container",
      "evidence": {
        "extid": "2709-2143",
        "extid_type": "issnl"
      },
      "primary_id": null
    }

    # 2567-8833
    # ahvaawe3fvd3zb4mfiaado7wyy is better primary (no releases)
    {
      "duplicate_ids": [
        "ahvaawe3fvd3zb4mfiaado7wyy",
        "e7q6rq2ebbbp3m3xxccqxi3fnm"
      ],
      "entity_type": "container",
      "evidence": {
        "extid": "2567-8833",
        "extid_type": "issnl"
      },
      "primary_id": null
    }

In retrospect, I probably should have also enforced title fuzzy match in
automated merging, instead of just ISSN-L match.
