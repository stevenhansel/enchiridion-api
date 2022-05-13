package usecase

import (
	"context"

	deviceRepository "github.com/stevenhansel/enchiridion-api/internal/device/repository"
)

func (u *Usecase) ListDevicesByAnnouncementID(ctx context.Context, announcementID int) ([]*deviceRepository.Device, error) {
	return u.db.FindByAnnouncementID(ctx, announcementID)
}
