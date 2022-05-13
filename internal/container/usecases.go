package container

import (
	announcementUsecase "github.com/stevenhansel/enchiridion-api/internal/announcement/usecase"
	deviceUsecase "github.com/stevenhansel/enchiridion-api/internal/device/usecase"
)

type Usecase struct {
	UsecaseAnnouncement *announcementUsecase.Usecase
	UsecaseDevice       *deviceUsecase.Usecase
}

func createUsecaseLayer(internal *Internal, repository *Repository) (*Usecase, error) {
	deviceUsecase := deviceUsecase.New(repository.RepositoryDevice)
	announcementUsecase := announcementUsecase.New(
		repository.RepositoryAnnouncement,
		deviceUsecase,
		internal.Cloudinary,
		internal.Rmq,
	)

	return &Usecase{
		UsecaseAnnouncement: announcementUsecase,
		UsecaseDevice:       deviceUsecase,
	}, nil
}
