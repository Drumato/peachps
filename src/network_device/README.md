# Network Device

## Raw Socket

`socket(AF_PACKET, SOCK_RAW, htons(ETH_P_ALL))` を使用してRaw Socketを作成している．  
このソケットはイーサネットフレームのヘッダを含むデータをプロセスに提供してくれるので，  
プロトコルスタックを自作する時はよく用いられる(ソースコードで体感する本でも使用されている)  

Linuxにおいてsocketに対するオペレーションはファイルディスクリプタと同様に扱えるので，  
`NetworkDevice::read()` 等の関数もそのようにして実現されている．  
具体的には，単に `libc::read()` (つまり `read(2)`)を実行しているだけである．  
