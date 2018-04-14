
"""
states for identifiers:
- pre-live: points to a rev (during edit/accept period)
- live: points to a rev
- redirect: live, points to upstream rev, also points to redirect id
    => if live and redirect non-null, all other fields copied from redirect target
- deleted: live, but doesn't point to a rev

possible refactors:
- '_rev' instead of '_rev'
- use mixins for entities
"""

from marshmallow import post_dump
from fatcat import db, ma


### Inter-Entity Relationships ###############################################

class ReleaseContrib(db.Model):
    __tablename__ = "release_contrib"
    release_rev = db.Column(db.ForeignKey('release_rev.id'), nullable=False, primary_key=True)
    creator_ident_id = db.Column(db.ForeignKey('creator_ident.id'), nullable=False, primary_key=True)
    type = db.Column(db.String, nullable=True)
    stub = db.Column(db.String, nullable=True)

    creator = db.relationship("CreatorIdent")
    release = db.relationship("ReleaseRev")

class ReleaseRef(db.Model):
    __tablename__ = "release_ref"
    id = db.Column(db.Integer, primary_key=True, nullable=False)
    release_rev = db.Column(db.ForeignKey('release_rev.id'), nullable=False)
    target_release_ident_id = db.Column(db.ForeignKey('release_ident.id'), nullable=True)
    index = db.Column(db.Integer, nullable=True)
    stub = db.Column(db.String, nullable=True)
    doi = db.Column(db.String, nullable=True)

    release = db.relationship("ReleaseRev")
    target = db.relationship("ReleaseIdent")

class FileRelease(db.Model):
    __tablename__ = "file_release"
    id = db.Column(db.Integer, primary_key=True, nullable=False)
    file_rev= db.Column(db.ForeignKey('file_rev.id'), nullable=False)
    release_ident_id = db.Column(db.ForeignKey('release_ident.id'), nullable=False)

    release = db.relationship("ReleaseIdent")
    file = db.relationship("FileRev")


### Entities #################################################################

class WorkRev(db.Model):
    __tablename__ = 'work_rev'
    id = db.Column(db.Integer, primary_key=True)
    extra_json_id = db.Column(db.ForeignKey('extra_json.sha1'), nullable=True)
    extra_json = db.relationship("ExtraJson") # XXX: for all entities

    title = db.Column(db.String)
    work_type = db.Column(db.String)
    primary_release_id = db.Column(db.ForeignKey('release_ident.id'), nullable=True)
    primary_release = db.relationship('ReleaseIdent')

class WorkIdent(db.Model):
    """
    If rev_id is null, this was deleted.
    If redirect_id is not null, this has been merged with the given id. In this
        case rev_id is a "cached" copy of the redirect's rev_id, as
        an optimization. If the merged work is "deleted", rev_id can be
        null and redirect_id not-null.
    """
    __tablename__ = 'work_ident'
    id = db.Column(db.Integer, primary_key=True, nullable=False)
    is_live = db.Column(db.Boolean, nullable=False, default=False)
    rev_id = db.Column(db.ForeignKey('work_rev.id'), nullable=True)
    redirect_id = db.Column(db.ForeignKey('work_ident.id'), nullable=True)
    rev = db.relationship("WorkRev")

class WorkEdit(db.Model):
    __tablename__ = 'work_edit'
    id = db.Column(db.Integer, primary_key=True)
    ident_id = db.Column(db.ForeignKey('work_ident.id'), nullable=True)
    rev_id = db.Column(db.ForeignKey('work_rev.id'), nullable=True)
    redirect_id = db.Column(db.ForeignKey('work_ident.id'), nullable=True)
    edit_group_id = db.Column(db.ForeignKey('edit_group.id'), nullable=True)
    extra_json = db.Column(db.ForeignKey('extra_json.sha1'), nullable=True)
    ident = db.relationship("WorkIdent", foreign_keys="WorkEdit.ident_id") # XXX: add to all other entities
    rev = db.relationship("WorkRev")
    edit_group = db.relationship("EditGroup")


