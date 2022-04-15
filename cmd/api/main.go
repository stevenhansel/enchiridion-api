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
)

func main() {
	var environment Environment

	flag.Var(
		&environment,
		"env",
		"application environment, could be either (development|staging|production)",
	)

	flag.Parse()

	config, err := NewConfiguration(environment, ".")
	if err != nil {
		fmt.Fprintf(os.Stderr, "err: %v\n", err)
		os.Exit(1)
	}

	cld, err := cloudinary.NewFromParams(config.CloudinaryCloudName, config.CloudinaryApiKey, config.CloudinaryApiSecret)
	if err != nil {
		fmt.Fprintf(os.Stderr, "err: %v\n", err)
		os.Exit(1)
	}

	q, err := rmq.OpenConnection("producer", "tcp", config.REDIS_QUEUE_ADDR, 1, nil)
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

	e.Logger.Fatal(e.Start(":8080"))
}
