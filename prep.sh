#!/usr/local/bin/bash
#set -x

declare -a DIRS=("bin" "dev" "mnt" "proc" "tmp" "etc/defaults")
declare -a EXECS=("COPYRIGHT" "/libexec/ld-elf.so.1" "bin/sh" "/sbin/ifconfig" "usr/sbin/jail")

#### End user editable vars

ID=$(uuidgen)

zfs create -p zroot/jails/$ID

>&2 echo "Prepping outside jail..."

declare -a FILES

for d in "${DIRS[@]}"
do
    mkdir -p /zroot/jails/$ID/root/$d
    chown root:wheel /zroot/jails/$ID/root/$d
    chmod 775 /zroot/jails/$ID/root/$d
done

cp /etc/defaults/devfs.rules /zroot/jails/$ID/root/etc/defaults

for e in "${EXECS[@]}"
do
    FILES=("${FILES[@]}" $(ldd -a /$e 2> /dev/null | awk '/=>/{print $(NF-1)}'))
    FILES=("${FILES[@]}" "$e")
done

for f in "${FILES[@]}"
do
    mkdir -p /zroot/jails/$ID/root/$(dirname $f)
    cp /$f /zroot/jails/$ID/root/$f
done


>&2 echo "Prepping solitary confinement"
mkdir -p /zroot/jails/$ID/root/jail
fetch ftp://ftp.freebsd.org/pub/FreeBSD/releases/amd64/11.0-RELEASE/base.txz -o /tmp/base.txz
tar -xf /tmp/base.txz -C /zroot/jails/$ID/root/jail/

zfs snapshot zroot/jails/$ID@final

>&2 echo "Jail is ready. Snapshot if needed"
echo $ID
