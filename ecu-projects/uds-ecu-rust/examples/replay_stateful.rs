use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process,
};

use uds_ecu_rust::ecu::{EcuState, handle_request};

// Stateful corpus record: [Action: u8][Length: u16 Big Endian][Payload]
const SEND_REQUEST: u8 = 0x01;
const RESET: u8 = 0x04;

// -----------------------------------------------------------------------------
// 핵심 기능: corpus 레코드 파싱과 ECU 요청 실행
// -----------------------------------------------------------------------------

/// Corpus에서 파싱한 레코드 하나입니다.
struct Record<'a> {
    action: u8,
    payload: &'a [u8],
}

/// Corpus 파일 하나를 Replay한 결과입니다.
#[derive(Default)]
struct ReplayResult {
    valid_requests: usize,
    resets: usize,
    unknown_actions: usize,
    trailing_bytes: usize,
    positive_50_03: bool,
    positive_62_f1_90: bool,
    positive_67_01: bool,
    positive_67_02: bool,
    did_sequence: bool,
    security_sequence: bool,
}

/// `offset` 위치에서 `[Action][Length][Payload]` 레코드 하나를 읽습니다.
///
/// `Ok(Some(...))`은 정상 레코드, `Ok(None)`은 입력 끝,
/// `Err(...)`는 완성되지 않은 마지막 레코드의 바이트 수를 뜻합니다.
fn next_record<'a>(data: &'a [u8], offset: &mut usize) -> Result<Option<Record<'a>>, usize> {
    if *offset == data.len() {
        return Ok(None);
    }

    let record_start = *offset;
    if data.len() - *offset < 3 {
        return Err(data.len() - record_start);
    }

    let action = data[*offset];
    let length = u16::from_be_bytes([data[*offset + 1], data[*offset + 2]]) as usize;
    *offset += 3;

    if length > data.len() - *offset {
        return Err(data.len() - record_start);
    }

    let payload = &data[*offset..*offset + length];
    *offset += length;

    Ok(Some(Record { action, payload }))
}

/// 바이트를 `10 03`, `62 F1 90`처럼 읽기 쉬운 16진수 문자열로 바꿉니다.
fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// corpus 파일 하나의 레코드를 같은 `EcuState`에 순서대로 실행합니다.
fn replay(data: &[u8], detailed: bool) -> ReplayResult {
    let mut result = ReplayResult::default();
    let mut sequences = SequenceTracker::default();

    // 한 corpus 파일에 포함된 요청들은 같은 ECU 상태를 공유합니다.
    let mut state = EcuState::new();
    let mut offset = 0;
    let mut record_index = 0;

    loop {
        let record = match next_record(data, &mut offset) {
            Ok(Some(record)) => record,
            Ok(None) => break,
            Err(trailing_bytes) => {
                result.trailing_bytes = trailing_bytes;
                if detailed {
                    println!("[{record_index}] TRUNCATED: {trailing_bytes} byte(s) remain");
                }
                break;
            }
        };

        execute_record(
            record,
            record_index,
            detailed,
            &mut state,
            &mut sequences,
            &mut result,
        );
        record_index += 1;
    }

    result.did_sequence = sequences.did_found;
    result.security_sequence = sequences.security_found;
    result
}

/// 파싱된 Action에 따라 UDS 요청 실행, RESET 또는 무시 처리를 합니다.
fn execute_record(
    record: Record<'_>,
    record_index: usize,
    detailed: bool,
    state: &mut EcuState,
    sequences: &mut SequenceTracker,
    result: &mut ReplayResult,
) {
    match record.action {
        SEND_REQUEST => {
            let response = handle_request(state, record.payload);
            result.valid_requests += 1;

            record_positive_response(&response, result);
            sequences.observe(&response);

            if detailed {
                print_request(record_index, record.payload, &response);
            }
        }
        RESET => {
            // RESET 뒤의 요청은 초기 ECU 상태에서 다시 시작합니다.
            *state = EcuState::new();
            sequences.reset_stages();
            result.resets += 1;

            if detailed {
                println!("[{record_index}] RESET");
            }
        }
        _ => {
            // 알 수 없는 Action은 퍼징 하네스와 동일하게 실행하지 않습니다.
            result.unknown_actions += 1;
        }
    }
}

