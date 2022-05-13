package container

import (
	"github.com/adjust/rmq/v4"
	"github.com/cloudinary/cloudinary-go"

	"github.com/stevenhansel/enchridion-api/internal/config"
)

type Internal struct {
	Config     *config.Configuration
	Cloudinary *cloudinary.Cloudinary
	Rmq        rmq.Connection
}

func createInternalLayer(env config.Environment) (*Internal, error) {
	config, err := config.New(env)
	if err != nil {
		return nil, err
	}

	cloudinary, err := cloudinary.NewFromParams(
		config.CloudinaryCloudName,
		config.CloudinaryApiKey,
		config.CloudinaryApiSecret,
	)
	if err != nil {
		return nil, err
	}

	rmq, err := rmq.OpenConnection(
		"producer",
		"tcp",
		config.RedisQueueAddr,
		1,
		nil,
	)
	if err != nil {
		return nil, err
	}

	return &Internal{
		Config:     config,
		Cloudinary: cloudinary,
		Rmq:        rmq,
	}, nil
}
