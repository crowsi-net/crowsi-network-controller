use crowsi_network_controller::{
    DecisionStatusV1, MAX_POLICY_RULES, MAX_RULE_REQUESTERS, PolicyEvaluator, Validate,
    parse_policy_json, parse_request_json,
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
fn policy_collections_have_hard_limits() {
    let (_, mut policy) = fixtures();
    policy.rules = vec![policy.rules[0].clone(); MAX_POLICY_RULES + 1];
    assert!(policy.validate().is_err());

    let (_, mut policy) = fixtures();
    policy.rules[0].requesters = (0..=MAX_RULE_REQUESTERS)
        .map(|index| format!("requester-{index}"))
        .collect();
    assert!(policy.validate().is_err());
}

#[test]
fn request_window_is_limited_to_one_hour() {
    let (mut request, _) = fixtures();
    request.expires_at = "2026-08-01T01:00:00.001Z".to_owned();
    assert!(request.validate().is_err());
}

#[test]
fn conflict_ids_are_unique_and_bounded_for_maximum_request_ids() {
    let (original, policy) = fixtures();
    let mut evaluator = PolicyEvaluator::default();
    evaluator.evaluate(&original, &policy, AT).unwrap();

    let mut first = original.clone();
    first.target_id = "other-edge".to_owned();
    first.request_id = format!("r{}", "a".repeat(127));
    let first_report = evaluator.evaluate(&first, &policy, AT).unwrap();

    let mut second = first;
    second.scope = "other-scope".to_owned();
    let second_report = evaluator.evaluate(&second, &policy, AT).unwrap();
    for id in [
        first_report.decisions()[0].id(),
        first_report.receipts()[0].id(),
        second_report.decisions()[0].id(),
        second_report.receipts()[0].id(),
    ] {
        assert!(id.len() <= 256);
    }
    assert_ne!(
        first_report.decisions()[0].id(),
        second_report.decisions()[0].id()
    );
}

#[test]
fn evaluator_rejects_time_regression_after_purge_boundary() {
    let (request, policy) = fixtures();
    let mut evaluator = PolicyEvaluator::default();
    evaluator
        .evaluate(&request, &policy, "2026-08-01T02:00:00.000Z")
        .unwrap();
    assert!(evaluator.evaluate(&request, &policy, AT).is_err());
}

#[test]
fn expired_entries_are_purged_before_capacity_check() {
    let (request, policy) = fixtures();
    let mut evaluator = PolicyEvaluator::default();
    evaluator.evaluate(&request, &policy, AT).unwrap();
    let mut later = request;
    later.request_id = "request-later".to_owned();
    later.idempotency_key = "idem-later".to_owned();
    later.requested_at = "2026-08-01T02:00:00.000Z".to_owned();
    later.expires_at = "2026-08-01T03:00:00.000Z".to_owned();
    let report = evaluator
        .evaluate(&later, &policy, "2026-08-01T02:05:00.000Z")
        .unwrap();
    assert_eq!(
        report.decisions()[0].status(),
        DecisionStatusV1::AllowedDryRun
    );
}
