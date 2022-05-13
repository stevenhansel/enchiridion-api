package usecases

import (
	"context"
	"encoding/json"
	"fmt"
	"mime/multipart"
	"strings"

	"github.com/cloudinary/cloudinary-go/api/uploader"

	announcementRepository "github.com/stevenhansel/enchiridion-api/internal/announcement/repository"
)

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

func (u *Usecase) ListAnnouncement(ctx context.Context) ([]*announcementRepository.Announcement, error) {
	return u.db.Find(ctx)
}

func (u *Usecase) AnnouncementDetail(ctx context.Context, id int) (*announcementRepository.Announcement, error) {
	return nil, nil
}

type CreateAnnouncementParams struct {
	Title     string
	Media     *multipart.FileHeader
	Duration  int
	Notes     string
	DeviceIDs []int
}

func (u *Usecase) CreateAnnouncement(ctx context.Context, params *CreateAnnouncementParams) error {
	m, err := params.Media.Open()
	if err != nil {
		return err
	}

	res, err := u.cloudinary.Upload.Upload(ctx, m, uploader.UploadParams{})
	if err != nil {
		return err
	}

	duration := 60 * 60 * 24 * params.Duration
	contentType := params.Media.Header.Get("Content-Type")
	fileExt := strings.Split(contentType, "/")[1]

	err = u.db.Insert(ctx, &announcementRepository.InsertAnnouncementParams{
		Title:     params.Title,
		Media:     res.SecureURL,
		Filename:  fmt.Sprintf("%s.%s", res.AssetID, fileExt),
		Duration:  duration,
		Notes:     params.Notes,
		DeviceIDs: params.DeviceIDs,
	})
	if err != nil {
		return err
	}

	return nil
}

func (u *Usecase) UpdateAnnouncementApproval(ctx context.Context, id int, approve bool) error {
	announcement, err := u.db.FindOne(ctx, id)
	if err != nil {
		return err
	}

	devices, err := u.device.ListDevicesByAnnouncementID(ctx, id)
	if err != nil {
		return err
	}

	if len(devices) == 0 {
		return NewErrAnnouncementHasNoDevice(fmt.Errorf("Announcement does not have any devices"))
	}

	var status announcementRepository.AnnouncementStatus

	if approve {
		status = announcementRepository.WaitingForSync
	} else {
		status = announcementRepository.Rejected
	}

	if err := u.db.UpdateApprovalStatus(ctx, id, status); err != nil {
		return err
	}

	expirationQueue, err := u.rmq.OpenQueue(getExpirationQueueName())
	if err != nil {
		return nil
	}

	for _, device := range devices {
		syncQueue, err := u.rmq.OpenQueue(getSyncQueueName(device.ID))
		if err != nil {
			return err
		}

		syncJobPayload, err := json.Marshal(SyncJobPayload{
			Operation: "append",
			URL:       announcement.Media,
			Filename:  announcement.Filename,
		})
		if err != nil {
			return err
		}

		// This should be handled in the consumer coming from the device itself

		// TODO: refactor this to expiredAt
		expirationJobPayload, err := json.Marshal(ExpirationJobPayload{
			URL:            announcement.Media,
			Filename:       announcement.Filename,
			DeviceID:       device.ID,
			ExpirationTime: announcement.Duration,
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
	}

	return nil
}

func getSyncQueueName(deviceID int) string {
	return fmt.Sprintf("sync-device-%d", deviceID)
}
func getExpirationQueueName() string {
	return "expiration-device"
}
