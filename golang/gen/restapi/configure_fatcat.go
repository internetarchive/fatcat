// This file is safe to edit. Once it exists it will not be overwritten

package restapi

import (
	"crypto/tls"
	"net/http"

    log "github.com/sirupsen/logrus"
	errors "github.com/go-openapi/errors"
	runtime "github.com/go-openapi/runtime"
	middleware "github.com/go-openapi/runtime/middleware"
	graceful "github.com/tylerb/graceful"
    "github.com/carbocation/interpose"
    "github.com/carbocation/interpose/adaptors"
    //"github.com/getsentry/raven-go"
    "github.com/meatballhat/negroni-logrus"

	"git.archive.org/bnewbold/fatcat/golang/restapi/operations"
)

//go:generate swagger generate server --target .. --name Fatcat --spec ../fatcat-openapi2.yml

func configureFlags(api *operations.FatcatAPI) {
	// api.CommandLineOptionsGroups = []swag.CommandLineOptionsGroup{ ... }
}

func configureAPI(api *operations.FatcatAPI) http.Handler {
	// configure the api here
	api.ServeError = errors.ServeError

	// Set your custom logger if needed. Default one is log.Printf
	// Expected interface func(string, ...interface{})
	api.Logger = log.Printf

	api.JSONConsumer = runtime.JSONConsumer()

	api.JSONProducer = runtime.JSONProducer()

	api.GetCreatorIDHandler = operations.GetCreatorIDHandlerFunc(func(params operations.GetCreatorIDParams) middleware.Responder {
        // "get or 404" using params.ID. join creator_ident and creator_rev.
        // populate result data
        // return that
		return middleware.NotImplemented("operation .GetCreatorID has not yet been implemented")
	})
	api.PostCreatorHandler = operations.PostCreatorHandlerFunc(func(params operations.PostCreatorParams) middleware.Responder {
        // get-or-create editgroup based on current editor (session)
        // insert new rev, ident, and edit
		return middleware.NotImplemented("operation .PostCreator has not yet been implemented")
	})

	api.ServerShutdown = func() {}

	return setupGlobalMiddleware(api.Serve(setupMiddlewares))
}

// The TLS configuration before HTTPS server starts.
func configureTLS(tlsConfig *tls.Config) {
	// Make all necessary changes to the TLS configuration here.
}

// As soon as server is initialized but not run yet, this function will be called.
// If you need to modify a config, store server instance to stop it individually later, this is the place.
// This function can be called multiple times, depending on the number of serving schemes.
// scheme value will be set accordingly: "http", "https" or "unix"
func configureServer(s *graceful.Server, scheme, addr string) {
}

// The middleware configuration is for the handler executors. These do not apply to the swagger.json document.
// The middleware executes after routing but before authentication, binding and validation
func setupMiddlewares(handler http.Handler) http.Handler {
	return handler
}

// The middleware configuration happens before anything, this middleware also applies to serving the swagger.json document.
// So this is a good place to plug in a panic handling middleware, logging and metrics
func setupGlobalMiddleware(handler http.Handler) http.Handler {

    middle := interpose.New()

    // sentry and upstream
    //middle.UseHandler(sentry.Recovery(raven.DefaultClient, false))

    // logging
    negroniMiddleware := negronilogrus.NewMiddleware()
    middle.Use(adaptors.FromNegroni(negroniMiddleware))

    // add clacks
    middle.UseHandler(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
        rw.Header().Set("X-Clacks-Overhead:", "GNU Aaron Swartz, John Perry Barlow")
    }))

    middle.UseHandler(handler)

	return middle
}
