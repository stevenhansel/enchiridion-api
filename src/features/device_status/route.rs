use std::time::Instant;

use actix::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::features::device_status;

pub async fn socket_handler(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<i32>,
    srv: web::Data<Addr<device_status::socket::StatusSocketServer>>,
) -> Result<HttpResponse, Error> {
    let device_id = path.into_inner();

    ws::start(
        device_status::session::StatusSocketSession {
            id: 0,
            device_id,
            hb: Instant::now(),
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}
