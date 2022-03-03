#![allow(dead_code)]

use reqwest::{
    header::{HeaderMap, ACCEPT, CONTENT_LENGTH},
    Client, Url,
};
use serde::{Deserialize, Serialize};

use super::SyncCredentials;

pub struct SyncApiV3 {
    server_url: Url,
    extra_headers: HeaderMap,
    client: Client,
    credentials: SyncCredentials,
}

fn extra_headers(side_id: u32) -> anyhow::Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert("msupply-site-uuid", format!("{}", side_id).parse()?);
    headers.insert(CONTENT_LENGTH, "application/json".parse()?);
    headers.insert(ACCEPT, "application/json".parse()?);
    Ok(headers)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SyncTypeV3 {
    #[serde(rename = "I")]
    Insert,
    #[serde(rename = "U")]
    Update,
    #[serde(rename = "D")]
    Delete,
    #[serde(rename = "M")]
    Merge,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteGetRecordV3 {
    #[serde(rename = "RecordType")]
    pub record_type: String,
    #[serde(rename = "SyncID")]
    pub sync_id: String,
    #[serde(rename = "KeyFieldID")]
    pub key_field_id: i64,
    #[serde(rename = "mergeIDtokeep")]
    pub merge_id_to_keep: String,
    #[serde(rename = "StoreID")]
    pub store_id: String,
    #[serde(rename = "RecordID")]
    pub record_id: String,
    #[serde(rename = "SyncType")]
    pub sync_type: SyncTypeV3,
    #[serde(rename = "mergeIDtodelete")]
    pub merge_id_to_delete: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemotePostRecordV3 {
    #[serde(rename = "SyncID")]
    pub sync_id: String,
    #[serde(rename = "RecordType")]
    pub record_type: String,
    #[serde(rename = "RecordID")]
    pub record_id: String,
    #[serde(rename = "SyncType")]
    pub sync_type: SyncTypeV3,
    #[serde(rename = "StoreID")]
    pub store_id: Option<String>,
    // if sync type is Delete data is None
    #[serde(rename = "Data")]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct RemoteSyncAckV3 {
    #[serde(rename = "SyncRecordIDs")]
    sync_record_ids: Vec<String>,
}

impl SyncApiV3 {
    pub fn new(
        server_url: Url,
        credentials: SyncCredentials,
        client: Client,
        site_id: u32,
    ) -> anyhow::Result<Self> {
        Ok(SyncApiV3 {
            server_url,
            extra_headers: extra_headers(site_id)?,
            client,
            credentials,
        })
    }

    pub async fn get_initial_dump(
        &self,
        from_site: u32,
        to_site: u32,
    ) -> anyhow::Result<serde_json::Value> {
        let query = [("from_site", from_site), ("to_site", to_site)];

        let response = self
            .client
            .get(self.server_url.join("/sync/v3/initial_dump")?)
            .basic_auth(
                &self.credentials.username,
                Some(&self.credentials.password_sha256),
            )
            .query(&query)
            .headers(self.extra_headers.clone())
            .send()
            .await?
            .error_for_status()?;

        let response = response.json().await?;
        Ok(response)
    }

    pub async fn get_queued_records(
        &self,
        from_site: u32,
        to_site: u32,
        limit: u32,
    ) -> anyhow::Result<Vec<RemoteGetRecordV3>> {
        let query = [
            ("from_site", from_site),
            ("to_site", to_site),
            ("limit", limit),
        ];

        let response = self
            .client
            .get(self.server_url.join("/sync/v3/queued_records")?)
            .basic_auth(
                &self.credentials.username,
                Some(&self.credentials.password_sha256),
            )
            .query(&query)
            .headers(self.extra_headers.clone())
            .send()
            .await?
            .error_for_status()?;

        let response = response.json().await?;
        Ok(response)
    }

    pub async fn post_queued_records(
        &self,
        from_site: u32,
        to_site: u32,
        records: &Vec<RemotePostRecordV3>,
    ) -> anyhow::Result<serde_json::Value> {
        let query = [("from_site", from_site), ("to_site", to_site)];

        let response = self
            .client
            .post(self.server_url.join("/sync/v3/queued_records")?)
            .basic_auth(
                &self.credentials.username,
                Some(&self.credentials.password_sha256),
            )
            .query(&query)
            .headers(self.extra_headers.clone())
            .json(records)
            .send()
            .await?
            .error_for_status()?;

        let response = response.json().await?;
        Ok(response)
    }

    pub async fn post_acknowledged_records(
        &self,
        from_site: u32,
        to_site: u32,
        records: &RemoteSyncAckV3,
    ) -> anyhow::Result<serde_json::Value> {
        let query = [("from_site", from_site), ("to_site", to_site)];

        let response = self
            .client
            .post(self.server_url.join("/sync/v3/acknowledged_records")?)
            .basic_auth(
                &self.credentials.username,
                Some(&self.credentials.password_sha256),
            )
            .query(&query)
            .headers(self.extra_headers.clone())
            .json(records)
            .send()
            .await?
            .error_for_status()?;

        let response = response.json().await?;
        Ok(response)
    }
}
