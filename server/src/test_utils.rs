use repository::test_db::get_test_db_settings;
use service::sync_settings::SyncSettings;
use util::inline_init;

use super::settings::{AuthSettings, ServerSettings, Settings};

// The following settings work for PG and Sqlite (username, password, host and port are
// ignored for the later)
pub fn get_test_settings(db_name: &str) -> Settings {
    Settings {
        server: ServerSettings {
            host: "localhost".to_string(),
            port: 5432,
            debug_no_access_control: true,
        },
        database: get_test_db_settings(db_name),
        sync: inline_init(|r: &mut SyncSettings| {
            r.interval = 100000000;
        }),
        auth: AuthSettings {
            token_secret: "testtokensecret".to_string(),
        },
    }
}
