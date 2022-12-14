use async_trait::async_trait;
use sqlx::{postgres::PgRow, Pool, Postgres, Row};

use super::definition::{
    DeviceLivestreamContent, LivestreamInterval, LivestreamMessagePayload, LivestreamRange,
};

#[async_trait]
pub trait LivestreamRepositoryInterface: Send + Sync + 'static {
    async fn insert(&self, message: LivestreamMessagePayload) -> Result<(), sqlx::Error>;
    async fn query_max_num_of_faces(
        &self,
        device_id: i32,
        interval: LivestreamInterval,
        range: LivestreamRange,
    ) -> Result<Vec<DeviceLivestreamContent>, sqlx::Error>;
    async fn query_avg_num_of_faces(
        &self,
        device_id: i32,
        interval: LivestreamInterval,
        range: LivestreamRange,
    ) -> Result<Vec<DeviceLivestreamContent>, sqlx::Error>;
}

pub struct LivestreamRepository {
    _db: Pool<Postgres>,
}

impl LivestreamRepository {
    pub fn new(_db: Pool<Postgres>) -> Self {
        LivestreamRepository { _db }
    }
}

#[async_trait]
impl LivestreamRepositoryInterface for LivestreamRepository {
    async fn insert(&self, message: LivestreamMessagePayload) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
                insert into "device_livestream"
                ("time", "device_id", "num_of_faces")
                values ($1, $2, $3)
            "#,
        )
        .bind(message.timestamp)
        .bind(message.device_id)
        .bind(message.num_of_faces)
        .execute(&self._db)
        .await?;

        Ok(())
    }

    async fn query_max_num_of_faces(
        &self,
        device_id: i32,
        interval: LivestreamInterval,
        range: LivestreamRange,
    ) -> Result<Vec<DeviceLivestreamContent>, sqlx::Error> {
        let result = sqlx::query(
            r#"
                select
                    time_bucket($1::interval, time) as bucket,
                    cast(max(num_of_faces) as float8) as value
                from "device_livestream"
                where time > now() - $2::interval and device_id = $3
                group by bucket
                order by bucket;
                "#,
        )
        .bind(interval.to_string())
        .bind(range.to_string())
        .bind(device_id)
        .map(|row: PgRow| DeviceLivestreamContent {
            timestamp: row.get("bucket"),
            value: row.get("value"),
        })
        .fetch_all(&self._db)
        .await?;

        Ok(result)
    }

    async fn query_avg_num_of_faces(
        &self,
        device_id: i32,
        interval: LivestreamInterval,
        range: LivestreamRange,
    ) -> Result<Vec<DeviceLivestreamContent>, sqlx::Error> {
        let result = sqlx::query(
            r#"
                select
                    time_bucket($1::interval, time) as bucket,
                    cast(avg(num_of_faces) as float8) as value
                from "device_livestream"
                where time > now() - $2::interval and device_id = $3
                group by bucket
                order by bucket;
                "#,
        )
        .bind(interval.to_string())
        .bind(range.to_string())
        .bind(device_id)
        .map(|row: PgRow| DeviceLivestreamContent {
            timestamp: row.get("bucket"),
            value: row.get("value"),
        })
        .fetch_all(&self._db)
        .await?;

        Ok(result)
    }
}
