use clap::StructOpt;
use graphql::schema_builder;
use repository::{get_storage_connection_manager, test_db, RemoteSyncBufferRepository};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use server::{
    configuration,
    settings::Settings,
    sync::{
        central_data_synchroniser::{
            self, central_sync_batch_records_to_buffer_rows, insert_one_and_update_cursor,
        },
        remote_data_synchroniser::{self, remote_sync_batch_records_to_buffer_rows},
        sync_api_v5::{CentralSyncBatchV5, RemoteSyncBatchV5},
        SyncApiV5, SyncCredentials,
    },
};
use service::{
    apis::login_v4::LoginUserInfoV4,
    login::{LoginInput, LoginService},
    service_provider::ServiceContext,
    sync_settings::SyncSettings,
};
use std::fs;

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
    /// Export initilisation data
    ExportInitilisation {
        /// File name for export of initilisation data
        #[clap(short, long)]
        data: String,
        /// Users to sync in format "username:password,username2:password2"
        #[clap(short, long)]
        users: String,
    },
    /// Initialise database from exported data (will re-initialise database, removing existing data)
    InitialiseFromExport {
        /// File name for import of initilisation data
        #[clap(short, long)]
        data: String,
    },
}

#[derive(Serialize, Deserialize)]
struct InitilisationData {
    central: CentralSyncBatchV5,
    remote: RemoteSyncBatchV5,
    users: Vec<(LoginInput, LoginUserInfoV4)>,
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
        Action::ExportInitilisation { data, users } => {
            let SyncSettings {
                username,
                password,
                url,
                ..
            } = settings.sync;

            let mut synced_user_info_rows = Vec::new();
            for user in users.split(",") {
                let user = user.split(':').collect::<Vec<&str>>();
                let input = LoginInput {
                    username: user[0].to_string(),
                    password: user[1].to_string(),
                    central_server_url: url.to_string(),
                };
                synced_user_info_rows.push((
                    input.clone(),
                    LoginService::fetch_user_from_central(&input).await.unwrap(),
                ));
            }

            let client = Client::new();
            let url = Url::parse(&url).unwrap();

            let credentials = SyncCredentials::new(&username, &password);
            let sync_api_v5 = SyncApiV5::new(url.clone(), credentials.clone(), client.clone());
            sync_api_v5.post_initialise().await.unwrap();

            fs::write(
                data,
                serde_json::to_string_pretty(&InitilisationData {
                    central: sync_api_v5.get_central_records(0, 1000000).await.unwrap(),
                    remote: sync_api_v5.get_queued_records(1000000).await.unwrap(),
                    users: synced_user_info_rows,
                })
                .unwrap(),
            )
            .unwrap();
        }

        Action::InitialiseFromExport { data } => {
            test_db::setup(&settings.database).await;

            let connection_manager = get_storage_connection_manager(&settings.database);
            let connection = connection_manager.connection().unwrap();

            let data: InitilisationData = serde_json::from_slice(&fs::read(data).unwrap()).unwrap();

            for central_sync_record in
                central_sync_batch_records_to_buffer_rows(data.central.data).unwrap()
            {
                insert_one_and_update_cursor(&connection, &central_sync_record)
                    .await
                    .unwrap()
            }

            central_data_synchroniser::do_integrate_records(&connection)
                .await
                .unwrap();

            if let Some(data) = data.remote.data {
                RemoteSyncBufferRepository::new(&connection)
                    .upsert_many(&remote_sync_batch_records_to_buffer_rows(data).unwrap())
                    .unwrap();
                remote_data_synchroniser::do_integrate_records(&connection).unwrap()
            }

            let ctx = ServiceContext { connection };

            for (input, user_info) in data.users {
                LoginService::update_user_from_central(&ctx, &input, user_info).unwrap();
            }
        }
        Action::InitialiseDatabase => {
            test_db::setup(&settings.database).await;
        }
    }
}
