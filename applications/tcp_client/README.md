# tcp_client

プロトコルスタックをテストするときに使用するシンプルなクライアント．  
このクライアント自体にpeachpsは使用していないので注意．  

## How to use

```shell
nc -kl 34567 # server process
cargo run 127.0.0.1 34567 # other process
```