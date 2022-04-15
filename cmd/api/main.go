package main

import (
	"flag"
	"fmt"
	"net/http"
	"os"

	"github.com/adjust/rmq/v4"
	"github.com/cloudinary/cloudinary-go"
	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
	"github.com/stevenhansel/enchridion-api/internal/config"
)

func main() {
	var environment config.Environment

	flag.Var(
		&environment,
		"env",
		"application environment, could be either (development|staging|production)",
	)

	flag.Parse()

	config, err := config.NewConfiguration(environment)
	if err != nil {
		fmt.Fprintf(os.Stderr, "err: %v\n", err)
		os.Exit(1)
	}

	cld, err := cloudinary.NewFromParams(config.CloudinaryCloudName, config.CloudinaryApiKey, config.CloudinaryApiSecret)
	if err != nil {
		fmt.Fprintf(os.Stderr, "err: %v\n", err)
		os.Exit(1)
	}

	q, err := rmq.OpenConnection("producer", "tcp", config.RedisQueueAddr, 1, nil)
	if err != nil {
		fmt.Fprintf(os.Stderr, "err: %v\n", err)
		os.Exit(1)
	}

	e := echo.New()
	e.Use(middleware.Logger())

	controller := NewController(e, cld, &q)

	e.GET("/", func(c echo.Context) error {
		return c.String(http.StatusOK, "Enchridion API; status: healthy")
	})

	e.POST("/upload", controller.Upload)
	e.GET("/test", controller.TestPublish)

	e.Logger.Fatal(e.Start(fmt.Sprintf(":%d", config.Port)))
}
