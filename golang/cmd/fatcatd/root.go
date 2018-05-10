
package main

import (
	"os"
	"fmt"

    log "github.com/sirupsen/logrus"
    "github.com/spf13/viper"
    "github.com/spf13/cobra"
    "github.com/getsentry/raven-go"
)

var cfgFile = "";

var rootCmd = &cobra.Command{
    Use:   "fatcatd",
    Short: "REST API Server",
	Long:  "A scalable, versioned, API-oriented catalog of bibliographic entities and file metadata",
    Version: "0.1.0",
}

func init() {
    cobra.OnInitialize(initConfig)
    rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "config file (default is ./fatcatd.toml)")
    rootCmd.PersistentFlags().BoolP("verbose", "v", false, "increase logging volume")
    viper.BindPFlag("verbose", rootCmd.PersistentFlags().Lookup("verbose"))

    serveCmd.Flags().String("db-url", "postgres://", "database connecion string")

    rootCmd.AddCommand(serveCmd)
    rootCmd.AddCommand(reinitCmd)
}

func initConfig() {

    viper.SetDefault("port", 9411)
    viper.SetEnvPrefix("FATCAT")
    viper.AutomaticEnv()

    if cfgFile != "" {
        // Use config file from the flag.
        viper.SetConfigFile(cfgFile)
    } else {
        viper.SetConfigType("toml")
        viper.AddConfigPath(".")
        viper.SetConfigName("fatcatd")
    }

    err := viper.ReadInConfig()
    if err != nil {
        log.Fatalf("Error loading config: %s \n", err)
    }

    // not default of stderr
    log.SetOutput(os.Stdout);

    if viper.GetBool("verbose") == true {
        log.SetLevel(log.DebugLevel)
    }
    log.Debug("It's verbose!")

    raven.SetDSN(viper.GetString("sentry_dsn"));

}

func Execute() {
    if err := rootCmd.Execute(); err != nil {
        fmt.Println(err)
        os.Exit(1)
    }
}
