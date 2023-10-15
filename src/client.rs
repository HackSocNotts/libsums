use std::{collections::HashMap, num::ParseIntError};

use chrono::NaiveDate;
use futures::executor::block_on;
use once_cell::sync::Lazy;
use thirtyfour::{
    fantoccini::error::CmdError, prelude::WebDriverError, By, DesiredCapabilities, WebDriver,
};
use thiserror::Error;
use url::Url;

use crate::{
    client,
    member::{Member, MemberType, StudentId},
};

/// The base URL of the SUMS website. This is a string instead of a Url since
/// Fantoccini takes URLs as strings.
const BASE_URL: &str = "https://su.nottingham.ac.uk";

/// The source code for the addShowAllEntries() function. See the source code in
/// the associated file for more information.
const ADD_SHOW_ALL_ENTRIES_JS: &'static str = include_str!("js/add_show_all_entries.js");

static DASHBOARD_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://student-dashboard.sums.su").unwrap());

#[derive(Debug, Error)]
pub enum SumsClientError {
    #[error("A WebDriver command failed")]
    WebDriverCmdError(#[from] WebDriverError),

    #[error("An error occured within Fantoccini")]
    FantocciniError(#[from] CmdError),
}

#[derive(Debug, Error)]
pub enum SumsClientNewError {
    #[error("Failed to create new WebDriver session")]
    WebDriverNewSessionError(#[from] WebDriverError),
}

#[derive(Debug, Error)]
pub enum SumsClientAuthError {
    #[error("A generic error occured (details within SumsClientError)")]
    SumsClientError(#[from] SumsClientError),

    #[error("Authentication failed with message {0}")]
    AuthFailedError(String),
}

impl From<WebDriverError> for SumsClientAuthError {
    fn from(err: WebDriverError) -> Self {
        SumsClientAuthError::SumsClientError(SumsClientError::WebDriverCmdError(err))
    }
}

impl From<CmdError> for SumsClientAuthError {
    fn from(err: CmdError) -> Self {
        SumsClientAuthError::SumsClientError(SumsClientError::FantocciniError(err))
    }
}

#[derive(Debug, Error)]
pub enum SumsClientMembersError {
    #[error("A generic error occured (details within SumsClientError)")]
    SumsClientError(#[from] SumsClientError),

    #[error("Failed to convert string to integer. Usually means invalid student ID.")]
    ParseIntError(#[from] ParseIntError),

    #[error("Failed to parse date joined.")]
    ChronoParseError(#[from] chrono::ParseError),
}

impl From<WebDriverError> for SumsClientMembersError {
    fn from(err: WebDriverError) -> Self {
        SumsClientMembersError::SumsClientError(SumsClientError::WebDriverCmdError(err))
    }
}

pub struct SumsClient {
    client: WebDriver,
    group_id: u16,
}

impl Drop for SumsClient {
    fn drop(&mut self) {
        // thirtyfour doesn't clean up after itself so we do so here
        block_on(self.client.quit());
    }
}

impl SumsClient {
    pub async fn new(group_id: u16, webdriver_address: &str) -> Result<Self, SumsClientNewError> {
        let caps = DesiredCapabilities::chrome();
        let client = WebDriver::new(webdriver_address, caps).await?;

        Ok(Self { client, group_id })
    }

    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<(), SumsClientAuthError> {
        self.client.goto(BASE_URL).await?;

        // Click on the user icon in the top right
        self.client
            .find(By::Id("userActionsInvoker"))
            .await?
            .click()
            .await?;

        // Click on the student login button
        self.client
            .find(By::XPath("//*[@id=\"userActions\"]/ul/li[1]/a[1]"))
            .await?
            .click()
            .await?;

        // Find the UoN login form
        let login_form = self
            .client
            .form(By::XPath("/html/body/div/div/div/div[1]/form"))
            .await?;

        // Fill in the username/password
        login_form.set_by_name("j_username", username).await?;
        login_form.set_by_name("j_password", password).await?;

        login_form.submit().await?;

        // Try to look for an error message on the login screen
        let login_error = self
            .client
            .find(By::XPath("/html/body/div/div/div/div[1]/section/p"))
            .await;

        // If an error message was found, we're still on the login screen, so
        // auth failed. Otherwise, we are on the SU screen and auth succeeded.
        match login_error {
            Ok(element) => Err(SumsClientAuthError::AuthFailedError(element.text().await?)),
            Err(_) => Ok(()),
        }
    }

    pub async fn members(&self) -> Result<Vec<Member>, SumsClientMembersError> {
        self.go_to_member_page().await?;

        self.client
            .goto("https://student-dashboard.sums.su/groups/213/members")
            .await?;

        let entry_count = self
            .client
            .execute(ADD_SHOW_ALL_ENTRIES_JS, Vec::new())
            .await?;

        // let entry_count_u64 = entry_count.as_u64().unwrap_or(100000);
        let entry_count_u64 = 100000;

        let entry_count_selector = self
            .client
            .find(By::Css(
                "#group-member-list-datatable_length > label:nth-child(1) > select:nth-child(1)",
            ))
            .await?;

        // entry_count_selector
        //     .select_by_value(&entry_count_u64.to_string())
        //     .await?;

        let table_body = self
            .client
            .find(By::Css("#group-member-list-datatable > tbody:nth-child(2)"))
            .await?;

        let member_elements = table_body.find_all(By::Css("tr")).await?;

        let mut members = Vec::new();

        for member_element in member_elements {
            let member_table_data = member_element.find_all(By::Css("td")).await?;

            let member = Member::new(
                member_table_data[0].text().await?.parse()?,
                member_table_data[1].text().await?,
                MemberType::Student,
                member_table_data[3].text().await?,
                NaiveDate::parse_from_str(&member_table_data[4].text().await?, "%Y-%m-%d")?,
            );

            members.push(member);
        }

        Ok(members)
    }

    async fn go_to_member_page(&self) -> Result<(), SumsClientError> {
        self.client.goto(BASE_URL).await?;

        let user_button = self.client.find(By::Id("userActionsInvoker")).await?;
        user_button.click().await?;

        let login_button = self.client.find(By::Id("studentDashboardLink")).await?;
        login_button.click().await?;

        self.client.wait().for_url(DASHBOARD_URL.clone()).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::client::{SumsClient, SumsClientAuthError, SumsClientNewError};

    use super::SumsClientMembersError;

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

    #[tokio::test]
    async fn test_members() -> Result<(), SumsClientMembersError> {
        let client = SumsClient::new(GROUP_ID, WEBDRIVER_ADDRESS).await.unwrap();

        let username = env::var("SUMS_USERNAME").expect("Invalid username environment variable");
        let password = env::var("SUMS_PASSWORD").expect("Invalid password environment variable");

        client
            .authenticate(username, password)
            .await
            .expect("Auth failed");

        client.members().await?;

        Ok(())
    }
}
