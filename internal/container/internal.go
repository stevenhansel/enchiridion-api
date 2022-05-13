package container

type Internal struct {}

func createInternalLayer() (Internal, error) {
	return Internal{}, nil
}
