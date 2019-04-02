
"""
Note: in thoery could use, eg, https://github.com/christabor/swagger_wtforms,
but can't find one that is actually maintained.
"""

from flask_wtf import FlaskForm
from wtforms import SelectField, DateField, StringField, IntegerField, \
    HiddenField, FormField, FieldList, validators

from fatcat_client import ContainerEntity, CreatorEntity, FileEntity, \
    ReleaseEntity, ReleaseContrib

release_type_options = [
    ('', 'Unknown'),
    ('article-journal', 'Journal Article'),
    ('paper-conference', 'Conference Proceeding'),
    ('article', 'Article (non-journal)'),
    ('book', 'Book'),
    ('chapter', 'Book Chapter'),
    ('dataset', 'Dataset'),
    ('stub', 'Invalid/Stub'),
]
release_status_options = [
    ('', 'Unknown'),
    ('draft', 'Draft'),
    ('submitted', 'Submitted'),
    ('accepted', 'Accepted'),
    ('published', 'Published'),
    ('updated', 'Updated'),
]
role_type_options = [
    ('author', 'Author'),
    ('editor', 'Editor'),
    ('translator', 'Translator'),
]

class EntityEditForm(FlaskForm):
    editgroup_id = StringField('Editgroup ID',
        [validators.Optional(True)])
    editgroup_description = StringField('Editgroup Description',
        [validators.Optional(True)])
    edit_description = StringField('Description of Changes',
        [validators.Optional(True)])

class ReleaseContribForm(FlaskForm):
    class Meta:
        # this is a sub-form, so disable CSRF
        csrf = False
    #surname
    #given_name
    #creator_id (?)
    #orcid (for match?)
    prev_index = HiddenField('prev_revision index', default=None)
    raw_name = StringField('Display Name',
        [validators.DataRequired()])
    role = SelectField(
        [validators.DataRequired()],
        choices=role_type_options,
        default='author')

RELEASE_SIMPLE_ATTRS = ['title', 'original_title', 'work_id', 'container_id',
    'release_type', 'release_status', 'release_date', 'doi', 'wikidata_qid',
    'isbn13', 'pmid', 'pmcid', 'volume', 'issue', 'pages', 'publisher',
    'language', 'license_slug']

class ReleaseEntityForm(EntityEditForm):
    """
    TODO:
    - field types: fatcat id
    - date
    """
    title = StringField('Title',
        [validators.InputRequired()])
    original_title = StringField('Original Title')
    work_id = StringField('Work FCID')
    container_id = StringField('Container FCID')
    release_type = SelectField('Release Type',
        [validators.DataRequired()],
        choices=release_type_options,
        default='')
    release_status = SelectField(choices=release_status_options)
    release_date = DateField('Release Date',
        [validators.Optional(True)])
    #release_year
    doi = StringField('DOI',
        [validators.Regexp('^10\..*\/.*', message="DOI must be valid"),
         validators.Optional(True)])
    wikidata_qid = StringField('Wikidata QID')
    isbn13 = StringField('ISBN-13')
    pmid = StringField('PubMed Id')
    pmcid = StringField('PubMed Central Id')
    #core_id
    #arxiv_id
    #jstor_id
    volume = StringField('Volume')
    issue = StringField('Issue')
    pages = StringField('Pages')
    publisher = StringField('Publisher (optional)')
    language = StringField('Language (code)')
    license_slug = StringField('License (slug)')
    contribs = FieldList(FormField(ReleaseContribForm))
    #refs
    #abstracts

    def from_entity(re):
        """
        Initializes form with values from an existing release entity.
        """
        ref = ReleaseEntityForm()
        for simple_attr in RELEASE_SIMPLE_ATTRS:
            a = getattr(ref, simple_attr)
            a.data = getattr(re, simple_attr)
        for i, c in enumerate(re.contribs):
            rcf = ReleaseContribForm()
            rcf.prev_index = i
            rcf.role = c.role
            rcf.raw_name = c.raw_name
            ref.contribs.append_entry(rcf)
        return ref

    def to_entity(self):
        assert(self.title.data)
        entity = ReleaseEntity(title=self.title.data)
        self.update_entity(entity)
        return entity

    def update_entity(self, re):
        """
        Mutates a release entity in place, updating fields with values from
        this form.

        Form must be validated *before* calling this function.
        """
        for simple_attr in RELEASE_SIMPLE_ATTRS:
            a = getattr(self, simple_attr).data
            # special case blank strings
            if a == '':
                a = None
            setattr(re, simple_attr, a)
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
            if c.prev_index.data not in ('', None):
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

