//! Jail Configuration

use std::error::Error;
use std::fs::File;
use std::io::Read;
#[cfg(target_os = "freebsd")]
use std::process::Command;
#[cfg(target_os = "freebsd")]
use errors::GenericError;

use serde_json;
use uuid::Uuid;


/// Jail configuration values
#[derive(Debug, Serialize, Deserialize)]
pub struct NIC {
    /// Interface name
    interface: String,
    #[serde(default = "new_mac")]
    mac: String,
    nic_tag: String,
    ip: String,
    netmask: String,
    gateway: String,
    #[serde(default = "dflt_false")]
    primary: bool,
}

fn dflt_false() -> bool {
    false
}

#[cfg(target_os = "freebsd")]
static IFCONFIG: &'static str = "/sbin/ifconfig";

/// Interface after creating
pub struct IFace {
    /// epair
    pub epair: String,
    /// Startup script for the jail
    pub start_script: String,
    // end_script: String,
    /// post stop script
    pub poststop_script: String,
}
impl NIC {
    /// Creates the related interface
    #[cfg(target_os = "freebsd")]
    pub fn get_iface(self: &NIC, uuid: &str) -> Result<IFace, Box<Error>> {
        let output = Command::new(IFCONFIG)
            .args(&["epair", "create", "up"])
            .output()
            .expect("failed ifconfig");
        if !output.status.success() {
            return Err(GenericError::bx("could not create interface"));
        }
        let reply = String::from_utf8_lossy(&output.stdout);
        let epaira = reply.trim();
        let mut epair = String::from(epaira);

        epair.pop();
        let output = Command::new(IFCONFIG)
            .args(&["bridge0", "addm", epaira])
            .output()
            .expect("failed ifconfig");

        if !output.status.success() {
            return Err(GenericError::bx("could not add epair to bridge"));
        }

        let mut script = format!(
            "/sbin/ifconfig {epair}b inet {ip} {mask}; \
        /sbin/ifconfig {epair}b name {iface}; ",
            epair = epair,
            ip = self.ip,
            mask = self.netmask,
            iface = self.interface
        );
        if (self.primary) {
            let route = format!("/sbin/route add default -gateway {}; ", self.gateway);
            script.push_str(route.as_str())
        }
        let mut desc = String::from("VNic from jail ");
        desc.push_str(uuid);
        let output = Command::new(IFCONFIG)
            .args(&[epaira, "description", desc.as_str()])
            .output()
            .expect("failed to add descirption");
        if !output.status.success() {
            return Err(GenericError::bx("could not set description"));
        }
        let poststop = format!("{} {} destroy;", IFCONFIG, epaira);
        Ok(IFace {
            epair: epair,
            start_script: script,
            poststop_script: poststop,
        })
    }
    /// Creates the related interface
    #[cfg(not(target_os = "freebsd"))]
    pub fn get_iface(self: &NIC, _uuid: &str) -> Result<IFace, Box<Error>> {
        let epair = "epair0";
        let script = format!(
            "/sbin/ifconfig {epair}b inet {ip} {mask};\
        /sbin/ifconfig {epair}b name {iface};",
            epair = epair,
            ip = self.ip,
            mask = self.netmask,
            iface = self.interface
        );

        Ok(IFace {
            epair: String::from(epair),
            start_script: script,
            poststop_script: String::from(""),
        })
    }
}

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

    /// networks
    #[serde(default = "empty_nics")]
    pub nics: Vec<NIC>,

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

fn empty_nics() -> Vec<NIC> {
    Vec::new()
}

fn new_mac() -> String {
    String::from("00:00:00:00:00:00")
}