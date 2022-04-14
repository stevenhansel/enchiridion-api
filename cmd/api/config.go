package main

import "github.com/spf13/viper"

type Configuration struct {
	CloudinaryCloudName string `mapstructure:"CLOUDINARY_CLOUD_NAME"`
	CloudinaryApiKey    string `mapstructure:"CLOUDINARY_API_KEY"`
	CloudinaryApiSecret string `mapstructure:"CLOUDINARY_API_SECRET"`
}

func NewConfiguration(path string) (*Configuration, error) {
	var config Configuration

	viper.SetDefault("PORT", "8080")

	viper.AutomaticEnv()

	viper.AddConfigPath(path)
	viper.SetConfigFile(".env")

	err := viper.ReadInConfig()

	if err != nil {
		return nil, err
	}

	err = viper.Unmarshal(&config)

	if err != nil {
		return nil, err
	}

	return &config, nil
}
