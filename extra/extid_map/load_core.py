#!/usr/bin/env python3
"""
Run like:

    zcat CORE_FILE | ./load_core.py release_ids.db
"""

import sys
import csv
import sqlite3

def run(db_path):
    #db = sqlite3.connect("file:{}?mode=ro".format(db_path)
    db = sqlite3.connect(db_path)
    c = db.cursor()
    count = 0
    inserted = 0
    for row in sys.stdin:
        row = row.strip().split("\t")
        if len(row) != 2:
            continue
        (doi, coreid) = row[:2]
        if count % 1000 == 0:
            print(f"read {count}, wrote {inserted}")
            db.commit()
        count = count + 1
        coreid = int(coreid)
        doi = doi.lower()
        # UPSERTS were only added to sqlite3 in summer 2018 (not in xenial version)
        try:
            c.execute("""INSERT INTO ids (doi, core) VALUES (?, ?)""", (doi, coreid))
        except sqlite3.IntegrityError:
            c.execute("""UPDATE ids SET core = ? WHERE doi = ?""", (coreid, doi))
        inserted = inserted + 1
    db.commit()
    db.close()

if __name__=="__main__":
    if len(sys.argv) != 2:
        print("Need single argument: db_path")
        sys.exit(-1)
    run(sys.argv[1])
