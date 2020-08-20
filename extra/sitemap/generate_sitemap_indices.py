#!/usr/bin/env python3

import sys
import glob
import datetime

def index_entity(entity_type, output):

    now = datetime.date.today().isoformat()
    print("""<?xml version="1.0" encoding="UTF-8"?>""", file=output)
    print("""<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">""", file=output)

    for filename in glob.glob(f"sitemap-{entity_type}-*.txt.gz"):
        print("  <sitemap>", file=output)
        print(f"    <loc>https://fatcat.wiki/{filename}</loc>", file=output)
        print(f"    <lastmod>{now}</lastmod>", file=output)
        print("  </sitemap>", file=output)

    print("</sitemapindex>", file=output)

def main():
    with open('sitemap-index-containers.xml', 'w') as output:
        index_entity("containers", output)
    with open('sitemap-index-releases.xml', 'w') as output:
        index_entity("releases", output)

if __name__=="__main__":
    main()
