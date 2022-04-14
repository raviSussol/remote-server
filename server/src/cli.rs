use chrono::Utc;
use clap::StructOpt;
use graphql::schema_builder;
use repository::{
    get_storage_connection_manager, schema::KeyValueType, test_db, ChangelogRepository,
    KeyValueStoreRepository, RefreshDatesRepository,
};
use server::{configuration, settings::Settings, sync::Synchroniser};
use service::{
    auth_data::AuthData,
    login::{LoginInput, LoginService},
    service_provider::ServiceProvider,
    token_bucket::TokenBucket,
};
use std::{fs, sync::RwLock};

/// omSupply remote server cli
#[derive(clap::Parser)]
#[clap(version, about)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    /// Export graphql schema
    ExportSchema,
    /// Initialise empty database (existing database will be dropped, and new one created and migrated)
    InitialiseDatabase,
    /// Initilise from running mSupply server
    InitialiseFromCentral {
        /// Users to sync in format "username:password,username2:password2"
        #[clap(short, long)]
        users: String,
    },
    /// Make data current, base on latest date difference to now
    RefreshData,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let settings: Settings =
        configuration::get_configuration().expect("Failed to parse configuration settings");

    match args.action {
        Action::ExportSchema => {
            let schema = schema_builder().finish();
            fs::write("schema.graphql", &schema.sdl()).unwrap();
        }
        Action::InitialiseDatabase => {
            test_db::setup(&settings.database).await;
        }
        Action::InitialiseFromCentral { users } => {
            test_db::setup(&settings.database).await;

            let connection_manager = get_storage_connection_manager(&settings.database);
            let service_provider = ServiceProvider::new(connection_manager.clone());

            let sync_settings = settings.sync.clone();

            let auth_data = AuthData {
                auth_token_secret: "secret".to_string(),
                token_bucket: RwLock::new(TokenBucket::new()),
                debug_no_ssl: true,
                debug_no_access_control: false,
            };
            Synchroniser::new(sync_settings, connection_manager)
                .unwrap()
                .initial_pull()
                .await
                .unwrap();

            for user in users.split(",") {
                let user = user.split(':').collect::<Vec<&str>>();
                let input = LoginInput {
                    username: user[0].to_string(),
                    password: user[1].to_string(),
                    central_server_url: settings.sync.url.clone(),
                };
                LoginService::login(&service_provider, &auth_data, input.clone(), 0)
                    .await
                    .expect(&format!("Cannot login with user {:?}", input));
            }
        }
        Action::RefreshData => {
            let connection_manager = get_storage_connection_manager(&settings.database);
            let connection = connection_manager.connection().unwrap();

            let result = RefreshDatesRepository::new(&connection)
                .refresh_dates(Utc::now().naive_local())
                .expect("Error while refreshing data");

            println!("Refresh data result: {:#?}", result);

            // Update cursor
            let latest_change_log = ChangelogRepository::new(&connection)
                .latest_changelog()
                .unwrap();
            if let Some(latest_change_log) = latest_change_log {
                let new_cursor = latest_change_log.id as i32 + 1;
                KeyValueStoreRepository::new(&connection)
                    .set_i32(KeyValueType::RemoteSyncPushCursor, Some(new_cursor))
                    .unwrap();
                println!("Cursor updated to {}", new_cursor)
            }
        }
    }
}
