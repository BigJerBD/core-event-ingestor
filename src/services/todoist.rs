use actix_web::web::Bytes;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use cloud_pubsub::{EncodedMessage, Topic};
use data_encoding::BASE64;

use ring::hmac;
use serde::{Deserialize, Serialize};
use serde_json;

use reqwest::header::AUTHORIZATION;
use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use log::debug;

#[derive(Deserialize, Clone)]
pub struct TodoistConfig {
    pub client_id: String,
    pub client_secret: String,
    pub access_token: String,
    #[serde(default = "default_topic")]
    pub topic: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoistEvent {
    user_id: String,
    version: String,
    initiator: serde_json::Value,
    event_name: String,
    event_data: TodoistEventData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoistEventData {
    id: String,
    parent_id: Option<String>,
    project_id: Option<String>,
    section_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoistProject {
    id: String,
    name: String,
    parent_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TodoistSection {
    id: String,
    name: String,
}

pub async fn webhook(
    req: HttpRequest,
    body: Bytes,
    topic: web::Data<Arc<Topic>>,
    config: web::Data<TodoistConfig>,
) -> impl Responder {
    authorize_request(&body, req, config.client_secret.clone())
        .unwrap();

    // For simplicity, TodoistEvent only contains only some data
    //  payload is used for the complete publishing.
    let event: TodoistEvent = serde_json::from_slice(&body).unwrap();
    let payload: serde_json::Value =
        serde_json::from_slice(&body).unwrap();

    let projects = get_projects(&config).await.unwrap();

    debug!("event: {:?}", event);

    let cur_project = match event.event_data.project_id {
        None => None,
        Some(project_id) => match &projects
            .iter()
            .find(|project| project.id == project_id)
        {
            None => None,
            Some(project) => Some(<&TodoistProject>::clone(project)),
        },
    };

    debug!("project: {:?}", cur_project);

    let cur_project_name = match &cur_project {
        None => "".to_string(),
        Some(project) => project.name.clone(),
    };

    let cur_parent = match &cur_project {
        None => None,
        Some(cur_project) => match &projects.iter().find(|project| {
            Some(project.id.clone()) == cur_project.parent_id
        }) {
            None => None,
            Some(project) => Some(project.clone()),
        },
    };
    let cur_parent_name = match &cur_parent {
        None => "".to_string(),
        Some(project) => project.name.clone()
    };

    let cur_parent_parent_name = match &cur_parent {
        None => "".to_string(),
        Some(cur_project) => match &projects.iter().find(|project| {
            Some(project.id.clone()) == cur_project.parent_id
        }) {
            None => "".to_string(),
            Some(project) => project.name.clone(),
        },
    };

    let cur_section_name = if event.event_name.starts_with("section:") {
        match get_section(event.event_data.id.clone(), &config).await {
            Ok(section) => section.name.clone(),
            _ => "".to_string(),
        }
    } else {
         match &event.event_data.section_id {
            None => "".to_string(),
            Some(section_id) => {
                match get_section(section_id.clone(), &config).await {
                    Ok(section) => section.name.clone(),
                    _ => "".to_string(),
                }
            }
        }
    };

    log::info!(
        "message published: event_name={}, project_name={}, parent_name={}, parent_parent_name={}, section_name={}",
        &event.event_name,
        &cur_project_name,
        &cur_parent_name,
        &cur_parent_parent_name,
        &cur_section_name
    );

    topic
        .clone()
        .publish_message(EncodedMessage::new(
            &payload.clone(),
            Some(HashMap::from([
                ("event_name".to_string(), event.event_name.clone()),
                ("project_name".to_string(), cur_project_name),
                ("parent_name".to_string(), cur_parent_name),
                ("parent_parent_name".to_string(), cur_parent_parent_name),
                ("section_name".to_string(), cur_section_name),
            ])),
            Some(event.event_data.id.clone())
        ))
        .await
        .unwrap();

    HttpResponse::Ok()
}

async fn get_projects(
    config: &TodoistConfig,
) -> Result<Vec<TodoistProject>> {
    Ok(reqwest::Client::new()
        .get("https://api.todoist.com/rest/v2/projects")
        .header(
            AUTHORIZATION,
            format!("Bearer {}", config.access_token),
        )
        .send()
        .await?
        .json()
        .await?)
}

async fn get_section(
    id: String,
    config: &TodoistConfig,
) -> Result<TodoistSection> {
    Ok(reqwest::Client::new()
        .get(format!(
            "https://api.todoist.com/rest/v2/sections/{}",
            id
        ))
        .header(
            AUTHORIZATION,
            format!("Bearer {}", config.access_token),
        )
        .send()
        .await?
        .json()
        .await?)
}

fn authorize_request(
    body: &[u8],
    request: HttpRequest,
    client_secret: String,
) -> Result<()> {
    let signature = request
        .headers()
        .get("X-Todoist-HMAC-SHA256")
        .ok_or(anyhow!("Missing header."))?
        .to_str()?;

    let key_value = client_secret.as_bytes();
    let key = hmac::Key::new(hmac::HMAC_SHA256, &key_value);

    let hash = BASE64.encode(hmac::sign(&key, &body).as_ref());

    if hash == String::from(signature) {
        Ok(())
    } else {
        Err(anyhow!("Invalid Signature."))
    }
}

fn default_topic() -> String {
    "todoist".to_string()
}
