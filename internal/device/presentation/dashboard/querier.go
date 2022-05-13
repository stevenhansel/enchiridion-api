package presentation

import (
	"context"

	deviceRepository "github.com/stevenhansel/enchiridion-api/internal/device/repository"
	deviceUsecase "github.com/stevenhansel/enchiridion-api/internal/device/usecase"
)

type DeviceUsecaseQuerier interface {
	ListDevice(ctx context.Context) ([]*deviceRepository.Device, error)
	CreateDevice(ctx context.Context, params *deviceUsecase.CreateDeviceParams) error
}
