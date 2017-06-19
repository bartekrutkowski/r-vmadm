
use std::error::Error;
use std::process::Command;
use errors::GenericError;


#[derive(Debug)]
/// Basic information about a ZFS dataset
pub struct ZFSEntry {
    name: String,
    used: u64,
    avail: u64,
    refer: u64,
    mountpoint: String,
}

/// reads the zfs datasets in a pool
pub fn list(pool: &str) -> Result<Vec<ZFSEntry>, Box<Error>> {
    let output = Command::new("zfs")
        .args(&["list", "-p", "-H", "-r", "-d1", pool])
        .output()
        .expect("zfs list failed");
    let reply = String::from_utf8_lossy(&output.stdout);
    let mut res = Vec::new();

    //Ok(reply.split('\n').filter(|x| *x != "").map(&deconstruct_entry).collect())
    for line in reply.split('\n').filter(|x| *x != "") {
        let entry = deconstruct_entry(line)?;
        res.push(entry)
    }
    Ok(res)
}

/// reads the zfs datasets in a pool
pub fn get(pool: &str, dataset: &str) -> Result<ZFSEntry, Box<Error>> {
    let mut path = String::from(pool);
    path.push('/');
    path.push_str(dataset);
    let output = Command::new("zfs")
        .args(&["list", "-p", "-H", path.as_str()])
        .output()
        .expect("zfs list failed");
    if output.status.success() {
        let reply = String::from_utf8_lossy(&output.stdout).to_string();
        deconstruct_entry(reply.as_str())
    } else {
        Err(GenericError::bx("Failed to get dataset"))
    }
}

/// create a zfs datasets in a pool
pub fn create(dataset: &str) -> Result<i32, Box<Error>> {
    let output = Command::new("zfs")
        .args(&["create", dataset])
        .output()
        .expect("zfs create failed");
    if output.status.success() {
        Ok(0)
    } else {
        Err(GenericError::bx("Failed create dataset"))
    }
}

/// destroy the zfs datasets in a pool
pub fn destroy(dataset: &str) -> Result<i32, Box<Error>> {
    let output = Command::new("zfs")
        .args(&["destroy", dataset])
        .output()
        .expect("zfs create failed");
    if output.status.success() {
        Ok(0)
    } else {
        Err(GenericError::bx("Failed destroy dataset"))
    }
}

/// deconstructs a line from zfs list into an `ZFSEntry`.
fn deconstruct_entry(line: &str) -> Result<ZFSEntry, Box<Error>> {
    let mut parts = line.split('\t');
    let name = parts.next().ok_or_else(
        || GenericError::bx("NAME field missing"),
    )?;
    let n0 = parts.next().ok_or_else(
        || GenericError::bx("USED field missing"),
    )?;
    let used: u64 = n0.parse()?;
    let n1 = parts.next().ok_or_else(
        || GenericError::bx("AVAIL field missing"),
    )?;
    let avail: u64 = n1.parse()?;
    let n2 = parts.next().ok_or_else(
        || GenericError::bx("REFER field missing"),
    )?;
    let refer: u64 = n2.parse()?;
    let mountpoint = parts.next().ok_or_else(
        || GenericError::bx("MOUNTPOINT field missing"),
    )?;

    Ok(ZFSEntry {
        name: String::from(name),
        used: used,
        avail: avail,
        refer: refer,
        mountpoint: String::from(mountpoint),
    })
}