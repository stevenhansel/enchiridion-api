# Enchiridion API

```
docker run \
	--name enchiridion-api-db \
	--volume enchiridion-api-data:/var/lib/postgresql/data \
	-p 5432:5432 \
	--rm \
	--detach \
	-e POSTGRES_USER=postgres \
	-e POSTGRES_PASSWORD=postgres \
	-e POSTGRES_DB=enchiridion \
	postgres
```
