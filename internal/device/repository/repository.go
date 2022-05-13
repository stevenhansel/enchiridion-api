package repository

import "github.com/jackc/pgx/v4"

type Repository struct {
	db *pgx.Conn
}

func New(db *pgx.Conn) *Repository {
	return &Repository{db: db}
}
