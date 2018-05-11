
package handlers

import (
    "github.com/go-pg/pg"
    //"github.com/go-openapi/swag"
    "github.com/go-openapi/runtime/middleware"
    //log "github.com/sirupsen/logrus"

	//"git.archive.org/bnewbold/fatcat/golang/gen/models"
	"git.archive.org/bnewbold/fatcat/golang/gen/restapi/operations"
)

type Editor struct {
    Id          int64
    Username    string
    IsAdmin     bool
    ActiveEditgroupId   int64
    ActiveEditgroup     *Editgroup
}

type Editgroup struct {
    Id              int64
    ExtraJson       string
    EditorId        int64
    Editor          *Editor
    Description     string
}

type Changelog struct {
    Id              int64
    EditgroupId     int64
    Editgroup       *Editgroup
    Timestamp       string // XXX: timestamp
}

func GetOrCreateEditgroup() Editgroup {
    // XXX
    eg := Editgroup{}
    return eg
}

////
func NewGetEditorUsernameHandler(db *pg.DB) operations.GetEditorUsernameHandler {
    return &getEditorUsername{db: db}
}
type getEditorUsername struct {
    db *pg.DB
}
func (d *getEditorUsername) Handle(params operations.GetEditorUsernameParams) middleware.Responder {
    return middleware.NotImplemented("operation .PostCreatorID has not yet been implemented")
}

////
func NewGetEditorUsernameChangelogHandler(db *pg.DB) operations.GetEditorUsernameChangelogHandler {
    return &getEditorUsernameChangelog{db: db}
}
type getEditorUsernameChangelog struct {
    db *pg.DB
}
func (d *getEditorUsernameChangelog) Handle(params operations.GetEditorUsernameChangelogParams) middleware.Responder {
    return middleware.NotImplemented("operation .PostCreatorID has not yet been implemented")
}

////
func NewGetEditgroupIDHandler(db *pg.DB) operations.GetEditgroupIDHandler {
    return &getEditgroupID{db: db}
}
type getEditgroupID struct {
    db *pg.DB
}
func (d *getEditgroupID) Handle(params operations.GetEditgroupIDParams) middleware.Responder {
    return middleware.NotImplemented("operation .PostCreatorID has not yet been implemented")
}

////
func NewPostEditgroupHandler(db *pg.DB) operations.PostEditgroupHandler {
    return &postEditgroup{db: db}
}
type postEditgroup struct {
    db *pg.DB
}
func (d *postEditgroup) Handle(params operations.PostEditgroupParams) middleware.Responder {
    return middleware.NotImplemented("operation .PostEditgroupID has not yet been implemented")
}

////
func NewPostEditgroupIDAcceptHandler(db *pg.DB) operations.PostEditgroupIDAcceptHandler {
    return &postEditgroupIDAccept{db: db}
}
type postEditgroupIDAccept struct {
    db *pg.DB
}
func (d *postEditgroupIDAccept) Handle(params operations.PostEditgroupIDAcceptParams) middleware.Responder {
    return middleware.NotImplemented("operation .PostEditgroupIDAcceptID has not yet been implemented")
}
