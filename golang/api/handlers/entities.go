
package handlers

import (
    "github.com/go-pg/pg"
    "github.com/go-openapi/swag"
    "github.com/go-openapi/runtime/middleware"
    log "github.com/sirupsen/logrus"

	"git.archive.org/bnewbold/fatcat/golang/gen/models"
	"git.archive.org/bnewbold/fatcat/golang/gen/restapi/operations"
)

type CreatorRev struct {
    Id          int64
    ExtraJson   string
    Name        string
    Orcid       string
    tableName struct{} `sql:"creator_rev"`
}

type CreatorIdent struct {
    Id              string
    IsLive          bool
    RevId           int64
    Rev             *CreatorRev
    RedirectId      string
    Redirect        *CreatorIdent
    tableName       struct{} `sql:"creator_ident"`
}

type CreatorEdit struct {
    Id              int64
    ExtraJson       string
    IdentId         string
    Ident           *CreatorIdent
    RevId           int64
    Rev             *CreatorRev
    RedirectId      string
    Redirect        *CreatorIdent
    EditgroupId     int64
    Editgroup       *Editgroup
    tableName       struct{} `sql:"creator_edit"`
}

func (ci *CreatorIdent) State() string {
    if ci.IsLive && (ci.RedirectId == "") && (ci.RevId == 0) {
        return "deleted"
    } else if ci.IsLive && (ci.RedirectId != "") {
        return "redirect"
    } else if ci.IsLive && (ci.RedirectId == "") && (ci.RevId != 0) {
        return "active"
    } else if !ci.IsLive && (ci.RedirectId == "") && (ci.RevId != 0) {
        return "wip"
    } else {
        log.Fatalf("Invalid CreatorIdent state: %v", ci)
        panic("fail")
    }
}

////
func NewGetCreatorIDHandler(db *pg.DB) operations.GetCreatorIDHandler {
    return &getCreatorID{db: db}
}
type getCreatorID struct {
    db *pg.DB
}
func (d *getCreatorID) Handle(params operations.GetCreatorIDParams) middleware.Responder {
    // "get or 404" using params.ID. join creator_ident and creator_rev.
    // populate result data
    // return that
    db_entity_ident := &CreatorIdent{}
    err := d.db.Model(db_entity_ident).
        Column("creator_ident.*", "Rev").
        Where("creator_ident.id = ?", swag.StringValue(&params.ID)).
        First()
    if err == pg.ErrNoRows {
        return operations.NewGetCreatorIDNotFound().WithPayload(&models.Error{Message: swag.String("no such entity")})
    } else if err != nil {
        log.Fatal(err)
    }
    api_entity := &models.CreatorEntity{
        Ident: db_entity_ident.Id,
        State: db_entity_ident.State(),
        Name: db_entity_ident.Rev.Name,
        Orcid: db_entity_ident.Rev.Orcid,
    }
    return operations.NewGetCreatorIDOK().WithPayload(api_entity)
}

////
func NewGetCreatorLookupHandler(db *pg.DB) operations.GetCreatorLookupHandler {
    return &getCreatorLookup{db: db}
}
type getCreatorLookup struct {
    db *pg.DB
}
func (d *getCreatorLookup) Handle(params operations.GetCreatorLookupParams) middleware.Responder {
    // get-or-create editgroup based on current editor (session)
    // insert new rev, ident, and edit
    return middleware.NotImplemented("operation .GetCreatorLookup has not yet been implemented")
}

////
func NewPostCreatorHandler(db *pg.DB) operations.PostCreatorHandler {
    return &postCreator{db: db}
}
type postCreator struct {
    db *pg.DB
}
func (d *postCreator) Handle(params operations.PostCreatorParams) middleware.Responder {
    // get-or-create editgroup based on current editor (session)
    var eg Editgroup
    if true {
        eg = Editgroup{Id: 1}
    } else {
        eg = GetOrCreateEditgroup()
    }
    ce := CreatorEdit{}

    // big honking SQL to update 3 tables in a single INSERT
    _, err := d.db.QueryOne(
    //Model(ce).ExecOne(
        &ce,
        `WITH rev AS ( INSERT INTO creator_rev (name, orcid)
                       VALUES (?, ?)
                       RETURNING id ),
              ident AS ( INSERT INTO creator_ident (rev_id)
                         VALUES ((SELECT rev.id FROM rev))
                         RETURNING id )
        INSERT INTO creator_edit (editgroup_id, ident_id, rev_id) VALUES
            (?, (SELECT ident.id FROM ident), (SELECT rev.id FROM rev))
        RETURNING *`,
        params.Body.Name,
        params.Body.Orcid,
        eg.Id)
    if err != nil {
        log.Fatal(err)
    }
    if err != nil {
        log.Fatal(err)
    }
    // redirect? or return the edit?
    api_edit:= models.EntityEdit {
        ID: ce.Id,
        Ident: ce.IdentId,
        Revision: ce.RevId,
        EditgroupID: ce.EditgroupId,
    }
    return operations.NewPostCreatorCreated().WithPayload(&api_edit)
}
