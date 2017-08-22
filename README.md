# Introduction

The goal of this is to provide a fast, efficient utility to manage BSD jails. The CLI is designed to mirror SmartOS's vmadm, and we borrow ideas from other Solaris zone utilities where applicable.

[![asciicast](https://asciinema.org/a/M8sjN0FC64JPBWZqjKIG5sx2q.png)](https://asciinema.org/a/M8sjN0FC64JPBWZqjKIG5sx2q)

## Design

vmadm describes jails as JSON files. These files are compatible with vmadm's files but represent only a subset of the total options.

Data lives in `/usr/local/etc/vmadm`, being an index file and description file per zone. We do this to mimic the way zomeadm works on Solaris but replaces xml+plaintext with JSON.

Images are ZFS datasets that get cloned for a new jail, both living under a given prefix (that can be defined).

## Prerequirements


### libraries

```bash

pkg install pkgconf openssl
```

### bridge interface
We need to add bridge0 interface to the `/etc/rc.conf` (`em0` might differ for you)

```bash
# set up a bridge interfaces for jails
cloned_interfaces="bridge0"

# plumb interface em0 into bridge0
ifconfig_bridge0="addm em0"
```


### vnet
vnet needs to be enabled in the kerne.


### rctrl

Rctrl needs to be enabled
```bash
echo kern.racct.enable=1 >> /boot/loader.conf
```

### zfs
We need a dataset for the jails:

```bash
zfs create zroot/jails
```

### reboot

Some of the steps above require a reboot, there is however no reason not just do it once at the very end.

## installation

1. Install rust (https://rustup.rs/)
2. Clone this repository
3. Build the binary `cargo build --release`
4. Copy the executable `cp target/release/vmadm /usr/local/sbin`
5. Create the jails folder: `mkdir /usr/local/etc/vmadm`
6. Create the images folder `mkdir -p /var/imgadm/images`
7. Create the main config file: `echo 'pool = "zroot/jails"\n[networks]\nadmin = "bridge0"' > /usr/local/etc/vmadm.toml`
8. Download a/the datase `curl -O https://s3.amazonaws.com/datasets.project-fifo.net/freebsd/e022d0f8-5630-11e7-b660-9b2d243d4404.xz`
9. Extract the dataset `xzcat e022d0f8-5630-11e7-b660-9b2d243d4404.xz | zfs receive zroot/jails/e022d0f8-5630-11e7-b660-9b2d243d4404`
10. Create a jail: cat example.json | vmadm create


The devfs ruleset to used can be adjusted in the `/usr/local/etc/vmadm.toml` by adding `devfs_ruleset = <rule number>`.

## update

If you ran 0.1.0 of the vmadm some path's have changed:

`/etc/vmadm.toml` is now `/usr/local/etc/vmadm.toml`

And

`/etc/jails` is now `/usr/local/etc/vmadm`

Moving those directories and files is all that's required.

## usage
```
vmadm 0.1.0
Heinz N. Gies <heinz@project-fifo.net>
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
    images     image subcommands
    info       gets a info for a hardware virtualized vm
    list       lists jails
    reboot     reboot a jail
    start      starts a jail
    stop       stops a jail
    update     updates a jail
```

Travis CI scripts form: https://github.com/japaric/trust
