#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate uuid;

use std::path::Path;
use std::io::{self, Read};

use std::error::Error;
use std::fmt;

mod jdb;
use jdb::{JDB, Config};

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
            ("", None) => println!("startup"),
            _ => println!("Can not use startup with a subcommand"),
        }
    } else {
        match matches.subcommand() {
            ("list", Some(list_matches)) => {
                match JDB::open(Path::new(INDEX)) {
                    Err(e) => println!("cound not open index: {}", e),
                    Ok(db) => db.print(),
                }
            }
            ("create", Some(create_matches)) => {
                let conf = create();
                println!("create jail: {:?}", conf);
            }
            ("update", Some(update_matches)) => println!("update jail"),
            ("destroy", Some(destroy_matches)) => println!("destroy jail"),
            ("start", Some(destroy_matches)) => println!("start jail"),
            ("stop", Some(destroy_matches)) => println!("stop jail"),
            ("", None) => help_app.print_help().unwrap(),
            _ => unreachable!(),
        };
    };
}

fn create() -> Result<jdb::Config, Box<Error>> {
    let mut db = JDB::open(Path::new(INDEX))?;
    let conf: jdb::Config = serde_json::from_reader(io::stdin())?;
    match db.find(conf.uuid.clone()) {
        None => db.insert(conf),
        Some(_) => Err(Box::new(ConflictError { uuid: conf.uuid })),
    }
}