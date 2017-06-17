#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate uuid;

use std::path::Path;
use std::error::Error;
use std::io;



pub mod jdb;
use jdb::JDB;

pub mod errors;

static INDEX: &'static str = "/etc/jails/index";



fn main() {
    use clap::App;
    let yaml = load_yaml!("cli.yml");
    let mut help_app = App::from_yaml(yaml).version(crate_version!());
    let matches = App::from_yaml(yaml).version(crate_version!()).get_matches();
    if matches.is_present("startup") {
        match matches.subcommand() {
            ("", None) => {
                println!("startup");
                0
            }
            _ => {
                println!("Can not use startup with a subcommand");
                1
            }
        }
    } else {
        let r = match matches.subcommand() {
            ("list", Some(list_matches)) => list(list_matches),
            ("create", Some(create_matches)) => create(create_matches),
            ("update", Some(update_matches)) => update(update_matches),
            ("destroy", Some(destroy_matches)) => destroy(destroy_matches),
            ("start", Some(start_matches)) => start(start_matches),
            ("stop", Some(stop_matches)) => stop(stop_matches),
            ("", None) => {
                help_app.print_help().unwrap();
                Ok(0)
            }
            _ => unreachable!(),
        };
        match r {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(e) => {
                println!("error: {}", e);
                std::process::exit(1)
            }
        }
    };
}

fn start(_matches: &clap::ArgMatches) -> Result<i32, Box<Error>> {
    Ok(0)
}

fn stop(_matches: &clap::ArgMatches) -> Result<i32, Box<Error>> {
    Ok(0)
}

fn update(_matches: &clap::ArgMatches) -> Result<i32, Box<Error>> {
    Ok(0)
}

fn list(_matches: &clap::ArgMatches) -> Result<i32, Box<Error>> {
    let db = JDB::open(Path::new(INDEX))?;
    db.print();
    Ok(0)
}

fn create(_matches: &clap::ArgMatches) -> Result<i32, Box<Error>> {
    let mut db = JDB::open(Path::new(INDEX))?;
    let conf: jdb::Config = serde_json::from_reader(io::stdin())?;
    db.insert(conf)?;
    Ok(0)
}

fn destroy(matches: &clap::ArgMatches) -> Result<i32, Box<Error>> {
    let mut db = JDB::open(Path::new(INDEX))?;
    let uuid = value_t!(matches, "uuid", String).unwrap();
    db.remove(uuid)?;
    Ok(0)
}