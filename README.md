# peachps

A TCP/IP protocol stack in Rust.  

## How to Use

```shell
sudo ip tuntap add mode tap tap0
sudo ip link set tap0 up
cd examples/etherdump
cargo build
sudo ./tardet/debug/etherdump tap0
```
