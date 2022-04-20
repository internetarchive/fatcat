
The domain isiarticles.com hosts a bunch of partial spam PDFs.

As a first pass, we can remove these via the domain itself.

A "blocklist" for this domain has been added to sandcrawler, so they should not
get auto-ingested in the future.

    # 2022-04-20
    fatcat-cli search file domain:isiarticles.com --count
    25067

## Prod Cleanup

See bulk edits log.
