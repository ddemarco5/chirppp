source /home/dominic/.profile

# build for x86_64 to test locally
#cargo build

# build for mipsel-musl for openwrt on our vocore
cargo build --target mipsel-unknown-linux-musl

# build for armv7 for our C.H.I.P. device - havn't tested yet
cargo build --target armv7-unknown-linux-gnueabihf

# scp our binary over to the vocore
echo "Vocore"
scp target/mipsel-unknown-linux-musl/debug/chirppp root@192.168.1.14:/mnt/sda1

# scp out binary over to the C.H.I.P.
echo "C.H.I.P."
scp target/armv7-unknown-linux-gnueabihf/debug/chirppp root@192.168.1.16:/root
