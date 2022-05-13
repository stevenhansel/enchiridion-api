package presentation

import (
	"log"
	"net/http"

	"github.com/labstack/echo/v4"

	deviceUsecase "github.com/stevenhansel/enchiridion-api/internal/device/usecase"
)

func (p *Presentation) Attach(e *echo.Group) {
	e.GET("/v1/devices", p.listDevice)
	e.POST("/v1/devices", p.createDevice)
	e.PUT("/v1/devices/:deviceId", p.updateDevice)
	e.DELETE("/v1/devices/:deviceId", p.deleteDevice)
}

type listDeviceResponse struct {
	Contents []listDeviceContent `json:"contents"`
}

type listDeviceContent struct {
	ID          int       `json:"id"`
	// Name        string    `json:"name"`
	// Description string    `json:"description"`
	MachineID   string    `json:"machineId"`
	// CreatedAt   time.Time `json:"createdAt"`
	// UpdatedAt   time.Time `json:"updatedAt"`
}

func (p *Presentation) listDevice(c echo.Context) error {
	ctx := c.Request().Context()

	devices, err := p.device.ListDevice(ctx)
	if err != nil {
		log.Print(err)
		return echo.NewHTTPError(http.StatusInternalServerError, "Something went wrong")
	}

	contents := make([]listDeviceContent, len(devices))
	for i := 0; i < len(devices); i++ {
		contents[i].ID = devices[i].ID
		// contents[i].Name = devices[i].Name
		// contents[i].Description = devices[i].Description
		contents[i].MachineID = devices[i].MachineID
		// contents[i].CreatedAt = devices[i].CreatedAt
		// contents[i].UpdatedAt = devices[i].UpdatedAt
	}

	return c.JSON(http.StatusOK, &listDeviceResponse{Contents: contents})
}

func (p *Presentation) createDevice(c echo.Context) error {
	ctx := c.Request().Context()

	params := &deviceUsecase.CreateDeviceParams{}
	if err := c.Bind(params); err != nil {
		return echo.NewHTTPError(http.StatusBadRequest, "Something went wrong when parsing the input")
	}

	if err := p.device.CreateDevice(ctx, params); err != nil {
		log.Print(err)
		return echo.NewHTTPError(http.StatusInternalServerError, "Something went wrong")
	}

	return c.NoContent(http.StatusNoContent)
}

func (p *Presentation) updateDevice(c echo.Context) error {
	panic("not implemented")
}

func (p *Presentation) deleteDevice(c echo.Context) error {
	panic("not implemented")
}
