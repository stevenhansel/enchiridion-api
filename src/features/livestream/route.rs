use std::time::Instant;

use actix::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::features::livestream;

pub async fn socket_handler(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<i32>,
    srv: web::Data<Addr<livestream::socket::LivestreamSocketServer>>,
) -> Result<HttpResponse, Error> {
    let device_id = path.into_inner();

    ws::start(
        livestream::session::LivestreamSocketSession {
            id: 0,
            device_id,
            hb: Instant::now(),
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}
