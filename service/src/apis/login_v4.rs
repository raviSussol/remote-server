use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum LoginV4Error {
    Unauthorised,
    ConnectionError(reqwest::Error),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LoginUserTypeV4 {
    #[serde(alias = "user")]
    User,
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum LoginStatusV4 {
    #[serde(alias = "success")]
    Success,
    #[serde(alias = "error")]
    Error,
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Serialize)]
pub struct LoginInputV4 {
    pub username: String,
    pub password: String,
    #[serde(rename = "loginType")]
    pub login_type: LoginUserTypeV4,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoginUserV4 {
    #[serde(rename = "ID")]
    pub id: String,
    pub name: String,
    pub startup_method: String,
    //Signature: "[object Picture]",
    pub nblogins: i32,
    //lastlogin: "2020-03-24",
    pub group_id: String,
    pub mode: String,
    // qdump_offset_b: null,
    pub active: bool,
    // permissions_spare: null,
    pub lasttime: i32,
    pub initials: String,
    pub first_name: String,
    pub last_name: String,
    //date_of_birth: "0000-00-00",
    pub address_1: String,
    pub address_2: String,
    pub e_mail: String,
    pub phone1: String,
    pub phone2: String,
    //date_created: "2017-10-11",
    //date_left: "0000-00-00",
    pub job_title: String,
    pub responsible_officer: bool,
    #[serde(rename = "Language")]
    pub language: i32,
    pub use_ldap: bool,
    pub ldap_login_string: String,
    pub receives_sms_errors: bool,
    pub is_group: bool,
    // dashboard_tabs: { "tabs": [] },
    // custom_data: null,
    pub windows_user_name: String,
    pub license_category_id: String,
    // tags: { "tags": [] },
    // type: { "types": ["desktop"]},
    #[serde(rename = "isInactiveAuthoriser")]
    pub is_inactive_authoriser: bool,
    pub spare_1: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoginUserStoresV4 {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "user_ID")]
    pub user_id: String,
    #[serde(rename = "store_ID")]
    pub store_id: String,
    pub can_login: bool,
    pub store_default: bool,
    pub can_action_replenishments: bool,
    pub permissions: Vec<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoginUserInfoV4 {
    pub user: LoginUserV4,
    #[serde(rename = "userStores")]
    pub user_stores: Vec<LoginUserStoresV4>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoginResponseV4 {
    pub status: LoginStatusV4,
    pub authenticated: bool,
    pub username: String,
    #[serde(rename = "userFirstName")]
    pub user_first_name: String,
    #[serde(rename = "userLastName")]
    pub user_last_name: String,
    #[serde(rename = "userJobTitle")]
    pub user_job_title: String,
    #[serde(rename = "userType")]
    pub user_type: LoginUserTypeV4,
    pub service: String,
    #[serde(rename = "storeName")]
    pub store_name: String,
    #[serde(rename = "userInfo")]
    pub user_info: Option<LoginUserInfoV4>,
}

pub struct LoginApiV4 {
    server_url: Url,
    client: Client,
}

impl LoginApiV4 {
    pub fn new(client: Client, server_url: Url) -> Self {
        LoginApiV4 { server_url, client }
    }

    pub async fn login(&self, input: LoginInputV4) -> Result<LoginResponseV4, LoginV4Error> {
        let response = self
            .client
            .post(self.server_url.join("/api/v4/login").unwrap())
            .json(&input)
            .send()
            .await
            .map_err(|e| LoginV4Error::ConnectionError(e))?;

        if reqwest::StatusCode::UNAUTHORIZED == response.status() {
            return Err(LoginV4Error::Unauthorised);
        }

        let response = response
            .error_for_status()
            .map_err(|e| LoginV4Error::ConnectionError(e))?
            .json()
            .await
            .map_err(|e| LoginV4Error::ConnectionError(e))?;

        Ok(response)
    }
}
