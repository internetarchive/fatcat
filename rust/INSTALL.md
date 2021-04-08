
Canonical IA production/QA ansible scripts are in the journal-infra repo. These
directions are likely to end up out-of-date.

## Simple Server Deployment

To install manually, on a bare server, as root:

    adduser fatcat
    apt install postgresql-13 postgresql-contrib postgresql-client-13 \
        nginx build-essential git pkg-config libssl-dev libpq-dev \
        htop screen
    mkdir -p /srv/fatcat
    chown fatcat:fatcat /srv/fatcat

    # setup new postgres user
    su - postgres
    createuser -P -s fatcat     # strong random password

    # as fatcat user
    su - fatcat
    ssh-keygen
    curl https://sh.rustup.rs -sSf | sh
    source $HOME/.cargo/env
    cargo install diesel_cli --no-default-features --features "postgres"
    cd /srv/fatcat
    git clone https://github.com/internetarchive/fatcat.git
    cd rust
    cargo build
    echo "DATABASE_URL=postgres://fatcat@localhost/fatcat" > .env
    diesel database reset

    # as fatcat, in a screen or something
    cd /srv/fatcat/fatcat/rust
    cargo run --bin fatcatd
