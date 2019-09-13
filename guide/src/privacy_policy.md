
## Privacy Policy

*It is important to note that this section is currently aspirational: the
servers hosting early deployments of Fatcat are largely in a defaults
configuration and have not been audited to ensure that these guidelines are
being followed.*

It is a goal for Fatcat to conduct as little surveillance of reader and editor
behavior and activities as possible. In practical terms, this means minimizing
the overall amount of logging and collection of identifying information. This
is in contrast to *submitted edit content*, which is captured, preserved, and
republished as widely as possible.

The general intention is to:

- not use third-party tracking (via extract browser-side requests or
  javascript)
- collect aggregate *metrics* (overall hit numbers), but not *log* individual
  interactions ("this IP visited this page at this time")

Exceptions will likely be made:

- temporary caching of IP addresses may be necessary to implement rate-limiting
  and debug traffic spikes
- exception logging, abuse detection, and other exceptional 

Some uncertain areas of privacy include:

- should third-party authentication identities be linked to editor ids? what
  about the specific case of ORCID if used for login?
- what about discussion and comments on edits? should conversations be included
  in full history dumps? should editors be allowed to update or remove
  comments?

