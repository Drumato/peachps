# peachps

A TCP/IP protocol stack in Rust.  

## How to Use

```shell
sudo ip tuntap add mode tap tap0
sudo ip link set tap0 up
sudo ip addr add 192.168.33.155/24 dev tap0
cd examples/etherdump
cargo build
sudo ./tardet/debug/etherdump tap0
```
