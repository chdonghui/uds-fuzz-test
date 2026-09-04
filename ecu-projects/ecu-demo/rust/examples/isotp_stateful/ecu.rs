// 0x22, SID, ReadDataByIdentifier, RDBI
// 0x31, NRC, RequestOutOfRange, ROOR
// 0x11, NRC, ServiceNotSupported, SNS
// 0x13, NRC, IncorrectMessageLengthOrInvalidFormat, IMLOIF
// 0x7F, NRC, ServiceNotSupportedInActiveSession
// 0xF190, DID, Vehicle Identification Number, VIN
// 0x10, SID, DiagnosticSessionControl, DSC
//
// 0x01, Default Session
// 0x02, Programming Session
// 0x03, Extended Diagnostic Session
// 0x04, Safety System Diagnostic Session

pub fn handle_request(session: &mut u8, req: &[u8]) -> Vec<u8> {
    match req {
        //DiagnosticSessionControl
        [0x10, 0x01] => {
            *session = 0x01;
            vec![0x10 + 0x40, 0x01]
        }

        [0x10, 0x02] => {
            *session = 0x02;
            vec![0x10 + 0x40, 0x02]
        }

        [0x10, 0x03] => {
            *session = 0x03;
            vec![0x10 + 0x40, 0x03]
        }

        [0x10, 0x04] => {
            *session = 0x04;
            vec![0x10 + 0x40, 0x04]
        }

        [0x10, _] => vec![0x7F, 0x10, 0x12],
        [0x10, ..] => vec![0x7F, 0x10, 0x13],

        // ReadDataByIdentifier
        // 학습용으로 Extended Session에서만 허용
        [0x22, 0xF1, 0x90] if *session == 0x03 => {
            vec![0x22 + 0x40, 0xF1, 0x90, 0x01]
        }

        // 0x22 지원하지만 현재 세션에서 사용 불가
        [0x22, _, _] if *session != 0x03 => {
            vec![0x7F, 0x22, 0x7F]
        }

        // Extended Session인데 지원하지 않는 DID
        [0x22, _, _] => {
            vec![0x7F, 0x22, 0x31]
        }

        [0x22, ..] => {
            vec![0x7F, 0x22, 0x13]
        }

        // 지원하지 않는 SID
        [sid, ..] => {
            vec![0x7F, *sid, 0x11]
        }

        [] => vec![],
    }
}
