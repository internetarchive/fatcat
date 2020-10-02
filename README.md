
              __       _            _   
             / _| __ _| |_ ___ __ _| |_ 
            | |_ / _` | __/ __/ _` | __|
            |  _| (_| | || (_| (_| | |_ 
            |_|  \__,_|\__\___\__,_|\__|

       perpetual access to the scholarly record


[![pipeline status](https://gitlab.com/bnewbold/fatcat/badges/master/pipeline.svg)](https://gitlab.com/bnewbold/fatcat/commits/master)
[![coverage report](https://gitlab.com/bnewbold/fatcat/badges/master/coverage.svg)](https://gitlab.com/bnewbold/fatcat/commits/master)

This repository contains source code for **fatcat**, an editable catalog of
published written works (mostly journal articles), with a focus on tracking the
location and status of full-text copies to ensure "perpetual access". The
primary public instance runs at [fatcat.wiki](https://fatcat.wiki). Both the
software project and primary instance are a project of the [Internet
Archive](https://archive.org).

Some resources for learning more about the aims, goals, and structure of this
overall project:

* **[FORCE11 2019 Presentation Video](https://www.youtube.com/watch?v=PARqfbYIdXQ)**: "Perpetual Access Machines: Archiving Web-Published Scholarship at Scale" (30 minutes)
* **["How the Internet Archive is Ensuring Permanent Access to Open Access Journal Articles"](https://blog.archive.org/2020/09/15/how-the-internet-archive-is-ensuring-permanent-access-to-open-access-journal-articles/)**: archive.org blog post (September 2020)
* **[guide.fatcat.wiki](https://guide.fatcat.wiki)**: project documentation,
  including schema overview, HOWTOs, policies, and more
* **[Fatcat RFC](./fatcat-rfc.md)**: original project design proposal


## Getting Started for Developers

There are three main components:

- backend API server and database (in Rust)
- API client libraries and bots (in Python)
- front-end web interface (in Python; built on API and library)

The `python/` and `rust/` folders have their own READMEs describing how to set
up development environments and requirements for those languages. Each also has
Makefiles to help with builds and running tests.

The python client library, which is automatically generated from the API
schema, lives under `./python_openapi_client/`.

To do unified development involving both the python code (web interface, bot
code) and the rust code (API server), you will likely need to install and run a
PostgreSQL (11+) database locally. For more advanced development involving
Kafka data pipelines or the metadata search index, there is a docker compose
file in `./extra/docker/` to run these services locally.

Contributors are asked run all of the following and correct any (new) lint
warnings before submitting patches:

    make fmt
    make lint
    make test

It is very appreciated if new features and code comes with full test coverage,
but maintainers can review code and help if this is difficult.


## Contributing

Software, documentation, new bots, and other contributions to this repository
are welcome! When considering a non-trivial contribution, it can save review
time and duplicated work to jump in the chatroom or post an issue with your
idea and intentions before starting work. Large changes and features usually
have a design proposal drafted and merged first (see `./proposals/`).

There is a public chatroom where you can discuss and ask questions at
[gitter.im/internetarchive/fatcat](https://gitter.im/internetarchive/fatcat).

Contributors in this project are asked to abide by our
[Code of Conduct](https://guide.fatcat.wiki/code_of_conduct.html).

See the `LICENSE` file for detailed permissions and licensing of both python
and rust code. In short, the auto-generated client libraries are permissively
released, while the API server and web interface are strong copyleft (AGPLv3).

For software developers, the "help wanted" tag in Github Issues is a way to
discover bugs and tasks that external folks could contribute to.

