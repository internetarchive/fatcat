"""
Note: in thoery could use, eg, https://github.com/christabor/swagger_wtforms,
but can't find one that is actually maintained.
"""

import datetime
from typing import Any, Dict, List, Tuple

import toml
from fatcat_openapi_client import (
    ContainerEntity,
    FileEntity,
    FileUrl,
    ReleaseContrib,
    ReleaseEntity,
    ReleaseExtIds,
)
from flask_wtf import FlaskForm
from wtforms import (
    DateField,
    FieldList,
    FormField,
    HiddenField,
    IntegerField,
    SelectField,
    StringField,
    TextAreaField,
    ValidationError,
    validators,
)

from fatcat_tools.transforms import entity_to_toml

release_type_options: List[Tuple[str, str]] = [
    ("", "Unknown (blank)"),
    ("article-journal", "Journal Article"),
    ("paper-conference", "Conference Proceeding"),
    ("article", "Article (non-journal)"),
    ("book", "Book"),
    ("chapter", "Book Chapter"),
    ("dataset", "Dataset"),
    ("stub", "Invalid/Stub"),
]
release_stage_options: List[Tuple[str, str]] = [
    ("", "Unknown (blank)"),
    ("draft", "Draft"),
    ("submitted", "Submitted"),
    ("accepted", "Accepted"),
    ("published", "Published"),
    ("updated", "Updated"),
]
withdrawn_status_options: List[Tuple[str, str]] = [
    ("", "Not Withdrawn (blank)"),
    ("retracted", "Retracted"),
    ("withdrawn", "Withdrawn"),
    ("concern", "Concern Noted"),
    ("spam", "Spam"),
    ("legal", "Legal Taketown"),
    ("safety", "Public Safety"),
    ("national-security", "National Security"),
]
role_type_options: List[Tuple[str, str]] = [
    ("author", "Author"),
    ("editor", "Editor"),
    ("translator", "Translator"),
]


class EntityEditForm(FlaskForm):
    editgroup_id = StringField(
        "Editgroup ID", [validators.Optional(True), validators.Length(min=26, max=26)]
    )
    editgroup_description = StringField("Editgroup Description", [validators.Optional(True)])
    edit_description = StringField("Description of Changes", [validators.Optional(True)])


class ReleaseContribForm(FlaskForm):
    class Meta:
        # this is a sub-form, so disable CSRF
        csrf = False

    # surname
    # given_name
    # creator_id (?)
    # orcid (for match?)
    prev_index = HiddenField("prev_revision index", default=None)
    raw_name = StringField("Display Name", [validators.DataRequired()])
    role = SelectField([validators.DataRequired()], choices=role_type_options, default="author")


RELEASE_SIMPLE_ATTRS: List[str] = [
    "title",
    "original_title",
    "work_id",
    "container_id",
    "release_type",
    "release_stage",
    "withdrawn_status",
    "release_date",
    "release_year",
    "volume",
    "issue",
    "pages",
    "publisher",
    "language",
    "license_slug",
]

RELEASE_EXTID_ATTRS: List[str] = ["doi", "wikidata_qid", "isbn13", "pmid", "pmcid"]


def valid_year(form: Any, field: Any) -> None:
    if field.data > datetime.date.today().year + 5:
        raise ValidationError(f"Year is too far in the future: {field.data}")
    if field.data < 10:
        raise ValidationError(f"Year is too far in the past: {field.data}")


def valid_2char_ascii(form: Any, field: Any) -> None:
    if (
        len(field.data) != 2
        or len(field.data.encode("utf-8")) != 2
        or not field.data.isalpha()
        or field.data != field.data.lower()
    ):
        raise ValidationError(f"Must be 2-character ISO format, lower case: {field.data}")


