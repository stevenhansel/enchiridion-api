package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"

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

type SyncJobPayload struct {
	Operation string `json:"operation"`
	URL       string `json:"imageUrl"`
	Filename  string `json:"filename"`
}

type ExpirationJobPayload struct {
	URL            string `json:"imageUrl"`
	Filename       string `json:"filename"`
	DeviceID       int    `json:"deviceId"`
	ExpirationTime int    `json:"expirationTime"`
}

func getSyncQueueName(deviceID int) string {
	return fmt.Sprintf("sync-device-%d", deviceID)
}
func getExpirationQueueName() string {
	return "expiration-device"
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

	deviceID, err := strconv.Atoi(ctx.FormValue("deviceId"))
	if err != nil {
		return err
	}

	filename := ctx.FormValue("filename")

	f, err := file.Open()
	if err != nil {
		return err
	}

	res, err := c.cld.Upload.Upload(ctx.Request().Context(), f, uploader.UploadParams{})
	if err != nil {
		return err
	}

	syncQueue, err := c.q.OpenQueue(getSyncQueueName(deviceID))
	if err != nil {
		return err
	}

	expirationQueue, err := c.q.OpenQueue(getExpirationQueueName())
	if err != nil {
		return err
	}

	syncJobPayload, err := json.Marshal(SyncJobPayload{
		Operation: "append",
		URL:       res.SecureURL,
		Filename:  filename,
	})
	if err != nil {
		return err
	}

	expirationJobPayload, err := json.Marshal(ExpirationJobPayload{
		URL:            res.SecureURL,
		Filename:       filename,
		DeviceID:       deviceID,
		ExpirationTime: expirationTime,
	})
	if err != nil {
		return err
	}

	if err := syncQueue.Publish(string(syncJobPayload)); err != nil {
		return err
	}
	if err := expirationQueue.Publish(string(expirationJobPayload)); err != nil {
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

	if err := queue.Publish("just a test payload"); err != nil {
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
