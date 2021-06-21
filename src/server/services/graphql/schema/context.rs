use crate::database::DataLoader;
use crate::database::DatabaseConnection;

impl juniper::Context for DatabaseConnection {}
impl juniper::Context for DataLoader {}
