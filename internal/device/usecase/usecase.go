package usecase

type Usecase struct {
	db DeviceRepositoryQuerier
}

func New(db DeviceRepositoryQuerier) *Usecase {
	return &Usecase{db: db}
}
