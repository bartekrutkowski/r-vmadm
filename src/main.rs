#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate uuid;

use std::path::Path;
use std::io;

use std::error::Error;
use std::fmt;

mod jdb;
use jdb::{JDB};

static INDEX: &'static str = "/etc/jails/index";


#[derive(Debug)]
struct ConflictError {
    uuid: String,
}

impl fmt::Display for ConflictError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Duplicated UUID:{}", self.uuid)
    }
}
impl Error for ConflictError {
    fn description(&self) -> &str {
        "Conflict"
    }
}


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
            ("update", Some(update_matches)) => dummy(update_matches),
            ("destroy", Some(destroy_matches)) => dummy(destroy_matches),
            ("start", Some(start_matches)) => dummy(start_matches),
            ("stop", Some(stop_matches)) => dummy(stop_matches),
            ("", None) => {
                help_app.print_help().unwrap();
                Ok(0)
            }
            _ => unreachable!(),
        };
        match r {
            Ok(v) => v,
            Err(_) => 1,
        }
    };
}

fn dummy(_matches: &clap::ArgMatches) -> Result<u8, Box<Error>> {
    Ok(0)
}

fn list(_matches: &clap::ArgMatches) -> Result<u8, Box<Error>> {
    let db = JDB::open(Path::new(INDEX))?;
    db.print();
    Ok(0)
}
fn create(_matches: &clap::ArgMatches) -> Result<u8, Box<Error>> {
    let mut db = JDB::open(Path::new(INDEX))?;
    let conf: jdb::Config = serde_json::from_reader(io::stdin())?;
    match db.find(conf.uuid.clone()) {
        None => {
            db.insert(conf)?;
            Ok(0)
        }
        Some(_) => Err(Box::new(ConflictError { uuid: conf.uuid })),
    }
}