container_type_options = (
    ('journal', 'Journal'),
    ('proceedings', 'Conference Proceedings'),
    ('blog', 'Blog'),
)

CONTAINER_SIMPLE_ATTRS = ['name', 'container_type', 'publisher', 'issnl',
    'wikidata_qid']

class ContainerEntityForm(EntityEditForm):
    name = StringField('Name/Title',
        [validators.InputRequired()])
    container_type = SelectField('Container Type',
        [validators.Optional(True)],
        choices=container_type_options,
        default='')
    publisher = StringField("Publisher")
    issnl = StringField("ISSN-L")
    wikidata_qid = StringField('Wikidata QID')

    def from_entity(re):
        """
        Initializes form with values from an existing container entity.
        """
        ref = ContainerEntityForm()
        for simple_attr in CONTAINER_SIMPLE_ATTRS:
            a = getattr(ref, simple_attr)
            a.data = getattr(re, simple_attr)
        return ref

    def to_entity(self):
        assert(self.name.data)
        entity = ContainerEntity(name=self.name.data)
        self.update_entity(entity)
        return entity

    def update_entity(self, ce):
        """
        Mutates a container entity in place, updating fields with values from
        this form.

        Form must be validated *before* calling this function.
        """
        for simple_attr in CONTAINER_SIMPLE_ATTRS:
            a = getattr(self, simple_attr).data
            # special case blank strings
            if a == '':
                a = None
            setattr(ce, simple_attr, a)
        if self.edit_description.data:
            ce.edit_extra = dict(description=self.edit_description.data)

url_rel_options = [
    ('web', 'Public Web'),
    ('webarchive', 'Web Archive'),
    ('repository', 'Repository'),
    ('social', 'Academic Social Network'),
    ('publisher', 'Publisher'),
    ('dweb', 'Decentralized Web'),
]

FILE_SIMPLE_ATTRS = ['size', 'md5', 'sha1', 'sha256', 'mimetype']

class FileUrlForm(FlaskForm):
    class Meta:
        # this is a sub-form, so disable CSRF
        csrf = False

    url = StringField('Display Name',
        [validators.DataRequired()])
    rel = SelectField(
        [validators.DataRequired()],
        choices=url_rel_options,
        default='web')

class FileEntityForm(EntityEditForm):
    size = IntegerField('Size (bytes)',
        [validators.DataRequired()])
        # TODO: positive definite
    md5 = StringField("MD5")
    sha1 = StringField("SHA-1")
    sha256 = StringField("SHA-256")
    urls = FieldList(FormField(FileUrlForm))
    mimetype = StringField("Mimetype")
    release_ids = FieldList(StringField("Release FCID"))

    def from_entity(re):
        """
        Initializes form with values from an existing file entity.
        """
        ref = FileEntityForm()
        for simple_attr in FILE_SIMPLE_ATTRS:
            a = getattr(ref, simple_attr)
            a.data = getattr(re, simple_attr)
        return ref

    def to_entity(self):
        assert(self.name.data)
        entity = FileEntity()
        self.update_entity(entity)
        return entity

    def update_entity(self, fe):
        """
        Mutates in place, updating fields with values from this form.

        Form must be validated *before* calling this function.
        """
        for simple_attr in FILE_SIMPLE_ATTRS:
            a = getattr(self, simple_attr).data
            # special case blank strings
            if a == '':
                a = None
            setattr(fe, simple_attr, a)
        re.urls = []
        for u in self.urls:
            re.contribs.append(FileUrl(
                rel=u.role.data or None,
                url=u.url.data or None,
            ))
        re.release_ids = []
        for ri in self.release_ids:
            re.release_ids.append(ri.data)
        if self.edit_description.data:
            fe.edit_extra = dict(description=self.edit_description.data)

