use std::collections::HashMap;

use async_trait::async_trait;
use secrecy::{ExposeSecret, Secret};

pub enum EmailError {
    NetworkError,
}

impl From<reqwest::Error> for EmailError {
    fn from(_: reqwest::Error) -> Self {
        EmailError::NetworkError
    }
}

#[derive(Default)]
pub struct EmailParams {
    from: String,
    to: String,
    subject: String,
    html: String,
}

#[async_trait]
pub trait EmailClient {
    async fn send(&self, params: EmailParams) -> Result<(), EmailError>;
}

pub struct Client {
    _email: Box<dyn EmailClient + Send + Sync>,
}

impl Client {
    pub fn new(email: Box<dyn EmailClient + Send + Sync>) -> Client {
        Client { _email: email }
    }

    pub async fn send(&self, params: EmailParams) -> Result<(), EmailError> {
        self._email.send(params).await
    }
}

pub struct MailgunAdapter {
    _client: reqwest::Client,

    base_url: String,
    domain: Secret<String>,
    api_key: Secret<String>,
}

impl MailgunAdapter {
    pub fn new(base_url: String, domain: Secret<String>, api_key: Secret<String>) -> MailgunAdapter {
        let _client = reqwest::Client::new();

        MailgunAdapter {
            _client,
            base_url,
            domain,
            api_key,
        }
    }
}

#[async_trait]
impl EmailClient for MailgunAdapter {
    async fn send(&self, params: EmailParams) -> Result<(), EmailError> {
        let mut form = HashMap::new();

        form.insert("from", params.from.to_owned());
        form.insert("to", params.to.to_owned());
        form.insert("subject", params.subject.to_owned());
        form.insert("html", params.html.to_owned());

        let url = format!("{}/{}/messages", self.base_url, self.domain.expose_secret());

        self._client
            .post(&url)
            .basic_auth("api", Some(self.api_key.expose_secret().clone()))
            .form(&form)
            .send()
            .await?;

        Ok(())
    }
}
