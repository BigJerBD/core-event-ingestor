mod services;

use actix_web::{middleware, web, web::ServiceConfig, App, HttpResponse, HttpServer};

use crate::services::todoist;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if !std::env::vars().any(|(k, _)| k == "RUST_LOG") {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let service_host = std::env::var("EVENT_INGESTOR_HOST").unwrap_or("0.0.0.0:8080".to_owned());

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(config)
    })
    .run()
    .await
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(todoist::webhook);
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use serde_json::json;

    #[actix_rt::test]
    async fn test_posts_todoist_webhook() {
        let mut app = test::init_service(App::new().configure(config)).await;

        let data = json!({
            "event_data": {
              "added_by_uid": 12345678,
              "assigned_by_uid": null,
              "checked": 1,
              "child_order": 53,
              "collapsed": 0,
              "content": "set recycle day",
              "date_added": "2021-12-01T23:05:51Z",
              "date_completed": "2021-12-02T01:55:25Z",
              "description": "",
              "due": {
                "date": "2021-12-01",
                "is_recurring": false,
                "lang": "fr",
                "string": "Déc 1",
                "timezone": null
              },
              "id": 1234565677,
              "in_history": 1,
              "is_deleted": 0,
              "labels": [],
              "parent_id": null,
              "priority": 1,
              "project_id": 1234565678,
              "responsible_uid": null,
              "section_id": null,
              "sync_id": null,
              "url": "https://todoist.com/showTask?id=5379261242",
              "user_id": 12345678
            },
            "event_name": "item:completed",
            "initiator": {
              "email": "abcd@gmail.com",
              "full_name": "Adrian Bigras Coddy Dunberry",
              "id": 12345678,
              "image_id": "bb00b9766c9243b7b445cddb3537a222",
              "is_premium": true
            },
            "user_id": 12345678,
            "version": "8"
        });

        let req = test::TestRequest::post()
            .uri("/todoist/webhook")
            .set_json(&data)
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());
    }
}
