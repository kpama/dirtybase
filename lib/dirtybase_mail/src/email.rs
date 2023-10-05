use std::collections::HashMap;

use crate::Mailman;

#[derive(Debug, Default, Clone)]
pub struct Email {
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub from: Option<String>,
    pub reply_to: Option<String>,
    pub subject: String,
    pub body: String,
    pub adapter_config: Option<HashMap<String, String>>,
}

impl Email {
    pub async fn send(self) -> Result<bool, anyhow::Error> {
        Ok(true)
    }

    pub fn dispatch(self) {
        // TODO: Implement
    }
}

#[derive(Debug, Default)]
pub struct Envelope {
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub headers: Option<HashMap<String, String>>,
    pub metadata: Option<HashMap<String, String>>,
    pub attachments: Option<HashMap<String, Vec<u8>>>,
    pub subject: Option<String>,
    pub mail: Option<Email>,
}

impl From<Mailman> for Envelope {
    fn from(value: Mailman) -> Self {
        Self {
            to: value.to,
            cc: value.cc,
            bcc: value.bcc,
            headers: value.headers,
            metadata: value.metadata,
            attachments: value.attachments,
            subject: value.subject,
            mail: value.mail,
        }
    }
}

#[derive(Debug, Default)]
pub struct EmailBuilder {
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub from: Option<String>,
    pub reply_to: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub adapter_config: Option<HashMap<String, String>>,
}

impl EmailBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Email {
        Email::default()
    }
}
