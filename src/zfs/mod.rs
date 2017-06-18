
use std::process::Command;

#[derive(Debug)]
pub struct ZFSEntry {
    name: String,
    used: u64,
    avail: u64,
    refer: u64,
    mountpoint: String,
}

pub fn list(pool: &str) -> Vec<ZFSEntry> {
    let output = Command::new("zfs").args(&["list", "-p", "-H", "-r", pool]).output().expect(
        "zfs list failed",
    );
    let reply = String::from_utf8_lossy(&output.stdout);
    println!("{:?}", reply);
    reply.split("\n").filter(|x| *x != "").map(&deconstruct_entry).collect()
}

fn deconstruct_entry(line: &str) -> ZFSEntry {
    let mut parts = line.split("\t");
    let name = parts.next().unwrap();
    let used: u64 = parts.next().unwrap().parse().unwrap();
    let avail: u64 = parts.next().unwrap().parse().unwrap();
    let refer: u64 = parts.next().unwrap().parse().unwrap();
    let mountpoint = parts.next().unwrap();

    ZFSEntry {
        name: String::from(name),
        used: used,
        avail: avail,
        refer: refer,
        mountpoint: String::from(mountpoint),
    }
}