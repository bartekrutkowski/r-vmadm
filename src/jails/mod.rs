//! Wrapper around the freebsd jail commands

use std::error::Error;
use errors::GenericError;
use std::collections::HashMap;
use jdb::Jail;
// We don't need command on non bsd systems
#[cfg(target_os = "freebsd")]
use std::process::Command;

#[derive(Debug)]
/// Basic information about a ZFS dataset
pub struct JailOSEntry {
    /// uuid of the jail
    pub uuid: String,
    /// os id of the jail
    pub id: u64,
}

/// starts a jail
#[cfg(target_os = "freebsd")]
pub fn start(jail: &Jail) -> Result<i32, Box<Error>> {
    let args = create_args(jail);
    let limits = rctl_limits(jail);
    debug!("Setting jail limits"; "vm" => jail.idx.uuid.clone());
    let output = Command::new("rctl").args(limits).output().expect(
        "limit failed",
    );
    debug!("Start jail"; "vm" => jail.idx.uuid.clone());
    let output = Command::new("jail").args(args).output().expect(
        "jail failed",
    );
    if output.status.success() {
        Ok(0)
    } else {
        Err(GenericError::bx("Could not delete jail"))
    }
}

/// pretend to starts a jail
#[cfg(not(target_os = "freebsd"))]
pub fn start(jail: &Jail) -> Result<i32, Box<Error>> {
    let args = create_args(jail);
    let limits = rctl_limits(jail);
    debug!("Setting jail limits"; "vm" => jail.idx.uuid.clone());
    println!("jail {:?}", limits);
    debug!("Start jail"; "vm" => jail.idx.uuid.clone());
    println!("jail {:?}", args);
    Ok(0)
}

fn rctl_limits(jail: &Jail) -> Vec<String> {
    let uuid = jail.config.uuid.clone();
    let mut base = String::from("jail:");
    base.push_str(uuid.as_str());

    let mut mem = base.clone();
    mem.push_str(":memoryuse:deny=");
    mem.push_str(jail.config.max_physical_memory.to_string().as_str());
    mem.push_str("M");

    let mut pcpu = base.clone();
    pcpu.push_str(":pcpu:deny=");
    pcpu.push_str(jail.config.cpu_cap.to_string().as_str());

    vec![String::from("-a"), mem, pcpu]

}

fn create_args(jail: &Jail) -> Vec<String> {
    let uuid = jail.idx.uuid.clone();
    let mut name = String::from("name=");
    name.push_str(uuid.as_str());
    let mut path = String::from("path=/");
    path.push_str(jail.idx.root.as_str());
    path.push_str("/root");
    let mut hostuuid = String::from("host.hostuuid=");
    hostuuid.push_str(uuid.as_str());
    let mut hostname = String::from("host.hostname=");
    hostname.push_str(jail.config.hostname.as_str());
    vec![
        String::from("-c"),
        String::from("persist"),
        name,
        path,
        hostuuid,
        hostname,
    ]
}

/// stops a jail
#[cfg(target_os = "freebsd")]
pub fn stop(uuid: &str) -> Result<i32, Box<Error>> {
    debug!("Dleting jail"; "vm" => uuid);
    let output = Command::new("jail").args(&["-r", uuid]).output().expect(
        "zfs list failed",
    );
    if output.status.success() {
        Ok(0)
    } else {
        Err(GenericError::bx("Could not delete jail"))
    }
}

/// pretend to stop a jail
#[cfg(not(target_os = "freebsd"))]
pub fn stop(uuid: &str) -> Result<i32, Box<Error>> {
    debug!("Dleting jail"; "vm" => uuid);
    Ok(0)
}

/// reads the zfs datasets in a pool
#[cfg(target_os = "freebsd")]
pub fn list() -> Result<HashMap<String, JailOSEntry>, Box<Error>> {
    debug!("Listing jails");
    let output = Command::new("jls")
        .args(&["-q", "jid", "name"])
        .output()
        .expect("zfs list failed");
    let reply = String::from_utf8_lossy(&output.stdout);
    let mut res = HashMap::new();


    for line in reply.split('\n').filter(|x| *x != "") {
        let entry = deconstruct_entry(line)?;
        res.insert(entry.uuid.clone(), entry);
        ()
    }
    Ok(res)
}

/// Reads a dummy jail
#[cfg(not(target_os = "freebsd"))]
pub fn list() -> Result<HashMap<String, JailOSEntry>, Box<Error>> {
    let reply = "1 fe0b9b05-1f3e-4b11-b0ae-8494bb6ecd53\n";
    let mut res = HashMap::new();

    for line in reply.split('\n').filter(|x| *x != "") {
        let entry = deconstruct_entry(line)?;
        res.insert(entry.uuid.clone(), entry);
        ()
    }
    Ok(res)
}

/// deconstructs a line from zfs list into an `ZFSEntry`.
fn deconstruct_entry(line: &str) -> Result<JailOSEntry, Box<Error>> {
    let mut parts = line.split(' ');
    let n0 = parts.next().ok_or_else(
        || GenericError::bx("JID field missing"),
    )?;
    let id: u64 = n0.parse()?;
    let uuid = parts.next().ok_or_else(
        || GenericError::bx("NAME field missing"),
    )?;

    Ok(JailOSEntry {
        uuid: String::from(uuid),
        id: id,
    })
}
