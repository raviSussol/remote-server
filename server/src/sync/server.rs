use reqwest::Url;
use std::fmt::{self, Debug, Display};

pub const BASE_URL: &'static str = "/sync/v5/";

pub const INITIALISE: &'static str = "initialise";
pub const QUEUED_RECORDS: &'static str = "queued_records";
pub const CENTRAL_RECORDS: &'static str = "central_records";
pub const ACKNOWLEDGE_RECORDS: &'static str = "acknowledged_records";

#[derive(Debug)]
pub struct SyncServer {
    pub url: Url,
}

impl SyncServer {
    pub fn new(url: &str) -> anyhow::Result<SyncServer> {
        // TODO: add error handling.
        let url = Url::parse(url)?;
        Ok(SyncServer { url })
    }

    pub fn from_url(url: Url) -> SyncServer {
        SyncServer { url }
    }

    pub fn base_url(&self) -> Url {
        self.url.join(BASE_URL).unwrap()
    }

    pub fn initialise_url(&self) -> Url {
        self.base_url().join(INITIALISE).unwrap()
    }

    pub fn queued_records_url(&self) -> Url {
        self.base_url().join(QUEUED_RECORDS).unwrap()
    }

    pub fn central_records_url(&self) -> Url {
        self.base_url().join(CENTRAL_RECORDS).unwrap()
    }

    pub fn acknowledge_records_url(&self) -> Url {
        self.base_url().join(ACKNOWLEDGE_RECORDS).unwrap()
    }
}

impl Display for SyncServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}
