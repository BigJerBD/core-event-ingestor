mod configs;
mod logging;
mod pubsub;
mod services;

use actix_web::{
    middleware, web, web::ServiceConfig, App, HttpServer,
};

use crate::{configs::GoogleConfig, configs::IngestorConfig};

use crate::services::todoist::TodoistConfig;

use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logging::set();

    let ingestor_config = envy::prefixed("EVENT_INGESTOR_")
        .from_env::<IngestorConfig>()
        .unwrap();

    let google_config = envy::prefixed("GOOGLE_")
        .from_env::<GoogleConfig>()
        .unwrap();

    let pubsub = pubsub::new(google_config).await;

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .configure(new_service_config(pubsub.clone()))
    })
    .bind(ingestor_config.host.to_owned())?
    .run()
    .await
}

fn new_service_config(
    pubsub: cloud_pubsub::Client,
) -> Box<dyn FnOnce(&mut ServiceConfig)> {
    let todoist_config = envy::prefixed("TODOIST_")
        .from_env::<TodoistConfig>()
        .unwrap();

    Box::new(move |cfg: &mut web::ServiceConfig| {
        cfg.service(
            web::scope("/todoist")
                .app_data(web::Data::new(Arc::new(
                    pubsub.topic(todoist_config.topic.clone()),
                )))
                .app_data(web::Data::new(todoist_config.clone()))
                .route(
                    "/webhook",
                    web::post().to(services::todoist::webhook),
                ),
        );
    })
}
