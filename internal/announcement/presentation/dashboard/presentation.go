package presentation

type Presentation struct {
	announcement AnnouncementUsecaseQuerier

}

func New(announcement AnnouncementUsecaseQuerier) *Presentation {
	return &Presentation{
		announcement: announcement,
	}
}
