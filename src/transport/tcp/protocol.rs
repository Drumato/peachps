use crate::{
    internet::ip::{self, IPv4Addr},
    network_device,
    transport::tcp::*,
    transport::TransportProtocolError,
    Items, RxResult,
};

struct TxPreparation {
    sequence: u32,
    acknowledgement: u32,
    flg: u8,
    window_size: u16,
    length: usize,
}

pub async fn rx<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
    rx_result: RxResult,
    buf: &[u8],
) -> Result<(SegmentHeader, Vec<u8>), TransportProtocolError> {
    let header = SegmentHeader::new_from_bytes(buf, TransportProtocolError::CannotParseTCPSegment)?;

    if table.opt.debug {
        eprintln!("++++++++ rx tcp segment ++++++++");
        eprintln!("{}", header);
    }

    let local = EndPoint {
        addr: table.opt.ip_addr,
        port: header.dst_port,
    };
    let foreign = EndPoint {
        addr: rx_result.src_ip_addr,
        port: header.src_port,
    };
    let conn = Connection { local, foreign };

    /*
    if header.certain_flag_is_up(control_flag::SYN) {
        segment.len += 1;
    }

    if header.certain_flag_is_up(control_flag::FIN) {
        segment.len += 1;
    }
    */

    segment_arrives(table, header, conn, rx_result).await?;

    let (_, rest) = buf.split_at(header.data_offset() as usize);
    Ok((header, rest.to_vec()))
}

async fn segment_arrives<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
    header: SegmentHeader,
    conn: Connection,
    rx_result: RxResult,
) -> Result<(), TransportProtocolError> {
    let pcb = find_pcb(table, conn);
    if no_connection_established(header, pcb) {
        return Ok(());
    }

    let (pcb_idx, pcb) = pcb.unwrap();

    match pcb.state {
        ConnState::Listen => {
            segment_arrives_on_listen(table, header, conn, pcb, pcb_idx, rx_result).await?;
        }
        // ConnState::SynSent => { segment_arrives_on_syn_sent()},
        _ => {}
    }
    Ok(())
}

async fn segment_arrives_on_listen<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
    header: SegmentHeader,
    conn: Connection,
    mut pcb: ProtocolControlBlock,
    pcb_idx: usize,
    rx_result: RxResult,
) -> Result<(), TransportProtocolError> {
    if header.flag_is_up(control_flag::RST) {
        // セグメントを無視
        return Ok(());
    }

    if header.flag_is_up(control_flag::ACK) {
        // TODO: RSTパケットを送信
        return Ok(());
    }

    if header.flag_is_up(control_flag::SYN) {
        // TODO: security/compartment check
        // TODO: precedence check

        // TODO: Connection Socket Mode
        pcb.conn.local = conn.local;
        pcb.conn.foreign = conn.foreign;
        pcb.receive.window_size = pcb.buffer.len() as u16;
        pcb.receive.next = pcb.iss + 1;
        pcb.irs = header.sequence + 1;
        pcb.iss = rand::random::<u32>();

        // SYN+ACKセグメントの送信(3way-handshake STEP2)
        let tx_prep = prepare_for_tx(control_flag::SYN | control_flag::ACK, 0, pcb);
        tx(table, Default::default(), tx_prep, conn, rx_result).await?;

        pcb.send.next = pcb.iss + 1;
        pcb.send.unacknowleage = pcb.iss;
        pcb.state = ConnState::SynReceived;

        if let Ok(ref mut connections) = table.tcp_connection.lock() {
            connections[pcb_idx] = pcb;
        }
        return Ok(());
    }

    Ok(())
}

fn prepare_for_tx(flg: u8, length: u32, pcb: ProtocolControlBlock) -> TxPreparation {
    let seq = if control_flag::is_up(flg, control_flag::SYN) {
        pcb.iss
    } else {
        pcb.send.next
    };
    TxPreparation {
        flg,
        length: length as usize,
        sequence: seq,
        acknowledgement: pcb.receive.next,
        window_size: pcb.receive.window_size,
    }
}

async fn tx<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
    data: Vec<u8>,
    tx_prep: TxPreparation,
    conn: Connection,
    rx_result: RxResult,
) -> Result<(), TransportProtocolError> {
    // TODO: retransmit_queue_add if SYN and FIN are up

    tx_segment(table, tx_prep, data, conn, rx_result).await?;
    Ok(())
}

async fn tx_segment<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
    tx_prep: TxPreparation,
    data: Vec<u8>,
    conn: Connection,
    rx_result: RxResult,
) -> Result<(), TransportProtocolError> {
    let mut header = SegmentHeader::default();
    header.src_port = conn.local.port;
    header.dst_port = conn.foreign.port;
    header.sequence = tx_prep.sequence;
    header.acknowledgement = tx_prep.acknowledgement;
    header.offset = (SegmentHeader::LEAST_LENGTH as u8 >> 2) << 4;
    header.flg = tx_prep.flg;
    header.window_size = tx_prep.window_size as u16;

    ip::tx(
        table,
        crate::transport::TransportProtocol::TCP,
        rx_result,
        data,
    )
    .await?;

    Ok(())
}

/// コネクションが確立されているかチェック．
/// 確立されていない or クローズの場合のみ処理を行う
fn no_connection_established(
    header: SegmentHeader,
    conn: Option<(usize, ProtocolControlBlock)>,
) -> bool {
    let process = conn.is_none() || conn.as_ref().unwrap().1.state == ConnState::Close;
    if !process {
        return process;
    }

    if header.flag_is_up(control_flag::RST) {
        return process;
    }

    if header.flag_is_up(control_flag::ACK) {
        eprintln!("TODO: send RST segment");
    } else {
        eprintln!("TODO: send RST + ACK segment");
    }

    process
}

/// PCBを検索し返す
fn find_pcb<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
    conn: Connection,
) -> Option<(usize, ProtocolControlBlock)> {
    // 該当するPCBを検索
    if let Ok(connections) = table.tcp_connection.lock() {
        for (i, pcb) in connections.iter().enumerate() {
            if (pcb.conn.local.addr == IPv4Addr::ANY || pcb.conn.local.addr == conn.local.addr)
                && pcb.conn.local.port == conn.local.port
            {
                if pcb.conn.foreign == conn.foreign {
                    return Some((i, *pcb));
                }

                if let ConnState::Listen = pcb.state {
                    if pcb.conn.foreign.addr == IPv4Addr::ANY && pcb.conn.foreign.port == 0 {
                        return Some((i, *pcb));
                    }
                }
            }
        }
    }
    None
}