/// 상세 모드에서 요청과 ECU가 반환한 응답을 출력합니다.
fn print_request(record_index: usize, request: &[u8], response: &[u8]) {
    println!("[{record_index}] SEND_REQUEST");
    println!("    Request:  {}", hex(request));
    println!("    Response: {}", hex(response));
}

// -----------------------------------------------------------------------------
// 부가 분석 기능: Positive Response와 목표 시퀀스 탐색
// -----------------------------------------------------------------------------

/// 파일 안에서 특정 Positive Response가 한 번이라도 나왔는지 기록합니다.
fn record_positive_response(response: &[u8], result: &mut ReplayResult) {
    result.positive_50_03 |= response.starts_with(&[0x50, 0x03]);
    result.positive_62_f1_90 |= response.starts_with(&[0x62, 0xF1, 0x90]);
    result.positive_67_01 |= response.starts_with(&[0x67, 0x01]);
    result.positive_67_02 |= response.starts_with(&[0x67, 0x02]);
}

/// 목표 UDS 응답들이 올바른 상태 순서로 나왔는지 추적합니다.
#[derive(Default)]
struct SequenceTracker {
    did_stage: u8,
    security_stage: u8,
    did_found: bool,
    security_found: bool,
}

impl SequenceTracker {
    /// 응답 하나를 보고 DID와 SecurityAccess 시퀀스 단계를 갱신합니다.
    fn observe(&mut self, response: &[u8]) {
        if response.starts_with(&[0x50, 0x03]) {
            self.did_stage = 1;
            self.security_stage = 1;
        } else if self.did_stage == 1 && response.starts_with(&[0x62, 0xF1, 0x90]) {
            self.did_found = true;
        }

        if self.security_stage == 1 && response.starts_with(&[0x67, 0x01]) {
            self.security_stage = 2;
        } else if self.security_stage == 2 && response.starts_with(&[0x67, 0x02]) {
            self.security_found = true;
        }

        // Default Session으로 돌아가면 이전 세션에서 진행하던 순서를 취소합니다.
        if response.starts_with(&[0x50, 0x01]) {
            self.reset_stages();
        }
    }

    /// 발견 결과는 유지하고 현재 진행 중인 단계만 초기화합니다.
    fn reset_stages(&mut self) {
        self.did_stage = 0;
        self.security_stage = 0;
    }
}

// -----------------------------------------------------------------------------
// 부가 편의 기능: 파일 상세 출력과 폴더 전체 요약
// -----------------------------------------------------------------------------

/// corpus 파일 하나를 읽고 Replay합니다.
fn replay_file(path: &Path, detailed: bool) -> io::Result<ReplayResult> {
    let data = fs::read(path)?;

    if detailed {
        println!("File: {}", path.display());
        println!();
    }

    let result = replay(&data, detailed);

    if detailed {
        print_file_summary(&result);
    }

    Ok(result)
}

/// 파일 상세 모드의 마지막에 요청 수와 시퀀스 결과를 출력합니다.
fn print_file_summary(result: &ReplayResult) {
    println!();
    println!("Valid requests: {}", result.valid_requests);
    println!("RESET actions: {}", result.resets);
    println!("Ignored unknown actions: {}", result.unknown_actions);
    println!("Trailing bytes: {}", result.trailing_bytes);
    println!("10 03 -> 22 F1 90: {}", found_text(result.did_sequence));
    println!(
        "10 03 -> 27 01 -> 27 02 B8 9E: {}",
        found_text(result.security_sequence)
    );
}

/// Boolean 발견 여부를 출력용 문자열로 바꿉니다.
fn found_text(found: bool) -> &'static str {
    if found { "found" } else { "not found" }
}

