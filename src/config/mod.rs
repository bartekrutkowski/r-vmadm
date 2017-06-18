
use std::io::Read;
use std::error::Error;
use std::fs::File;

use toml;
extern crate slog;

static CONFIG: &'static str = "/etc/vmadm.toml";

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub pool: String,
    #[serde(default = "default_conf_dir")]
    pub conf_dir: String,
}

#[derive(Debug)]
pub struct Config {
    pub settings: Settings,
    pub logger: slog::Logger
}

fn default_conf_dir() -> String {
    "/etc/jails".to_string()
}

impl Config {
    pub fn new(logger: slog::Logger) -> Result<Self, Box<Error>> {
        let mut file = File::open(CONFIG)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(
            "Failed to read config file.",
        );
        let settings: Settings = toml::from_str(contents.as_str())?;
        Ok(Config{settings: settings, logger: logger})
    }
    // add code here
}