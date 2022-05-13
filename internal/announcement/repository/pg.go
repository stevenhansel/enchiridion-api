package repositories

import (
	"context"
	"time"

	"github.com/georgysavva/scany/pgxscan"
)

type AnnouncementStatus string

var (
	WaitingForApproval AnnouncementStatus = "waiting_for_approval"
	WaitingForSync     AnnouncementStatus = "waiting_for_sync"
	Active             AnnouncementStatus = "active"
	Done               AnnouncementStatus = "done"
	Canceled           AnnouncementStatus = "canceled"
	Rejected           AnnouncementStatus = "removed"
)

type Announcement struct {
	ID             int                `db:"id"`
	Title          string             `db:"title"`
	Media          string             `db:"media"`
	Filename       string             `db:"filename"`
	Status         AnnouncementStatus `db:"status"`
	Notes          string             `db:"notes"`
	Duration       int                `db:"duration"`
	RejectionNotes string             `db:"rejection_notes"`
	ApprovedAt     *time.Time         `db:"approved_at"`
	CreatedAt      time.Time          `db:"created_at"`
	UpdatedAt      time.Time          `db:"updated_at"`
	// UserID int `db:"user_id"`
}

func (r *Repository) Find(ctx context.Context) ([]*Announcement, error) {
	var announcements []*Announcement

	query := `
		select 
			"id", 
			"title", 
			"media", 
			"filename",
			"status", 
			"notes", 
			"duration", 
			"rejection_notes", 
			"approved_at", 
			"created_at", 
			"updated_at"
		from "announcement"
	`

	rows, err := r.db.Query(ctx, query)
	if err != nil {
		return nil, err
	}

	if err := pgxscan.ScanAll(&announcements, rows); err != nil {
		return nil, err
	}

	return announcements, nil
}

func (r *Repository) FindOne(ctx context.Context, id int) (*Announcement, error) {
	var announcement *Announcement

	query := `
		select 
			"id", 
			"title", 
			"media", 
			"filename",
			"status", 
			"notes", 
			"duration", 
			"rejection_notes", 
			"approved_at", 
			"created_at", 
			"updated_at"
		from "announcement"
		where "id" = $1
	`

	rows, err := r.db.Query(ctx, query)
	if err != nil {
		return nil, err
	}

	if err := pgxscan.ScanOne(&announcement, rows); err != nil {
		return nil, err
	}

	return announcement, nil
}

type AnnouncementDetail struct {
	ID             int                `db:"announcement_id"`
	Title          string             `db:"announcement_title"`
	Media          string             `db:"announcement_media"`
	Filename       string             `db:"announcement_filename"`
	Status         AnnouncementStatus `db:"announcement_status"`
	Notes          string             `db:"announcement_notes"`
	Duration       int                `db:"announcement_duration"`
	RejectionNotes string             `db:"announcement_rejection_notes"`
	ApprovedAt     *time.Time         `db:"announcement_approved_at"`
	CreatedAt      time.Time          `db:"announcement_created_at"`
	UpdatedAt      time.Time          `db:"announcement_updated_at"`
}

// TODO: not completed
func (r *Repository) FindOneDetailed(ctx context.Context, id int) (*AnnouncementDetail, error) {
	var announcement *AnnouncementDetail

	query := `
		select 
			"announcement"."id" as "announcement_id", 
			"announcement"."title" as "announcement_title", 
			"announcement"."media" as "announcement_media", 
			"announcement"."status" as "announcement_status", 
			"announcement"."notes" as "announcement_notes", 
			"announcement"."duration" as "announcement_duration", 
			"announcement"."rejection_notes" as "announcement_rejection_notes", 
			"announcement"."approved_at" as "announcement_approved_at", 
			"announcement"."created_at" as "announcement_created_at", 
			"announcement"."updated_at" as "announcement_updated_at",
		from "announcement"
		join "device_announcement" on "device_announcement"."announcement_id" = "announcement"."id"
		join "device" on "device"."id" = "device_announcement"."device_id"
		join "floor" on "floor"."id" = "device"."floor_id"
		where "id" = $1
	`

	rows, err := r.db.Query(ctx, query, id)
	if err != nil {
		return nil, err
	}

	if err := pgxscan.ScanOne(announcement, rows); err != nil {
		return nil, err
	}

	return announcement, nil
}

type InsertAnnouncementParams struct {
	Title    string
	Media    string
	Filename string
	Duration int
	Notes    string
}

func (r *Repository) Insert(ctx context.Context, params *InsertAnnouncementParams) error {
	query := `
		insert into "announcement"
		(
			"title",
			"media",
			"filename",
			"duration",
			"notes"
		)
		values ($1, $2, $3, $4, $5)
	`

	if _, err := r.db.Exec(
		ctx,
		query,
		params.Title,
		params.Media,
		params.Filename,
		params.Duration,
		params.Notes,
	); err != nil {
		return err
	}

	return nil
}

func (r *Repository) UpdateStatus(ctx context.Context, id int, status AnnouncementStatus) error {
	query := `
		update "announcement"
		set "status" = $1
		where "id" = $2
	`

	if _, err := r.db.Exec(ctx, query, status, id); err != nil {
		return err
	}

	return nil
}

func (r *Repository) UpdateApprovalStatus(ctx context.Context, id int, status AnnouncementStatus) error {
	query := `
		update "announcement"
		set 
			"status" = $1,
			"approved_at" = now()
		where "id" = $2
	`

	if _, err := r.db.Exec(ctx, query, status, id); err != nil {
		return err
	}

	return nil
}
