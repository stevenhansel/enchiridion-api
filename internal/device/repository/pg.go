package repository

import (
	"context"
	"fmt"

	"github.com/georgysavva/scany/pgxscan"
)

type Device struct {
	ID          int       `db:"id"`
	// Name        string    `db:"name"`
	// Description string    `db:"description"`
	MachineID   string    `db:"machine_id"`
	// CreatedAt   time.Time `db:"created_at"`
	// UpdatedAt   time.Time `db:"updated_at"`
}

func (r *Repository) Find(ctx context.Context) ([]*Device, error) {
	var devices []*Device

	query := `
		select
			"id",
			"machine_id"
		from "device"
	`

	rows, err := r.db.Query(ctx, query)
	if err != nil {
		return nil, err
	}

	if err := pgxscan.ScanAll(&devices, rows); err != nil {
		return nil, err
	}

	return devices, nil
}

func (r *Repository) Insert(ctx context.Context, MachineID string) error {
	query := `
		insert into "device" ("machine_id")
		values ($1)
	`
	if _, err := r.db.Exec(
		ctx,
		query,
		MachineID,
	); err != nil {
		return err
	}

	return nil
}

type BulkFindAnnouncementByID struct {
	ID int `db:"device_id"`
	// Name        string    `db:"device_name"`
	// Description string    `db:"device_description"`
	// MachineID string    `db:"device_machine_id"`
	// CreatedAt time.Time `db:"device_created_at"`
	// UpdatedAt time.Time `db:"device_updated_at"`
}

func (r *Repository) FindByAnnouncementID(ctx context.Context, announcementID int) ([]*Device, error) {
	var bulk []*BulkFindAnnouncementByID

	query := `
		select
			"device"."id" as "device_id"
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
	fmt.Println("bulk: ", bulk)
	fmt.Println("len", len(bulk))
	fmt.Println("bulk[0]", bulk[0].ID)
	fmt.Println("len devices", len(devices))
	fmt.Println("devices", devices)
	for i := 0; i < len(bulk); i++ {
		devices[i] = &Device{}

		devices[i].ID = bulk[i].ID
		// devices[i].Name = bulk[i].Name
		// devices[i].Description = bulk[i].Description
		// devices[i].MachineID = bulk[i].MachineID
		// devices[i].CreatedAt = bulk[i].CreatedAt
		// devices[i].UpdatedAt = bulk[i].UpdatedAt
	}

	return devices, nil
}
