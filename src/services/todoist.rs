use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

//#[api_v2_operation]
#[post("/todoist/webhook")]
pub async fn webhook(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
