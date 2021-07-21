use crate::util::environment::{
    Environment, LOCAL as LOCAL_ENVIRONMENT, PRODUCTION as PRODUCTION_ENVIRONMENT,
};
use crate::util::settings::Settings;

use config::{Config, File};
use std::fmt::{Debug, Display};
use std::{borrow::Cow, env};

static CONFIGURATION_DIRECTORY_PATH: &str = "configuration";
static BASE_CONFIGURATION_FILE_PATH: &str = "base";
static LOCAL_CONFIGURATION_FILE_PATH: &str = LOCAL_ENVIRONMENT;
static PRODUCTION_CONFIGURATION_FILE_PATH: &str = PRODUCTION_ENVIRONMENT;

static APP_ENVIRONMENT_VAR: &str = "APP_ENVIRONMENT";
static DEFAULT_APP_ENVIRONMENT: &str = LOCAL_ENVIRONMENT;

static CONFIGURATION_ENVIRONMENT_PREFIX: &str = "app";
static CONFIGURATION_ENVIRONMENT_SEPARATOR: &str = "__";

pub struct DisplayDebug {
    display: String,
    debug: String,
    // Also tried this but failed with borrowing issue when boxing one
    // display: Box<&dyn Debug>,
    // debug: Box<&dyn Display>,
}

impl<T> From<T> for DisplayDebug
where
    T: Display + Debug,
{
    fn from(err: T) -> Self {
        DisplayDebug {
            display: format!("{}", err),
            debug: format!("{:?}", err),
        }
    }
}

// If I add below, getting: "conflicting implementations of trait `std::convert::From<util::configuration::Yow>` for type `util::configuration::Yow`", no idea, even if just having one bound trait

// impl Debug for Yow {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.debug)
//     }
// }

// impl Display for Yow {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.display)
//     }
// }

pub struct DisplayDebugWrapper(DisplayDebug);

impl From<DisplayDebug> for DisplayDebugWrapper {
    fn from(yow: DisplayDebug) -> Self {
        DisplayDebugWrapper(yow)
    }
}

impl Debug for DisplayDebugWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.debug)
    }
}

impl Display for DisplayDebugWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display)
    }
}
pub fn get_configuration() -> Result<Settings, DisplayDebug> {
    let mut configuration = Config::default();

    let current_directory_path = env::current_dir()?;

    let configuration_directory = current_directory_path.join(CONFIGURATION_DIRECTORY_PATH);

    configuration.merge(
        File::from(configuration_directory.join(BASE_CONFIGURATION_FILE_PATH)).required(true),
    )?;

    let environment: Environment = env::var(APP_ENVIRONMENT_VAR)
        .map(Cow::from)
        .unwrap_or_else(|_| DEFAULT_APP_ENVIRONMENT.into())
        .parse()?;

    let app_environment_file_path = match environment {
        Environment::Local => LOCAL_CONFIGURATION_FILE_PATH,
        Environment::Production => PRODUCTION_CONFIGURATION_FILE_PATH,
    };

    configuration.merge(
        File::from(configuration_directory.join(app_environment_file_path)).required(true),
    )?;

    configuration.merge(
        config::Environment::with_prefix(CONFIGURATION_ENVIRONMENT_PREFIX)
            .separator(CONFIGURATION_ENVIRONMENT_SEPARATOR),
    )?;
    // Still need .map_err, why ?? ConfigError implements debug and display just like other errors above that work with ?
    configuration
        .try_into()
        .map_err(|err| DisplayDebug::from(err))
}
