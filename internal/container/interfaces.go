package container

import (
	announcementDashboard "github.com/stevenhansel/enchridion-api/internal/announcement/presentation/dashboard"
)

type Presentation struct {
	PresentationAnnouncementDashboard *announcementDashboard.Presentation
}

func createPresentationLayer() (*Presentation, error) {
	announcementDashboard := announcementDashboard.New()

	return &Presentation{
		PresentationAnnouncementDashboard: announcementDashboard,
	}, nil
}
