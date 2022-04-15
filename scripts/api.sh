#!/bin/sh
docker run \
  --name enchridion-api \
  --rm \
  --network enchridion \
  --env-file .env \
  -p 8080:8080 \
  ghcr.io/stevenhansel/enchridion-api api -env staging
