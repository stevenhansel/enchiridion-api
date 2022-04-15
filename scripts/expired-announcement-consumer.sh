#!/bin/sh
docker run \
  --name enchridion-api \
  --rm \
  --network enchridion \
  --env-file .env \
  ghcr.io/stevenhansel/enchridion-api expired-announcement-consumer -env staging
