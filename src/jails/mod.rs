//! Wrapper around the freebsd jail commands

use std::error::Error;
use errors::GenericError;
use std::collections::HashMap;
use std::process::Command;
use jail_config::IFace;
use config::Config;
use uuid::Uuid;
use jdb::IdxEntry;
use jail_config::JailConfig;

#[derive(Debug)]
/// Basic information about a ZFS dataset
pub struct JailOSEntry {
    /// uuid of the jail
    pub uuid: String,
    /// os id of the jail
    pub id: u64,
}

struct CreateArgs {
    args: Vec<String>,
    ifs: Vec<IFace>,
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

#[cfg(target_os = "freebsd")]
static IFCONFIG: &'static str = "/sbin/ifconfig";
#[cfg(not(target_os = "freebsd"))]
static IFCONFIG: &'static str = "echo";


/// Jail config
pub struct Jail<'a> {
    /// Index refference
    pub idx: &'a IdxEntry,
    /// Jail configuration
    pub config: JailConfig,
    /// Record from the OS
    pub inner: Option<&'a JailOSEntry>,
    /// Record from the outer OS jail
    pub outer: Option<&'a JailOSEntry>,
}

impl<'a> Jail<'a> {
    /// starts a jail
    pub fn start(&self, config: &Config) -> Result<i32, Box<Error>> {
        self.set_rctl()?;
        self.mount_devfs()?;
        let CreateArgs { args, ifs } = create_args(config, self)?;
        debug!("Start jail"; "vm" => self.idx.uuid.hyphenated().to_string(), "args" => args.clone().join(" "));
        let id = start_jail(&self.idx.uuid, args)?;
        let id_str = id.to_string();
        let mut jprefix = String::from("j");
        jprefix.push_str(id_str.as_str());
        jprefix.push(':');
        for iface in ifs.iter() {
            let mut epair = String::from(iface.epair.clone());
            epair.push('a');
            let mut target_name = jprefix.clone();
            target_name.push_str(iface.iface.as_str());
            let args = vec![epair, String::from("name"), target_name];
            debug!("renaiming epair"; "vm" => self.idx.uuid.hyphenated().to_string(), "args" => args.clone().join(" "));
            let output = Command::new(IFCONFIG).args(args.clone()).output().expect(
                "ifconfig failed",
            );
            if !output.status.success() {
                crit!("failed to rename interface"; "vm" => self.idx.uuid.hyphenated().to_string());
            }
        }
        Ok(0)
    }

    /// stops a jail
    pub fn stop(&self) -> Result<i32, Box<Error>> {
        debug!("Dleting jail"; "vm" => self.idx.uuid.hyphenated().to_string());
        let output = Command::new(JAIL)
            .args(&["-r", self.idx.uuid.hyphenated().to_string().as_str()])
            .output()
            .expect("zfs list failed");
        if !output.status.success() {
            crit!("Failed to stop jail"; "vm" => self.idx.uuid.hyphenated().to_string());
            return Err(GenericError::bx("Could not stop jail"));
        }

        let mut devfs = String::from("/");
        devfs.push_str(self.idx.root.as_str());
        devfs.push_str("/root/dev");
        let devfs_args = vec![devfs.as_str()];

        debug!("un mounting devfs in outer jail"; "vm" => self.idx.uuid.hyphenated().to_string(), "args" => devfs_args.clone().join(" "));
        let output = Command::new(UMOUNT).args(devfs_args).output().expect(
            "failed to mount devfs in outer jail",
        );
        if !output.status.success() {
            crit!("failed to mount devfs in outer jail"; "vm" => self.idx.uuid.hyphenated().to_string());
        }

        let mut devfs = String::from("/");
        devfs.push_str(self.idx.root.as_str());
        devfs.push_str("/root/jail/dev");
        let devfs_args = vec![devfs.as_str()];

        debug!("un mounting devfs in inner jail"; "vm" => self.idx.uuid.hyphenated().to_string(), "args" =>devfs_args.clone().join(" "));
        let output = Command::new(UMOUNT).args(devfs_args).output().expect(
            "failed to mount devfs in inner jail",
        );
        if !output.status.success() {
            crit!("failed to mount devfs in inner jail"; "vm" => self.idx.uuid.hyphenated().to_string());
        }

        let _ = self.remove_rctl();
        match self.outer {
            Some(outer) => {
                let id_str = outer.id.to_string();
                let mut jprefix = String::from("j");
                jprefix.push_str(id_str.as_str());
                jprefix.push(':');
                for nic in self.config.nics.clone() {
                    let mut target_name = jprefix.clone();
                    target_name.push_str(nic.interface.as_str());
                    let args = vec![target_name, String::from("destroy")];
                    debug!("renaiming epair"; "vm" => self.idx.uuid.hyphenated().to_string(), "args" => args.clone().join(" "));
                    let output = Command::new(IFCONFIG).args(args.clone()).output().expect(
                        "ifconfig failed",
                    );
                    if !output.status.success() {
                        crit!("failed to rename interface"; "vm" => self.idx.uuid.hyphenated().to_string());
                    }
                }
            }
            None => {
            crit!("Failed to get outer jail id to delete interfaces"; "vm" => self.idx.uuid.hyphenated().to_string())
            }
        }

        Ok(0)
    }

