# Introduction

The goal of this is to provide a fast, efficient utility to manage BSD jails. The CLI is designed to mirror SmartOS's vmadm and other ideas are borrowed from other Solaris zone utilities where applicable.

## Design

Jails are described as JSON files. These files are compatible with vmadm's files but represent only a subset of the total options.

Data lives in `/etc/jails`, being an index file and description file per zone. This mostly mimics the way zomeadm works on Solaris, but replaces xml+plaintext with json.

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
10. Download a/the datase `curl -O curl -O https://s3.amazonaws.com/datasets.project-fifo.net/freebsd/e022d0f8-5630-11e7-b660-9b2d243d4404.xz`
11. Extract the dataset `zxcat e022d0f8-5630-11e7-b660-9b2d243d4404.xz | zfs receive zroot/jails/e022d0f8-5630-11e7-b660-9b2d243d4404`
12. Create a jail: cat example.json | vmadm create
