package container

import "github.com/stevenhansel/enchridion-api/internal/config"

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

	repository, err := createRepositoryLayer()
	if err != nil {
		return nil, err
	}

	usecase, err := createUsecaseLayer()
	if err != nil {
		return nil, err
	}

	presentation, err := createPresentationLayer()
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
