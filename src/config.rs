use anyhow::Result;
use envconfig::Envconfig;
use validator::Validate;

const DEFAULT_PORT: u16 = 9950;
const DEFAULT_STORAGE_DIR: &'static str = "./baste_storage";
const DEFAULT_ADDRESS: &'static str = "0.0.0.0";

#[derive(Envconfig, Validate, Default, Clone, Debug)]
pub struct Config {
    #[validate(length(min = 10))]
    #[envconfig(from = "BASTE_SECRET_TOKEN")]
    pub secret_token: String,

    #[envconfig(from = "BASTE_PORT")]
    pub port: Option<u16>,

    #[envconfig(from = "BASTE_ADDRESS")]
    pub address: Option<String>,

    #[envconfig(from = "BASTE_STORAGE_DIR")]
    pub storage_directory: Option<String>,
}

impl Config {
    pub fn load() -> Result<Config> {
        let mut c = Config::init_from_env()?;

        c.validate()?;

        c.port = match c.port {
            None => Some(DEFAULT_PORT),
            Some(data) => Some(data),
        };

        c.storage_directory = match c.storage_directory {
            Some(path) => Some(path),
            None => Some(String::from(DEFAULT_STORAGE_DIR)),
        };

        c.address = match c.address {
            Some(address) => Some(address),
            None => Some(String::from(DEFAULT_ADDRESS)),
        };

        Ok(c)
    }
}
