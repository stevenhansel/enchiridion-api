#!/bin/sh
docker run \
  --name enchiridion-postgres \
  --network enchiridion \
  --volume enchiridion-postgres:/var/lib/postgresql/data \
  -p 5432:5432 \
  --rm \
  --detach \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=enchiridion \
  -e POSTGRES_HOST_AUTH_METHOD=trust \
  postgres
