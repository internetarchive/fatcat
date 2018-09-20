# Implementation

The canonical backend datastore exposes a microservice-like HTTP API, which
could be extended with gRPC or GraphQL interfaces. The initial datastore is a
transactional SQL database, but this implementation detail is abstracted by the
API.

As little "application logic" as possible should be embedded in this back-end;
as much as possible would be pushed to bots which could be authored and
operated by anybody. A separate web interface project talks to the API backend
and can be developed more rapidly with less concern about data loss or
corruption.

A cronjob will creae periodic database dumps, both in "full" form (all tables
and all edit history, removing only authentication credentials) and "flattened"
form (with only the most recent version of each entity).

A goal is to be linked-data/RDF/JSON-LD/semantic-web "compatible", but not
necessarily "first". It should be possible to export the database in a
relatively clean RDF form, and to fetch data in a variety of formats, but
internally fatcat will not be backed by a triple-store, and will not be bound
to a rigid third-party ontology or schema.

Microservice daemons should be able to proxy between the primary API and
standard protocols like ResourceSync and OAI-PMH, and third party bots could
ingest or synchronize the databse in those formats.
