package usecases

import (
	"context"

	announcementRepository "github.com/stevenhansel/enchiridion-api/internal/announcement/repository"
	deviceRepository "github.com/stevenhansel/enchiridion-api/internal/device/repository"
)

type AnnouncementRepositoryQuerier interface {
	Find(ctx context.Context) ([]*announcementRepository.Announcement, error)
	FindOne(ctx context.Context, id int) (*announcementRepository.Announcement, error)
	Insert(ctx context.Context, params *announcementRepository.InsertAnnouncementParams) error
	UpdateApprovalStatus(ctx context.Context, id int, status announcementRepository.AnnouncementStatus) error
}

type DeviceUsecaseQuerier interface {
	ListDevicesByAnnouncementID(ctx context.Context, announcementID int) ([]*deviceRepository.Device, error)
}
