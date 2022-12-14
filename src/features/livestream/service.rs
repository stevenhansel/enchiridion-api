use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, DurationRound, Utc};

use super::{
    definition::{
        DeviceLivestreamContent, DeviceLivestreamQueryResult, LivestreamInterval,
        LivestreamMessagePayload, LivestreamQueryAction, LivestreamRange,
    },
    error::{InsertLivestreamError, QueryLivestreamError},
    repository::LivestreamRepositoryInterface,
};

#[async_trait]
pub trait LivestreamServiceInterface: Send + Sync + 'static {
    async fn insert(&self, message: LivestreamMessagePayload) -> Result<(), InsertLivestreamError>;
    async fn query(
        &self,
        device_id: i32,
        action: LivestreamQueryAction,
        interval: LivestreamInterval,
        range: LivestreamRange,
    ) -> Result<DeviceLivestreamQueryResult, QueryLivestreamError>;
}

pub struct LivestreamService {
    _livestream_repository: Arc<dyn LivestreamRepositoryInterface>,
}

impl LivestreamService {
    pub fn new(_livestream_repository: Arc<dyn LivestreamRepositoryInterface>) -> Self {
        LivestreamService {
            _livestream_repository,
        }
    }
}

#[async_trait]
impl LivestreamServiceInterface for LivestreamService {
    async fn insert(&self, message: LivestreamMessagePayload) -> Result<(), InsertLivestreamError> {
        Ok(self._livestream_repository.insert(message).await?)
    }

    async fn query(
        &self,
        device_id: i32,
        action: LivestreamQueryAction,
        interval: LivestreamInterval,
        range: LivestreamRange,
    ) -> Result<DeviceLivestreamQueryResult, QueryLivestreamError> {
        #[allow(unused_assignments)]
        let mut plots = 60;

        #[allow(unused_assignments)]
        let mut truncate_duration = Duration::minutes(1);

        #[allow(unused_assignments)]
        let mut subtract_duration = Duration::minutes(1);

        if interval == LivestreamInterval::Minute && range == LivestreamRange::Hour {
            plots = 60;
            truncate_duration = Duration::minutes(1);
            subtract_duration = Duration::minutes(1);
        } else if interval == LivestreamInterval::Hour && range == LivestreamRange::Day {
            plots = 24;
            truncate_duration = Duration::hours(1);
            subtract_duration = Duration::hours(1);
        } else if interval == LivestreamInterval::Day && range == LivestreamRange::Week {
            plots = 7;
            truncate_duration = Duration::days(1);
            subtract_duration = Duration::days(1);
        } else {
            return Err(QueryLivestreamError::UnsupportedQuery)
        }

        let query_interval = interval.clone();
        let query_range = range.clone();

        let livestream_contents = match action {
            LivestreamQueryAction::Max => {
                self._livestream_repository
                    .query_max_num_of_faces(device_id, query_interval, query_range)
                    .await?
            }
            LivestreamQueryAction::Average => {
                self._livestream_repository
                    .query_avg_num_of_faces(device_id, query_interval, query_range)
                    .await?
            }
        };

        let mut contents: Vec<DeviceLivestreamContent> = Vec::new();

        let mut timestamps: Vec<chrono::DateTime<chrono::Utc>> = Vec::with_capacity(plots);
        let mut now = Utc::now().duration_trunc(truncate_duration).unwrap();

        for i in 0..plots {
            if i == 0 {
                timestamps.push(now);
            } else {
                now -= subtract_duration;
                timestamps.push(now);
            }
        }

        for timestamp in timestamps {
            if let Some(ts) = livestream_contents
                .iter()
                .find(|c| c.timestamp == timestamp)
            {
                contents.push(DeviceLivestreamContent {
                    timestamp: ts.timestamp,
                    value: ts.value,
                })
            } else {
                contents.push(DeviceLivestreamContent {
                    timestamp,
                    value: 0.0,
                })
            }
        }

        Ok(DeviceLivestreamQueryResult {
            action,
            interval,
            range,
            contents,
        })
    }
}
