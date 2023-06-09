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
pub enum SumsClientError {
    #[error("A WebDriver command failed")]
    WebDriverCmdError(#[from] CmdError),
}

#[derive(Debug, Error)]
pub enum SumsClientNewError {
    #[error("Failed to create new WebDriver session")]
    WebDriverNewSessionError(#[from] NewSessionError),
}

#[derive(Debug, Error)]
pub enum SumsClientAuthError {
    #[error("A generic error occured (details within SumsClientError)")]
    SumsClientError(#[from] SumsClientError),

    #[error("Authentication failed with message {0}")]
    AuthFailedError(String),
}

impl From<CmdError> for SumsClientAuthError {
    fn from(err: CmdError) -> Self {
        SumsClientAuthError::SumsClientError(SumsClientError::WebDriverCmdError(err))
    }
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

    pub async fn authenticate<S>(&self, username: S, password: S) -> Result<(), SumsClientAuthError>
    where
        S: AsRef<str>,
    {
        self.client.goto(BASE_URL).await?;

        // Click on the user icon in the top right
        self.client
            .find(Locator::Id("userActionsInvoker"))
            .await?
            .click()
            .await?;

        // Click on the student login button
        self.client
            .find(Locator::XPath("//*[@id=\"userActions\"]/ul/li[1]/a[1]"))
            .await?
            .click()
            .await?;

        // Find the UoN login form
        let login_form = self
            .client
            .form(Locator::XPath("/html/body/div/div/div/div[1]/form"))
            .await?;

        // Fill in the username/password
        login_form
            .set(Locator::Id("username"), username.as_ref())
            .await?;
        login_form
            .set(Locator::Id("password"), password.as_ref())
            .await?;

        login_form.submit().await?;

        let login_error = self
            .client
            .find(Locator::XPath("/html/body/div/div/div/div[1]/section/p"))
            .await;

        match login_error {
            Ok(element) => Err(SumsClientAuthError::AuthFailedError(element.text().await?)),
            Err(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::client::{SumsClient, SumsClientAuthError, SumsClientNewError};

    use super::SumsClientError;

    const GROUP_ID: u16 = 213;
    const WEBDRIVER_ADDRESS: &str = "http://localhost:9515";

    #[tokio::test]
    async fn test_create_client() -> Result<(), SumsClientNewError> {
        let client = SumsClient::new(GROUP_ID, WEBDRIVER_ADDRESS).await?;

        assert_eq!(GROUP_ID, client.group_id);

        Ok(())
    }

    #[tokio::test]
    async fn test_auth() -> Result<(), SumsClientAuthError> {
        // test_create_client should handle this
        let client = SumsClient::new(GROUP_ID, WEBDRIVER_ADDRESS).await.unwrap();

        let username = env::var("SUMS_USERNAME").expect("Invalid username environment variable");
        let password = env::var("SUMS_PASSWORD").expect("Invalid password environment variable");

        client.authenticate(username, password).await?;

        Ok(())
    }
}
