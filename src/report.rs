use crate::{
    CONTROL_RECEIPTS_SCHEMA_V1, ControlDecisionV1, ControlPolicyV1, ControlReceiptV1,
    ControlReportV1, ControlRequestV1, DecisionStatusV1, ReceiptStatusV1,
};

pub(crate) fn report(
    request: &ControlRequestV1,
    policy: &ControlPolicyV1,
    at: &str,
    matched_rule: Option<&str>,
    reason: &str,
    suffix_override: Option<&str>,
) -> ControlReportV1 {
    let allowed = matched_rule.is_some() && reason == "allowlist-match";
    let suffix = suffix_override.unwrap_or(&request.idempotency_key);
    let decision_id = format!("decision-{suffix}");
    let decision = ControlDecisionV1 {
        id: decision_id.clone(),
        target_id: request.target_id.clone(),
        action: request.action,
        status: if allowed {
            DecisionStatusV1::AllowedDryRun
        } else {
            DecisionStatusV1::Denied
        },
        decided_at: at.to_owned(),
        expires_at: request.expires_at.clone(),
        reason_code: reason.to_owned(),
        requested_by: request.requested_by.clone(),
        decided_by: policy.policy_id.clone(),
        matched_rule_id: matched_rule.map(str::to_owned),
    };
    let receipt = ControlReceiptV1 {
        id: format!("receipt-{suffix}"),
        decision_id,
        target_id: request.target_id.clone(),
        status: ReceiptStatusV1::Recorded,
        recorded_at: at.to_owned(),
        result_code: reason.to_owned(),
        changed_state: false,
    };
    ControlReportV1 {
        schema: CONTROL_RECEIPTS_SCHEMA_V1.to_owned(),
        generated_at: at.to_owned(),
        external_actions: false,
        decision_count: 1,
        receipt_count: 1,
        decisions: vec![decision],
        receipts: vec![receipt],
    }
}
