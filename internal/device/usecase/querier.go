package usecase

import (
	"context"

	deviceRepository "github.com/stevenhansel/enchiridion-api/internal/device/repository"
)

type DeviceRepositoryQuerier interface {
	Find(ctx context.Context) ([]*deviceRepository.Device, error)
	FindByAnnouncementID(ctx context.Context, announcementID int) ([]*deviceRepository.Device, error)
	Insert(ctx context.Context, MachineID string) error
}
