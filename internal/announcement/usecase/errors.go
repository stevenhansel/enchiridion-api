package usecases

type (
	ErrAnnouncementHasNoDevice struct{ error }
)

func (e ErrAnnouncementHasNoDevice) Unwrap() error { return e.error }

func NewErrAnnouncementHasNoDevice(err error) ErrAnnouncementHasNoDevice {
	return ErrAnnouncementHasNoDevice{err}
}
