package main

import (
	"reflect"

	"github.com/spf13/viper"
)

type Configuration struct {
	CloudinaryCloudName string `mapstructure:"CLOUDINARY_CLOUD_NAME"`
	CloudinaryApiKey    string `mapstructure:"CLOUDINARY_API_KEY"`
	CloudinaryApiSecret string `mapstructure:"CLOUDINARY_API_SECRET"`

	REDIS_QUEUE_ADDR string `mapstructure:"REDIS_QUEUE_ADDR"`
}

func InitializeConfiguration(config *Configuration) {
	fields := reflect.VisibleFields(reflect.TypeOf(*config))

	for _, field := range fields {
		viper.SetDefault(field.Name, reflect.Zero(field.Type))
	}
}

func NewConfiguration(environment Environment, path string) (*Configuration, error) {
	var config Configuration

	if environment == DEVELOPMENT {
		viper.AddConfigPath(path)
		viper.SetConfigFile(".env")

		err := viper.ReadInConfig()

		if err != nil {
			return nil, err
		}
	} else {
		InitializeConfiguration(&config)
	}

	viper.AutomaticEnv()

	err := viper.Unmarshal(&config)

	if err != nil {
		return nil, err
	}

	return &config, nil
}
