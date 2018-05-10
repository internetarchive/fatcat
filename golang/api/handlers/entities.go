
package handlers

import (
    "github.com/go-pg/pg"
    "github.com/go-openapi/runtime/middleware"

	//"git.archive.org/bnewbold/fatcat/golang/gen/models"
	"git.archive.org/bnewbold/fatcat/golang/gen/restapi/operations"
)

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
    return middleware.NotImplemented("operation .GetCreatorID has not yet been implemented. Coming soon!")
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
