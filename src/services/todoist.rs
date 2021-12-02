use actix_web::web::Json;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize)]
struct TodoistEvent {
    user_id: u64,
    version: String,
    initiator: serde_json::Value,
    event_name: String,
    event_data: serde_json::Map<String, serde_json::Value>,
}

//#[api_v2_operation]
#[post("/todoist/webhook")]
pub async fn webhook(event: web::Json<TodoistEvent>) -> impl Responder {
    //todo validate HMAC
    let project_id = event
        .event_data
        .get("project_id")
        .unwrap()
        .as_u64()
        .unwrap();

    HttpResponse::Ok()
}
