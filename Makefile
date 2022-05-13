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

# Migration
.PHONY: migration/up
migration/up:
	go run cmd/database/*.go -env $(env) -command up

.PHONY: migration/down
migration/down:
	go run cmd/database/*.go -env $(env) -command down

.PHONY: migration/create
migration/create:
	migrate create -seq -ext=.sql -dir=./database/migrations $(name)

.PHONY: migration/build
migration/build:
	go build -o bin/migration cmd/database/*.go

