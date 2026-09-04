use socketcan_isotp::{IsoTpSocket, StandardId};
use uds_ecu_rust::ecu::{EcuState, handle_request};

fn main() -> Result<(), socketcan_isotp::Error> {
    let mut socket = IsoTpSocket::open(
        "vcan0",
        StandardId::new(0x7E0).expect("invalid RX ID"),
        StandardId::new(0x7E8).expect("invalid TX ID"),
    )?;

    let mut state = EcuState::new();

    loop {
        println!("waiting...");

        let req = socket.read()?;
        let response = handle_request(&mut state, &req);

        socket.write(&response)?;
    }
}
