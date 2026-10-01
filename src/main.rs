use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};

use crowsi_network_controller::{
    MAX_DOCUMENT_BYTES, PolicyEvaluator, parse_policy_json, parse_request_json,
};

mod utc;

const SAMPLE_REQUEST: &[u8] = include_bytes!("../examples/control.request.json");
const SAMPLE_POLICY: &[u8] = include_bytes!("../examples/control.policy.json");
const SAMPLE_TIME: &str = "2026-08-01T00:05:00.000Z";

fn main() {
    if let Err(error) = run() {
        eprintln!("network control evaluation failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let (request_source, policy_source, evaluated_at) = match arguments.as_slice() {
        [] => (
            SAMPLE_REQUEST.to_vec(),
            SAMPLE_POLICY.to_vec(),
            SAMPLE_TIME.to_owned(),
        ),
        [command] if command == "sample" => (
            SAMPLE_REQUEST.to_vec(),
            SAMPLE_POLICY.to_vec(),
            SAMPLE_TIME.to_owned(),
        ),
        [command, request, policy] if command == "evaluate" => {
            let request_source = read_bounded(request)?;
            let policy_source = read_bounded(policy)?;
            let evaluated_at = utc::now()?;
            (request_source, policy_source, evaluated_at)
        }
        [command] if matches!(command.as_str(), "--help" | "-h" | "help") => {
            println!("usage: crowsi-network-controller [sample|evaluate REQUEST POLICY]");
            return Ok(());
        }
        _ => return Err("expected sample or evaluate REQUEST POLICY".into()),
    };
    let request = parse_request_json(&request_source)?;
    let policy = parse_policy_json(&policy_source)?;
    let report = PolicyEvaluator::default().evaluate(&request, &policy, &evaluated_at)?;
    let stdout = io::stdout();
    let mut output = stdout.lock();
    serde_json::to_writer_pretty(&mut output, &report)?;
    writeln!(output)?;
    Ok(())
}

fn read_bounded(path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let file = File::open(path)?;
    if !file.metadata()?.file_type().is_file() {
        return Err("input document must be a regular file".into());
    }
    let mut source = Vec::new();
    let limit = u64::try_from(MAX_DOCUMENT_BYTES + 1)?;
    file.take(limit).read_to_end(&mut source)?;
    if source.len() > MAX_DOCUMENT_BYTES {
        return Err("input document exceeds 65536 bytes".into());
    }
    Ok(source)
}