/// 폴더 안의 일반 파일 경로만 모아 이름순으로 정렬합니다.
fn corpus_files(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = fs::read_dir(directory)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

/// 폴더 전체 Replay 결과를 합쳐 출력하기 위한 통계입니다.
#[derive(Default)]
struct DirectorySummary {
    files_scanned: usize,
    files_with_valid_actions: usize,
    positive_50_03: usize,
    positive_62_f1_90: usize,
    positive_67_01: usize,
    positive_67_02: usize,
    did_sequence_files: Vec<PathBuf>,
    security_sequence_files: Vec<PathBuf>,
}

/// 폴더의 모든 corpus를 Replay하고 요약합니다.
/// `show_all`이 true이면 각 파일의 상세 내용도 함께 출력합니다.
fn replay_directory(directory: &Path, show_all: bool) -> io::Result<()> {
    let files = corpus_files(directory)?;
    let mut summary = DirectorySummary::default();

    for path in files {
        print_file_separator(show_all, summary.files_scanned);

        // replay_file()이 호출될 때마다 새로운 EcuState로 시작합니다.
        let result = replay_file(&path, show_all)?;
        add_to_directory_summary(&path, &result, &mut summary);
    }

    print_directory_summary(directory, &summary);
    Ok(())
}

/// `--all` 출력에서 파일 사이를 구분합니다.
fn print_file_separator(show_all: bool, files_scanned: usize) {
    if !show_all {
        return;
    }

    if files_scanned > 0 {
        println!("\n{}", "-".repeat(72));
    }
    println!();
}

/// 파일 하나의 결과를 폴더 전체 통계에 더합니다.
fn add_to_directory_summary(path: &Path, result: &ReplayResult, summary: &mut DirectorySummary) {
    summary.files_scanned += 1;

    if result.valid_requests > 0 || result.resets > 0 {
        summary.files_with_valid_actions += 1;
    }

    // bool을 0 또는 1로 바꿔 해당 응답이 나온 파일 수를 셉니다.
    summary.positive_50_03 += usize::from(result.positive_50_03);
    summary.positive_62_f1_90 += usize::from(result.positive_62_f1_90);
    summary.positive_67_01 += usize::from(result.positive_67_01);
    summary.positive_67_02 += usize::from(result.positive_67_02);

    if result.did_sequence {
        summary.did_sequence_files.push(path.to_path_buf());
    }
    if result.security_sequence {
        summary.security_sequence_files.push(path.to_path_buf());
    }
}

/// 폴더 전체의 응답 통계와 목표 시퀀스가 들어 있는 파일을 출력합니다.
fn print_directory_summary(directory: &Path, summary: &DirectorySummary) {
    println!("Directory: {}", directory.display());
    println!("Files scanned: {}", summary.files_scanned);
    println!(
        "Files containing valid actions: {}",
        summary.files_with_valid_actions
    );
    println!();
    println!("Positive responses (number of files):");
    println!("  50 03: {}", summary.positive_50_03);
    println!("  62 F1 90: {}", summary.positive_62_f1_90);
    println!("  67 01: {}", summary.positive_67_01);
    println!("  67 02: {}", summary.positive_67_02);
    println!();
    print_sequence("10 03 -> 22 F1 90", &summary.did_sequence_files);
    print_sequence(
        "10 03 -> 27 01 -> 27 02 B8 9E",
        &summary.security_sequence_files,
    );
}

/// 시퀀스 발견 여부와 해당 corpus 파일 이름을 출력합니다.
fn print_sequence(name: &str, files: &[PathBuf]) {
    if files.is_empty() {
        println!("{name}: not found");
        return;
    }

    println!("{name}: found in {} file(s)", files.len());
    for path in files {
        if let Some(file_name) = path.file_name() {
            println!("  {}", file_name.to_string_lossy());
        }
    }
}

// -----------------------------------------------------------------------------
// 실행 제어: 명령행 인수와 오류 처리
// -----------------------------------------------------------------------------

/// 올바른 실행 방법을 출력합니다.
fn usage(program: &str) {
    eprintln!("Usage:");
    eprintln!("  {program} <corpus-file>");
    eprintln!("  {program} <corpus-directory>");
    eprintln!("  {program} <corpus-directory> --all");
}

/// 명령행 인수를 검사하고 파일 모드 또는 폴더 모드를 실행합니다.
fn run() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 || args.len() > 3 {
        usage(&args[0]);
        return Err("a corpus file or directory path is required".to_owned());
    }

    let path = Path::new(&args[1]);
    let show_all = parse_show_all(args.get(2))?;

    if path.is_file() {
        if show_all {
            return Err("--all can only be used with a directory".to_owned());
        }
        replay_file(path, true).map_err(|error| error.to_string())?;
    } else if path.is_dir() {
        replay_directory(path, show_all).map_err(|error| error.to_string())?;
    } else {
        return Err(format!("path does not exist: {}", path.display()));
    }

    Ok(())
}

/// 선택 인수가 없거나 `--all`인지 확인합니다.
fn parse_show_all(option: Option<&String>) -> Result<bool, String> {
    match option.map(String::as_str) {
        Some("--all") => Ok(true),
        Some(option) => Err(format!("unknown option: {option}")),
        None => Ok(false),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        process::exit(1);
    }
}
