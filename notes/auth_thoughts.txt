
For users: use openid connect (oauth2) to sign up and login to web app. From
web app, can create (and disable?) API tokens

For impl: fatcat-web has private key to create tokens. tokens used both in
cookies and as API keys. tokens are macaroons (?). fatcatd only verifies
tokens. optionally, some redis or other fast shared store to verify that tokens
haven't been revoked.

Could use portier with openid connect as an email-based option. Otherwise,
orcid, github, google.

---------

Use macaroons!

editor/user table has a "auth_epoch" timestamp; only macaroons generated
after this timestamp are valid. revocation is done by incrementing this
timestamp ("touch").

Rust CLI tool for managing users:
- create editor

Special users/editor that can create editor accounts via API; eg, one for
fatcat-web.

Associate one oauth2 id per domain per editor/user.

Users come to fatcat-web and do oauth2 to login or create an account. All
oauth2 internal to fatcat-web. If successful, fatcat-web does an
(authenticated) lookup to API for that identifier. If found, requests a
new macaroon to use as a cookie for auth. All future requests pass this
cookie through as bearer auth. fatcat-web remains stateless! macaroon
contains username (for display); no lookup-per page. Need to logout/login for
this to update?

Later, can do a "add additional account" feature.

Backend:
- oauth2 account table, foreign key to editor table
    => this is the only private table
- auth_epoch timestamp column on editor table
- lock editor by setting auth_epoch to deep future

Deploy process:
- auto-create root (admin), import-bootstrap (admin,bot), and demo-user
  editors, with fixed editor_id and "early" auth_epoch, as part of SQL. save
  tokens in env files, on laptop and QA instance.
- on live QA instance, revoke all keys when live (?)

TODO: privacy policy

fatcat API doesn't *require* auth, but if auth is provided, it will check
macaroon, and validate against editor table's timestamp.

support oauth2 against:
- orcid
- git.archive.org
- github
? google

Macaroon details:
- worth looking at "bakery" projects (python and golang) for example of how to
  actually implement macaroon authentication/authorization
- location is fatcat.wiki (and/or qa.fatcat.wiki, or test or localhost or test.fatcat.wiki?)
- identifier is a UUID in upper-case string format
- will need some on-disk key storage thing?
    => how to generate new keys? which one should be used, most recent?
       conception of revoking keys? simple JSON/TOML, or LMDB?
- call them "authentication tokens"?
- params/constraints
    - editor_id: always, fcid format
    - created: always, some date format (seconds/iso)
    - expires: optional, same date format

It's a huge simplification to have webface generate macaroons as well, using a
root key. webface doesn't need multiple keys because it only creates, doesn't
verify.

Code structure:
- auth service/struct is generated at startup; reads environment and on-disk keys
- verify helper does the thing
- some sort of auth/edit context

Roles?
- public: unauthenticated
- editor: any authenticated, active account
- bot
- admin

Caveats:
- general model is that macaroon is omnipotent and passes all verification,
  unless caveats are added. eg, adding verification checks doesn't constrain
  auth, only the caveats constrain auth; verification check *allow* additional
  auth. each caveat only needs to be allowed by one verifiation.
- can (and should?) add as many caveat checkers/constrants in code as possible

http://evancordell.com/2015/09/27/macaroons-101-contextual-confinement.html
