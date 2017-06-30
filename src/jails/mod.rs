//! Wrapper around the freebsd jail commands

use std::error::Error;
use errors::GenericError;
use std::collections::HashMap;
use jdb::Jail;
use std::process::Command;
use jail_config::IFace;

#[derive(Debug)]
/// Basic information about a ZFS dataset
pub struct JailOSEntry {
    /// uuid of the jail
    pub uuid: String,
    /// os id of the jail
    pub id: u64,
}

#[cfg(target_os = "freebsd")]
static UMOUNT: &'static str = "umount";
#[cfg(target_os = "freebsd")]
static MOUNT: &'static str = "mount";
#[cfg(target_os = "freebsd")]
static RCTL: &'static str = "rctl";
#[cfg(target_os = "freebsd")]
static JAIL: &'static str = "jail";
#[cfg(not(target_os = "freebsd"))]
static MOUNT: &'static str = "echo";
#[cfg(not(target_os = "freebsd"))]
static UMOUNT: &'static str = "echo";
#[cfg(not(target_os = "freebsd"))]
static RCTL: &'static str = "echo";
#[cfg(not(target_os = "freebsd"))]
static JAIL: &'static str = "echo";

/// starts a jail
pub fn start(jail: &Jail) -> Result<i32, Box<Error>> {
    let args = create_args(jail)?;
    let limits = jail.config.rctl_limits();
    debug!("Setting jail limits"; "vm" => jail.idx.uuid.clone(), "limits" => limits.clone().join(" "));
    let output = Command::new(RCTL).args(limits.clone()).output().expect(
        "limit failed",
    );
    if !output.status.success() {
        crit!("failed to set resource limits"; "vm" => jail.idx.uuid.clone());
        return Err(GenericError::bx("Could not set jail limits"));
    }

    let mut devfs = String::from("/");
    devfs.push_str(jail.idx.root.as_str());
    devfs.push_str("/root/dev");
    let devfs_args = vec!["-t", "devfs", "devfs", devfs.as_str()];

    debug!("mounting devfs"; "vm" => jail.idx.uuid.clone(), "args" =>devfs_args.clone().join(" "));
    let _output = Command::new(MOUNT).args(devfs_args).output().expect(
        "failed to mount devfs",
    );

    debug!("Start jail"; "vm" => jail.idx.uuid.clone(), "args" => args.clone().join(" "));
    let output = Command::new(JAIL).args(args.clone()).output().expect(
        "jail failed",
    );
    if output.status.success() {
        Ok(0)
    } else {
        crit!("Failed to start jail"; "vm" => jail.idx.uuid.clone());
        Err(GenericError::bx("Could not start jail"))
    }
}

fn create_args(jail: &Jail) -> Result<Vec<String>, Box<Error>> {
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
    let mut res = vec![
        String::from("-c"),
        String::from("persist"),
        name,
        path,
        hostuuid,
        hostname,
    ];

    // Basic stuff I don't know what it does
    res.push(String::from("devfs_ruleset=4"));
    res.push(String::from("securelevel=2"));
    res.push(String::from("sysvmsg=new"));
    res.push(String::from("sysvsem=new"));
    res.push(String::from("sysvshm=new"));

    // let mut exec_stop = String::from("exec.stop=");
    let mut exec_start = String::from("exec.start=");
    let mut exec_poststop = String::from("exec.poststop=");
    res.push(String::from("vnet=new"));

    for nic in jail.config.nics.iter() {
        // see https://lists.freebsd.org/pipermail/freebsd-jail//2016-December/003305.html
        let iface: IFace = nic.get_iface(uuid.as_str())?;
        let mut vnet_iface = String::from("vnet.interface=");
        vnet_iface.push_str(iface.epair.as_str());
        vnet_iface.push('b');

        res.push(vnet_iface);

        exec_start.push_str(iface.start_script.as_str());
        exec_poststop.push_str(iface.poststop_script.as_str());
    }
    res.push(exec_start);
    if !jail.config.nics.is_empty() {
        // exec_stop.push('"');
        // res.push(exec_stop);
        res.push(exec_poststop);
    };
    Ok(res)

}

/// stops a jail
pub fn stop(jail: &Jail) -> Result<i32, Box<Error>> {
    debug!("Dleting jail"; "vm" => jail.idx.uuid.clone());
    let output = Command::new(JAIL)
        .args(&["-r", jail.idx.uuid.clone().as_str()])
        .output()
        .expect("zfs list failed");
    if !output.status.success() {
        crit!("Failed to stop jail"; "vm" => jail.idx.uuid.clone());
        return Err(GenericError::bx("Could not stop jail"));
    }

    let mut devfs = String::from("/");
    devfs.push_str(jail.idx.root.as_str());
    devfs.push_str("/root/dev");
    let devfs_args = vec![devfs.as_str()];

    debug!("un mounting devfs"; "vm" => jail.idx.uuid.clone(), "args" =>devfs_args.clone().join(" "));
    let _output = Command::new(UMOUNT).args(devfs_args).output().expect(
        "failed to mount devfs",
    );
    Ok(0)
}

/// reboots a jail
pub fn reboot(jail: &Jail) -> Result<i32, Box<Error>> {
    debug!("Dleting jail"; "vm" => jail.idx.uuid.clone());
    let output = Command::new(JAIL)
        .args(&["-rc", jail.idx.uuid.clone().as_str()])
        .output()
        .expect("zfs list failed");
    if !output.status.success() {
        crit!("Failed to restart jail"; "vm" => jail.idx.uuid.clone());
        return Err(GenericError::bx("Could not restart jail"));
    }
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
    let reply = "1 ge0b9b05-1f3e-4b11-b0ae-8494bb6ecd53\n";
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
