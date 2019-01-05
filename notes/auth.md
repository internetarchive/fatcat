
This file summarizes the current fatcat authentication schema, which is based
on 3rd party OAuth2/OIDC sign-in and macaroon tokens.

## Overview

The informal high-level requirements for the auth system were:

- public read-only (HTTP GET) API and website require no login or
  authentication
- all changes to the catalog happen through the API and are associated with an
  abstract editor (the entity behind an editor could be human, a bots, an
  organization, change over time, etc). basic editor metadata (eg, identifier)
  is public for all time.
- editors can signup (create account) and login using the web interface
- bots and scripts access the API directly; their actions are associated with
  an editor (which could be a bot account)
- authentication can be managed via the web interface (eg, creating any tokens
  or bot accounts)
- there is a mechanism to revoke API access and lock editor accounts (eg, to
  block spam); this mechanism doesn't need to be a web interface, but shouldn't
  be raw SQL commands
- store an absolute minimum of PII (personally identifiable intformation) that
  can't be "mixed in" with public database dumps, or would make the database a
  security target. eg, if possible don't store emails or passwords
- the web interface should, as much as possible, not be "special". Eg, should
  work through the API and not have secret keys, if possible
- be as simple an efficient as possible (eg, minimize per-request database
  hits)

The initial design that came out of these requirements is to use bearer tokens
(in the form of macaroons) for all API authentication needs, and to have editor
account creation and authentication offloaded to third parties via OAuth2
(specifically OpenID Connect (OIDC) when available). By storing only OIDC
identifiers in a single database table (linked but separate from the editor
table), PII collection is minimized, and no code needs to be written to handle
password recovery, email verification, etc. Tokens can be embedded in web
interface session cookies and "passed through" in API calls that require
authentication, so the web interface is effectively stateless (in that it does
not hold any session or user information internally).

Macaroons, like JSON Web Tokens (JWT) contain signed (verifiable) constraints,
called caveats. Unlike JWT, these caveats can easily be "further constrained"
by any party. There is additional support for signed third party caveats, but
we don't use that feature currently. Caveats can be used to set an expiry time
for each token, which is appropriate for cookies (requiring a fresh login). We
also use creation timestamps and per-editor "authentication epoches" (publicly
stored in the editor table, non-sensitive) to revoke API tokens per-editor (or
globally, if necessary). Basically, only macaroons that were "minted" after the
current `auth_epoch` for the editor are considered valid. If a token is lost,
the `auth_epoch` is reset to the current time (after the compromised token was
minted, or any subsequent tokens possibly created by an attacker), all existing
tokens are considered invalid, and the editor must log back in (and generate
new API tokens for any bots/scripts). In the event of a serious security
compromise (like the secret signing key being compromised, or a bug in macaroon
generation is found), all `auth_epoch` timestamps are updated at once (and a
new key is used).

The account login/signup flow for new editors is to visit the web interface and
select an OAuth provider (from a fixed list) where they have an account. After
they approve Fatcat as an application on the third party site, they bounce back
to the web interface. If they had signed up previously they are signed in,
otherwise a new editor account is automatically created. A username is
generated based on the OAuth remote account name, but the editor can change
this immediately. The web interface allows (or will, when implemented) creation
of bot accounts (linked to a "wrangler" editor account), generation of tokens,
etc.

In theory, the API tokens, as macaroons, can be "attenuated" by the user with
additional caveats before being used. Eg, the expiry could be throttled down to
a minute or two, or constrained to edits of a specific editgroup, or to a
specific API endpoint. A use-case for this would be pasting a token in a
single-page app or untrusted script with minimal delgated authority. Not all of
these caveat checks have been implemented in the server yet though.

As an "escape hatch", there is a rust command (`fatcat-auth`) for debugging,
creating new keys and tokens, revoking tokens (via `auth_epoch`), etc. There is
also a web interface mechanism to "login via existing token". These mechanisms
aren't intended for general use, but are helpful when developing (when login
via OAuth may not be configured or accessible) and for admins/operators.

## Current Limitations

No mechanism for linking (or unlinking) multiple remote OAuth accounts into a
single editor account. The database schema supports this, there just aren't API
endpoints or a web interface.

There is no obvious place to store persistent non-public user information:
things like preferences, or current editgroup being operated on via the web
interface. This info can go in session cookies, but is lost when user logs
out/in or uses another device.

## API Tokens (Macaroons)

Macaroons contain "caveats" which constrain their scope. In the context of
fatcat, macaroons should always be constrained to a single editor account (by
`editor_id`) and a valid creation timestamp; this enables revocation.

In general, want to keep caveats, identifier, and other macaroon contents as
short as possible, because they can bloat up the token size.

Use identifiers (unique names for looking up signing keys) that contain the
date and (short) domain, like `20190110-qa`.