class ReleaseEntityForm(EntityEditForm):
    """
    TODO:
    - field types: fatcat id
    - date
    """

    title = StringField("Title", [validators.DataRequired()])
    original_title = StringField("Title in Original Language (if different)")
    work_id = StringField(
        "Work FCID", [validators.Optional(True), validators.Length(min=26, max=26)]
    )
    container_id = StringField(
        "Container FCID", [validators.Optional(True), validators.Length(min=26, max=26)]
    )
    release_type = SelectField(
        "Release Type", [validators.DataRequired()], choices=release_type_options, default=""
    )
    release_stage = SelectField(choices=release_stage_options)
    withdrawn_status = SelectField(
        "Withdrawn Status",
        [validators.Optional(True)],
        choices=withdrawn_status_options,
        default="",
    )
    release_date = DateField("Release Date", [validators.Optional(True)])
    release_year = IntegerField("Release Year", [validators.Optional(True), valid_year])
    doi = StringField(
        "DOI",
        [
            validators.Regexp(r"^10\..*\/.*", message="DOI must be valid"),
            validators.Optional(True),
        ],
    )
    wikidata_qid = StringField("Wikidata QID")
    isbn13 = StringField("ISBN-13")
    pmid = StringField("PubMed Id")
    pmcid = StringField("PubMed Central Id")
    # core_id
    # arxiv_id
    # jstor_id
    # oai
    # hdl
    volume = StringField("Volume")
    issue = StringField("Issue")
    pages = StringField("Pages")
    publisher = StringField("Publisher (optional)")
    language = StringField("Language (code)", [validators.Optional(True), valid_2char_ascii])
    license_slug = StringField("License (slug)")
    contribs = FieldList(FormField(ReleaseContribForm))
    # refs
    # abstracts

    @staticmethod
    def from_entity(re: ReleaseEntity) -> "ReleaseEntityForm":
        """
        Initializes form with values from an existing release entity.
        """
        ref = ReleaseEntityForm()
        for simple_attr in RELEASE_SIMPLE_ATTRS:
            a = getattr(ref, simple_attr)
            a.data = getattr(re, simple_attr)
        for extid_attr in RELEASE_EXTID_ATTRS:
            a = getattr(ref, extid_attr)
            a.data = getattr(re.ext_ids, extid_attr)
        for i, c in enumerate(re.contribs):
            rcf = ReleaseContribForm()
            rcf.prev_index = i
            rcf.role = c.role
            rcf.raw_name = c.raw_name
            ref.contribs.append_entry(rcf)
        return ref

    def to_entity(self) -> ReleaseEntity:
        assert self.title.data
        entity = ReleaseEntity(title=self.title.data, ext_ids=ReleaseExtIds())
        self.update_entity(entity)
        return entity

    def update_entity(self, re: ReleaseEntity) -> None:
        """
        Mutates a release entity in place, updating fields with values from
        this form.

        Form must be validated *before* calling this function.
        """
        for simple_attr in RELEASE_SIMPLE_ATTRS:
            a = getattr(self, simple_attr).data
            # special case blank strings
            if a == "":
                a = None
            setattr(re, simple_attr, a)
        for extid_attr in RELEASE_EXTID_ATTRS:
            a = getattr(self, extid_attr).data
            # special case blank strings
            if a == "":
                a = None
            setattr(re.ext_ids, extid_attr, a)
        if self.release_date.data:
            re.release_year = self.release_date.data.year
        # bunch of complexity here to preserve old contrib metadata (eg,
        # affiliation and extra) not included in current forms
        # TODO: this may be broken; either way needs tests
        if re.contribs:
            old_contribs = re.contribs.copy()
            re.contribs = []
        else:
            old_contribs = []
            re.contribs = []
        for c in self.contribs:
            if c.prev_index.data not in ("", None):
                rc = old_contribs[int(c.prev_index.data)]
                rc.role = c.role.data or None
                rc.raw_name = c.raw_name.data or None
            else:
                rc = ReleaseContrib(
                    role=c.role.data or None,
                    raw_name=c.raw_name.data or None,
                )
            re.contribs.append(rc)
        if self.edit_description.data:
            re.edit_extra = dict(description=self.edit_description.data)


container_type_options: List[Tuple[str, str]] = [
    ("", "Unknown (blank)"),
    ("journal", "Scholarly Journal"),
    ("proceedings", "Proceedings"),
    ("book-series", "Book Series"),
    ("blog", "Blog"),
    ("magazine", "Magazine"),
    ("trade", "Trade Magazine"),
    ("test", "Test / Dummy"),
]

CONTAINER_SIMPLE_ATTRS: List[str] = [
    "name",
    "container_type",
    "publisher",
    "issnl",
    "wikidata_qid",
    "issne",
    "issnp",
]
CONTAINER_EXTRA_ATTRS: List[str] = ["original_name", "country"]


