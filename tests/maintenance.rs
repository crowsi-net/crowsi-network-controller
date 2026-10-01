use crowsi_network_controller::{
    ControlActionV1, DecisionStatusV1, PolicyEvaluator, parse_policy_json, parse_request_json,
};

const AT: &str = "2026-08-01T00:05:00.000Z";

#[test]
fn maintenance_actions_require_the_same_exact_allowlist_match() {
    let mut request =
        parse_request_json(include_bytes!("../examples/control.request.json")).unwrap();
    let mut policy = parse_policy_json(include_bytes!("../examples/control.policy.json")).unwrap();
    request.action = ControlActionV1::EnterMaintenance;
    request.scope = "public-maintenance".to_owned();
    policy.rules[0].action = ControlActionV1::EnterMaintenance;
    policy.rules[0].scope = "public-maintenance".to_owned();
    let report = PolicyEvaluator::default()
        .evaluate(&request, &policy, AT)
        .unwrap();
    assert_eq!(
        report.decisions()[0].status(),
        DecisionStatusV1::AllowedDryRun
    );
    assert!(!report.external_actions());
    assert!(!report.receipts()[0].changed_state());
}

#[test]
fn coela_policies_are_bounded_and_default_deny_unknown_services() {
    let enter = parse_policy_json(include_bytes!(
        "../policies/coela-enter-maintenance.policy.json"
    ))
    .unwrap();
    let exit = parse_policy_json(include_bytes!(
        "../policies/coela-exit-maintenance.policy.json"
    ))
    .unwrap();
    assert_eq!(enter.rules.len(), 21);
    assert_eq!(exit.rules.len(), 21);

    let mut request =
        parse_request_json(include_bytes!("../examples/control.request.json")).unwrap();
    request.target_id = "unregistered-service".to_owned();
    request.action = ControlActionV1::EnterMaintenance;
    request.scope = "public-maintenance".to_owned();
    let report = PolicyEvaluator::default()
        .evaluate(&request, &enter, AT)
        .unwrap();
    assert_eq!(report.decisions()[0].reason_code(), "default-deny");
}
