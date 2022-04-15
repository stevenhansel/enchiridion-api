image_tag := latest

.PHONY: run
run: 
	go run cmd/api/*.go -env development

.PHONY: build
build:
	go build -o bin/api cmd/api/*.go
	go build -o bin/expired-announcement-consumer cmd/expired-announcement-consumer/*.go

.PHONY: docker/build
docker/build:
	docker build -f ./docker/Dockerfile -t ghcr.io/stevenhansel/enchridion-api:$(image_tag) .	

.PHONY: docker/run
docker/run:
	./scripts/api.sh