class ContainerEntityForm(EntityEditForm):
    name = StringField("Name/Title", [validators.DataRequired()])
    container_type = SelectField(
        "Container Type",
        [validators.Optional(True)],
        choices=container_type_options,
        default="",
    )
    publisher = StringField("Publisher")
    issnl = StringField("ISSN-L (linking)")
    issne = StringField("ISSN (electronic)")
    issnp = StringField("ISSN (print)")
    original_name = StringField("Name in Original Language (if different)")
    country = StringField(
        "Country of Publication (ISO code)", [validators.Optional(True), valid_2char_ascii]
    )
    wikidata_qid = StringField("Wikidata QID")
    urls = FieldList(
        StringField(
            "Container URLs", [validators.DataRequired(), validators.URL(require_tld=False)]
        )
    )

    @staticmethod
    def from_entity(ce: ContainerEntity) -> "ContainerEntityForm":
        """
        Initializes form with values from an existing container entity.
        """
        cef = ContainerEntityForm()
        for simple_attr in CONTAINER_SIMPLE_ATTRS:
            a = getattr(cef, simple_attr)
            a.data = getattr(ce, simple_attr)
        if ce.extra:
            for k in CONTAINER_EXTRA_ATTRS:
                if ce.extra.get(k):
                    a = getattr(cef, k)
                    a.data = ce.extra[k]
            if ce.extra.get("urls"):
                for url in ce.extra["urls"]:
                    cef.urls.append_entry(url)
        return cef

    def to_entity(self) -> ContainerEntity:
        assert self.name.data
        entity = ContainerEntity(name=self.name.data)
        self.update_entity(entity)
        return entity

    def update_entity(self, ce: ContainerEntity) -> None:
        """
        Mutates a container entity in place, updating fields with values from
        this form.

        Form must be validated *before* calling this function.
        """
        for simple_attr in CONTAINER_SIMPLE_ATTRS:
            a = getattr(self, simple_attr).data
            # special case blank strings
            if a == "":
                a = None
            setattr(ce, simple_attr, a)
        if not ce.extra:
            ce.extra = dict()
        for extra_attr in CONTAINER_EXTRA_ATTRS:
            a = getattr(self, extra_attr).data
            if a and a != "":
                ce.extra[extra_attr] = a
        extra_urls = []
        for url in self.urls:
            extra_urls.append(url.data)
        if extra_urls:
            ce.extra["urls"] = extra_urls
        if self.edit_description.data:
            ce.edit_extra = dict(description=self.edit_description.data)
        if not ce.extra:
            ce.extra = None


url_rel_options: List[Tuple[str, str]] = [
    ("web", "Public Web"),
    ("webarchive", "Web Archive"),
    ("repository", "Repository"),
    ("archive", "Preservation Archive"),
    ("academicsocial", "Academic Social Network"),
    ("publisher", "Publisher"),
    ("dweb", "Decentralized Web"),
    ("aggregator", "Aggregator"),
]

FILE_SIMPLE_ATTRS: List[str] = ["size", "md5", "sha1", "sha256", "mimetype"]


class FileUrlForm(FlaskForm):
    class Meta:
        # this is a sub-form, so disable CSRF
        csrf = False

    url = StringField(
        "Display Name", [validators.DataRequired(), validators.URL(require_tld=False)]
    )
    rel = SelectField([validators.DataRequired()], choices=url_rel_options, default="web")


class FileEntityForm(EntityEditForm):
    # TODO: positive definite
    size = IntegerField("Size (bytes)", [validators.DataRequired()])
    md5 = StringField("MD5", [validators.Optional(True), validators.Length(min=32, max=32)])
    sha1 = StringField("SHA-1", [validators.DataRequired(), validators.Length(min=40, max=40)])
    sha256 = StringField(
        "SHA-256", [validators.Optional(True), validators.Length(min=64, max=64)]
    )
    urls = FieldList(FormField(FileUrlForm))
    mimetype = StringField("Mimetype")
    release_ids = FieldList(
        StringField(
            "Release FCID", [validators.DataRequired(), validators.Length(min=26, max=26)]
        )
    )

    @staticmethod
    def from_entity(fe: FileEntity) -> "FileEntityForm":
        """
        Initializes form with values from an existing file entity.
        """
        ref = FileEntityForm()
        for simple_attr in FILE_SIMPLE_ATTRS:
            a = getattr(ref, simple_attr)
            a.data = getattr(fe, simple_attr)
        for i, c in enumerate(fe.urls):
            ruf = FileUrlForm()
            ruf.rel = c.rel
            ruf.url = c.url
            ref.urls.append_entry(ruf)
        for r in fe.release_ids:
            ref.release_ids.append_entry(r)
        return ref

    def to_entity(self) -> FileEntity:
        assert self.sha1.data
        entity = FileEntity()
        self.update_entity(entity)
        return entity

    def update_entity(self, fe: FileEntity) -> None:
        """
        Mutates in place, updating fields with values from this form.

        Form must be validated *before* calling this function.
        """
        for simple_attr in FILE_SIMPLE_ATTRS:
            a = getattr(self, simple_attr).data
            # be flexible about hash capitalization
            if simple_attr in ("md5", "sha1", "sha256"):
                a = a.lower()
            # special case blank strings
            if a == "":
                a = None
            setattr(fe, simple_attr, a)
        fe.urls = []
        for u in self.urls:
            fe.urls.append(
                FileUrl(
                    rel=u.rel.data or None,
                    url=u.url.data or None,
                )
            )
        fe.release_ids = []
        for ri in self.release_ids:
            fe.release_ids.append(ri.data)
        if self.edit_description.data:
            fe.edit_extra = dict(description=self.edit_description.data)


