use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::str;
use std::cmp::PartialEq;
use uuid::Uuid;

use serde_json;

use errors::{NotFoundError, ConflictError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "new_uuid")]
    pub uuid: String,
    alias: String,
    hostname: String,
    ram: u64,
    cpu: u64,
    disk: u64,
    #[serde(default = "bfalse")]
    autostart: bool
}
fn new_uuid() -> String {
    Uuid::new_v4().hyphenated().to_string()
}

fn bfalse() -> bool {
    false
}

#[derive(Debug, Serialize, Deserialize)]
struct IdxEntry {
    version: u32,
    uuid: String,
    root: String,
    state: String,
    jail_type: String,
}

impl PartialEq for IdxEntry {
    fn eq(&self, other: &IdxEntry) -> bool {
        self.uuid == other.uuid
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct Index {
    pub version: u32,
    pub entries: Vec<IdxEntry>,
}

#[derive(Debug)]
pub struct JDB<'a> {
    dir: &'a Path,
    index: Index,
}

impl<'a> JDB<'a> {
    /// Opens an JDB index file.
    ///„ # Arguments
    ///
    /// * `path` - Path of the **index file**, the locatio of the
    ///            file is also where the seperate configs live.
    ///
    /// # Example
    ///
    /// ```
    /// // Open jail config folder in /etc/jails
    /// use jdb::JDB;
    /// let db = JDB::open("/etc/jails/index");
    /// ```

    pub fn open(path: &'a str) -> Result<Self, Box<Error>> {
        let idx_file = Path::new(path);
        match File::open(idx_file.join("index")) {
            Ok(file) => {
                let index: Index = serde_json::from_reader(file)?;
                Ok(JDB {
                    index: index,
                    dir: Path::new(path),
                })
            }
            Err(_) => {
                let entries: Vec<IdxEntry> = Vec::new();
                let index: Index = Index {
                    version: 0,
                    entries: entries,
                };
                let db = JDB {
                    index: index,
                    dir:  Path::new(path),
                };
                db.save()?;
                Ok(db)

            }

        }
    }

    /// Inserts a config into the database, writes the config file
    /// and adds it to the index.
    pub fn insert(self: &'a mut JDB<'a>, config: Config) -> Result<Config, Box<Error>> {
        match self.find(&config.uuid) {
            None => {
                let mut path = self.dir.join(config.uuid.clone());
                path.set_extension("json");
                let file = File::create(&path)?;
                let mut root = String::from("/jails/");
                root.push_str(&config.uuid.clone());
                let e = IdxEntry {
                    version: 0,
                    uuid: config.uuid.clone(),
                    state: String::from("installing"),
                    jail_type: String::from("base"),
                    root: root,
                };
                self.index.entries.push(e);
                self.save()?;
                serde_json::to_writer(file, &config)?;
                Ok(config)
            }
            Some(_) => Err(ConflictError::bx(config.uuid.as_str())),
        }
    }

    /// Removes a jail with a given uuid from the index and removes it's
    /// config file.
    pub fn remove(self: &'a mut JDB<'a>, uuid: & str) -> Result<usize, Box<Error>> {
        match self.find(uuid) {
            None => Err(NotFoundError::bx(uuid)),
            Some(index) => {
                // remove the config file first
                let mut path = self.dir.join(uuid);
                path.set_extension("json");
                fs::remove_file(&path)?;
                self.index.entries.remove(index);
                self.save()?;
                Ok(index)
            }
        }
    }

    /// Prints the jdb database
    pub fn print(self: &'a JDB<'a>) {
        println!(
            "{:37} {:5} {:8} {:17} {}",
            "UUID",
            "TYPE",
            "RAM",
            "STATE",
            "ALIAS"
        );
        for e in &(self.index.entries) {
            self.print_entry(e);
        }
    }

    /// Reads the config file for a given entry
    fn config(self: &'a JDB<'a>, entry: &IdxEntry) -> Result<Config, Box<Error>> {
        let mut config_path = self.dir.join(entry.uuid.clone());
        config_path.set_extension("json");
        let config_file = File::open(config_path)?;
        let conf: Config = serde_json::from_reader(config_file)?;
        Ok(conf)
    }
    /// Saves the database
    fn save(self: &'a JDB<'a>) -> Result<usize, Box<Error>> {
        let path = self.dir.join("index");
        let file = File::create(path)?;
        serde_json::to_writer(file, &self.index)?;
        Ok(self.index.entries.len())
    }
    /// Finds an entry for a given uuid
    fn find(self: &'a JDB<'a>, uuid: & str) -> Option<usize> {
        self.index.entries.iter().position(
            |x| *x.uuid == *uuid,
        )
    }

    fn print_entry(self: &'a JDB<'a>, entry: &IdxEntry) {
        let conf = self.config(entry).unwrap();
        println!(
            "{:37} {:5} {:8} {:17} {}",
            conf.uuid,
            "OS",
            conf.ram,
            entry.state,
            conf.alias
        )
    }
}