class ReleaseRev(db.Model):
    __tablename__ = 'release_rev'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    extra_json = db.Column(db.ForeignKey('extra_json.sha1'), nullable=True)

    work_ident_id = db.ForeignKey('work_ident.id')
    container_ident_id = db.Column(db.ForeignKey('container_ident.id'), nullable=True)
    title = db.Column(db.String, nullable=False)
    license = db.Column(db.String, nullable=True)   # TODO: oa status foreign key
    release_type = db.Column(db.String)             # TODO: foreign key
    date = db.Column(db.String, nullable=True)      # TODO: datetime
    doi = db.Column(db.String, nullable=True)       # TODO: identifier table
    volume = db.Column(db.String, nullable=True)
    pages = db.Column(db.String, nullable=True)
    issue = db.Column(db.String, nullable=True)

    #work = db.relationship("WorkIdent", lazy='subquery')
    container = db.relationship("ContainerIdent", lazy='subquery')
    creators = db.relationship('ReleaseContrib', lazy='subquery')
    refs = db.relationship('ReleaseRef', lazy='subquery')

class ReleaseIdent(db.Model):
    __tablename__ = 'release_ident'
    id = db.Column(db.Integer, primary_key=True)
    is_live = db.Column(db.Boolean, nullable=False, default=False)
    rev_id = db.Column(db.ForeignKey('release_rev.id'))
    redirect_id = db.Column(db.ForeignKey('release_ident.id'), nullable=True)
    rev = db.relationship("ReleaseRev")

class ReleaseEdit(db.Model):
    __tablename__ = 'release_edit'
    id = db.Column(db.Integer, primary_key=True)
    ident_id = db.Column(db.ForeignKey('release_ident.id'), nullable=True)
    rev_id = db.Column(db.ForeignKey('release_rev.id'), nullable=True)
    redirect_id = db.Column(db.ForeignKey('release_ident.id'), nullable=True)
    edit_group_id = db.Column(db.ForeignKey('edit_group.id'), nullable=True)
    extra_json = db.Column(db.ForeignKey('extra_json.sha1'), nullable=True)


class CreatorRev(db.Model):
    __tablename__ = 'creator_rev'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    extra_json = db.Column(db.ForeignKey('extra_json.sha1'), nullable=True)

    name = db.Column(db.String)
    sortname = db.Column(db.String)
    orcid = db.Column(db.String)            # TODO: identifier table

class CreatorIdent(db.Model):
    __tablename__ = 'creator_ident'
    id = db.Column(db.Integer, primary_key=True)
    is_live = db.Column(db.Boolean, nullable=False, default=False)
    rev_id = db.Column(db.ForeignKey('creator_rev.id'))
    redirect_id = db.Column(db.ForeignKey('creator_ident.id'), nullable=True)
    rev = db.relationship("CreatorRev")

class CreatorEdit(db.Model):
    __tablename__ = 'creator_edit'
    id = db.Column(db.Integer, primary_key=True)
    ident_id = db.Column(db.ForeignKey('creator_ident.id'), nullable=True)
    rev_id = db.Column(db.ForeignKey('creator_rev.id'), nullable=True)
    redirect_id = db.Column(db.ForeignKey('creator_ident.id'), nullable=True)
    edit_group_id = db.Column(db.ForeignKey('edit_group.id'), nullable=True)
    extra_json = db.Column(db.ForeignKey('extra_json.sha1'), nullable=True)


class ContainerRev(db.Model):
    __tablename__ = 'container_rev'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    extra_json = db.Column(db.ForeignKey('extra_json.sha1'), nullable=True)

    name = db.Column(db.String)
    parent_id = db.Column(db.ForeignKey('container_ident.id', use_alter=True))
    publisher = db.Column(db.String)        # TODO: foreign key
    sortname = db.Column(db.String)
    issn = db.Column(db.String)             # TODO: identifier table
    parent = db.relationship("ContainerIdent", foreign_keys="ContainerRev.parent_id")

