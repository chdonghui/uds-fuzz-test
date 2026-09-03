// 0x22, SID, ReadDataByIdentifier, RDBI
// 0x31, NRC, RequestOutOfRange, ROOR
// 0x11, NRC, ServiceNotSupported, SNS
// 0x13, NRC, IncorrectMessageLengthOrInvalidFormat, IMLOIF
// 0xF190, DID, Vehicle Identification Number, VIN
// 0x10, SID, DiagnosticSessionControl, DSC

fn handle_request(req: &[u8]) -> Vec<u8> {
    match req {
        [0x22, 0xF1, 0x90] => vec![0x22 + 0x40, 0xF1, 0x90, 0x01],
        [0x22, _, _] => vec![0x7F, 0x22, 0x31],
        [0x22, ..] => vec![0x7F, 0x22, 0x13],
        [0x10, 0x01] => vec![0x10 + 0x40, 0x01],
        [0x10, 0x02] => vec![0x10 + 0x40, 0x02],
        [0x10, 0x03] => vec![0x10 + 0x40, 0x03],
        [0x10, 0x04] => vec![0x10 + 0x40, 0x04],
        [0x10, _] => vec![0x7F, 0x10, 0x12],
        [0x10, ..] => vec![0x7F, 0x10, 0x13],
        [sid, ..] => vec![0x7F, *sid, 0x11],
        [] => vec![],
    }
}

fn main() {
    // [u8]       바이트 여러 개
    // &[u8]      바이트 slice 하나
    // [&[u8]]    그런 slice 여러 개
    // &[&[u8]]   그 목록을 참조
    let requests: &[&[u8]] = &[
        &[0x22, 0xF1, 0x90], // 정상 DID
        &[0x22, 0x12, 0x34], // 지원하지 않는 DID
        &[0x22, 0xF1],       // 잘못된 길이
        &[0x99],             // 지원하지 않는 SID
        &[],                 // 빈 요청
        &[0x10, 0x01],       // DSC, Default Session
        &[0x10, 0x02],       // DSC, Programming Session
        &[0x10, 0x03],       // DSC, Extended Diagnostic Session
        &[0x10, 0x04],       // DSC, Safety System Diagnostic Session
        &[0x10, 0x05],       // 지원하지 않는 sub-function
        &[0x10],             // 잘못된 길이
    ];

    for req in requests {
        // {}    값을 출력
        // ?     Debug 형식
        // X     16진수 대문자
        // 02    두 자리로 출력
        println!("{:02X?} -> {:02X?}", req, handle_request(req));
    }
}
