
import enum
from fatcat import db


# TODO: EntityMixin, EntityIdMixin

work_contrib = db.Table("work_contrib",
    db.Column("work_rev", db.ForeignKey('work_revision.id'), nullable=False, primary_key=True),
    db.Column("creator_id", db.ForeignKey('creator_id.id'), nullable=False, primary_key=True),
    db.Column("stub", db.String, nullable=True))

release_contrib = db.Table("release_contrib",
    db.Column("release_rev", db.ForeignKey('release_revision.id'), nullable=False, primary_key=True),
    db.Column("creator_id", db.ForeignKey('creator_id.id'), nullable=False, primary_key=True),
    db.Column("stub", db.String, nullable=True))

class WorkId(db.Model):
    __tablename__ = 'work_id'
    id = db.Column(db.Integer, primary_key=True)
    revision_id = db.Column(db.ForeignKey('work_revision.id'))

class WorkRevision(db.Model):
    __tablename__ = 'work_revision'
    id = db.Column(db.Integer, primary_key=True)
    previous = db.Column(db.ForeignKey('work_revision.id'), nullable=True)
    state = db.Column(db.String)
    redirect_id = db.Column(db.ForeignKey('work_id.id'), nullable=True)
    edit_id = db.Column(db.ForeignKey('edit.id'))
    extra_json = db.Column(db.ForeignKey('extra_json.sha1'), nullable=True)
    #work_ids = db.relationship("WorkId", backref="revision", lazy=True)

    title = db.Column(db.String)
    work_type = db.Column(db.String)
    date = db.Column(db.String)

    creators = db.relationship('CreatorId', secondary=work_contrib,
        lazy='subquery', backref=db.backref('works', lazy=True))

class ReleaseId(db.Model):
    __tablename__ = 'release_id'
    id = db.Column(db.Integer, primary_key=True) #autoincrement=False
    revision_id = db.Column(db.ForeignKey('release_revision.id'))

class ReleaseRevision(db.Model):
    __tablename__ = 'release_revision'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    previous = db.Column(db.ForeignKey('release_revision.id'), nullable=True)
    state = db.Column(db.String)              # TODO: enum
    redirect_id = db.Column(db.ForeignKey('release_id.id'), nullable=True)
    edit_id = db.Column(db.ForeignKey('edit.id'))
    extra_json = db.Column(db.ForeignKey('extra_json.sha1'), nullable=True)
    #release_ids = db.relationship("ReleaseId", backref="revision", lazy=False)

    work_id = db.ForeignKey('work_id.id')
    container = db.Column(db.ForeignKey('container_id.id'))
    title = db.Column(db.String)
    license = db.Column(db.String)          # TODO: oa status foreign key
    release_type = db.Column(db.String)     # TODO: foreign key
    date = db.Column(db.String)             # TODO: datetime
    doi = db.Column(db.String)              # TODO: identifier table

    creators = db.relationship('CreatorId', secondary=release_contrib,
        lazy='subquery')
        #backref=db.backref('releases', lazy=True))

class CreatorId(db.Model):
    __tablename__ = 'creator_id'
    id = db.Column(db.Integer, primary_key=True) #autoincrement=False)
    revision_id = db.Column(db.ForeignKey('creator_revision.id'))

class CreatorRevision(db.Model):
    __tablename__ = 'creator_revision'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    previous = db.Column(db.ForeignKey('creator_revision.id'), nullable=True)
    state = db.Column(db.String)
    redirect_id = db.Column(db.ForeignKey('creator_id.id'), nullable=True)
    edit_id = db.Column(db.ForeignKey('edit.id'))
    extra_json = db.Column(db.ForeignKey('extra_json.sha1'), nullable=True)
    #creator_ids = db.relationship("CreatorId", backref="revision", lazy=False)

    name = db.Column(db.String)
    sortname = db.Column(db.String)
    orcid = db.Column(db.String)            # TODO: identifier table

class ContainerId(db.Model):
    __tablename__ = 'container_id'
    id = db.Column(db.Integer, primary_key=True) #autoincrement=False)
    revision_id = db.Column(db.ForeignKey('container_revision.id'))

class ContainerRevision(db.Model):
    __tablename__ = 'container_revision'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    previous = db.Column(db.ForeignKey('container_revision.id'), nullable=True)
    state = db.Column(db.String)
    redirect_id = db.Column(db.ForeignKey('container_id.id'), nullable=True)
    edit_id = db.Column(db.ForeignKey('edit.id'))
    extra_json = db.Column(db.ForeignKey('extra_json.sha1'), nullable=True)

    name = db.Column(db.String)
    container = db.Column(db.ForeignKey('container_id.id'))
    publisher = db.Column(db.String)        # TODO: foreign key
    sortname = db.Column(db.String)
    issn = db.Column(db.String)             # TODO: identifier table

class FileId(db.Model):
    __tablename__ = 'file_id'
    id = db.Column(db.Integer, primary_key=True, autoincrement=False)
    revision_id = db.Column('revision', db.ForeignKey('container_revision.id'))

class FileRevision(db.Model):
    __tablename__ = 'file_revision'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    previous = db.Column(db.ForeignKey('file_revision.id'), nullable=True)
    state = db.Column(db.String)
    redirect_id = db.Column(db.ForeignKey('file_id.id'), nullable=True)
    edit_id = db.Column(db.ForeignKey('edit.id'))
    extra_json = db.Column(db.ForeignKey('extra_json.sha1'), nullable=True)

    size = db.Column(db.Integer)
    sha1 = db.Column(db.Integer)            # TODO: hash table... only or in addition?
    url = db.Column(db.Integer)             # TODO: URL table

class ReleaseFil(db.Model):
    __tablename__ = 'release_file'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    release_rev = db.Column(db.ForeignKey('release_revision.id'), nullable=False)
    file_id = db.Column(db.ForeignKey('file_id.id'), nullable=False)

class Edit(db.Model):
    __tablename__ = 'edit'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    edit_group = db.Column(db.ForeignKey('edit_group.id'))
    editor = db.Column(db.ForeignKey('editor.id'))
    description = db.Column(db.String)

class EditGroup(db.Model):
    __tablename__ = 'edit_group'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    editor = db.Column(db.ForeignKey('editor.id'))
    description = db.Column(db.String)

class Editor(db.Model):
    __tablename__ = 'editor'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    username = db.Column(db.String)

class ChangelogEntry(db.Model):
    __tablename__= 'changelog'
    id = db.Column(db.Integer, primary_key=True, autoincrement=True)
    edit_id = db.Column(db.ForeignKey('edit.id'))
    timestamp = db.Column(db.Integer)

class ExtraJson(db.Model):
    __tablename__ = 'extra_json'
    sha1 = db.Column(db.String, primary_key=True)
    json = db.Column(db.String)

