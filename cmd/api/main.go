package main

import (
	"fmt"
	"net/http"
	"os"

	"github.com/cloudinary/cloudinary-go"
	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
)

func main() {
	config, err := NewConfiguration(".")
	if err != nil {
		fmt.Fprintf(os.Stderr, "err: %v\n", err)
		os.Exit(1)
	}

	cld, err := cloudinary.NewFromParams(config.CloudinaryCloudName, config.CloudinaryApiKey, config.CloudinaryApiSecret)
	if err != nil {
		fmt.Fprintf(os.Stderr, "err: %v\n", err)
		os.Exit(1)
	}

	e := echo.New()
	e.Use(middleware.Logger())

	controller := NewController(e, cld)

	e.GET("/", func(c echo.Context) error {
		return c.String(http.StatusOK, "Enchridion API; status: healthy")
	})

	e.POST("/upload", controller.Upload)

	e.Logger.Fatal(e.Start(":8080"))
}
