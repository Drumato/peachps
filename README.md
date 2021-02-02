# peachps
TCP/IP protocol stack written by Rust.

## How To Run

Vagrantfileを用意している．  
この設定から，`node1/2` という2つのVMを起動可能．  
プライベートネットワークを作っているので，ゲストOS同士で通信が可能，という状態．  
`node1` には `192.168.11.11` が，  
`node2` には `192.168.11.12` が割り当てられている．  
これらを用いてpeachpsをテストする．  

```text
$ vagrant up
$ vagrant ssh node1/node2

# In VM 
$ sudo apt update -y
$ sudo apt upgrade -y
$ sudo apt install -y git build-essential curl
$ sudo dpkg --configure -a
```