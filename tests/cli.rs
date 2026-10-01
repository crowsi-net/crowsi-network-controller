use std::fs;
use std::process::Command;

use crowsi_network_controller::ControlReportV1;

#[test]
fn evaluate_uses_execution_time_instead_of_request_time() {
    let request = include_str!("../examples/control.request.json")
        .replace("2026-08-01T00:00:00.000Z", "2000-01-01T00:00:00.000Z")
        .replace("2026-08-01T01:00:00.000Z", "2000-01-01T01:00:00.000Z");
    let path = std::env::temp_dir().join(format!(
        "crowsi-controller-expired-{}.json",
        std::process::id()
    ));
    fs::write(&path, request).unwrap();
    let policy = format!(
        "{}/examples/control.policy.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new(env!("CARGO_BIN_EXE_crowsi-network-controller"))
        .args(["evaluate", path.to_str().unwrap(), &policy])
        .output()
        .unwrap();
    fs::remove_file(path).unwrap();
    assert!(output.status.success());
    let report: ControlReportV1 = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report.decisions()[0].reason_code(),
        "request-outside-window"
    );
    assert_eq!(
        report.decisions()[0].expires_at(),
        "2000-01-01T01:00:00.000Z"
    );
}

#[test]
fn evaluate_rejects_non_regular_input_paths() {
    let policy = format!(
        "{}/examples/control.policy.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new(env!("CARGO_BIN_EXE_crowsi-network-controller"))
        .args(["evaluate", std::env::temp_dir().to_str().unwrap(), &policy])
        .output()
        .unwrap();
    assert!(!output.status.success());
}
