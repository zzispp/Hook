use std::collections::HashMap;

use reqwest::{RequestBuilder, Url};
use serde::de::DeserializeOwned;

use crate::ClientError;

#[derive(Clone, Debug)]
pub struct ReqwestClient {
    client: reqwest::Client,
}

impl ReqwestClient {
    fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub fn from_builder(builder: reqwest::ClientBuilder) -> Result<Self, ClientError> {
        builder.build().map(Self::new).map_err(ClientError::from)
    }

    pub fn get(&self, url: Url) -> RequestBuilder {
        self.client.get(url)
    }

    pub fn post(&self, url: String) -> RequestBuilder {
        self.client.post(url)
    }

    pub async fn get_json<R>(&self, url: &str) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        self.get_json_with_headers(url, None).await
    }

    pub async fn get_json_with_headers<R>(&self, url: &str, headers: Option<HashMap<String, String>>) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let request = self.apply_headers(self.client.get(url), headers);
        let response = request.send().await.map_err(ClientError::from)?;
        self.read_json_response(response).await
    }

    pub async fn get_text(&self, url: &str) -> Result<String, ClientError> {
        self.get_text_with_headers(url, None).await
    }

    pub async fn get_text_with_headers(&self, url: &str, headers: Option<HashMap<String, String>>) -> Result<String, ClientError> {
        let request = self.apply_headers(self.client.get(url), headers);
        let response = request.send().await.map_err(ClientError::from)?;
        self.read_text_response(response).await
    }

    pub async fn execute(&self, request: reqwest::Request) -> Result<reqwest::Response, ClientError> {
        self.client.execute(request).await.map_err(ClientError::from)
    }

    pub fn build_request(&self, request: RequestBuilder) -> Result<reqwest::Request, ClientError> {
        request.build().map_err(ClientError::from)
    }

    fn apply_headers(&self, request: RequestBuilder, headers: Option<HashMap<String, String>>) -> RequestBuilder {
        if let Some(headers) = headers {
            headers
                .into_iter()
                .fold(request, |request, (key, value)| request.header(key.as_str(), value.as_str()))
        } else {
            request
        }
    }

    async fn read_json_response<R>(&self, response: reqwest::Response) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let (status, body, body_text) = self.read_response(response).await?;
        if status.is_success() {
            serde_json::from_slice(&body).map_err(|error| ClientError::Serialization(error.to_string()))
        } else {
            Err(ClientError::Http {
                status: status.as_u16(),
                body: body_text,
            })
        }
    }

    async fn read_text_response(&self, response: reqwest::Response) -> Result<String, ClientError> {
        let (status, _body, body_text) = self.read_response(response).await?;
        if status.is_success() {
            Ok(body_text)
        } else {
            Err(ClientError::Http {
                status: status.as_u16(),
                body: body_text,
            })
        }
    }

    async fn read_response(&self, response: reqwest::Response) -> Result<(reqwest::StatusCode, Vec<u8>, String), ClientError> {
        let status = response.status();
        let body = response.bytes().await.map_err(ClientError::from)?.to_vec();
        let body_text = String::from_utf8_lossy(&body).to_string();
        Ok((status, body, body_text))
    }
}

impl Default for ReqwestClient {
    fn default() -> Self {
        Self::from_builder(crate::builder()).expect("default req client builder should be valid")
    }
}
