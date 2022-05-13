#!/bin/bash
docker run \
  --name enchiridion-db \
  --network enchiridion \
  --volume enchiridion-db-data:/var/lib/postgresql/data \
  -p 5432:5432 \
  --rm \
  --detach \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=enchiridion \
  postgres
