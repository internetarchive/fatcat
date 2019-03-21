
"""
Note: in thoery could use, eg, https://github.com/christabor/swagger_wtforms,
but can't find one that is actually maintained.
"""

from flask_wtf import FlaskForm
from wtforms import SelectField, DateField, StringField, FormField, FieldList, validators

release_type_options = [
    ('article-journal', 'Journal Article'),
    ('paper-conference', 'Conference Proceeding'),
    ('article', 'Article (non-journal)'),
    ('book', 'Book'),
    ('chapter', 'Book Chapter'),
    ('dataset', 'Dataset'),
    ('stub', 'Invalid/Stub'),
]
release_status_options = [
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
        [validators.DataRequired()])
    editgroup_description = StringField('Editgroup Description',
        [validators.Optional(True)])
    edit_description = StringField('Description of Changes',
        [validators.Optional(True)])

class ReleaseContribForm(FlaskForm):
    #surname
    #given_name
    #creator_id (?)
    #orcid (for match?)
    raw_name = StringField('Display Name')
    role = SelectField(choices=role_type_options)

class ReleaseEntityForm(EntityEditForm):
    """
    TODO:
    - field types: fatcat id
    - date
    """
    title = StringField('Title', [validators.InputRequired()])
    original_title = StringField('Original Title')
    work_id = StringField('Work FCID')
    container_id = StringField('Container FCID')
    release_type = SelectField(choices=release_type_options)
    release_status = SelectField(choices=release_status_options)
    release_date = DateField('Release Date',
        [validators.Optional(True)])
    #release_year
    doi = StringField('DOI',
        [validators.Regexp('^10\..*\/.*', message="DOI must be valid")])
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

