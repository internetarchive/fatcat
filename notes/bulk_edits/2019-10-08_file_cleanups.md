
These cleanups are primarily intended to fix bogus 'None' datetime links to
wayback for files that are actually in petabox (archive.org not
web.archive.org). These URLs were created accidentally during fatcat
boostrapping; there are about 300k such file enties to fix.

Will also update archive.org link reltype to 'archive' (instead of
'repository'), which is the new prefered style.

Generated the set of files to update like:

    zcat file_export.2019-07-07.json.gz | rg 'web.archive.org/web/None' | gzip > file_export.2019-07-07.None.json.gz

    zcat /srv/fatcat/datasets/file_export.2019-07-07.None.json.gz | wc -l
    304308

## QA

Running at git rev:

    984a1b157990f42f8c57815f4b3c00f6455a114f

Created a new 'cleanup-bot' account and credentials. Put token in local env.

Ran with a couple hundred entities first; edits look good.

    zcat /srv/fatcat/datasets/file_export.2019-07-07.None.json.gz | head -n200 | ./fatcat_cleanup.py files -

Then the full command, with batchsize=100:

    time zcat /srv/fatcat/datasets/file_export.2019-07-07.None.json.gz | pv -l | ./fatcat_cleanup.py --batch-size 100 files -

Should finish in a couple hours.

    304k 1:05:19 [77.6 /s]

    Counter({'cleaned': 304308, 'lines': 304308, 'updated': 297308, 'skip-revision': 7000})

    real    65m20.613s
    user    20m40.828s
    sys     0m34.492s

## Production

Again ran with a couple hundred entities first; edits look good.

    zcat /srv/fatcat/datasets/file_export.2019-07-07.None.json.gz | head -n200 | ./fatcat_cleanup.py files -

Then the full command, with batchsize=100:

    time zcat /srv/fatcat/datasets/file_export.2019-07-07.None.json.gz | pv -l | ./fatcat_cleanup.py --batch-size 100 files -
    [...]
    304k 1:03:10 [80.3 /s]
    Counter({'cleaned': 304308, 'lines': 304308, 'updated': 304107, 'skip-revision': 201})

    real    63m11.631s
    user    21m8.504s
    sys     0m31.888s

