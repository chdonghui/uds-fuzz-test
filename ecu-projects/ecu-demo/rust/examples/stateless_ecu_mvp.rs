// 0x22, ReadDataByIdentifier, RDBI
// 0x31, NRC, RequestOutOfRange, ROOR
// 0x11, NRC, ServiceNotSupported, SNS
// 0xF190, DID, Vehicle Identification Number, VIN

fn handle_request(req: &[u8]) -> Vec<u8> {
    match req {
        [0x22, 0xF1, 0x90] => vec![0x22 + 0x40, 0xF1, 0x90, 0x01],
        [0x22, ..] => vec![0x7F, 0x22, 0x31],
        [sid, ..] => vec![0x7F, *sid, 0x11],
        [] => vec![],
    }
}

fn main() {
    let requests: &[&[u8]] = &[
        &[0x22, 0xF1, 0x90], // 정상 요청
        &[0x22, 0x12, 0x34], // 지원하지 않는 DID
        &[0x22, 0xF1],       // 잘못된 0x22 요청
        &[0x99],             // 지원하지 않는 SID
        &[],                 // 빈 요청
    ];

    for req in requests {
        println!("{:02X?} -> {:02X?}", req, handle_request(req));
    }
}
