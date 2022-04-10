#[derive(serde::Deserialize, Clone)]
pub struct SyncSettings {
    pub url: String,
    pub username: String,
    pub password: String,
    /// sync interval in sec
    pub interval: u64,
    pub central_server_site_id: u32,
    pub site_id: u32,
    pub site_hardware_id: String,
    pub batch_size: u32,
}

impl Default for SyncSettings {
    fn default() -> Self {
        // As per configurations/base.yaml
        Self {
            url: "http://localhost:2048".to_string(),
            username: "username".to_string(),
            password: "password".to_string(),
            interval: 300,
            central_server_site_id: 1,
            site_id: 2,
            site_hardware_id: Default::default(),
            batch_size: 500,
        }
    }
}