    fn set_rctl(&self) -> Result<i32, Box<Error>> {
        let limits = self.config.rctl_limits();
        debug!("Setting jail limits"; "vm" => self.idx.uuid.hyphenated().to_string(), "limits" => limits.clone().join(" "));
        let output = Command::new(RCTL).args(limits.clone()).output().expect(
            "limit failed",
        );
        if !output.status.success() {
            crit!("failed to set resource limits"; "vm" => self.idx.uuid.hyphenated().to_string());
            return Err(GenericError::bx("Could not set jail limits"));
        }
        Ok(0)
    }

    fn mount_devfs(&self) -> Result<i32, Box<Error>> {
        let mut devfs = String::from("/");
        devfs.push_str(self.idx.root.as_str());
        devfs.push_str("/root/dev");
        let devfs_args = vec!["-t", "devfs", "devfs", devfs.as_str()];

        debug!("mounting devfs in outer jail"; "vm" => self.idx.uuid.hyphenated().to_string(), "args" =>devfs_args.clone().join(" "));
        let output = Command::new(MOUNT).args(devfs_args).output().expect(
            "failed to mount devfs in outer jail",
        );

        if !output.status.success() {
            crit!("failed to mount ounter devfs"; "vm" => self.idx.uuid.hyphenated().to_string());
            return Err(GenericError::bx("Could mount outer devfs"));
        }

        let mut devfs = String::from("/");
        devfs.push_str(self.idx.root.as_str());
        devfs.push_str("/root/jail/dev");
        let devfs_args = vec!["-t", "devfs", "devfs", devfs.as_str()];

        debug!("mounting devfs in inner jail"; "vm" => self.idx.uuid.hyphenated().to_string(), "args" =>devfs_args.clone().join(" "));
        let output = Command::new(MOUNT).args(devfs_args).output().expect(
            "failed to mount devfs in inner jail",
        );
        if !output.status.success() {
            crit!("failed to mount inner devfs"; "vm" => self.idx.uuid.hyphenated().to_string());
            return Err(GenericError::bx("Could not remove resource limits"));
        }

        Ok(0)
    }

    fn remove_rctl(&self) -> Result<i32, Box<Error>> {
        let mut prefix = String::from("jail:");
        prefix.push_str(self.idx.uuid.hyphenated().to_string().as_str());
        let limit_args = vec!["-r", prefix.as_str()];
        debug!("removing rctl limits"; "vm" => self.idx.uuid.hyphenated().to_string(), "args" => limit_args.clone().join(" "));
        let output = Command::new(RCTL).args(limit_args).output().expect(
            "rctl failed",
        );

        if !output.status.success() {
            crit!("failed to remove resource limits"; "vm" => self.idx.uuid.hyphenated().to_string());
            return Err(GenericError::bx("Could not remove resource limits"));
        }
        Ok(0)
    }
}

#[cfg(not(target_os = "freebsd"))]
fn start_jail(_uuid: &Uuid, _args: Vec<String>) -> Result<u64, Box<Error>> {
    Ok(42)
}

