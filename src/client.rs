use std::collections::HashMap;

use fantoccini::{
    error::{CmdError, NewSessionError},
    Client, ClientBuilder, Locator,
};
use thiserror::Error;
use url::Url;

use crate::{
    client,
    member::{Member, StudentId},
};

const BASE_URL: &str = "https://su.nottingham.ac.uk";

#[derive(Debug, Error)]
pub enum SumsClientNewError {
    // #[error("Failed to parse base URL.")]
    // BaseUrlParseError(#[from] ParseError),

    // #[error("Failed to create default cookie header. Is su_session in the valid header format?")]
    // CookieHeaderParseError(#[from] InvalidHeaderValue),
    // #[error("Failed to initialise client.")]
    // ClientBuildError(#[from] reqwest::Error),
    #[error("Failed to create new WebDriver session")]
    WebDriverNewSessionError(#[from] NewSessionError),
}

#[derive(Debug, Error)]
pub enum SumsClientError {
    // #[error("Failed to join base URL.")]
    // BaseUrlJoinError(#[from] ParseError),

    // #[error("Failed to make request.")]
    // RequestError(#[from] reqwest::Error),
    #[error("A WebDriver command failed")]
    WebDriverCmdError(#[from] CmdError),
}

pub struct SumsClient {
    client: Client,
    group_id: u16,
}

impl SumsClient {
    pub async fn new<S>(group_id: u16, webdriver_address: S) -> Result<Self, SumsClientNewError>
    where
        S: AsRef<str>,
    {
        let client = ClientBuilder::rustls()
            .connect(webdriver_address.as_ref())
            .await?;

        Ok(Self { client, group_id })
    }

    pub async fn authenticate<S>(&self, username: S, password: S) -> Result<(), SumsClientError>
    where
        S: AsRef<str>,
    {
        self.client.goto(BASE_URL).await?;

        self.client
            .find(Locator::LinkText("Student Login"))
            .await?
            .click()
            .await?;

        println!("{}", self.client.current_url().await?);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::client::{SumsClient, SumsClientNewError};

    const GROUP_ID: u16 = 213;
    const WEBDRIVER_ADDRESS: &str = "http://localhost:4444";

    #[tokio::test]
    async fn test_create_client() -> Result<(), SumsClientNewError> {
        let client = SumsClient::new(GROUP_ID, WEBDRIVER_ADDRESS).await?;

        assert_eq!(GROUP_ID, client.group_id);

        Ok(())
    }

    #[tokio::test]
    async fn test_auth() {
        // let client = SumsClient::new(GROUP_ID, WEBDRIVER_ADDRESS).await?;
    }
}
