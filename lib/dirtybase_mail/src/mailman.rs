use std::collections::HashMap;

use crate::{adapter_manager::REGISTERED_ADAPTERS, email::Envelope, AdapterTrait, Email};

#[derive(Debug, Default)]
pub struct Mailman {
    pub(crate) to: Vec<String>,
    pub(crate) cc: Option<Vec<String>>,
    pub(crate) bcc: Option<Vec<String>>,
    pub(crate) headers: Option<HashMap<String, String>>,
    pub(crate) metadata: Option<HashMap<String, String>>,
    pub(crate) attachments: Option<HashMap<String, Vec<u8>>>,
    pub(crate) subject: Option<String>,
    pub(crate) mail: Option<Email>,
}

impl Mailman {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to(mut self, to: &str) -> Self {
        self.to.push(to.to_string());

        self
    }

    pub fn to_many<T>(mut self, to: Vec<T>) -> Self
    where
        T: Into<String>,
    {
        self.to.extend(to.into_iter().map(|s| s.into()));

        self
    }

    pub fn cc(mut self, cc: &str) -> Self {
        if self.cc.is_none() {
            self.cc = Some(Vec::new());
        }

        self.cc.as_mut().unwrap().push(cc.to_string());

        self
    }

    pub fn cc_many<T>(mut self, cc: Vec<T>) -> Self
    where
        T: Into<String>,
    {
        if self.cc.is_none() {
            self.cc = Some(Vec::new());
        }

        self.cc
            .as_mut()
            .unwrap()
            .extend(cc.into_iter().map(|s| s.into()));

        self
    }

    pub fn bcc(mut self, bcc: &str) -> Self {
        if self.bcc.is_none() {
            self.bcc = Some(Vec::new());
        }

        self.bcc.as_mut().unwrap().push(bcc.to_string());

        self
    }

    pub fn bcc_many<T>(mut self, bcc: Vec<T>) -> Self
    where
        T: Into<String>,
    {
        if self.cc.is_none() {
            self.bcc = Some(Vec::new());
        }

        self.bcc
            .as_mut()
            .unwrap()
            .extend(bcc.into_iter().map(|s| s.into()));

        self
    }

    pub fn header<V: Into<String>>(mut self, key: &str, value: V) -> Self {
        if self.headers.is_none() {
            self.headers = Some(HashMap::new());
        }

        self.headers
            .as_mut()
            .unwrap()
            .insert(key.to_string(), value.into());

        self
    }

    pub fn headers(mut self, entries: HashMap<String, String>) -> Self {
        if self.headers.is_none() {
            self.headers = Some(HashMap::new());
        }

        self.headers.as_mut().unwrap().extend(entries);

        self
    }

    pub fn subject(mut self, subject: &str) -> Self {
        self.subject = Some(subject.to_string());

        self
    }

    pub fn mail(mut self, mail: Email) -> Self {
        self.mail = Some(mail);

        self
    }

    pub async fn send(self) -> Result<bool, anyhow::Error> {
        if let Some(adapters) = REGISTERED_ADAPTERS.get() {
            let lock = adapters.read().await;
            return match lock.get("smtp") {
                Some(sender) => {
                    let envelope: Envelope = self.into();
                    sender.send(envelope).await
                }
                None => Err(anyhow::anyhow!("Adapter not found")),
            };
        }

        Err(anyhow::anyhow!("Adapters not register "))
    }
}
