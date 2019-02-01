
First create ident files, following `../extra/sql_dumps/README.md`.

Then, to dump locally to stdout:

    cat /tmp/fatcat_ident_releases.tsv | ./target/debug/fatcat-export releases

Or, in production:

    cat /tmp/fatcat_ident_releases.tsv | ./target/release/fatcat-export release --expand files,filesets,webcaptures,container -j8 | gzip > /srv/fatcat/snapshots/release_export_expanded.json.gz
    cat /tmp/fatcat_ident_releases.tsv | ./target/release/fatcat-export release -j8 | gzip > /srv/fatcat/snapshots/release_export.json.gz
    cat /tmp/fatcat_ident_containers.tsv | ./target/release/fatcat-export container -j8 | gzip > /srv/fatcat/snapshots/container_export.json.gz
    cat /tmp/fatcat_ident_files.tsv | ./target/release/fatcat-export file -j8 | gzip > /srv/fatcat/snapshots/file_export.json.gz

