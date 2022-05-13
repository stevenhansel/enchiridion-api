package presentation

import "github.com/labstack/echo/v4"

func (p *Presentation) Attach(e *echo.Group) {
	e.GET("/v1/devices", p.listDevice)
	e.POST("/v1/devices", p.createDevice)
	e.PUT("/v1/devices/:deviceId", p.updateDevice)
	e.DELETE("/v1/devices/:deviceId", p.deleteDevice)
}

func (p *Presentation) listDevice(ctx echo.Context) error {
	panic("not implemented")
}

func (p *Presentation) createDevice(ctx echo.Context) error {
	panic("not implemented")
}

func (p *Presentation) updateDevice(ctx echo.Context) error {
	panic("not implemented")
}

func (p *Presentation) deleteDevice(ctx echo.Context) error {
	panic("not implemented")
}
