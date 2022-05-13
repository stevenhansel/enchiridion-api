package container

import "github.com/stevenhansel/enchiridion-api/internal/config"

type Container struct {
	Internal
	Repository
	Usecase
	Presentation
}

func New(env config.Environment) (*Container, error) {
	internal, err := createInternalLayer(env)
	if err != nil {
		return nil, err
	}

	repository, err := createRepositoryLayer(internal)
	if err != nil {
		return nil, err
	}

	usecase, err := createUsecaseLayer(internal, repository)
	if err != nil {
		return nil, err
	}

	presentation, err := createPresentationLayer(internal, repository, usecase)
	if err != nil {
		return nil, err
	}

	return &Container{
		Internal: *internal,
		Repository: *repository,
		Usecase: *usecase,
		Presentation: *presentation,
	}, nil
}
