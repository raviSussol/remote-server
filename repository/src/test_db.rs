use crate::{
    database_settings::DatabaseSettings,
    db_diesel::{DBBackendConnection, StorageConnection, StorageConnectionManager},
    mock::{insert_all_mock_data, insert_mock_data, MockData, MockDataCollection, MockDataInserts},
};
use crate::{get_storage_connection_manager, run_db_migrations};
use diesel::r2d2::{ConnectionManager, Pool};

#[cfg(feature = "postgres")]
pub async fn setup(db_settings: &DatabaseSettings) {
    use diesel::{PgConnection, RunQueryDsl};

    let connection_manager =
        ConnectionManager::<PgConnection>::new(&db_settings.connection_string_without_db());
    let pool = Pool::new(connection_manager).expect("Failed to connect to database");
    let connection = pool.get().expect("Failed to open connection");

    diesel::sql_query(format!(
        "DROP DATABASE IF EXISTS \"{}\";",
        &db_settings.database_name
    ))
    .execute(&connection)
    .unwrap();

    diesel::sql_query(format!(
        "CREATE DATABASE \"{}\";",
        &db_settings.database_name
    ))
    .execute(&connection)
    .unwrap();

    let connection_manager = get_storage_connection_manager(&db_settings);
    let connection = connection_manager.connection().unwrap();
    run_db_migrations(&connection, false).unwrap()
}

#[cfg(not(feature = "postgres"))]
pub async fn setup(db_settings: &DatabaseSettings) {
    use std::fs;
    use std::path::Path;

    let db_path = format!("./{}.sqlite", db_settings.database_name);
    fs::remove_file(&db_path).ok();

    // create parent dirs
    let path = Path::new(&db_path);
    let prefix = path.parent().unwrap();
    fs::create_dir_all(prefix).unwrap();

    let connection_manager = get_storage_connection_manager(&db_settings);
    let connection = connection_manager.connection().unwrap();

    run_db_migrations(&connection, false).unwrap()
}

#[cfg(feature = "postgres")]
fn make_test_db_name(base_name: String) -> String {
    base_name
}

#[cfg(not(feature = "postgres"))]
fn make_test_db_name(base_name: String) -> String {
    // store all test db files in a test directory
    format!("test_output/{}", base_name)
}

// The following settings work for PG and Sqlite (username, password, host and port are
// ignored for the later)
pub fn get_test_db_settings(db_name: &str) -> DatabaseSettings {
    DatabaseSettings {
        username: "postgres".to_string(),
        password: "password".to_string(),
        port: 5432,
        host: "localhost".to_string(),
        database_name: make_test_db_name(db_name.to_owned()),
    }
}

/// Generic setup method to help setup test enviroment
/// - sets up database (create one and initialises schema), drops existing database
/// - creates connectuion
/// - inserts mock data
pub async fn setup_all(
    db_name: &str,
    inserts: MockDataInserts,
) -> (
    MockDataCollection,
    StorageConnection,
    StorageConnectionManager,
    DatabaseSettings,
) {
    setup_all_with_data(db_name, inserts, MockData::default()).await
}

pub async fn setup_all_with_data(
    db_name: &str,
    inserts: MockDataInserts,
    extra_mock_data: MockData,
) -> (
    MockDataCollection,
    StorageConnection,
    StorageConnectionManager,
    DatabaseSettings,
) {
    let settings = get_test_db_settings(db_name);

    setup(&settings).await;

    let connection_manager =
        ConnectionManager::<DBBackendConnection>::new(&settings.connection_string());
    let pool = Pool::new(connection_manager).expect("Failed to connect to database");

    let storage_connection_manager = StorageConnectionManager::new(pool.clone());

    let connection = storage_connection_manager.connection().unwrap();

    let core_data = insert_all_mock_data(&connection, inserts).await;

    insert_mock_data(
        &connection,
        MockDataInserts::all(),
        MockDataCollection {
            data: vec![("extra_data".to_string(), extra_mock_data)],
        },
    )
    .await;
    (core_data, connection, storage_connection_manager, settings)
}
