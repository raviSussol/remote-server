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
use service::sync_settings::SyncSettings;
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
        Action::ExportInitilisation { data } => {
            let SyncSettings {
                username,
                password,
                url,
                ..
            } = settings.sync;
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
        }
        Action::InitialiseDatabase => {
            test_db::setup(&settings.database).await;
        }
    }
}
