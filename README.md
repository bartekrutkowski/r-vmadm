# Introduction

The goal of this is to provide a fast, efficient utility to manage BSD jails. The CLI is designed to mirror SmartOS's vmadm, and we borrow ideas from other Solaris zone utilities where applicable.

## Design

vmadm describes jails as JSON files. These files are compatible with vmadm's files but represent only a subset of the total options.

Data lives in `/etc/jails`, being an index file and description file per zone. We do this to mimic the way zomeadm works on Solaris but replaces xml+plaintext with JSON.

Images are ZFS datasets that get cloned for a new jail, both living under a given prefix (that can be defined).
## installation

1. Install rust (https://rustup.rs/)
2. Clone this repository
3. Build the binary `cargo build --release`
4. Copy the executable `cp target/release/vmadm /usr/local/sbin`
5. Enable rctl: `echo kern.racct.enable=1 >> /boot/loader.conf`
6. Reboot :(
7. Create the jails folder: `mkdir /etc/jails`
8. Create the main config file: `echo 'pool = "zroot/jails"' > /etc/vmadm.toml`
9. Create a zfs dataset: `zfs create zroot/jails `
10. Download a/the datase `curl -O https://s3.amazonaws.com/datasets.project-fifo.net/freebsd/e022d0f8-5630-11e7-b660-9b2d243d4404.xz`
11. Extract the dataset `xzcat e022d0f8-5630-11e7-b660-9b2d243d4404.xz | zfs receive zroot/jails/e022d0f8-5630-11e7-b660-9b2d243d4404`
12. Create a jail: cat example.json | vmadm create

## usage
```
vmadm compatible jail manager

USAGE:
    vmadm [FLAGS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
        --startup
    -V, --version    Prints version information
    -v               Sets the level of verbosity

SUBCOMMANDS:
    console    connects to a jails console
    create     creates a new jail
    delete     deletes a jail
    get        gets a jails configuration
    help       Prints this message or the help of the given subcommand(s)
    list       lists jails
    reboot     reboot a jail
    start      starts a jail
    stop       stops a jail
    update     updates a jail
```