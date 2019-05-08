
Google has a limit of 50k lines / 10 MByte for text sitemap files, and 50K
lines / 50 MByte for XML site map files.

With a baseline of 100 million entities, that requires an index file pointing
to at least 2000x individual sitemaps. 3 hex characters is 12 bits, or 4096
options; seems like an ok granularity to start with.

Should look in to what archive.org does to generate their sitemap.xml, seems
simple, and comes in batches of exactly 50k.

## Text Sitemaps

Should be possible to create simple text-style sitemaps, one URL per line, and
link to these from a sitemap index. This is appealing because the sitemaps can
be generated very quickly from identifier SQL dump files, run through UNIX
commands (eg, to split and turn into URLs). Some script to create an XML
sitemap index to point at all the sitemaps would still be needed though.


## Resources

Google sitemap verifier: https://support.google.com/webmasters/answer/7451001
