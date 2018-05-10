
package main

import (
	"os"

    log "github.com/sirupsen/logrus"
	loads "github.com/go-openapi/loads"
	flags "github.com/jessevdk/go-flags"
    "github.com/spf13/viper"

	"git.archive.org/bnewbold/fatcat/golang/restapi"
	"git.archive.org/bnewbold/fatcat/golang/restapi/operations"
)

func init() {

    viper.SetEnvPrefix("FATCAT")
    viper.AutomaticEnv()
    viper.SetDefault("Port", 9411)
    viper.SetDefault("Verbose", true)

    viper.SetConfigName("fatcatd.toml")
    viper.AddConfigPath(".")
    err := viper.ReadInConfig()
    if err != nil {
        log.Fatalf("Fatal error config file: %s \n", err)
    }

    // not default of stderr
    log.SetOutput(os.Stdout);

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

	server.ConfigureAPI()

	if err := server.Serve(); err != nil {
		log.Fatalln(err)
	}

}
