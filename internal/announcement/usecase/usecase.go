package usecases

import (
	"github.com/adjust/rmq/v4"
	"github.com/cloudinary/cloudinary-go"
)

type Usecase struct {
	db         AnnouncementRepositoryQuerier
	device     DeviceUsecaseQuerier
	cloudinary *cloudinary.Cloudinary
	rmq        rmq.Connection
}

func New(
	db AnnouncementRepositoryQuerier,
	device DeviceUsecaseQuerier,
	cloudinary *cloudinary.Cloudinary,
	rmq rmq.Connection,
) *Usecase {
	return &Usecase{
		db:         db,
		device:     device,
		cloudinary: cloudinary,
		rmq:        rmq,
	}
}
