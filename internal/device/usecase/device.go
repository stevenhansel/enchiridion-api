package usecase

import (
	"context"

	deviceRepository "github.com/stevenhansel/enchiridion-api/internal/device/repository"
)

func (u *Usecase) ListDevice(ctx context.Context) ([]*deviceRepository.Device, error) {
	return u.db.Find(ctx)
}

func (u *Usecase) ListDevicesByAnnouncementID(ctx context.Context, announcementID int) ([]*deviceRepository.Device, error) {
	return u.db.FindByAnnouncementID(ctx, announcementID)
}

type CreateDeviceParams struct {
	MachineID string `json:"machineId"`
}

func (u *Usecase) CreateDevice(ctx context.Context, params *CreateDeviceParams) error {
	return u.db.Insert(ctx, params.MachineID)
}
