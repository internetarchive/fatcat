# Norms and Policies

These social norms are explicitly expected to evolve and mature if the number
of contributors to the project grows. It is important to have some policies as
a starting point, but also important not to set these policies in stone until
they have been reviewed.

## Social Norms and Conduct

Contributors (editors and software developers) are expected to treat each other
excellently, to assume good intentions, and to participate constructively.

## Metadata Licensing

The Fatcat catalog content license is the Creative Commons Zero ("CC-0")
license, which is effectively a public domain grant. This applies to the
catalog metadata itself (titles, entity relationships, citation metadata, URLs,
hashes, identifiers), as well as "meta-meta-data" provided by editors (edit
descriptions, provenance metadata, etc).

The core catalog is designed to contain only factual information: "this work,
known by this title and with these third-party identifiers, is believed to be
represented by these files and published under such-and-such venue". As a norm,
sourcing metadata (for attribution and provenance) is retained for each edit
made to the catalog.

A notable exception to this policy are abstracts, for which no copyright claims
or license is made. Abstract content is kept separate from core catalog
metadata; downstream users need to make their own decision regarding reuse and
distribution of this material.

As a social norm, it is expected (and appreciated!) that downstream users of
the public API and/or bulk exports provide attribution, and even transitive
attribution (acknowledging the original source of metadata contributed to
Fatcat). As an academic norm, researchers are encouraged to cite the corpus as
a dataset (when this option becomes available). However, neither of these norms
are enforced via the copyright mechanism.

As a strong norm, editors should expect full access to the full corpus and edit
history, including all of their contributions.

## Immutable History

All editors agree to the licensing terms, and understand that their full public
history of contributions is made irrevocably public. Edits and contributions
may be *reverted*, but the history (and content) of their edits are retained.
Edit history is not removed from the corpus on the request of an editor or when
an editor closes their account.

In an emergency situation, such as non-bibliographic content getting encoded in
the corpus by bypassing normal filters (eg, base64 encoding hate crime content
or exploitative photos, as has happened to some blockchain projects), the
ecosystem may decide to collectively, in a coordinated manner, expunge specific
records from their history.

## Documentation Licensing

This guide ("Fatcat: The Guide") is licensed under the Creative Commons
Attribution license.

## Software Licensing

The Fatcat software project licensing policy is to adopt strong copyleft
licenses for server software (where the majority of software development takes
place), and permissive licenses for client library and bot framework software,
and CC-0 (public grant) licensing for declarative interface specifications
(such as SQL schemas and REST API specifications).

## Privacy Policy

*It is important to note that this section is currently aspirational: the
servers hosting early deployments of fatcat are largely in a default
configuration and have not been audited to ensure that these guidelines are
being followed.*

It is a goal for fatcat to conduct as little surveillance of reader and editor
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

