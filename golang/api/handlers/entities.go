
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
    tableName struct{} `sql:"creator_rev"`
    Id          string
    ExtraJson   string
    Name        string
    Orcid       string
}

type CreatorIdent struct {
    tableName struct{} `sql:"creator_ident"`
    Id          string
    IsLive      bool
    RevId       int64
    //Rev         *CreatorRev
    RedirectId  int64
}

func (ci *CreatorIdent) State() string {
    if ci.IsLive && (ci.RedirectId == 0) && (ci.RevId == 0) {
        return "deleted"
    } else if ci.IsLive && (ci.RedirectId != 0) {
        return "redirect"
    } else if ci.IsLive && (ci.RedirectId == 0) && (ci.RevId != 0) {
        return "active"
    } else if !ci.IsLive && (ci.RedirectId == 0) && (ci.RevId != 0) {
        return "wip"
    } else {
        log.Fatalf("Invalid CreatorIdent state: %v", ci)
        panic("fail")
    }
}

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
    db_entity_ident := &CreatorIdent{Id: swag.StringValue(&params.ID)}

    err := d.db.Select(db_entity_ident)
    //err := d.db.Model(db_entity_ident).Select()
    //    Relation("Rev").
    //    Select()
    if err == pg.ErrNoRows {
        return operations.NewGetCreatorIDNotFound().WithPayload(&models.Error{Message: swag.String("no such entity")})
    } else if err != nil {
        log.Fatal(err)
    }
    api_entity := &models.CreatorEntity{
        Ident: &db_entity_ident.Id,
        State: swag.String(db_entity_ident.State()),
        //Name: db_entity_ident.Rev.Name,
        //Orcid: db_entity_ident.Rev.Orcid,
    }
    return operations.NewGetCreatorIDOK().WithPayload(api_entity)
}


func NewPostCreatorHandler(db *pg.DB) operations.PostCreatorHandler {
    return &postCreator{db: db}
}

type postCreator struct {
    db *pg.DB
}

func (d *postCreator) Handle(params operations.PostCreatorParams) middleware.Responder {
    // get-or-create editgroup based on current editor (session)
    // insert new rev, ident, and edit
    return middleware.NotImplemented("operation .PostCreatorID has not yet been implemented")
}
