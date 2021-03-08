# Transmission Control Protocol

詳細は[RFC793](https://tools.ietf.org/html/rfc793)を．  
`<SEQ=0><ACK=SEG.SEQ+SEG.LEN><CTL=RST,ACK>` のような文字列の読み方だけど，  
セグメントヘッダのsequenceに0を，acknowleageに `SEG.SEQ + SEG.LEN` の値を，  
control_flagに `RST | ACK` を立てたセグメント，みたいな意味になる．  

- [Transmission Control Protocol](#transmission-control-protocol)
  - [Segment Arrives](#segment-arrives)

## Segment Arrives

セグメント受信時の処理が書いてある．  

- connection stateがCLOSEDかチェック
  - 受信セグメントのデータ部はすべて破棄
  - RSTフラグが立っている場合，セグメント自体も破棄
  - RSTフラグが立っていない場合
    - ACKフラグが立っている場合，`<SEQ=SEG.ACK><CTL=RST>`なセグメントを送信
    - ACKフラグが立っていない場合，`<SEQ=0><ACK=SEG.SEQ+SEG.LEN><CTL=RST,ACK>`なセグメントを送信
- connection stateがLISTENかチェック
  - RSTフラグが立っている場合
    - セグメントは無視
  - ACKフラグが立っている場合
    - 不正なセグメントとする(`<SEQ=SEG.ACK><CTL=RST>` セグメントを送信する)
  - SYNフラグが立っている場合
    - セキュリティチェックを行い，チェックに引っかかった場合`<SEQ=SEG.ACK><CTL=RST>` を送信
    - 優先度チェックを行う
    - 3way-handshakeのSYN+ACKセグメントを送信
