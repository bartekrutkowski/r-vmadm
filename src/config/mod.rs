
use serde_derive;
use std::io::Read;
use std::io::BufReader;
use std::error::Error;
use std::fs::File;

use toml;


static CONFIG: &'static str = "/etc/vmadm.toml";

#[derive(Deserialize)]
pub struct Config {
    pub pool: String,
    #[serde(default = "default_conf_dir")]
    pub conf_dir: String
}

fn default_conf_dir() -> String {
    "/etc/jails".to_string()
}

impl Config {
    pub fn new() -> Result<Self, Box<Error>> {
        let mut file = File::open(CONFIG)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Failed to read config file.");
        let config: Config = toml::from_str(contents.as_str())?;
        Ok(config)
    }
    // add code here
}


