use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

use crate::{
    ControlActionV1, ControlDecisionV1, ControlReceiptV1, ControlReportV1, DecisionStatusV1,
    ReceiptStatusV1, Validate,
};

impl<'de> Deserialize<'de> for ControlReportV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let report = ReportWire::deserialize(deserializer)?.into_report();
        report.validate().map_err(D::Error::custom)?;
        Ok(report)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReportWire {
    schema: String,
    generated_at: String,
    external_actions: bool,
    decision_count: usize,
    receipt_count: usize,
    decisions: Vec<DecisionWire>,
    receipts: Vec<ReceiptWire>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DecisionWire {
    id: String,
    target_id: String,
    action: ControlActionV1,
    status: DecisionStatusV1,
    decided_at: String,
    expires_at: String,
    reason_code: String,
    requested_by: String,
    decided_by: String,
    matched_rule_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceiptWire {
    id: String,
    decision_id: String,
    target_id: String,
    status: ReceiptStatusV1,
    recorded_at: String,
    result_code: String,
    changed_state: bool,
}

impl ReportWire {
    fn into_report(self) -> ControlReportV1 {
        ControlReportV1 {
            schema: self.schema,
            generated_at: self.generated_at,
            external_actions: self.external_actions,
            decision_count: self.decision_count,
            receipt_count: self.receipt_count,
            decisions: self
                .decisions
                .into_iter()
                .map(DecisionWire::into_value)
                .collect(),
            receipts: self
                .receipts
                .into_iter()
                .map(ReceiptWire::into_value)
                .collect(),
        }
    }
}

impl DecisionWire {
    fn into_value(self) -> ControlDecisionV1 {
        ControlDecisionV1 {
            id: self.id,
            target_id: self.target_id,
            action: self.action,
            status: self.status,
            decided_at: self.decided_at,
            expires_at: self.expires_at,
            reason_code: self.reason_code,
            requested_by: self.requested_by,
            decided_by: self.decided_by,
            matched_rule_id: self.matched_rule_id,
        }
    }
}

impl ReceiptWire {
    fn into_value(self) -> ControlReceiptV1 {
        ControlReceiptV1 {
            id: self.id,
            decision_id: self.decision_id,
            target_id: self.target_id,
            status: self.status,
            recorded_at: self.recorded_at,
            result_code: self.result_code,
            changed_state: self.changed_state,
        }
    }
}