class ContainerIdent(db.Model):
    __tablename__ = 'container_ident'
    id = db.Column(db.Integer, primary_key=True)
    is_live = db.Column(db.Boolean, nullable=False, default=False)
    rev_id = db.Column(db.ForeignKey('container_rev.id'))
    redirect_id = db.Column(db.ForeignKey('container_ident.id'), nullable=True)
    rev = db.relationship("ContainerRev", foreign_keys="ContainerIdent.rev_id")

class ContainerEdit(db.Model):
    __tablename__ = 'container_edit'
    id = db.Column(db.Integer, primary_key=True)
    ident_id = db.Column(db.ForeignKey('container_ident.id'), nullable=True)
    rev_id = db.Column(db.ForeignKey('container_rev.id'), nullable=True)
    redirect_id = db.Column(db.ForeignKey('container_ident.id'), nullable=True)
    edit_group_id = db.Column(db.ForeignKey('edit_group.id'), nullable=True)
    extra_json = db.Column(db.ForeignKey('extra_json.sha1'), nullable=True)


class FileRev(db.Model):
    __tablename__ = 'file_rev'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    extra_json = db.Column(db.ForeignKey('extra_json.sha1'), nullable=True)

    size = db.Column(db.Integer)
    sha1 = db.Column(db.String)             # TODO: hash table... only or in addition?
    url = db.Column(db.Integer)             # TODO: URL table
    releases = db.relationship('FileRelease', lazy='subquery')

class FileIdent(db.Model):
    __tablename__ = 'file_ident'
    id = db.Column(db.Integer, primary_key=True)
    is_live = db.Column(db.Boolean, nullable=False, default=False)
    rev_id = db.Column(db.ForeignKey('file_rev.id'))
    redirect_id = db.Column(db.ForeignKey('file_ident.id'), nullable=True)
    rev = db.relationship("FileRev")

class FileEdit(db.Model):
    __tablename__ = 'file_edit'
    id = db.Column(db.Integer, primary_key=True)
    ident_id = db.Column(db.ForeignKey('file_ident.id'), nullable=True)
    rev_id = db.Column(db.ForeignKey('file_rev.id'), nullable=True)
    redirect_id = db.Column(db.ForeignKey('file_ident.id'), nullable=True)
    edit_group_id = db.Column(db.ForeignKey('edit_group.id'), nullable=True)
    extra_json = db.Column(db.ForeignKey('extra_json.sha1'), nullable=True)


### Editing #################################################################

class EditGroup(db.Model):
    __tablename__ = 'edit_group'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    editor_id = db.Column(db.ForeignKey('editor.id'), nullable=False)
    description = db.Column(db.String)
    editor = db.relationship('Editor', foreign_keys='EditGroup.editor_id')

class Editor(db.Model):
    __tablename__ = 'editor'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    username = db.Column(db.String)
    is_admin = db.Column(db.Boolean, nullable=False, default=False)
    active_edit_group_id = db.Column(db.ForeignKey('edit_group.id', use_alter=True))
    active_edit_group = db.relationship('EditGroup', foreign_keys='Editor.active_edit_group_id')

class ChangelogEntry(db.Model):
    __tablename__= 'changelog'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    edit_group_id = db.Column(db.ForeignKey('edit_group.id'))
    timestamp = db.Column(db.Integer)


### Other ###################################################################

class ExtraJson(db.Model):
    __tablename__ = 'extra_json'
    sha1 = db.Column(db.String, primary_key=True, nullable=False)
    json = db.Column(db.String, nullable=False)


### Marshmallow Wrappers ####################################################

class ExtraJsonSchema(ma.ModelSchema):

    class Meta:
        model = ExtraJson

    @post_dump(pass_many=False)
    def unflatten(self, data):
        assert(hashlib.sha1sum(data['json']).hexdigest() == data['sha1'])
        data.pop('sha1')
        raw = data.pop('json')
        data.update(json.loads(raw))

    @post_dump(pass_many=False)
    def flatten(self, data):
        raw = json.dumps(data, indent=None)
        for k in list(data.keys()):
            data.pop(k)
        data['sha1'] = hashlib.sha1sum(raw).hexdigest()
        data['json'] = raw

