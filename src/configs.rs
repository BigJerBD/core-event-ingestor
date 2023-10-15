use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct IngestorConfig {
    #[serde(default = "default_host")]
    pub host: String,
}

#[derive(Deserialize, Clone)]
pub struct GoogleConfig {
    pub application_credentials: String,
}

fn default_host() -> String {
    "0.0.0.0:8080".to_string()
}
