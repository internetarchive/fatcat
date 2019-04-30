
First create ident files, following `../extra/sql_dumps/README.md`.

Then, to dump locally to stdout:

    cat /tmp/fatcat_ident_releases.tsv | ./target/debug/fatcat-export releases

Or, in production, as the fatcat user:

    cat /tmp/fatcat_ident_releases.tsv | ./target/release/fatcat-export release --expand files,filesets,webcaptures,container -j8 | pigz > /srv/fatcat/snapshots/release_export_expanded.json.gz
    cat /tmp/fatcat_ident_releases.tsv | ./target/release/fatcat-export release -j8 | pigz > /srv/fatcat/snapshots/release_export.json.gz
    cat /tmp/fatcat_ident_creators.tsv | ./target/release/fatcat-export creator -j8 | pigz > /srv/fatcat/snapshots/creator_export.json.gz
    cat /tmp/fatcat_ident_containers.tsv | ./target/release/fatcat-export container -j8 | pigz > /srv/fatcat/snapshots/container_export.json.gz
    cat /tmp/fatcat_ident_files.tsv | ./target/release/fatcat-export file -j8 | pigz > /srv/fatcat/snapshots/file_export.json.gz
    cat /tmp/fatcat_ident_filesets.tsv | ./target/release/fatcat-export fileset -j8 | pigz > /srv/fatcat/snapshots/fileset_export.json.gz
    cat /tmp/fatcat_ident_webcaptures.tsv | ./target/release/fatcat-export webcapture -j8 | pigz > /srv/fatcat/snapshots/webcapture_export.json.gz

