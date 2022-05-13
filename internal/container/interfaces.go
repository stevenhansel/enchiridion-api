package container

import (
	announcementDashboard "github.com/stevenhansel/enchiridion-api/internal/announcement/presentation/dashboard"
	deviceDashboard "github.com/stevenhansel/enchiridion-api/internal/device/presentation/dashboard"
	floorDashboard "github.com/stevenhansel/enchiridion-api/internal/floor/presentation/dashboard"
)

type Presentation struct {
	PresentationAnnouncementDashboard *announcementDashboard.Presentation
	PresentationDeviceDashboard       *deviceDashboard.Presentation
	PresentationFloorDashboard        *floorDashboard.Presentation
}

func createPresentationLayer(internal *Internal, repository *Repository, usecase *Usecase) (*Presentation, error) {
	announcementDashboard := announcementDashboard.New(usecase.UsecaseAnnouncement)
	deviceDashboard := deviceDashboard.New(usecase.UsecaseDevice)
	floorDashboard := floorDashboard.New()

	return &Presentation{
		PresentationAnnouncementDashboard: announcementDashboard,
		PresentationDeviceDashboard:       deviceDashboard,
		PresentationFloorDashboard:        floorDashboard,
	}, nil
}
