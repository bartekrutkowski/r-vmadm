#![deny(trivial_numeric_casts,
// missing_docs,
        unstable_features,
        unused_import_braces,
)]

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate uuid;
extern crate toml;

use std::error::Error;
use std::io;

mod zfs;

pub mod jdb;
use jdb::JDB;

mod config;
use config::Config;

pub mod errors;
use errors::GenericError;

fn main() {
    use clap::App;
    let yaml = load_yaml!("cli.yml");
    let mut help_app = App::from_yaml(yaml).version(crate_version!());
    let matches = App::from_yaml(yaml).version(crate_version!()).get_matches();

    let config = Config::new().unwrap();
    let r = if matches.is_present("startup") {
        match matches.subcommand() {
            ("", None) => startup(&config),
            _ => Err(GenericError::bx("Can not use startup with a subcommand")),
        }
    } else {
        match matches.subcommand() {
            ("list", Some(list_matches)) => list(&config, list_matches),
            ("create", Some(create_matches)) => create(&config, create_matches),
            ("update", Some(update_matches)) => update(&config, update_matches),
            ("destroy", Some(destroy_matches)) => destroy(&config, destroy_matches),
            ("start", Some(start_matches)) => start(&config, start_matches),
            ("stop", Some(stop_matches)) => stop(&config, stop_matches),
            ("", None) => {
                help_app.print_help().unwrap();
                Ok(0)
            }
            _ => unreachable!(),
        }
    };

    match r {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(e) => {
            println!("error: {}", e);
            std::process::exit(1)
        }
    }
}

fn startup(_conf: &Config) -> Result<i32, Box<Error>> {
    println!("{:?}", zfs::list("tpool"));
    Ok(0)
}

fn start(_conf: &Config, _matches: &clap::ArgMatches) -> Result<i32, Box<Error>> {
    Ok(0)
}

fn stop(_conf: &Config, _matches: &clap::ArgMatches) -> Result<i32, Box<Error>> {
    Ok(0)
}

fn update(_conf: &Config, _matches: &clap::ArgMatches) -> Result<i32, Box<Error>> {
    Ok(0)
}

fn list(conf: &Config, _matches: &clap::ArgMatches) -> Result<i32, Box<Error>> {
    let db = JDB::open(&conf.conf_dir)?;
    db.print();
    Ok(0)
}

fn create(conf: &Config, _matches: &clap::ArgMatches) -> Result<i32, Box<Error>> {
    let mut db = JDB::open(&conf.conf_dir)?;
    let conf: jdb::Config = serde_json::from_reader(io::stdin())?;
    db.insert(conf)?;
    Ok(0)
}

// fn destroy(conf: &Config, matches: &clap::ArgMatches) -> Result<i32, Box<Error>> {
    let mut db = JDB::open(&conf.conf_dir)?;
    let uuid = value_t!(matches, "uuid", String).unwrap();
    db.remove(uuid.as_str())?;
    Ok(0)
}