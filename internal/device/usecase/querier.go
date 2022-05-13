package usecase

import (
	"context"

	deviceRepository "github.com/stevenhansel/enchiridion-api/internal/device/repository"
)

type DeviceRepositoryQuerier interface {
	FindByAnnouncementID(ctx context.Context, announcementID int) ([]*deviceRepository.Device, error)
}