#[cfg(target_os = "freebsd")]
fn start_jail(uuid: &Uuid, args: Vec<String>) -> Result<u64, Box<Error>> {
    let output = Command::new(JAIL).args(args.clone()).output().expect(
        "jail failed",
    );
    let reply = String::from_utf8_lossy(&output.stdout).into_owned();
    if output.status.success() {
        // the Jail command has a bug that it will not honor -q
        // so everything but the first line might be garbage we have to
        // ignore.
        let mut lines = reply.lines();
        let first = lines.next().unwrap();
        // this seems odd but we guarnatee our ID is a int this way
        let id: u64 = first.trim().parse().unwrap();
        Ok(id)
    } else {
        crit!("Failed to start jail"; "vm" => uuid.hyphenated().to_string().as_str());
        Err(GenericError::bx(reply.as_str()))
    }
}

fn create_args(config: &Config, jail: &Jail) -> Result<CreateArgs, Box<Error>> {
    let uuid = jail.idx.uuid.hyphenated().to_string();
    let mut name = String::from("name=");
    name.push_str(uuid.as_str());
    let mut path = String::from("path=/");
    path.push_str(jail.idx.root.as_str());
    path.push_str("/root");
    let mut hostuuid = String::from("host.hostuuid=");
    hostuuid.push_str(uuid.as_str());
    let mut hostname = String::from("host.hostname=");
    hostname.push_str(jail.config.hostname.as_str());
    let mut args = vec![
        String::from("-i"),
        String::from("-c"),
        String::from("persist"),
        name,
        path,
        hostuuid,
        hostname,
    ];
    let mut ifs = Vec::new();

    // Basic stuff I don't know what it does
    let mut devfs_ruleset = String::from("devfs_ruleset=");
    devfs_ruleset.push_str(config.settings.devfs_ruleset.to_string().as_str());
    args.push(devfs_ruleset);
    args.push(String::from("securelevel=2"));
    args.push(String::from("sysvmsg=new"));
    args.push(String::from("sysvsem=new"));
    args.push(String::from("sysvshm=new"));

    // for nested jails
    args.push(String::from("allow.raw_sockets"));
    args.push(String::from("children.max=1"));


    // let mut exec_stop = String::from("exec.stop=");
    let mut exec_start = String::from("exec.start=");
    args.push(String::from("vnet=new"));
    for nic in jail.config.nics.iter() {
        // see https://lists.freebsd.org/pipermail/freebsd-jail//2016-December/003305.html
        let iface: IFace = nic.get_iface(config, &jail.idx.uuid)?;
        ifs.push(iface.clone());
        let mut vnet_iface = String::from("vnet.interface=");
        vnet_iface.push_str(iface.epair.as_str());
        vnet_iface.push('b');

        args.push(vnet_iface);

        exec_start.push_str(iface.start_script.as_str());
    }
    if !jail.config.nics.is_empty() {
        exec_start.push_str("/sbin/ifconfig lo0 127.0.0.1 up; ");
    };
    // inner jail configuration
    exec_start.push_str("jail -c");
    exec_start.push_str(" persist name=");
    exec_start.push_str(uuid.as_str());
    exec_start.push_str(" host.hostname=");
    exec_start.push_str(jail.config.hostname.as_str());
    exec_start.push_str(" path=/jail");
    exec_start.push_str(" ip4=inherit");
    exec_start.push_str(" devfs_ruleset=4");
    exec_start.push_str(" securelevel=2");
    exec_start.push_str(" sysvmsg=new");
    exec_start.push_str(" sysvsem=new");
    exec_start.push_str(" sysvshm=new");
    exec_start.push_str(" allow.raw_sockets");
    exec_start.push_str(" exec.start='sh /etc/rc'");

    args.push(exec_start);
    Ok(CreateArgs { args, ifs })
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
    let reply = "1 00000000-1f3e-4b11-b0ae-8494bb6ecd52\n2 00000000-1f3e-4b11-b0ae-8494bb6ecd52.00000000-1f3e-4b11-b0ae-8494bb6ecd52\n";
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
