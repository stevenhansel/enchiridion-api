package repository

import (
	"context"
	"time"

	"github.com/georgysavva/scany/pgxscan"
)

type Device struct {
	ID          int       `db:"id"`
	Name        string    `db:"name"`
	Description string    `db:"description"`
	CreatedAt   time.Time `db:"created_at"`
	UpdatedAt   time.Time `db:"updated_at"`
}

type BulkFindAnnouncementByID struct {
	ID          int       `db:"device_id"`
	Name        string    `db:"device_name"`
	Description string    `db:"device_description"`
	CreatedAt   time.Time `db:"device_created_at"`
	UpdatedAt   time.Time `db:"device_updated_at"`
}

func (r *Repository) FindByAnnouncementID(ctx context.Context, announcementID int) ([]*Device, error) {
	var bulk []*BulkFindAnnouncementByID

	query := `
		select
			"device"."id" as "device_id",
			"device"."name" as "device_name",
			"device"."description" as "device_description",
			"device"."created_at" as "device_created_at",
			"device"."updated_at" as "device_updated_at"
		from "device_announcement"
		join "device" on "device"."id" = "device_announcement"."device_id"
		where "device_announcement"."announcement_id" = $1
	`
	rows, err := r.db.Query(ctx, query, announcementID)
	if err != nil {
		return nil, err
	}

	if err := pgxscan.ScanAll(&bulk, rows); err != nil {
		return nil, err
	}

	devices := make([]*Device, len(bulk))
	for i := 0; i < len(bulk); i++ {
		devices[i].ID = bulk[i].ID
		devices[i].Name = bulk[i].Name
		devices[i].Description = bulk[i].Description
		devices[i].CreatedAt = bulk[i].CreatedAt
		devices[i].UpdatedAt = bulk[i].UpdatedAt
	}

	return devices, nil
}
