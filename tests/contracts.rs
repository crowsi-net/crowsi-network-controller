use std::fs;

use crowsi_network_controller::{
    ControlReportV1, PolicyEvaluator, parse_policy_json, parse_request_json,
};

const EVALUATED_AT: &str = "2026-08-01T00:05:00.000Z";
const UTC_MILLIS_PATTERN: &str =
    "^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\\.[0-9]{3}Z$";

fn fixtures() -> (
    crowsi_network_controller::ControlRequestV1,
    crowsi_network_controller::ControlPolicyV1,
) {
    (
        parse_request_json(include_bytes!("../examples/control.request.json")).unwrap(),
        parse_policy_json(include_bytes!("../examples/control.policy.json")).unwrap(),
    )
}

#[test]
fn deterministic_sample_matches_checked_in_contract() {
    let (request, policy) = fixtures();
    let actual = PolicyEvaluator::default()
        .evaluate(&request, &policy, EVALUATED_AT)
        .unwrap();
    let expected: ControlReportV1 =
        serde_json::from_str(&fs::read_to_string("examples/control.receipt.sample.json").unwrap())
            .unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn output_never_claims_external_or_changed_state() {
    let (request, policy) = fixtures();
    let report = PolicyEvaluator::default()
        .evaluate(&request, &policy, EVALUATED_AT)
        .unwrap();
    assert!(!report.external_actions());
    assert!(
        report
            .receipts()
            .iter()
            .all(|receipt| !receipt.changed_state())
    );
}

#[test]
fn closed_input_rejects_credential_smuggling() {
    let source = include_str!("../examples/control.request.json")
        .replace("\"expires_at\"", "\"credential\":\"secret\",\"expires_at\"");
    assert!(parse_request_json(source.as_bytes()).is_err());
}

#[test]
fn timestamp_schema_shapes_match_the_executable_contract() {
    let request: serde_json::Value = serde_json::from_str(include_str!(
        "../schemas/network-control-request-v1.schema.json"
    ))
    .unwrap();
    let receipts: serde_json::Value = serde_json::from_str(include_str!(
        "../schemas/network-control-receipts-v1.schema.json"
    ))
    .unwrap();
    assert_eq!(
        request.pointer("/x-crowsi-max-request-window-ms").unwrap(),
        3_600_000
    );
    for pointer in [
        "/properties/requested_at/pattern",
        "/properties/expires_at/pattern",
    ] {
        assert_eq!(request.pointer(pointer).unwrap(), UTC_MILLIS_PATTERN);
    }
    for pointer in [
        "/properties/generated_at/pattern",
        "/$defs/decision/properties/decided_at/pattern",
        "/$defs/decision/properties/expires_at/pattern",
        "/$defs/receipt/properties/recorded_at/pattern",
    ] {
        assert_eq!(receipts.pointer(pointer).unwrap(), UTC_MILLIS_PATTERN);
    }
}

#[test]
fn deserialization_rejects_unsafe_output_states() {
    let source = include_str!("../examples/control.receipt.sample.json");
    for invalid in [
        source.replace("\"external_actions\": false", "\"external_actions\": true"),
        source.replace("\"decision_count\": 1", "\"decision_count\": 2"),
        source.replace(
            "\"matched_rule_id\": \"rule-sample-edge\"",
            "\"matched_rule_id\": null",
        ),
        source.replace("\"changed_state\": false", "\"changed_state\": true"),
    ] {
        assert!(serde_json::from_str::<ControlReportV1>(&invalid).is_err());
    }
}

#[test]
fn deserialization_rejects_broken_decision_receipt_relations() {
    let base: serde_json::Value =
        serde_json::from_str(include_str!("../examples/control.receipt.sample.json")).unwrap();
    let mut invalid = Vec::new();

    let mut value = base.clone();
    value["receipts"][0]["target_id"] = "different-target".into();
    invalid.push(value);

    let mut value = base.clone();
    value["receipts"][0]["result_code"] = "different-result".into();
    invalid.push(value);

    let mut value = base.clone();
    value["receipts"][0]["recorded_at"] = "2026-08-01T00:04:59.999Z".into();
    invalid.push(value);

    let mut value = base.clone();
    value["decisions"][0]["decided_at"] = "2026-08-01T00:05:00.001Z".into();
    invalid.push(value);

    let mut value = base.clone();
    value["receipts"][0]["recorded_at"] = "2026-08-01T00:05:00.001Z".into();
    invalid.push(value);

    let mut value = base.clone();
    value["decisions"][0]["expires_at"] = value["decisions"][0]["decided_at"].clone();
    invalid.push(value);

    let mut value = base;
    let mut second = value["decisions"][0].clone();
    second["id"] = "decision-second".into();
    value["decisions"].as_array_mut().unwrap().push(second);
    value["decision_count"] = 2.into();
    invalid.push(value);

    for value in invalid {
        assert!(serde_json::from_value::<ControlReportV1>(value).is_err());
    }
}
