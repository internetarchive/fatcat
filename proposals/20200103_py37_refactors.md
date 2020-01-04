
status: planning

If we update fatcat python code to python3.7, what code refactoring changes can
we make? We currently use/require python3.5.

Nice features in python3 I know of are:

- dataclasses (python3.7)
- async/await (mature in python3.7?)
- type annotations (python3.5)
- format strings (python3.6)
- walrus assignment (python3.8)

Not sure if the walrus operator is worth jumping all the way to python3.8.

While we might be at it, what other superficial factorings might we want to do?

- strict lint style (eg, maximum column width) with `black` (python3.6)
- logging/debugging/verbose
- type annotations and checking
- use named dicts or structs in place of dicts

## Linux Distro Support

The default python version shipped by current and planned linux releases are:

- ubuntu xenial 16.04 LTS:  python3.5
- ubuntu bionic 18.04 LTS:  python3.6
- ubuntu focal  20.04 LTS:  python3.8 (planned)
- debian buster 10 2019:    python3.7

Python 3.7 is the default in debian buster (10).

There are apt PPA package repositories that allow backporting newer pythons to
older releases. As far as I know this is safe and doesn't override any system
usage if we are careful not to set the defaults (aka, `python3` command should
be the older version unless inside a virtualenv).

It would also be possible to use `pyenv` to have `virtualenv`s with custom
python versions. We should probably do that for OS X and/or windows support if
we wanted those. But having a system package is probably a lot faster to
install.

## Dataclasses

`dataclasses` are a user-friendly way to create struct-like objects. They are
pretty similar to the existing `namedtuple`, but can be mutable and have
methods attached to them (they are just classes), plus several other usability
improvements.

Most places we are throwing around dicts with structure we could be using
dataclasses instead. There are some instances of this in fatcat, but many more
in sandcrawler.

## Async/Await

Where might we actually use async/await? I think more in sandcrawler than in
the python tools or web apps. The GROBID, ingest, and ML workers in particular
should be async over batches, as should all fetches from CDX/wayback.

Some of the kafka workers *could* be aync, but i'm not sure how much speedup
there would actually be. For example, the entity updates worker could fetch
entities for an editgroup concurrently.

Inserts (importers) should probably mostly happen serially, at least the kafka
importers, one editgroup at a time, so progress is correctly recorded in kafka.
Parallelization should probably happen at the partition level; would need to
think through whether async would actually help with code simplicity vs. thread
or process parallelization.

## Type Annotations

The meta-goals of (gradual) type annotations would be catching more bugs at
development time, and having code be more self-documenting and easier to
understand.

The two big wins I see with type annotation would be having annotations
auto-generated for the openapi classes and API calls, and to make string
munging in importer code less buggy.

## Format Strings

Eg, replace code like:

    "There are {} out of {} objects".format(found, total)

With:

    f"There are {found} out of {total} objects"

## Walrus Operator

New operator allows checking and assignment together:

    if (n := len(a)) > 10:
        print(f"List is too long ({n} elements, expected <= 10)")

I feel like we would actually use this pattern *a ton* in importer code, where
we do a lot of lookups or cleaning then check if we got a `None`.

