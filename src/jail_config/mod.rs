//! Jail Configuration

use std::error::Error;
use std::fs::File;
use std::io::Read;

use serde_json;
use uuid::Uuid;


/// Jail configuration values
#[derive(Debug, Serialize, Deserialize)]
pub struct JailConfig {
    /// UUID of the jail
    #[serde(default = "new_uuid")]
    pub uuid: String,
    /// UUID of the imaage
    pub image_uuid: String,
    /// readable alias for the jail
    pub alias: String,
    /// hostname of the jail
    pub hostname: String,

    /// weather to start this jail on --startup
    pub autostart: Option<bool>,

    // Resources
    /// max physical memory in MB (memoryuse)
    pub max_physical_memory: u64,
    /// mac cpu usage 100 = 1 core (pcpu)
    pub cpu_cap: u64,
    /// max quota (zfs quota)
    quota: u64,

    /// SysV shared memory size, in bytes (shmsize)
    pub max_shm_memory: Option<u64>,

    /// locked memory (memorylocked)
    pub max_locked_memory: Option<u64>,

    /// maximum number of porocesses (maxproc)
    #[serde(default = "dflt_max_lwp")]
    pub max_lwps: u64,
}

impl JailConfig {
    /// Reads a new config from a file
    pub fn from_file(config_path: &str) -> Result<Self, Box<Error>> {
        let config_file = File::open(config_path)?;
        JailConfig::from_reader(config_file)
    }

    /// Reads the config from a reader
    pub fn from_reader<R>(reader: R) -> Result<Self, Box<Error>>
    where
        R: Read,
    {
        let mut conf: JailConfig = serde_json::from_reader(reader)?;
        let max_physical_memory = conf.max_physical_memory;
        if conf.max_shm_memory.is_none() {
            conf.max_shm_memory = Some(max_physical_memory);
        }
        if conf.max_locked_memory.is_none() {
            conf.max_locked_memory = Some(max_physical_memory);
        }
        if conf.autostart.is_none() {
            conf.autostart = Some(false);
        }
        Ok(conf)
    }


    /// Translates the config into resource controle limts
    pub fn rctl_limits(self: &JailConfig) -> Vec<String> {
        let mut res = Vec::new();
        let uuid = self.uuid.clone();
        let mut base = String::from("jail:");
        base.push_str(uuid.as_str());

        res.push(String::from("-a"));

        let max_physical_memory = self.max_physical_memory.to_string();
        let mut mem = base.clone();
        mem.push_str(":memoryuse:deny=");
        mem.push_str(max_physical_memory.as_str());
        mem.push_str("M");
        res.push(mem);

        let mut memorylocked = base.clone();
        memorylocked.push_str(":memorylocked:deny=");
        match self.max_locked_memory {
            Some(max_locked_memory) => {
                memorylocked.push_str(max_locked_memory.to_string().as_str())
            }
            None => memorylocked.push_str(max_physical_memory.as_str()),
        }
        memorylocked.push_str("M");
        res.push(memorylocked);

        let mut shmsize = base.clone();
        shmsize.push_str(":shmsize:deny=");
        match self.max_shm_memory {
            Some(max_shm_memory) => shmsize.push_str(max_shm_memory.to_string().as_str()),
            None => shmsize.push_str(max_physical_memory.as_str()),
        }
        shmsize.push_str("M");
        res.push(shmsize);

        let mut pcpu = base.clone();
        pcpu.push_str(":pcpu:deny=");
        pcpu.push_str(self.cpu_cap.to_string().as_str());
        res.push(pcpu);


        let mut maxproc = base.clone();
        maxproc.push_str(":maxproc:deny=");
        maxproc.push_str(self.max_lwps.to_string().as_str());
        res.push(maxproc);

        res
    }
}

fn dflt_max_lwp() -> u64 {
    2000
}

fn new_uuid() -> String {
    Uuid::new_v4().hyphenated().to_string()
}