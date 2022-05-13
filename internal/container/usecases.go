package container

type Usecase struct {}

func createUsecaseLayer() (*Usecase, error) {
	return &Usecase{}, nil
}
