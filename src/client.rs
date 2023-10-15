use std::num::ParseIntError;

use chrono::NaiveDate;
use fantoccini::{
    error::{CmdError, NewSessionError},
    wd::Capabilities,
    Client, ClientBuilder, Locator,
};
use once_cell::sync::Lazy;
use thiserror::Error;
use url::Url;

use crate::member::{Member, MemberType};

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

#[derive(Debug, Error)]
pub enum SumsClientMembersError {
    #[error("A generic error occured (details within SumsClientError)")]
    SumsClientError(#[from] SumsClientError),

    #[error("Failed to convert string to integer. Usually means invalid student ID.")]
    ParseIntError(#[from] ParseIntError),

    #[error("Failed to parse date joined.")]
    ChronoParseError(#[from] chrono::ParseError),
}

impl From<CmdError> for SumsClientMembersError {
    fn from(err: CmdError) -> Self {
        SumsClientMembersError::SumsClientError(SumsClientError::WebDriverCmdError(err))
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
        let mut client_builder = ClientBuilder::rustls();

        // Selenium gets annoyed if we don't do this. We should probably let the user pass whatever
        // here in case they're using Firefox or something, but geckodriver doesn't support
        // simultaneous sessions so they probably shouldn't
        let mut capabilities = Capabilities::new();
        capabilities.insert("browserName".to_string(), "chromium".into());
        client_builder.capabilities(capabilities);

        let client = client_builder.connect(webdriver_address.as_ref()).await?;

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

        // Try to look for an error message on the login screen
        let login_error = self
            .client
            .find(Locator::XPath("/html/body/div/div/div/div[1]/section/p"))
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
            .goto(&format!(
                "https://student-dashboard.sums.su/groups/{}/members",
                self.group_id
            ))
            .await?;

        self.client
            .execute(ADD_SHOW_ALL_ENTRIES_JS, Vec::new())
            .await?;

        // let entry_count_u64 = entry_count.as_u64().unwrap_or(100000);
        let entry_count_u64 = 100000;

        let entry_count_selector = self
            .client
            .find(Locator::Css(
                "#group-member-list-datatable_length > label:nth-child(1) > select:nth-child(1)",
            ))
            .await?;

        entry_count_selector
            .select_by_value(&entry_count_u64.to_string())
            .await?;

        let table_body = self
            .client
            .find(Locator::Css(
                "#group-member-list-datatable > tbody:nth-child(2)",
            ))
            .await?;

        let member_elements = table_body.find_all(Locator::Css("tr")).await?;

        let mut members = Vec::new();

        for member_element in member_elements {
            let member_table_data = member_element.find_all(Locator::Css("td")).await?;

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

        let user_button = self.client.find(Locator::Id("userActionsInvoker")).await?;
        user_button.click().await?;

        let login_button = self
            .client
            .find(Locator::Id("studentDashboardLink"))
            .await?;
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
