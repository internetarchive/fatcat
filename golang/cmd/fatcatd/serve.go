
package main

import (
	"net/http"

    "github.com/spf13/viper"
    log "github.com/sirupsen/logrus"
	loads "github.com/go-openapi/loads"
    "github.com/carbocation/interpose"
    "github.com/carbocation/interpose/adaptors"
    "github.com/meatballhat/negroni-logrus"
    "github.com/bradleyg/go-sentroni"
    "github.com/go-pg/pg"
    "github.com/spf13/cobra"

	"git.archive.org/bnewbold/fatcat/golang/api/handlers"
	"git.archive.org/bnewbold/fatcat/golang/gen/restapi"
	"git.archive.org/bnewbold/fatcat/golang/gen/restapi/operations"
)

var serveCmd = &cobra.Command{
    Use:    "serve [options]",
    Short:  "Run fatcat REST API server",
    Run:    func(cmd *cobra.Command, args[] string) {
        main_serve()
    },
}

func main_serve() {

    // load embedded swagger file
    swaggerSpec, err := loads.Analyzed(restapi.SwaggerJSON, "")
    if err != nil {
        log.Fatalln(err)
    }

    // create new service API
	api := operations.NewFatcatAPI(swaggerSpec)

	// Set your custom logger if needed. Default one is log.Printf
	// Expected interface func(string, ...interface{})
    api.Logger = log.Printf

	server := restapi.NewServer(api)
	defer server.Shutdown()

    server.Port = viper.GetInt("port")

    db_options, err := pg.ParseURL(viper.GetString("db_url"))
    if err != nil {
        log.Panicf("parsing DB string: %v", err)
    }
    db := pg.Connect(db_options)
    defer db.Close()

    // register all the many handlers here
	api.GetCreatorIDHandler = handlers.NewGetCreatorIDHandler(db);
    api.PostCreatorHandler = handlers.NewPostCreatorHandler(db);
    api.GetCreatorLookupHandler = handlers.NewGetCreatorLookupHandler(db);
/*
	api.GetCreatorLookupHandler = operations.GetCreatorLookupHandlerFunc(func(params operations.GetCreatorLookupParams) middleware.Responder {
		return middleware.NotImplemented("operation .GetCreatorLookup has not yet been implemented")
	})
*/

    api.GetEditgroupIDHandler = handlers.NewGetEditgroupIDHandler(db);
    api.GetEditorUsernameHandler = handlers.NewGetEditorUsernameHandler(db);
    api.GetEditorUsernameChangelogHandler = handlers.NewGetEditorUsernameChangelogHandler(db);
    api.PostEditgroupHandler = handlers.NewPostEditgroupHandler(db);
    api.PostEditgroupIDAcceptHandler = handlers.NewPostEditgroupIDAcceptHandler(db);
/*
	api.GetEditgroupIDHandler = operations.GetEditgroupIDHandlerFunc(func(params operations.GetEditgroupIDParams) middleware.Responder {
		return middleware.NotImplemented("operation .GetEditgroupID has not yet been implemented")
	})
	api.GetEditorUsernameHandler = operations.GetEditorUsernameHandlerFunc(func(params operations.GetEditorUsernameParams) middleware.Responder {
		return middleware.NotImplemented("operation .GetEditorUsername has not yet been implemented")
	})
	api.GetEditorUsernameChangelogHandler = operations.GetEditorUsernameChangelogHandlerFunc(func(params operations.GetEditorUsernameChangelogParams) middleware.Responder {
		return middleware.NotImplemented("operation .GetEditorUsernameChangelog has not yet been implemented")
	})
	api.PostEditgroupHandler = operations.PostEditgroupHandlerFunc(func(params operations.PostEditgroupParams) middleware.Responder {
		return middleware.NotImplemented("operation .PostEditgroup has not yet been implemented")
	})
	api.PostEditgroupIDAcceptHandler = operations.PostEditgroupIDAcceptHandlerFunc(func(params operations.PostEditgroupIDAcceptParams) middleware.Responder {
		return middleware.NotImplemented("operation .PostEditgroupIDAccept has not yet been implemented")
	})
*/

    middle := interpose.New()

    // sentry
    middle.Use(adaptors.FromNegroni(sentroni.NewRecovery(viper.GetString("sentry_dsn"))))

    // logging
    middle.Use(adaptors.FromNegroni(negronilogrus.NewMiddleware()))

    // add clacks
    middle.UseHandler(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
        rw.Header().Set("X-Clacks-Overhead:", "GNU Aaron Swartz, John Perry Barlow")
    }))

    // actual handler
    middle.UseHandler(api.Serve(nil))
    server.SetHandler(middle)

	if err := server.Serve(); err != nil {
		log.Fatalln(err)
	}

}
