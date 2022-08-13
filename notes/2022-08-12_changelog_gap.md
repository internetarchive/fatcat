
On 2022-08-11, realized that we had a "gap" in the changelog: after a VM
reboot, the postgresql primary key sequence for the 'changelog' table had been
incremented, but rows were not inserted (transaction hadn't finished).

This was a known potential problem (naively relying on the sequence to
increment with no gaps).

As a work-around, implemented a simple "gap filler" which will create
empty/dummy editgroups and changelog entries.

This gap extends from 6153703 to 6153721, so just a couple dozen entries. The
fixup command was:

    ./target/release/fatcat-doctor backfill-changelog-gap 6153702 6153721
    Inserted changelog: 6153703
    Inserted changelog: 6153704
    Inserted changelog: 6153705
    Inserted changelog: 6153706
    Inserted changelog: 6153707
    Inserted changelog: 6153708
    Inserted changelog: 6153709
    Inserted changelog: 6153710
    Inserted changelog: 6153711
    Inserted changelog: 6153712
    Inserted changelog: 6153713
    Inserted changelog: 6153714
    Inserted changelog: 6153715
    Inserted changelog: 6153716
    Inserted changelog: 6153717
    Inserted changelog: 6153718
    Inserted changelog: 6153719
    Inserted changelog: 6153720
    Inserted changelog: 6153721

After that the changelog worker was happy:

    Aug 13 02:41:59 wbgrp-svc502.us.archive.org fatcat-worker[386037]: Most recent changelog index in Kafka seems to be 6153702
    Aug 13 02:41:59 wbgrp-svc502.us.archive.org fatcat-worker[386037]: Fetching changelogs from 6153703 through 6158547
    Aug 13 02:43:12 wbgrp-svc502.us.archive.org fatcat-worker[386037]: Sleeping 5.0 seconds...

