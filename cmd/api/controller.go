package main

import (
	"net/http"
	"strconv"
	"time"

	"github.com/adjust/rmq/v4"
	"github.com/cloudinary/cloudinary-go"
	"github.com/cloudinary/cloudinary-go/api/uploader"
	"github.com/labstack/echo/v4"
)

type Controller struct {
	e   *echo.Echo
	cld *cloudinary.Cloudinary
	q   rmq.Connection
}

func handleExpirationTime(expirationTime int) {
	duration := time.Duration(expirationTime) * time.Second
	time.AfterFunc(duration, func() {
		// TODO: handle expiration time, deleting the image in cloudinary + publishing a message to the queue
	})
}

func (c *Controller) Upload(ctx echo.Context) error {
	file, err := ctx.FormFile("file")
	if err != nil {
		return err
	}

	expirationTime, err := strconv.Atoi(ctx.FormValue("expirationTime"))
	if err != nil {
		return err
	}

	f, err := file.Open()
	if err != nil {
		return err
	}

	res, err := c.cld.Upload.Upload(ctx.Request().Context(), f, uploader.UploadParams{})
	if err != nil {
		return err
	}

	go handleExpirationTime(expirationTime)

	// TODO: publish the message from rmq
	queue, err := c.q.OpenQueue("device")
	if err != nil {
		return err
	}

	if err := queue.Publish("this is just a test payload"); err != nil {
		return err
	}

	return ctx.JSON(http.StatusOK, map[string]interface{}{
		"imageUrl": res.SecureURL,
	})
}

func (c *Controller) TestPublish(ctx echo.Context) error {
	queue, err := c.q.OpenQueue("device")
	if err != nil {
		return err
	}

	if err := queue.Publish("this is just a test payload"); err != nil {
		return err
	}

	return ctx.JSON(http.StatusOK, map[string]interface{}{
		"status": true,
	})
}

func NewController(e *echo.Echo, cld *cloudinary.Cloudinary, q *rmq.Connection) *Controller {
	return &Controller{
		e:   e,
		cld: cld,
		q:   *q,
	}
}
