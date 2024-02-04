use crate::error;
use base64::{self, Engine};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Clone, Serialize)]
pub struct EncodedMessage {
    data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    attributes: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename= "orderingKey")]
    ordering_key: Option<String>,
}

pub trait FromPubSubMessage
where
    Self: std::marker::Sized,
{
    fn from(message: EncodedMessage) -> Result<Self, error::Error>;
}

impl EncodedMessage {
    pub fn decode(&self) -> Result<Vec<u8>, base64::DecodeError> {
        base64::engine::general_purpose::STANDARD.decode(&self.data)
    }

    pub fn attributes(&self) -> Option<&HashMap<String, String>> {
        self.attributes.as_ref()
    }

    pub fn new<T: serde::Serialize>(
        data: &T,
        attributes: Option<HashMap<String, String>>,
        ordering_key: Option<String>
    ) -> Self {
        let json = serde_json::to_string(data).unwrap();
        Self::new_binary(&json, attributes, ordering_key)
    }

    pub fn new_binary<T: AsRef<[u8]> + std::marker::Sync>(
        incoming: &T,
        attributes: Option<HashMap<String, String>>,
        ordering_key: Option<String>
    ) -> Self {
        let data = base64::engine::general_purpose::STANDARD.encode(&incoming);
        EncodedMessage { data, attributes, ordering_key}
    }
}

#[derive(Deserialize)]
pub(crate) struct Message {
    #[serde(alias = "ackId")]
    pub(crate) ack_id: String,
    pub(crate) message: EncodedMessage,
}
