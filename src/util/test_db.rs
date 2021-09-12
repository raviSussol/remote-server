use crate::database::repository::ConnectionType;

use super::settings::{DatabaseSettings, ServerSettings, Settings, SyncSettings};

use diesel_migrations::{find_migrations_directory, mark_migrations_in_directory};

pub async fn setup(db_settings: &DatabaseSettings) {
    match db_settings.database_type {
        ConnectionType::Pg => setup_pg(db_settings).await,
        ConnectionType::Sqlite => setup_sqlite(db_settings).await,
    }
}

#[cfg(not(feature = "postgres"))]
async fn setup_pg(_: &DatabaseSettings) {
    panic!("postgres flag is not enabled")
}

#[cfg(feature = "postgres")]
async fn setup_pg(db_settings: &DatabaseSettings) {
    use diesel::{
        r2d2::{ConnectionManager, Pool},
        PgConnection, RunQueryDsl,
    };

    const MIGRATION_PATH: &str = "postgres";

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

    let connection_manager =
        ConnectionManager::<PgConnection>::new(&db_settings.connection_string());
    let pool = Pool::new(connection_manager).expect("Failed to connect to database");
    let connection = pool.get().expect("Failed to open connection");

    let mut migrations_dir =
        find_migrations_directory().expect("Failed to locate migrations directory");
    migrations_dir.push(MIGRATION_PATH);

    let mut migrations = mark_migrations_in_directory(&connection, &migrations_dir).unwrap();
    migrations.sort_by(|(m, ..), (n, ..)| m.version().cmp(&n.version()));

    for (migration, ..) in migrations.iter() {
        migration.run(&connection).unwrap();
    }
}
#[cfg(not(feature = "sqlite"))]
async fn setup_sqlite(_: &DatabaseSettings) {
    panic!("sqlite flag is not enabled")
}
#[cfg(feature = "sqlite")]
async fn setup_sqlite(db_settings: &DatabaseSettings) {
    use diesel::{Connection, SqliteConnection};
    use std::fs;

    const MIGRATION_PATH: &str = "sqlite";

    let db_path = format!("./{}.sqlite", db_settings.database_name);

    fs::remove_file(&db_path).ok();

    let connection = SqliteConnection::establish(&db_path).unwrap();

    let mut migrations_dir =
        find_migrations_directory().expect("Failed to locate migrations directory");
    migrations_dir.push(MIGRATION_PATH);

    let mut migrations = mark_migrations_in_directory(&connection, &migrations_dir).unwrap();
    migrations.sort_by(|(m, ..), (n, ..)| m.version().cmp(&n.version()));

    for (migration, ..) in migrations.iter() {
        migration.run(&connection).unwrap();
    }
}

// The following settings work for PG and Sqlite (username, password, host and port are
// ignored for the later)
pub fn get_test_settings(db_name: &str, database_type: ConnectionType) -> Settings {
    Settings {
        server: ServerSettings {
            host: "localhost".to_string(),
            port: 5432,
        },
        database: DatabaseSettings {
            username: "postgres".to_string(),
            password: "password".to_string(),
            port: 5432,
            host: "localhost".to_string(),
            database_name: db_name.to_owned(),
            database_type,
        },
        sync: SyncSettings {
            username: "postgres".to_string(),
            password: "password".to_string(),
            port: 5432,
            host: "localhost".to_string(),
            interval: 100000000,
        },
    }
}
