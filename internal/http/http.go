package http

import "github.com/stevenhansel/enchridion-api/internal/container"

type Server struct {
	*container.Container
}

func (s *Server) Routes() {}

func (s *Server) Serve() {}

func New(container *container.Container) *Server {
	return &Server{container}
}
