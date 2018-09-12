
First create ident files:

    psql fatcat < ../extra/quick_dump.sql

Then dump:

    cat /tmp/fatcat_ident_releases.tsv | ./target/debug/fatcat-export releases

Or, perhaps, in production:

    cat /tmp/fatcat_ident_releases.tsv | ./target/release/fatcat-export release --expand files,contaner -j8 | pv | gzip > all_releases.json.gz

