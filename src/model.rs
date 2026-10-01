use serde::{Deserialize, Serialize};

mod output;
mod output_wire;
pub use output::{ControlDecisionV1, ControlReceiptV1, ControlReportV1};

pub const CONTROL_REQUESTS_SCHEMA_V1: &str = "crowsi://network/control-requests/v1";
pub const CONTROL_POLICIES_SCHEMA_V1: &str = "crowsi://network/control-policies/v1";
pub const CONTROL_RECEIPTS_SCHEMA_V1: &str = "crowsi://network/control-receipts/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ControlModeV1 {
    DryRun,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ControlActionV1 {
    QuarantineInterface,
    RestrictEgress,
    RestoreEgress,
    EnterMaintenance,
    ExitMaintenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlRequestV1 {
    pub schema: String,
    pub request_id: String,
    pub idempotency_key: String,
    pub target_id: String,
    pub action: ControlActionV1,
    pub scope: String,
    pub mode: ControlModeV1,
    pub requested_by: String,
    pub requested_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllowRuleV1 {
    pub id: String,
    pub target_id: String,
    pub action: ControlActionV1,
    pub scope: String,
    pub requesters: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlPolicyV1 {
    pub schema: String,
    pub policy_id: String,
    pub enabled: bool,
    pub rules: Vec<AllowRuleV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DecisionStatusV1 {
    AllowedDryRun,
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReceiptStatusV1 {
    Recorded,
}
