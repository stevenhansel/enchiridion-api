use std::sync::Arc;

use actix_web::{
    web::{self, Query, Path},
    HttpResponse,
};
use serde::Deserialize;

use crate::{
    features::livestream::error::LivestreamErrorCode,
    http::{
        derive_authentication_middleware_error, derive_user_id, AuthenticationContext,
        HttpErrorResponse,
    },
};

use super::{
    definition::{LivestreamInterval, LivestreamQueryAction, LivestreamRange},
    error::QueryLivestreamError,
    service::LivestreamServiceInterface,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LivestreamQueryParams {
    pub action: LivestreamQueryAction,
    pub interval: LivestreamInterval,
    pub range: LivestreamRange,
}

pub async fn livestream(
    livestream_service: web::Data<Arc<dyn LivestreamServiceInterface>>,
    auth: AuthenticationContext,
    query_params: Query<LivestreamQueryParams>,
    device_id: Path<i32>,
) -> HttpResponse {
    if let Err(e) = derive_user_id(auth) {
        return derive_authentication_middleware_error(e);
    }

    let device_id = device_id.into_inner();

    let result = match livestream_service
        .query(
            device_id,
            query_params.action.clone(),
            query_params.interval.clone(),
            query_params.range.clone(),
        )
        .await
    {
        Ok(result) => result,
        Err(e) => match e {
            QueryLivestreamError::UnsupportedQuery => {
                return HttpResponse::BadRequest().json(HttpErrorResponse::new(
                    LivestreamErrorCode::UnsupportedQuery.to_string(),
                    vec![e.to_string()],
                ))
            },
            QueryLivestreamError::DatabaseError(e) => {
                return HttpResponse::InternalServerError().json(HttpErrorResponse::new(
                    LivestreamErrorCode::DatabaseError.to_string(),
                    vec![e.to_string()],
                ))
            }
        },
    };

    HttpResponse::Ok().json(result)
}
