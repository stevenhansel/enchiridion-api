package presentation

import "github.com/labstack/echo/v4"

func (p *Presentation) Attach(e *echo.Group) {
	e.GET("/v1/announcements", p.listAnnouncement)
	e.GET("v1/announcements/:announcementId", p.announcementDetail)
	e.POST("/v1/announcements", p.createAnnouncement)
	e.PUT("/v1/announcements/:announcementId/approval", p.updateAnnouncementApproval)
}

func (p *Presentation) listAnnouncement(ctx echo.Context) error {
	panic("not implemented")
}

func (p *Presentation) announcementDetail(ctx echo.Context) error {
	panic("not implemented")
}

func (p *Presentation) createAnnouncement(ctx echo.Context) error {
	panic("not implemented")
}

func (p *Presentation) updateAnnouncementApproval(ctx echo.Context) error {
	panic("not implemented")
}