Caveats:

- general model is that macaroon is omnipotent and passes all verification,
  unless caveats are added. eg, adding verification checks doesn't constrain
  auth, only the caveats constrain auth; verification check *allow* additional
  auth. each caveat only needs to be allowed by one verifiation.
- can (and should?) add as many caveat checkers/constrants in code as possible

## Web Signup/Login

OpenID Connect (OIDC) is basically a convention for servers and clients to use
OAuth2 for the specific purpose of just logging in or linking accounts, a la
"Sign In With ...". OAuth is often used to provider interoperability between
service (eg, a client app can take actions as the user, when granted
permissions, on the authenticating platform); OIDC doesn't grant any such
permissions, just refreshing logins at most.

The web interface (webface) does all the OAuth/OIDC trickery, and extracts a
simple platform identifier and user identifier if authentication was
successful. It sends this in a fatcat API request to the `/auth/oidc` endpoint,
using admin authentication (the web interface stores an internal token "for
itself" for this one purpose). The API will return both an Editor object and a
token for that editor in the response. If the user had signed in previously
using the same provider/service/user pair as before, the Editor object is the
user's login. If the pair is new, a new account is created automatically and
returned; the HTTP status code indicates which happened. The editor username is
automatically generated from the remote username and platform (user can change
it if they want).

The returned token and editor metadata are stored in session cookies. The flask
framework has a secure cookie implementation that prevents users from making up
cookies, but this isn't the real security mechanism; the real mechanism is that
they can't generate valid macaroons because they are signed. Cookie *theft* is
an issue, so aggressive cookie protections should be activated in the Flask
configuration.

The `auth_oidc` enforces uniqueness on accounts in a few ways:

- lowercase UNIQ constaint on usernames (can't register upper- and lower-case
  variants)
- UNIQ {`editor_id`, `platform`}: can't login using multiple remote accounts
  from the same platform
- UNIQ {`platform`, `remote_host`, `remote_id`}: can't login to multiple local
  accounts using the same remote account
- all fields are NOT NULL

## Role-Based Authentication (RBAC)

Current acknowledge roles:

- public (not authenticated)
- bot
- human
- editor (bot or human)
- admin
- superuser

Will probably rename these. Additionally, editor accounts have an `is_active`
flag (used to lock disabled/deleted/abusive/compromised accounts); no roles
beyond public are given for inactive accounts.

## Developer Affordances

A few core accounts are created automatically, with fixed `username`,
`auth_epoch` and `editor_id`, to make testing and administration easier across
database resets (aka, tokens keep working as long as the signing key stays the
same).

Tokens and other secrets can be store in environment variables, scripts, or
`.env` files.

## Future Work and Alternatives

Want to support more OAuth/OIDC endpoints:

- archive.org: bespoke "XAuth" thing; would be reasonable to hack in support.
  use user itemname as persistent 'sub' field
- orcid.org: supports OIDC
- wikipedia/wikimedia: OAuth; https://github.com/valhallasw/flask-mwoauth
- additional 

Additional macaroon caveats:

- `endpoint` (API method; caveat can include a list)
- `editgroup`
- (etc)

Looked at a few other options for managing use accounts:

- portier, the successor to persona, which basically uses email for magic-link
  login, unless the email provider supports OIDC or similar. There is a central
  hosted version to use for bootstrap. Appealing/minimal, but feels somewhat
  neglected.
- use something like 'dex' as a proxy to multiple OIDC (and other) providers
- deploy a huge all-in-one platform like keycloak for all auth anything ever.
  sort of wish Internet Archive, or somebody (Wikimedia?) ran one of these as
  public infrastructure.
- having webface generate macaroons itself

## Implementation Notes

To start, using the `loginpass` python library to handle logins, which is built
on `authlib`. May need to extend or just use `authlib` directly in the future.
Supports many large commercial providers, including gitlab.com, github.com, and
google.

There are many other flask/oauth/OIDC libraries out there, but this one worked
well with multiple popular providers, mostly by being flexible about actual
OIDC support. For example, Github doesn't support OIDC (only OAuth2), and
apparently Gitlab's is incomplete/broken.

### Background Reading

Other flask OIDC integrations:

- https://flask-oidc.readthedocs.io/en/latest/
- https://github.com/zamzterz/Flask-pyoidc

Background reading on macaroons:

- https://github.com/rescrv/libmacaroons
- http://evancordell.com/2015/09/27/macaroons-101-contextual-confinement.html
- https://blog.runscope.com/posts/understanding-oauth-2-and-openid-connect
- https://latacora.micro.blog/2018/06/12/a-childs-garden.html
- https://github.com/go-macaroon-bakery/macaroon-bakery (for the "bakery" API pattern)

