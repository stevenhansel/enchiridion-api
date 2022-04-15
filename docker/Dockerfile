# Stage 0
FROM golang:alpine AS build

RUN apk update && \
    apk add curl \
            git \
            bash \
            make \
            ca-certificates && \
    rm -rf /var/cache/apk/*

WORKDIR /app

COPY go.* ./
RUN go mod download
RUN go mod verify

COPY . .
RUN make build

# Stage 1
FROM alpine:latest

RUN apk --no-cache add ca-certificates bash

WORKDIR /app/

COPY --from=build /app/bin/api .
COPY --from=build /app/bin/expired-announcement-consumer .
COPY --from=build /app/docker/entrypoint.sh . 

RUN ls -la

ENTRYPOINT ["./entrypoint.sh"]
