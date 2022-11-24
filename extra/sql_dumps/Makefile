
SHELL=/bin/bash -euo pipefail
TODAY ?= $(shell date --iso --utc)
DATADIR ?= /srv/fatcat/snapshots/$(TODAY)
DATESLUG ?= $(shell date +%Y-%m-%d.%H%M%S)
DATABASE_URL ?= fatcat_prod

.PHONY: help
help: ## Print info about all commands
	@echo "Commands:"
	@echo
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "    \033[01;32m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: create_datadir
create_datadir:
	mkdir -p $(DATADIR)/
	sudo chmod a+rw $(DATADIR)/

$(DATADIR)/.IDENTS:
	sudo -u postgres DATABASE_URL=$(DATABASE_URL) ./ident_table_snapshot.sh $(DATADIR)
	sudo -u postgres mv /srv/fatcat/tmp/fatcat_ident_*.tsv $(DATADIR)
	touch $@

$(DATADIR)/release_export_expanded.json.gz: $(DATADIR)/.IDENTS
	cd ../../rust; cat $(DATADIR)/fatcat_ident_releases_by_work.tsv | sudo -u fatcat ./target/release/fatcat-export releasebywork --expand files,filesets,webcaptures,container -j8 | pigz > $@.wip
	mv  $@.wip  $@

$(DATADIR)/creator_export.json.gz: $(DATADIR)/.IDENTS
	cd ../../rust; cat $(DATADIR)/fatcat_ident_creators.tsv | sudo -u fatcat ./target/release/fatcat-export creator -j8 | pigz > $@.wip
	mv  $@.wip  $@

$(DATADIR)/container_export.json.gz: $(DATADIR)/.IDENTS
	cd ../../rust; cat $(DATADIR)/fatcat_ident_containers.tsv | sudo -u fatcat ./target/release/fatcat-export container -j8 | pigz > $@.wip
	mv  $@.wip  $@

$(DATADIR)/file_export.json.gz: $(DATADIR)/.IDENTS
	cd ../../rust; cat $(DATADIR)/fatcat_ident_files.tsv | sudo -u fatcat ./target/release/fatcat-export file -j8 | pigz > $@.wip
	mv  $@.wip  $@

$(DATADIR)/fileset_export.json.gz: $(DATADIR)/.IDENTS
	cd ../../rust; cat $(DATADIR)/fatcat_ident_filesets.tsv | sudo -u fatcat ./target/release/fatcat-export fileset -j8 | pigz > $@.wip
	mv  $@.wip  $@

$(DATADIR)/webcapture_export.json.gz: $(DATADIR)/.IDENTS
	cd ../../rust; cat $(DATADIR)/fatcat_ident_webcaptures.tsv | sudo -u fatcat ./target/release/fatcat-export webcapture -j8 | pigz > $@.wip
	mv  $@.wip  $@

$(DATADIR)/abstracts.json.gz:
	sudo -u postgres psql $(DATABASE_URL) < dump_abstracts.sql | egrep -v ^BEGIN$ | egrep -v ^ROLLBACK$ | pv -l | pigz > $@.wip
	mv  $@.wip  $@

$(DATADIR)/file_hashes.tsv.gz:
	sudo -u postgres psql $(DATABASE_URL) < dump_file_hashes.sql | egrep -v ^BEGIN$ | egrep -v ^ROLLBACK$ | pv -l | pigz > $@.wip
	mv  $@.wip  $@

$(DATADIR)/release_extid.tsv.gz:
	sudo -u postgres psql $(DATABASE_URL) < dump_release_extid.sql | egrep -v ^BEGIN$ | egrep -v ^ROLLBACK$ | pv -l | pigz > $@.wip
	mv  $@.wip  $@

$(DATADIR)/.METADATA_EXPORT: $(DATADIR)/.IDENTS $(DATADIR)/release_export_expanded.json.gz $(DATADIR)/creator_export.json.gz $(DATADIR)/container_export.json.gz $(DATADIR)/file_export.json.gz $(DATADIR)/fileset_export.json.gz $(DATADIR)/webcapture_export.json.gz $(DATADIR)/abstracts.json.gz $(DATADIR)/file_hashes.tsv.gz $(DATADIR)/release_extid.tsv.gz ## Dump bulk metadata to disk
	touch $@

