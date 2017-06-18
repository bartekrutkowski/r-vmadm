
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
    println!("{:?}", reply);
    let mut res = Vec::new();

    //Ok(reply.split('\n').filter(|x| *x != "").map(&deconstruct_entry).collect())
    for line in reply.split('\n').filter(|x| *x != "") {
        let entry = deconstruct_entry(line)?;
        res.push(entry)
    }
    Ok(res)
}
/// deconstructs a line from zfs list into an ZFSEntry.
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