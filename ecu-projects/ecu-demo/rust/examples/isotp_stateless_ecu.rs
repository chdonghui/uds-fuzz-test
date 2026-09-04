use socketcan_isotp::{IsoTpSocket, StandardId};

fn handle_request(req: &[u8]) -> Vec<u8> {
    match req {
        [0x22, 0xF1, 0x90] => vec![0x62, 0xF1, 0x90, 0x01, 0x02, 0x03, 0x04],
        [0x22, _, _] => vec![0x7F, 0x22, 0x31],
        [0x22, ..] => vec![0x7F, 0x22, 0x13],
        [sid, ..] => vec![0x7F, *sid, 0x11],
        [] => vec![],
    }
}

fn main() -> Result<(), socketcan_isotp::Error> {
    let mut socket = IsoTpSocket::open(
        "vcan0",
        StandardId::new(0x7E0).expect("invalid RX ID"),
        StandardId::new(0x7E8).expect("invalid TX ID"),
    )?;

    loop {
        let req = socket.read()?;
        let response = handle_request(&req);

        socket.write(&response)?;
    }
}
