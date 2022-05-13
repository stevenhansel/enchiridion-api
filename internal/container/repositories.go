package container

import (
	announcementRepository "github.com/stevenhansel/enchiridion-api/internal/announcement/repository"
	deviceRepository "github.com/stevenhansel/enchiridion-api/internal/device/repository"
)

type Repository struct {
	RepositoryAnnouncement *announcementRepository.Repository
	RepositoryDevice       *deviceRepository.Repository
}

func createRepositoryLayer(internal *Internal) (*Repository, error) {
	announcementRepository := announcementRepository.New(internal.DB)
	deviceRepository := deviceRepository.New(internal.DB)

	return &Repository{
		RepositoryAnnouncement: announcementRepository,
		RepositoryDevice:       deviceRepository,
	}, nil
}
