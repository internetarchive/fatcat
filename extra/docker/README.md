
This docker compose file can be used for local development without needing to
install some large dependencies. Currently it isn't *required* for core
development (the fatcat API server and most of the web interface), and also
doesn't bundle *all* dependencies.

This requires 

To start docker compose (kafka, zookeeper, elasticsearch) with logging in the
current terminal:

    docker-compose up

This mode is recommended for most development because the services consume a
lot of RAM and you don't want them sticking around by accident. You can run the
services in the background by adding the `-d` flag.

TODO:
- postgres
- fatcatd (rust)
- kibana
