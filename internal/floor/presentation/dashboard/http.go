package presentation

import "github.com/labstack/echo/v4"

func (p *Presentation) Attach(e *echo.Group) {
	e.GET("/v1/floors", p.listFloor)
	e.POST("/v1/floors", p.createFloor)
	e.PUT("/v1/floors/:floorId", p.updateFloor)
	e.DELETE("/v1/floors/:floorId", p.deleteFloor)
}

func (p *Presentation) listFloor(ctx echo.Context) error {
	panic("not implemented")
}

func (p *Presentation) createFloor(ctx echo.Context) error {
	panic("not implemented")
}

func (p *Presentation) updateFloor(ctx echo.Context) error {
	panic("not implemented")
}

func (p *Presentation) deleteFloor(ctx echo.Context) error {
	panic("not implemented")
}
