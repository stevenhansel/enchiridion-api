package container

type Container struct {
	Internal
	Repository
	Usecase
	Presentation
}

func New() (*Container, error) {
	internal, err := createInternalLayer()
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
		Internal: internal,
		Repository: repository,
		Usecase: usecase,
		Presentation: presentation,
	}, nil
}