class EntitySchema(ma.ModelSchema):

    @post_dump(pass_many=False)
    def merge_rev(self, data):
        if data.get('rev', None) != None:
            rev_id = data['rev'].pop('id')
            data.update(data['rev'])
            data['rev'] = rev_id
        else:
            data['rev'] = None
        # TODO: should be able to set as an allow_none field somewhere
        if not data.get('redirect_id', None):
            data['redirect_id'] = None

    extra_json = ma.Nested(ExtraJsonSchema)

class ReleaseContribSchema(ma.ModelSchema):
    class Meta:
        model = ReleaseContrib
    #creator = db.relationship("CreatorIdent")
    #release = db.relationship("ReleaseRev")

class ReleaseRefSchema(ma.ModelSchema):
    class Meta:
        model = ReleaseRef
    #release = db.relationship("ReleaseRev")
    #target = db.relationship("ReleaseIdent")

class FileReleaseSchema(ma.ModelSchema):
    class Meta:
        model = FileRelease
    #release = db.relationship("ReleaseIdent")
    #file = db.relationship("FileRev")

class WorkRevSchema(ma.ModelSchema):
    class Meta:
        model = WorkRev

class WorkSchema(EntitySchema):
    class Meta:
        model = WorkIdent
    rev = ma.Nested(WorkRevSchema)

class WorkEditSchema(ma.ModelSchema):
    class Meta:
        model = WorkEdit

work_schema = WorkSchema()
work_edit_schema = WorkEditSchema()


class ReleaseRevSchema(ma.ModelSchema):
    class Meta:
        model = ReleaseRev
    container = ma.Nested('ContainerSchema')
    creators = ma.Nested(ReleaseContribSchema, many=True)
    refs = ma.Nested(ReleaseRefSchema, many=True)

class ReleaseSchema(EntitySchema):
    class Meta:
        model = ReleaseIdent
    rev = ma.Nested(ReleaseRevSchema)
    # XXX: files = ma.Nested('FileSchema', many=True)

class ReleaseEditSchema(ma.ModelSchema):
    class Meta:
        model = ReleaseEdit

release_rev_schema = ReleaseRevSchema()
release_schema = ReleaseSchema()
release_edit_schema = ReleaseEditSchema()


class CreatorRevSchema(ma.ModelSchema):
    class Meta:
        model = CreatorRev

class CreatorSchema(EntitySchema):
    class Meta:
        model = CreatorIdent
    rev = ma.Nested(CreatorRevSchema)

class CreatorEditSchema(ma.ModelSchema):
    class Meta:
        model = CreatorEdit

creator_rev_schema = CreatorRevSchema()
creator_schema = CreatorSchema()
creator_edit_schema = CreatorEditSchema()


class ContainerRevSchema(ma.ModelSchema):
    class Meta:
        model = ContainerRev

class ContainerSchema(EntitySchema):
    class Meta:
        model = ContainerIdent
    rev = ma.Nested(ContainerRevSchema)

class ContainerEditSchema(ma.ModelSchema):
    class Meta:
        model = ContainerEdit

container_rev_schema = ContainerRevSchema()
container_schema = ContainerSchema()
container_edit_schema = ContainerEditSchema()


class FileRevSchema(ma.ModelSchema):
    class Meta:
        model = FileRev

class FileSchema(EntitySchema):
    class Meta:
        model = FileIdent
    rev = ma.Nested(FileRevSchema)

class FileEditSchema(ma.ModelSchema):
    class Meta:
        model = FileEdit

file_rev_schema = FileRevSchema()
file_schema = FileSchema()
file_edit_schema = FileEditSchema()


class EditorSchema(ma.ModelSchema):
    class Meta:
        model = Editor

class EditGroupSchema(ma.ModelSchema):
    class Meta:
        model = EditGroup

editor_schema = EditGroupSchema()
edit_group_schema = EditGroupSchema()
