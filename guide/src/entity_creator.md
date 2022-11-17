
# Creator Entity Reference

## Fields

- `display_name` (string, required): Full name, as will be displayed in user
  interfaces. Eg, "Grace Hopper"
- `given_name` (string): Also known as "first name". Eg, "Grace".
- `surname` (string): Also known as "last name". Eg, "Hooper".
- `orcid` (string): external identifier, as registered with ORCID.
- `wikidata_qid` (string): external linking identifier to a Wikidata entity.
  
See also ["Human Names"](./style_guide.md##human-names) sub-section of style guide.

#### `extra` Fields

All are optional.

- `also-known-as` (list of objects): additional names that this creator may be
  known under. For example, previous names, aliases, or names in different
  scripts. Can include any or all of `display_name`, `given_name`, or `surname`
  as keys.

## Human Names

Representing names of human beings in databases is a fraught subject. For some
background reading, see:

- [Falsehoods Programmers Believe About Names](https://www.kalzumeus.com/2010/06/17/falsehoods-programmers-believe-about-names/) (blog post)
- [Personal names around the world](https://www.w3.org/International/questions/qa-personal-names) (W3C informational)
- [Hubert Blaine Wolfeschlegelsteinhausenbergerdorff Sr.](https://en.wikipedia.org/wiki/Hubert_Blaine_Wolfeschlegelsteinhausenbergerdorff_Sr.) (Wikipedia article)

Particular difficult issues in the context of a bibliographic database include:

- the non-universal concept of "family" vs.  "given" names and their
  relationship to first and last names
- the inclusion of honorary titles and other suffixes and prefixes to a name
- the distinction between "preferred", "legal", and "bibliographic" names, or
  other situations where a person may not wish to be known under the name they
  are commonly referred
- language and character set issues
- different conventions for sorting and indexing names
- the sprawling world of citation styles
- name changes
- pseudonyms, anonymous publications, and fake personas (perhaps representing a
  group, like Bourbaki)

The general guidance for Fatcat is to:

- not be a "source of truth" for representing a persona or human being; ORCID
  and Wikidata are better suited to this task
- represent author personas, not necessarily 1-to-1 with human beings
- balance the concerns of readers with those of the author
- enable basic interoperability with external databases, file formats, schemas,
  and style guides
- when possible, respect the wishes of individual authors

The data model for the `creator` entity has three name fields:

- `surname` and `given_name`: needed for "aligning" with external databases,
  and to export metadata to many standard formats
- `display_name`: the "preferred" representation for display of the entire name,
  in the context of international attribution of authorship of a written work
  
Names to not necessarily need to expressed in a Latin character set, but also
does not necessarily need to be in the native language of the creator or the
language of their notable works

Ideally all three fields are populated for all creators.

It seems likely that this schema and guidance will need review.


