use crowsi_network_controller::{
    DecisionStatusV1, MAX_LEDGER_ENTRIES, PolicyEvaluator, parse_policy_json, parse_request_json,
};

const AT: &str = "2026-08-01T00:05:00.000Z";

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
fn exact_allowlist_match_is_dry_run_only() {
    let (request, policy) = fixtures();
    let report = PolicyEvaluator::default()
        .evaluate(&request, &policy, AT)
        .unwrap();
    assert_eq!(
        report.decisions()[0].status(),
        DecisionStatusV1::AllowedDryRun
    );
    assert_eq!(report.decisions()[0].reason_code(), "allowlist-match");
    assert_eq!(
        report.decisions()[0].matched_rule_id(),
        Some("rule-sample-edge")
    );
}

#[test]
fn scope_mismatch_is_denied_by_default() {
    let (mut request, policy) = fixtures();
    request.scope = "other".to_owned();
    let report = PolicyEvaluator::default()
        .evaluate(&request, &policy, AT)
        .unwrap();
    assert_eq!(report.decisions()[0].status(), DecisionStatusV1::Denied);
    assert_eq!(report.decisions()[0].reason_code(), "default-deny");
    assert_eq!(report.decisions()[0].matched_rule_id(), None);
}

#[test]
fn replay_and_conflict_are_distinguished() {
    let (request, policy) = fixtures();
    let mut evaluator = PolicyEvaluator::default();
    let first = evaluator.evaluate(&request, &policy, AT).unwrap();
    let replay = evaluator
        .evaluate(&request, &policy, "2026-08-01T00:06:00.000Z")
        .unwrap();
    assert_eq!(replay, first);

    let mut conflict = request.clone();
    conflict.target_id = "other-edge".to_owned();
    let denied = evaluator
        .evaluate(&conflict, &policy, "2026-08-01T00:06:00.000Z")
        .unwrap();
    assert_eq!(denied.decisions()[0].status(), DecisionStatusV1::Denied);
    assert_eq!(denied.receipts()[0].result_code(), "idempotency-conflict");
    assert_eq!(
        denied.decisions()[0].id(),
        "decision-conflict-0000000000000000-request-sample-edge"
    );
    assert_eq!(
        denied.receipts()[0].id(),
        "receipt-conflict-0000000000000000-request-sample-edge"
    );
    let replayed_conflict = evaluator
        .evaluate(&conflict, &policy, "2026-08-01T00:07:00.000Z")
        .unwrap();
    assert_eq!(denied, replayed_conflict);
}

#[test]
fn same_key_with_changed_policy_is_denied() {
    let (request, mut policy) = fixtures();
    let mut evaluator = PolicyEvaluator::default();
    evaluator.evaluate(&request, &policy, AT).unwrap();
    policy.rules[0].scope = "changed-scope".to_owned();

    let denied = evaluator.evaluate(&request, &policy, AT).unwrap();
    assert_eq!(denied.decisions()[0].status(), DecisionStatusV1::Denied);
    assert_eq!(denied.decisions()[0].reason_code(), "idempotency-conflict");
    assert!(!denied.external_actions());
}

#[test]
fn expired_request_is_denied() {
    let (request, policy) = fixtures();
    let report = PolicyEvaluator::default()
        .evaluate(&request, &policy, "2026-08-01T01:00:00.000Z")
        .unwrap();
    assert_eq!(
        report.decisions()[0].reason_code(),
        "request-outside-window"
    );
}

#[test]
fn malformed_calendar_and_evaluation_times_are_rejected() {
    let (mut request, policy) = fixtures();
    request.requested_at = "2026-02-30T00:00:00.000Z".to_owned();
    assert!(
        PolicyEvaluator::default()
            .evaluate(&request, &policy, AT)
            .is_err()
    );

    let (request, policy) = fixtures();
    assert!(
        PolicyEvaluator::default()
            .evaluate(&request, &policy, "2026-08-01T25:00:00.000Z")
            .is_err()
    );
}

#[test]
fn ledger_is_bounded_and_fails_closed() {
    let (base, policy) = fixtures();
    let mut evaluator = PolicyEvaluator::default();
    for index in 0..MAX_LEDGER_ENTRIES {
        let mut request = base.clone();
        request.request_id = format!("request-{index}");
        request.idempotency_key = format!("idem-{index}");
        evaluator.evaluate(&request, &policy, AT).unwrap();
    }
    let mut overflow = base;
    overflow.request_id = "request-overflow".to_owned();
    overflow.idempotency_key = "idem-overflow".to_owned();
    let denied = evaluator.evaluate(&overflow, &policy, AT).unwrap();
    assert_eq!(denied.decisions()[0].status(), DecisionStatusV1::Denied);
    assert_eq!(
        denied.decisions()[0].reason_code(),
        "idempotency-ledger-full"
    );
}
