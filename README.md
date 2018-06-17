
      __       _            _   
     / _| __ _| |_ ___ __ _| |_ 
    | |_ / _` | __/ __/ _` | __|
    |  _| (_| | || (_| (_| | |_ 
    |_|  \__,_|\__\___\__,_|\__|

                                        ... catalog all the things!


The [RFC](./farcat-rfc.md) is the original design document, and the best place
to start for background.

There will be three main components:

- backend API server and database
- front-end web interface (built on API)
- client libraries and bots

The API server was prototyped in python. "Real" implementation started in
golang, but shifted to Rust, and is work-in-progress. The beginings of a client
library and data ingesters exist in python (may or may not be re-written in
Rust).
