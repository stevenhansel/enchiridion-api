#!/bin/sh
docker run \
  --name enchiridion-redis \
  --volume enchiridion-redis-data:/data \
  --network enchiridion \
  -p 6379:6379 \
  --rm \
  --detach \
  redis
