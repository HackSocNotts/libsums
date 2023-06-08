use std::collections::HashMap;

use reqwest::{
    header::{HeaderMap, InvalidHeaderValue, COOKIE},
    Client, ClientBuilder, Url,
};
use thiserror::Error;
use url::ParseError;

use crate::member::{Member, StudentId};

#[derive(Debug, Error)]
pub enum SumsClientNewError {
    #[error("Failed to parse base URL.")]
    BaseUrlParseError(#[from] ParseError),

    #[error("Failed to create default cookie header. Is su_session in the valid header format?")]
    CookieHeaderParseError(#[from] InvalidHeaderValue),

    #[error("Failed to initialise client.")]
    ClientBuildError(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum SumsClientError {
    #[error("Failed to join base URL.")]
    BaseUrlJoinError(#[from] ParseError),

    #[error("Failed to make request.")]
    RequestError(#[from] reqwest::Error),
}

pub struct SumsClient {
    base_url: Url,
    client: Client,
    group_id: u16,
    su_session: String,
}

impl SumsClient {
    pub fn new(group_id: u16, su_session: String) -> Result<Self, SumsClientNewError> {
        let base_url = Url::parse(&format!(
            "https://student-dashboard.sums.su/groups/{}",
            group_id
        ))?;

        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, format!("su_session={}", su_session).parse()?);

        let client = ClientBuilder::new().default_headers(headers).build()?;

        Ok(SumsClient {
            base_url,
            client: client,
            group_id,
            su_session,
        })
    }

    pub async fn get_members(&self) -> Result<HashMap<StudentId, Member>, SumsClientError> {
        let url = self.base_url.join("members")?;

        let response = self.client.get(url).send().await?;

        Ok(HashMap::new())
    }
}
