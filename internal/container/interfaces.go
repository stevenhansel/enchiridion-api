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

func createPresentationLayer() (*Presentation, error) {
	announcementDashboard := announcementDashboard.New()
	deviceDashboard := deviceDashboard.New()
	floorDashboard := floorDashboard.New()

	return &Presentation{
		PresentationAnnouncementDashboard: announcementDashboard,
		PresentationDeviceDashboard:       deviceDashboard,
		PresentationFloorDashboard:        floorDashboard,
	}, nil
}
