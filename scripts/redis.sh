#!/bin/sh
docker run \
  --name enchiridion-redis \
  --network enchiridion \
  --volume enchiridion-redis:/data \
  -p 6379:6379 \
  --detach \
  --rm \
  redis redis-server --requirepass 15f699f37f0c30f2ec051cf2ea72f66055d525d5c6d44cea467e5ff0618fb2f3
