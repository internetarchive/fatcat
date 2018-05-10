
package main

import (
    "github.com/spf13/viper"
    log "github.com/sirupsen/logrus"
    "github.com/go-pg/pg"
    "github.com/spf13/cobra"
    "github.com/gobuffalo/packr"
)

var reinitCmd = &cobra.Command{
    Use:    "reinit [options]",
    Short:  "Reset database schema",
    Run:    func(cmd *cobra.Command, args[] string) {
        main_reinit()
    },
}

func main_reinit() {

    box := packr.NewBox("../../sql")
    sql_schema, err := box.MustString("fatcat-schema.sql")
    if err != nil {
        log.Panicf("finding SQL file: {}", err)
    }

    db_options, err := pg.ParseURL(viper.GetString("db_url"))
    if err != nil {
        log.Panicf("parsing DB string: {}", err)
    }
    db := pg.Connect(db_options)
    defer db.Close()

    log.Info("Starting load...")
    _, err = db.Exec(sql_schema)
    if err != nil {
        log.Fatalf("Error loading SQL: {}", err)
    }
    log.Info("Success!")

}
