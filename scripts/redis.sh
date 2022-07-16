#!/bin/sh
docker run \
  --name enchiridion-redis \
  --network enchiridion \
  --volume enchiridion-redis:/data \
  -p 6379:6379 \
  --detach \
  --rm \
  redis
