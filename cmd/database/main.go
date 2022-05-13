package main

import (
	"flag"
	"log"

	db "github.com/stevenhansel/enchiridion-api/database"
	"github.com/stevenhansel/enchiridion-api/internal/config"
)

type MigrationCommand string

const (
	MigrateUp   MigrationCommand = "up"
	MigrateDown MigrationCommand = "down"
)

func main() {
	var cmd string
	environment := config.DEVELOPMENT

	flag.Var(
		&environment,
		"env",
		"application environment, could be either (development|staging|production)",
	)

	flag.StringVar(&cmd, "command", "up", `The migration command, could be "up" or "down", the default is "up"`)
	flag.Parse()

	command := MigrationCommand(cmd)

	app, err := NewMigrationApp(environment)
	if err != nil {
		log.Fatal(err)
		return
	}

	if command == MigrateUp {
		err = db.MigrateUp(app.Instance)
		if err != nil {
			log.Fatal(err)
			return
		}

	} else if command == MigrateDown {
		err = db.MigrateDown(app.Instance)
		if err != nil {
			log.Fatal(err)
			return
		}
	} else {
		return
	}
}
