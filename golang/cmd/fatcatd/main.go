
package main

import (
	"os"
	"net/http"

    log "github.com/sirupsen/logrus"
	loads "github.com/go-openapi/loads"
	flags "github.com/jessevdk/go-flags"
	//middleware "github.com/go-openapi/runtime/middleware"
    "github.com/spf13/viper"
    "github.com/getsentry/raven-go"
    "github.com/carbocation/interpose"
    "github.com/carbocation/interpose/adaptors"
    "github.com/meatballhat/negroni-logrus"
    "github.com/go-pg/pg"

	"git.archive.org/bnewbold/fatcat/golang/api/handlers"
	"git.archive.org/bnewbold/fatcat/golang/gen/restapi"
	"git.archive.org/bnewbold/fatcat/golang/gen/restapi/operations"
)

func init() {

    viper.SetDefault("port", 9411)
    viper.SetDefault("verbose", true)

    viper.SetEnvPrefix("FATCAT")
    viper.AutomaticEnv()

    viper.SetConfigType("toml")
    viper.SetConfigName("fatcatd.toml")
    viper.AddConfigPath(".")
    //err := viper.ReadInConfig()
    //if err != nil {
    //    log.Fatalf("Error loading config: %s \n", err)
    //}

    // not default of stderr
    log.SetOutput(os.Stdout);

    raven.SetDSN(viper.GetString("sentry_dsn"));

}

func main() {

    log.Warn("Starting up...");

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

	parser := flags.NewParser(server, flags.Default)
	parser.ShortDescription = "fatcat"
	parser.LongDescription = "A scalable, versioned, API-oriented catalog of bibliographic entities and file metadata"

	server.ConfigureFlags()
	for _, optsGroup := range api.CommandLineOptionsGroups {
		_, err := parser.AddGroup(optsGroup.ShortDescription, optsGroup.LongDescription, optsGroup.Options)
		if err != nil {
			log.Fatalln(err)
		}
	}

	if _, err := parser.Parse(); err != nil {
		code := 1
		if fe, ok := err.(*flags.Error); ok {
			if fe.Type == flags.ErrHelp {
				code = 0
			}
		}
		os.Exit(code)
	}

    server.Port = viper.GetInt("port")

    db := pg.Connect(&pg.Options{
        User: "bnewbold", // XXX:
        Database: "fatcat",
    })
    defer db.Close()

    // register all the many handlers here
	api.GetCreatorIDHandler = handlers.NewGetCreatorIDHandler(db);
    api.PostCreatorHandler = handlers.NewPostCreatorHandler(db);

    middle := interpose.New()

    // sentry and upstream
    // XXX: middle.UseHandler(sentry.Recovery(raven.DefaultClient, false))

    // logging
    negroniMiddleware := negronilogrus.NewMiddleware()
    middle.Use(adaptors.FromNegroni(negroniMiddleware))

    // add clacks (TODO: only production)
    middle.UseHandler(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
        rw.Header().Set("X-Clacks-Overhead:", "GNU Aaron Swartz, John Perry Barlow")
    }))

    middle.UseHandler(api.Serve(nil))

    server.SetHandler(middle)

	if err := server.Serve(); err != nil {
		log.Fatalln(err)
	}

}
