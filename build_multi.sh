source /home/dominic/.profile

# build for x86_64 to test locally
#cargo build

# build for mipsel-musl for openwrt on our vocore
cargo build --target mipsel-unknown-linux-musl

# build for armv7 for our C.H.I.P. device - havn't tested yet
cargo build --target armv7-unknown-linux-gnueabihf

# scp our binary over to the vocore
scp target/armv7-unknown-linux-gnueabihf/debug/chirppp root@192.168.1.10:/root
