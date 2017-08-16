use std::io::Read;
use std::error::Error;
use std::fs::File;
use std::io::copy;

use config::Config;

use reqwest;
use serde_json;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use prettytable::Table;
use prettytable::format;
use prettytable::row::Row;
use prettytable::cell::Cell;



#[derive(Debug, Serialize, Deserialize, Clone)]
struct ImageFile {
    size: u64,
    compression: String,
    sha1: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Image {
    v: u32,
    uuid: Uuid,
    name: String,
    version: String,
    #[serde(rename = "type")]
    image_type: String,
    os: String,
    origin: Option<Uuid>,
    files: Vec<ImageFile>,
    published_at: DateTime<Utc>,
    public: bool,
    state: String,
    disabled: bool,
}

impl Image {
    pub fn from_reader<R>(reader: R) -> Result<Self, Box<Error>>
    where
        R: Read,
    {
        let image: Image = serde_json::from_reader(reader)?;
        return Ok(image);
    }

    pub fn list_from_reader<R>(reader: R) -> Result<Vec<Self>, Box<Error>>
    where
        R: Read,
    {
        let images: Vec<Image> = serde_json::from_reader(reader)?;
        return Ok(images);
    }
    fn print(&self, table: &mut Table, parsable: bool) {
        let date = format!("{}", self.published_at.format("%Y-%m-%d"));
        if parsable {
            println!(
                "{}:{}:{}:{}:{}:{}",
                self.uuid,
                self.name,
                self.version,
                self.os,
                self.image_type,
                date
            )
        } else {
            table.add_row(Row::new(vec![
                Cell::new(self.uuid.hyphenated().to_string().as_str()),
                Cell::new(self.name.as_str()),
                Cell::new(self.version.as_str()),
                Cell::new(self.os.as_str()),
                Cell::new(self.image_type.as_str()),
                Cell::new(date.as_str()),
            ]));

        }
    }
}

fn print_images(images: Vec<Image>, headerless: bool, parsable: bool) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_CLEAN);
    if !headerless {
        if parsable {
            println!(
                "{}:{}:{}:{}:{}:{}",
                "UUID",
                "NAME",
                "VERSION",
                "OS",
                "TYPE",
                "PUB"
            );
        } else {
            table.add_row(row!["UUID", "NAME", "VERSION", "OS", "TYPE", "PUB"]);
        }
    }
    for image in images.iter() {
        image.print(&mut table, parsable)
    }
    if !parsable {
        table.printstd()
    };

}
pub fn avail(config: &Config) -> Result<i32, Box<Error>> {
    debug!("Listing images"; "repo" => config.settings.repo.clone());
    let resp = reqwest::get(config.settings.repo.as_str())?;
    let images = Image::list_from_reader(resp)?;
    print_images(images, false, false);
    Ok(0)
}

pub fn get(config: &Config, uuid: Uuid) -> Result<i32, Box<Error>> {
    let mut url = config.settings.repo.clone();
    let uuid_str = uuid.hyphenated().to_string();
    url.push('/');
    url.push_str(uuid_str.as_str());
    debug!("Fethcing image"; "repo" => config.settings.repo.clone(),
           "uuid" => uuid_str.clone(), "url" => url.clone());
    let resp = reqwest::get(url.as_str())?;
    let image = Image::from_reader(resp)?;
    let j = serde_json::to_string_pretty(&image)?;
    println!("{}\n", j);
    //print_images(images, false, false);
    Ok(0)
}

pub fn import(config: &Config, uuid: Uuid) -> Result<i32, Box<Error>> {
    let mut url = config.settings.repo.clone();
    let uuid_str = uuid.hyphenated().to_string();
    url.push('/');
    url.push_str(uuid_str.as_str());
    debug!("Fethcing image"; "repo" => config.settings.repo.clone(),
           "uuid" => uuid_str.clone(), "url" => url.clone());
    let resp = reqwest::get(url.as_str())?;
    let image = Image::from_reader(resp)?;
    let file_info = image.files[0].clone();
    let mut file = uuid_str.clone();
    file.push_str(".");
    file.push_str(file_info.compression.as_str());
    url.push_str("/file");

    let mut out = File::create(file)?;
    let mut resp = reqwest::get(url.as_str())?;
    copy(&mut resp, &mut out)?;
    //print_images(images, false, false);
    Ok(0)
}
