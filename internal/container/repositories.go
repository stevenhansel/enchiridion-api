package container

type Repository struct {}

func createRepositoryLayer() (Repository, error) {
	return Repository{}, nil
}
