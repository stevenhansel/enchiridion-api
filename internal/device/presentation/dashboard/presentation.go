package presentation

type Presentation struct {
	device DeviceUsecaseQuerier
}

func New(device DeviceUsecaseQuerier) *Presentation {
	return &Presentation{device: device}
}