.PHONY: metadata-exports
metadata-exports: create_datadir $(DATADIR)/.METADATA_EXPORT  ## Dump bulk metadata to disk
	@echo


$(DATADIR)/.METADATA_UPLOADED: $(DATADIR)/.METADATA_EXPORT
	ia upload --checksum fatcat_bulk_exports_$(TODAY) ia_exports_item_readme.md --remote-name=README.md -m collection:fatcat_snapshots_and_exports -m mediatype:data -m creator:"Internet Archive Web Group" -m date:$(TODAY) -m title:"Fatcat Bulk Metadata Exports ($(TODAY))"
	ia upload fatcat_bulk_exports_$(TODAY) $(DATADIR)/*_export.json.gz $(DATADIR)/*_export_expanded.json.gz $(DATADIR)/abstracts.json.gz
	touch $@

.PHONY: upload-metadata-exports
upload-metadata-exports: create_datadir $(DATADIR)/.METADATA_UPLOADED ## Upload bulk metadata exports to archive.org
	@echo

$(DATADIR)/.PUBLIC_DB_DUMP:
	sudo -u postgres pg_dump --verbose --format=custom --exclude-table-data=auth_oidc fatcat_prod > $(DATADIR)/fatcat_public_dbdump_${DATESLUG}.pgdump.wip
	mv $(DATADIR)/fatcat_public_dbdump_${DATESLUG}.pgdump.wip $(DATADIR)/fatcat_public_dbdump_${DATESLUG}.pgdump
	touch $@

.PHONY: public-database-snapshot
public-database-snapshot: create_datadir $(DATADIR)/.PUBLIC_DB_DUMP ## Create SQL database snapshot which can be shared publicly
	@echo

$(DATADIR)/.PUBLIC_DB_UPLOADED: $(DATADIR)/.PUBLIC_DB_DUMP
	ia upload --checksum fatcat_sqldump_public_$(TODAY) ia_sqldump_item_readme.md --remote-name=README.md -m collection:fatcat_snapshots_and_exports -m mediatype:data -m creator:"Internet Archive Web Group" -m date:$(TODAY) -m title:"Fatcat Public Database Snapshot ($(TODAY))"
	ia upload --checksum fatcat_sqldump_public_$(TODAY) $(DATADIR)/fatcat_public_dbdump_*.pgdump
	touch $@

.PHONY: upload-public-database-snapshot
upload-public-database-snapshot: create_datadir public-database-snapshot $(DATADIR)/.PUBLIC_DB_UPLOADED ## Upload metadata snapshot to archive.org
	@echo

$(DATADIR)/.PRIVATE_DB_DUMP:
	sudo -u postgres pg_dump --verbose --format=custom fatcat_prod > $(DATADIR)/fatcat_private_dbdump_${DATESLUG}.pgdump.wip
	mv $(DATADIR)/fatcat_private_dbdump_${DATESLUG}.pgdump.wip $(DATADIR)/fatcat_private_dbdump_${DATESLUG}.pgdump
	touch $@

.PHONY: private-database-snapshot
private-database-snapshot: create_datadir $(DATADIR)/.PRIVATE_DB_DUMP ## Create SQL database snapshot which can be shared privately
	@echo

$(DATADIR)/.PRIVATE_DB_UPLOADED: $(DATADIR)/.PRIVATE_DB_DUMP
	ia upload --checksum fatcat_sqldump_private_$(TODAY) ia_sqldump_item_readme.md --remote-name=README.md -m collection:webgroup-internal-backups -m mediatype:data -m creator:"Internet Archive Web Group" -m date:$(TODAY) -m title:"Fatcat Private Database Snapshot ($(TODAY))"
	ia upload --checksum fatcat_sqldump_private_$(TODAY) $(DATADIR)/fatcat_private_dbdump_*.pgdump
	touch $@

.PHONY: upload-private-database-snapshot
upload-private-database-snapshot: create_datadir private-database-snapshot $(DATADIR)/.PRIVATE_DB_UPLOADED ## Upload metadata snapshot to archive.org
	@echo
