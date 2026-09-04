mod ecu;

use ecu::handle_request;
use socketcan_isotp::{IsoTpSocket, StandardId};

fn main() -> Result<(), socketcan_isotp::Error> {
    let mut socket = IsoTpSocket::open(
        "vcan0",
        StandardId::new(0x7E0).expect("invalid RX ID"),
        StandardId::new(0x7E8).expect("invalid TX ID"),
    )?;

    let mut session: u8 = 0x01;

    loop {
        println!("waiting...");

        let req = socket.read()?;
        let response = handle_request(&mut session, &req);

        socket.write(&response)?;
    }
}
