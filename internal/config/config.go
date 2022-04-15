package config

import (
	"reflect"

	"github.com/spf13/viper"
)

type Configuration struct {
	Port int `mapstructure:"PORT"`

	CloudinaryCloudName string `mapstructure:"CLOUDINARY_CLOUD_NAME"`
	CloudinaryApiKey    string `mapstructure:"CLOUDINARY_API_KEY"`
	CloudinaryApiSecret string `mapstructure:"CLOUDINARY_API_SECRET"`

	RedisQueueAddr string `mapstructure:"REDIS_QUEUE_ADDR"`
}

func initializeDevelopmentConfig() error {
	viper.AutomaticEnv()

	viper.AddConfigPath(".")
	viper.SetConfigFile(".env")

	err := viper.ReadInConfig()

	if err != nil {
		return err
	}

	return nil
}

func initializeProductionConfig(config *Configuration) error {
	fields := reflect.VisibleFields(reflect.TypeOf(*config))

	for _, field := range fields {
		viper.SetDefault(field.Tag.Get("mapstructure"), reflect.Zero(field.Type))
	}

	viper.AutomaticEnv()

	return nil
}

func NewConfiguration(environment Environment) (*Configuration, error) {
	var config Configuration

	if environment == DEVELOPMENT {
		if err := initializeDevelopmentConfig(); err != nil {
			return nil, err
		}
	} else {
		if err := initializeProductionConfig(&config); err != nil {
			return nil, err
		}
	}

	err := viper.Unmarshal(&config)

	if err != nil {
		return nil, err
	}

	return &config, nil
}
