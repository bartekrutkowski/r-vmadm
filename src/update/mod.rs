//! Update for a jail
use config::Config;
use jail_config::JailConfig;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use serde_json;
use uuid::Uuid;

/// Jail update
#[derive(Debug, Deserialize, Clone)]
pub struct JailUpdate {
    /// readable alias for the jail
    alias: Option<String>,
    /// hostname of the jail
    hostname: Option<String>,
    /// weather to start this jail on --startup
    autostart: Option<bool>,
    /// max physical memory in MB (memoryuse)
    max_physical_memory: Option<u64>,
    /// mac cpu usage 100 = 1 core (pcpu)
    cpu_cap: Option<u64>,
    /// max quota (zfs quota)
    //    quota: u64,
    /// SysV shared memory size, in bytes (shmsize)
    max_shm_memory: Option<u64>,

    /// locked memory (memorylocked)
    max_locked_memory: Option<u64>,

    /// maximum number of porocesses (maxproc)
    max_lwps: Option<u64>,

    // Metadata fields w/o effect on vmadm at the moment
    archive_on_delete: Option<bool>,
    billing_id: Option<Uuid>,
    do_not_inventory: Option<bool>,
    // Currently has no effect
    dns_domain: Option<String>,

    owner_uuid: Option<Uuid>,
    package_name: Option<String>,
    package_version: Option<String>,
}

macro_rules! update {
    ( $src:ident, $target:ident; $($field:ident),+)  => (
        $(
            match $src.$field {
                Some(ref value) => $target.$field = value.clone(),
                _ => ()
            }
        )*
    );
}
macro_rules! update_option {
    ( $src:ident, $target:ident; $($field:ident),+)  => (
        $(
            match $src.$field {
                Some(ref value) => $target.$field = Some(value.clone()),
                _ => ()
            }
        )*
    );
}
impl JailUpdate {
    /// Reads a new config from a file
    pub fn from_file(config_path: &str) -> Result<Self, Box<Error>> {
        let config_file = File::open(config_path)?;
        JailUpdate::from_reader(config_file)
    }

    /// Reads the config from a reader
    pub fn from_reader<R>(reader: R) -> Result<Self, Box<Error>>
    where
        R: Read,
    {
        let update: JailUpdate = serde_json::from_reader(reader)?;
        return Ok(update);
    }
    pub fn apply(&self, config: JailConfig) -> JailConfig {
        let mut c = config.clone();
        update!(self, c;
            autostart,
            alias,
            hostname,
            max_physical_memory,
            cpu_cap,
            max_lwps,
            dns_domain

        );
        update_option!(self, c;
            max_shm_memory,
            max_locked_memory,
            archive_on_delete,
            billing_id,
            do_not_inventory,
            owner_uuid,
            package_name,
            package_version
        );
        return c;
    }
}