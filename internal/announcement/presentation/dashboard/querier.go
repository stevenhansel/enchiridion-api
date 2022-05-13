package presentation

import (
	"context"

	announcementRepository "github.com/stevenhansel/enchiridion-api/internal/announcement/repository"
	announcementUsecase "github.com/stevenhansel/enchiridion-api/internal/announcement/usecase"
)

type AnnouncementUsecaseQuerier interface {
	ListAnnouncement(ctx context.Context) ([]*announcementRepository.Announcement, error)
	CreateAnnouncement(ctx context.Context, params *announcementUsecase.CreateAnnouncementParams) error
	UpdateAnnouncementApproval(ctx context.Context, id int, approve bool) error
}
