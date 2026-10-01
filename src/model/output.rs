use serde::Serialize;

use crate::{ControlActionV1, DecisionStatusV1, ReceiptStatusV1};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ControlDecisionV1 {
    pub(crate) id: String,
    pub(crate) target_id: String,
    pub(crate) action: ControlActionV1,
    pub(crate) status: DecisionStatusV1,
    pub(crate) decided_at: String,
    pub(crate) expires_at: String,
    pub(crate) reason_code: String,
    pub(crate) requested_by: String,
    pub(crate) decided_by: String,
    pub(crate) matched_rule_id: Option<String>,
}

impl ControlDecisionV1 {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub fn target_id(&self) -> &str {
        &self.target_id
    }
    #[must_use]
    pub const fn action(&self) -> ControlActionV1 {
        self.action
    }
    #[must_use]
    pub const fn status(&self) -> DecisionStatusV1 {
        self.status
    }
    #[must_use]
    pub fn decided_at(&self) -> &str {
        &self.decided_at
    }
    #[must_use]
    pub fn expires_at(&self) -> &str {
        &self.expires_at
    }
    #[must_use]
    pub fn reason_code(&self) -> &str {
        &self.reason_code
    }
    #[must_use]
    pub fn requested_by(&self) -> &str {
        &self.requested_by
    }
    #[must_use]
    pub fn decided_by(&self) -> &str {
        &self.decided_by
    }
    #[must_use]
    pub fn matched_rule_id(&self) -> Option<&str> {
        self.matched_rule_id.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ControlReceiptV1 {
    pub(crate) id: String,
    pub(crate) decision_id: String,
    pub(crate) target_id: String,
    pub(crate) status: ReceiptStatusV1,
    pub(crate) recorded_at: String,
    pub(crate) result_code: String,
    pub(crate) changed_state: bool,
}

impl ControlReceiptV1 {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub fn decision_id(&self) -> &str {
        &self.decision_id
    }
    #[must_use]
    pub fn target_id(&self) -> &str {
        &self.target_id
    }
    #[must_use]
    pub const fn status(&self) -> ReceiptStatusV1 {
        self.status
    }
    #[must_use]
    pub fn recorded_at(&self) -> &str {
        &self.recorded_at
    }
    #[must_use]
    pub fn result_code(&self) -> &str {
        &self.result_code
    }
    #[must_use]
    pub const fn changed_state(&self) -> bool {
        self.changed_state
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ControlReportV1 {
    pub(crate) schema: String,
    pub(crate) generated_at: String,
    pub(crate) external_actions: bool,
    pub(crate) decision_count: usize,
    pub(crate) receipt_count: usize,
    pub(crate) decisions: Vec<ControlDecisionV1>,
    pub(crate) receipts: Vec<ControlReceiptV1>,
}

impl ControlReportV1 {
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }
    #[must_use]
    pub fn generated_at(&self) -> &str {
        &self.generated_at
    }
    #[must_use]
    pub const fn external_actions(&self) -> bool {
        self.external_actions
    }
    #[must_use]
    pub const fn decision_count(&self) -> usize {
        self.decision_count
    }
    #[must_use]
    pub const fn receipt_count(&self) -> usize {
        self.receipt_count
    }
    #[must_use]
    pub fn decisions(&self) -> &[ControlDecisionV1] {
        &self.decisions
    }
    #[must_use]
    pub fn receipts(&self) -> &[ControlReceiptV1] {
        &self.receipts
    }
}
