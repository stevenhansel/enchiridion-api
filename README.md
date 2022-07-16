# Enchiridion API

Requirements:
- rust
- docker
- sqlx-rs (https://github.com/launchbadge/sqlx)

### Development setup

1. Create .env from .env.example
```
cp .env.example .env
```

2. Run the postgres docker instance:

```
./scripts/postgres.sh
```

3.  Migrate the database with sqlx, note that sqlx migrate won't work if you don't have `DATABASE_URL` in the .env
```
sqlx migrate run --source database/migrations
```

4. Run the app
```
cargo run
```

### Database Migrations

Adding new migration file `sqlx migrate add --source database/migrations <migration_name>`

### Resetting the docker postgres volume

1. Stop pg docker instance
```
docker stop enchiridion-postgres
```

2. Remove the pg docker volume
```
docker volume rm enchiridion-postgres
```

3. Run pg docker instance
```
./scripts/postgres.sh
```

4. Re-run the migration
```
sqlx migrate run --source database/migrations
```

Prepare for deployment:

cargo sqlx prepare -- --lib
