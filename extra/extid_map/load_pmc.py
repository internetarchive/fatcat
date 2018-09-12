#!/usr/bin/env python3
"""
Run like:

    zcat PMID_PMCID_DOI.csv.gz | ./load_pmc.py release_ids.db
"""

import sys
import csv
import sqlite3

def run(db_path):
    #db = sqlite3.connect("file:{}?mode=ro".format(db_path)
    db = sqlite3.connect(db_path)
    reader = csv.reader(sys.stdin)
    c = db.cursor()
    count = 0
    inserted = 0
    for row in reader:
        (pmid, pmcid, doi) = row
        if count % 1000 == 0:
            print("read {}, wrote {}".format(count, inserted))
            db.commit()
        count = count + 1
        if not doi.startswith("http"):
            continue
        doi = doi.replace("https://doi.org/", "").lower()
        if pmcid == '':
            pmcid = None
        if pmid == '':
            pmid = None
        else:
            pmid = int(pmid)
        # UPSERTS were only added to sqlite3 in summer 2018 (not in xenial version)
        try:
            c.execute("""INSERT INTO ids (doi, pmid, pmcid) VALUES (?, ?, ?)""", (doi, pmid, pmcid))
        except sqlite3.IntegrityError:
            c.execute("""UPDATE ids SET pmid = ?, pmcid = ? WHERE doi = ?""", (pmid, pmcid, doi))
        inserted = inserted + 1
    db.commit()
    db.close()

if __name__=="__main__":
    if len(sys.argv) != 2:
        print("Need single argument: db_path")
        sys.exit(-1)
    run(sys.argv[1])
