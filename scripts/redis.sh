#!/bin/sh
#!/bin/bash
docker run \
  --name enchridion-redis \
  --volume enchridion-redis-data:/data \
  --network enchridion \
  -p 6379:6379 \
  --rm \
  --detach \
  redis
