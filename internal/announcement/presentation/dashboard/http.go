package presentation

import (
	"log"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/labstack/echo/v4"

	announcementRepository "github.com/stevenhansel/enchiridion-api/internal/announcement/repository"
	announcementUsecase "github.com/stevenhansel/enchiridion-api/internal/announcement/usecase"
)

func (p *Presentation) Attach(e *echo.Group) {
	e.GET("/v1/announcements", p.listAnnouncement)
	e.GET("v1/announcements/:announcementId", p.announcementDetail)
	e.POST("/v1/announcements", p.createAnnouncement)
	e.PUT("/v1/announcements/:announcementId/approval", p.updateAnnouncementApproval)
}

type listAnnouncementResponse struct {
	Contents []listAnnouncementContent `json:"contents"`
}

type listAnnouncementContent struct {
	ID             int                                       `json:"id"`
	Title          string                                    `json:"title"`
	Media          string                                    `json:"media"`
	Filename       string                                    `json:"filename"`
	Status         announcementRepository.AnnouncementStatus `json:"status"`
	Notes          string                                    `json:"notes"`
	Duration       int                                       `json:"duration"`
	// RejectionNotes *string                                   `json:"rejectionNotes"`
	// ApprovedAt     *time.Time                                `json:"approvedAt"`
	CreatedAt      time.Time                                 `json:"createdAt"`
	UpdatedAt      time.Time                                 `json:"updatedAt"`
}

func (p *Presentation) listAnnouncement(c echo.Context) error {
	ctx := c.Request().Context()

	announcements, err := p.announcement.ListAnnouncement(ctx)
	if err != nil {
		log.Println(err)
		return echo.NewHTTPError(http.StatusInternalServerError, "Something went wrong")
	}

	contents := make([]listAnnouncementContent, len(announcements))
	for i := 0; i < len(announcements); i++ {
		contents[i].ID = announcements[i].ID
		contents[i].Title = announcements[i].Title
		contents[i].Media = announcements[i].Media
		contents[i].Filename = announcements[i].Filename
		contents[i].Status = announcements[i].Status
		contents[i].Notes = announcements[i].Notes
		contents[i].Duration = announcements[i].Duration
		// contents[i].RejectionNotes = announcements[i].RejectionNotes
		// contents[i].ApprovedAt = announcements[i].ApprovedAt
		contents[i].CreatedAt = announcements[i].CreatedAt
		contents[i].UpdatedAt = announcements[i].UpdatedAt
	}

	return c.JSON(http.StatusOK, &listAnnouncementResponse{Contents: contents})
}

func (p *Presentation) announcementDetail(c echo.Context) error {
	panic("not implemented")
}

func (p *Presentation) createAnnouncement(c echo.Context) error {
	ctx := c.Request().Context()

	title := c.FormValue("title")
	media, err := c.FormFile("media")
	if err != nil {
		return echo.NewHTTPError(http.StatusBadRequest, "Media is required")
	}
	duration, err := strconv.Atoi(c.FormValue("duration"))
	if err != nil {
		return echo.NewHTTPError(http.StatusBadRequest, "Duration must be a valid integer")
	}
	notes := c.FormValue("notes")

	rawDeviceIDs := strings.Split(c.FormValue("deviceIds"), ",")
	deviceIDs := make([]int, len(rawDeviceIDs))
	for i := 0; i < len(deviceIDs); i++ {
		id, err := strconv.Atoi(rawDeviceIDs[i])
		if err != nil {
			return echo.NewHTTPError(http.StatusBadRequest, "Device Ids must be a valid integer")
		}

		deviceIDs[i] = id
	}

	params := &announcementUsecase.CreateAnnouncementParams{
		Title:     title,
		Media:     media,
		Duration:  duration,
		Notes:     notes,
		DeviceIDs: deviceIDs,
	}

	if err := p.announcement.CreateAnnouncement(ctx, params); err != nil {
		log.Println(err)
		return echo.NewHTTPError(http.StatusInternalServerError, "Something went wrong")
	}

	return c.NoContent(http.StatusNoContent)
}

func (p *Presentation) updateAnnouncementApproval(c echo.Context) error {
	ctx := c.Request().Context()

	params := &announcementUsecase.UpdateAnnouncementApprovalParams{}
	if err := c.Bind(params); err != nil {
		return echo.NewHTTPError(http.StatusBadRequest, "Something went wrong when parsing the input")
	}
	announcementID, err := strconv.Atoi(c.Param("announcementId"))
	if err != nil {
		return echo.NewHTTPError(http.StatusBadRequest, "Announcement ID invalid")
	}

	params.AnnouncementID = announcementID

	if err := p.announcement.UpdateAnnouncementApproval(ctx, params); err != nil {
		log.Println(err)
		return echo.NewHTTPError(http.StatusInternalServerError, "Something went wrong")
	}

	return c.NoContent(http.StatusNoContent)
}
