use actix_web::HttpResponse;

pub async fn list_role() -> HttpResponse {
    HttpResponse::NoContent().finish()
}
