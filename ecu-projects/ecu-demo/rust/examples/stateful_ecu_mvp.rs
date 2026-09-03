// 0x22, SID, ReadDataByIdentifier, RDBI
// 0x11, NRC, ServiceNotSupported, SNS
// 0xF190, DID, Vehicle Identification Number, VIN
// 0x10, SID, DiagnosticSessionControl, DSC
// 0x7F, Response SID, Negative Response
// 0x7F, NRC, ServiceNotSupportedInActiveSession

fn handle_request(session: &mut u8, req: &[u8]) -> Vec<u8> {
    match req {
        [0x10, 0x01] => {
            *session = 0x01;
            vec![0x10 + 0x40, 0x01]
        }

        [0x10, 0x03] => {
            *session = 0x03;
            vec![0x10 + 0x40, 0x03]
        }

        [0x22, 0xF1, 0x90] if *session == 0x03 => {
            vec![0x22 + 0x40, 0xF1, 0x90, 0x01]
        }

        [0x22, ..] => {
            vec![0x7F, 0x22, 0x7F]
        }

        [sid, ..] => {
            vec![0x7F, *sid, 0x11]
        }

        [] => vec![],
    }
}

fn main() {
    let mut session: u8 = 0x01;

    let requests: &[&[u8]] = &[
        &[0x22, 0xF1, 0x90],
        &[0x10, 0x03],
        &[0x22, 0xF1, 0x90],
        &[0x10, 0x01],
        &[0x22, 0xF1, 0x90],
    ];

    for req in requests {
        println!("{:02X?} -> {:02X?}", req, handle_request(&mut session, req));
    }
}