INGEST_TYPE_OPTIONS: List[Tuple[str, str]] = [
    ("pdf", "PDF Fulltext"),
    ("html", "HTML Fulltext"),
    ("xml", "XML Fulltext"),
]


class SavePaperNowForm(FlaskForm):

    base_url = StringField("URL", [validators.DataRequired(), validators.URL()])
    ingest_type = SelectField(
        "Content Type", [validators.DataRequired()], choices=INGEST_TYPE_OPTIONS, default="pdf"
    )
    release_stage = SelectField(
        "Publication Stage",
        [validators.DataRequired()],
        choices=release_stage_options,
        default="",
    )

    def to_ingest_request(
        self, release: ReleaseEntity, ingest_request_source: str = "savepapernow"
    ) -> Dict[str, Any]:
        base_url = self.base_url.data
        ext_ids = release.ext_ids.to_dict()
        # by default this dict has a bunch of empty values
        ext_ids = dict([(k, v) for (k, v) in ext_ids.items() if v])
        ingest_request = {
            "ingest_type": self.ingest_type.data,
            "ingest_request_source": ingest_request_source,
            "link_source": "spn",
            "link_source_id": release.ident,
            "base_url": base_url,
            "fatcat": {
                "release_ident": release.ident,
                "work_ident": release.work_id,
            },
            "ext_ids": ext_ids,
        }
        if self.release_stage.data:
            ingest_request["release_stage"] = self.release_stage.data

        if release.ext_ids.doi and base_url == "https://doi.org/{}".format(release.ext_ids.doi):
            ingest_request["link_source"] = "doi"
            ingest_request["link_source_id"] = release.ext_ids.doi
        elif release.ext_ids.arxiv and base_url == "https://arxiv.org/pdf/{}.pdf".format(
            release.ext_ids.arxiv
        ):
            ingest_request["link_source"] = "arxiv"
            ingest_request["link_source_id"] = release.ext_ids.arxiv
        return ingest_request


def valid_toml(form: Any, field: Any) -> None:
    try:
        toml.loads(field.data)
    except toml.TomlDecodeError as tpe:
        raise ValidationError(tpe)


class EntityTomlForm(EntityEditForm):

    toml = TextAreaField(
        "TOML",
        [
            validators.DataRequired(),
            valid_toml,
        ],
    )

    @staticmethod
    def from_entity(entity: Any) -> "EntityTomlForm":
        """
        Initializes form with TOML version of existing entity
        """
        etf = EntityTomlForm()
        if entity.state == "active":
            pop_fields = ["ident", "state", "revision", "redirect"]
        else:
            pop_fields = ["ident", "state"]

        # remove "expand" fields
        pop_fields += [
            "releases",
            "container",
            "work",
            "creators",
            "files",
            "filesets",
            "webcaptures",
        ]

        etf.toml.data = entity_to_toml(entity, pop_fields=pop_fields)
        return etf


class ReferenceMatchForm(FlaskForm):
    class Meta:
        # this is an API, so disable CSRF
        csrf = False

    submit_type = SelectField(
        "submit_type", [validators.DataRequired()], choices=["parse", "match"]
    )

    raw_citation = TextAreaField("Citation String", render_kw={"rows": "3"})

    title = StringField("Title")
    journal = StringField("Journal or Conference")
    first_author = StringField("First Author")
    # author_names = StringField("Author Names")
    # year = IntegerField('Year Released',
    #    [validators.Optional(True), valid_year])
    year = StringField("Year Released")
    date = StringField("Date Released")
    volume = StringField("Volume")
    issue = StringField("Issue")
    pages = StringField("Pages")
    publisher = StringField("Publisher")
    doi = StringField("DOI")
    pmid = StringField("PubMed Identifier (PMID)")
    arxiv_id = StringField("arxiv.org Identifier")
    release_type = StringField("Release Type")
    release_stage = StringField("Release Stage")

    @staticmethod
    def from_grobid_parse(
        parse_dict: Dict[str, Any], raw_citation: str
    ) -> "ReferenceMatchForm":
        """
        Initializes form from GROBID extraction
        """
        rmf = ReferenceMatchForm()
        rmf.raw_citation.data = raw_citation

        direct_fields = ["title", "journal", "volume", "issue", "pages"]
        for k in direct_fields:
            if parse_dict.get(k):
                a = getattr(rmf, k)
                a.data = parse_dict[k]

        date = parse_dict.get("date")
        if date and len(date) >= 4 and date[0:4].isdigit():
            rmf.year.data = int(date[0:4])

        if parse_dict.get("authors"):
            rmf.first_author.data = parse_dict["authors"][0].get("name")

        return rmf
