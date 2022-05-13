package http

import (
	"context"
	"fmt"
	"net/http"
	"os"
	"os/signal"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/labstack/gommon/log"

	"github.com/stevenhansel/enchiridion-api/internal/container"
)

type Server struct {
	*container.Container
}

func (s *Server) registerMiddleware(e *echo.Echo) {
	e.Use(middleware.Logger())
	e.Use(middleware.RecoverWithConfig(middleware.RecoverConfig{
		StackSize: 1 << 10, // 1 KB
		LogLevel:  log.ERROR,
	}))
}

func (s *Server) registerRoutes(e *echo.Echo) {
	e.GET("/", func(c echo.Context) error {
		return c.String(http.StatusOK, "Enchridion API; status: healthy")
	})

	s.PresentationAnnouncementDashboard.Attach(e.Group("/dashboard"))
	s.PresentationDeviceDashboard.Attach(e.Group("/dashboard"))
	s.PresentationFloorDashboard.Attach(e.Group("/dashboard"))
}

func (s *Server) Serve() {
	e := echo.New()

	s.registerMiddleware(e)
	s.registerRoutes(e)

	go func() {
		if err := e.Start(fmt.Sprintf(":%d", s.Config.Port)); err != nil && err != http.ErrServerClosed {
			e.Logger.Fatal(err)
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, os.Interrupt)
	<-quit

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	if err := e.Shutdown(ctx); err != nil {
		e.Logger.Fatal(err)
	}
}

func New(container *container.Container) *Server {
	return &Server{container}
}
