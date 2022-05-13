package main

import (
	"github.com/golang-migrate/migrate/v4"

	"github.com/stevenhansel/enchiridion-api/database"
	"github.com/stevenhansel/enchiridion-api/internal/config"
)

type MigrationApplication struct {
	Instance *migrate.Migrate
	Config   *config.Configuration
}

func NewMigrationApp(environment config.Environment) (*MigrationApplication, error) {
	config, err := config.New(environment)
	if err != nil {
		return nil, err
	}

	m, err := database.NewMigrationInstance(config)
	if err != nil {
		return nil, err
	}

	return &MigrationApplication{
		Instance: m,
		Config:   config,
	}, nil
}
