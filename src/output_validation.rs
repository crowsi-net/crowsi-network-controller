use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CONTROL_RECEIPTS_SCHEMA_V1, ControlDecisionV1, ControlReceiptV1, ControlReportV1,
    DecisionStatusV1, Validate, ValidationError, validation::validate_timestamp,
};

const MAX_REPORT_ITEMS: usize = 256;

impl Validate for ControlReportV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.schema != CONTROL_RECEIPTS_SCHEMA_V1 {
            return Err(ValidationError::new("schema", "unsupported schema"));
        }
        validate_timestamp("generated_at", &self.generated_at)?;
        if self.external_actions
            || self.decision_count != self.decisions.len()
            || self.receipt_count != self.receipts.len()
            || self.decisions.len() != self.receipts.len()
            || self.decisions.len() > MAX_REPORT_ITEMS
            || self.receipts.len() > MAX_REPORT_ITEMS
        {
            return Err(ValidationError::new(
                "report",
                "contains inconsistent or unsafe output",
            ));
        }
        let mut decisions = BTreeMap::new();
        for decision in &self.decisions {
            validate_decision(decision)?;
            if decision.decided_at > self.generated_at {
                return Err(ValidationError::new(
                    "decision.decided_at",
                    "must not be later than generated_at",
                ));
            }
            if decisions.insert(decision.id.as_str(), decision).is_some() {
                return Err(ValidationError::new("decision.id", "must be unique"));
            }
        }
        let mut receipt_ids = BTreeSet::new();
        let mut referenced = BTreeSet::new();
        for receipt in &self.receipts {
            validate_receipt(receipt)?;
            if receipt.recorded_at > self.generated_at {
                return Err(ValidationError::new(
                    "receipt.recorded_at",
                    "must not be later than generated_at",
                ));
            }
            let Some(decision) = decisions.get(receipt.decision_id.as_str()) else {
                return Err(ValidationError::new(
                    "receipt.decision_id",
                    "must reference a decision",
                ));
            };
            if !receipt_ids.insert(receipt.id.as_str())
                || !referenced.insert(receipt.decision_id.as_str())
                || receipt.target_id != decision.target_id
                || receipt.result_code != decision.reason_code
                || receipt.recorded_at < decision.decided_at
            {
                return Err(ValidationError::new(
                    "receipt",
                    "must uniquely and consistently reference a decision",
                ));
            }
        }
        if referenced.len() == decisions.len() {
            Ok(())
        } else {
            Err(ValidationError::new(
                "receipt",
                "every decision must have one receipt",
            ))
        }
    }
}

fn validate_decision(decision: &ControlDecisionV1) -> Result<(), ValidationError> {
    identifier("decision.id", &decision.id)?;
    identifier("decision.target_id", &decision.target_id)?;
    identifier("decision.reason_code", &decision.reason_code)?;
    identifier("decision.requested_by", &decision.requested_by)?;
    identifier("decision.decided_by", &decision.decided_by)?;
    validate_timestamp("decision.decided_at", &decision.decided_at)?;
    validate_timestamp("decision.expires_at", &decision.expires_at)?;
    if decision.status == DecisionStatusV1::AllowedDryRun
        && decision.expires_at <= decision.decided_at
    {
        return Err(ValidationError::new(
            "decision.expires_at",
            "must be later than decided_at",
        ));
    }
    let matched = decision.matched_rule_id.as_deref();
    if let Some(rule) = matched {
        identifier("decision.matched_rule_id", rule)?;
    }
    let valid_match = match decision.status {
        DecisionStatusV1::AllowedDryRun => {
            matched.is_some() && decision.reason_code == "allowlist-match"
        }
        DecisionStatusV1::Denied => matched.is_none() && decision.reason_code != "allowlist-match",
    };
    if valid_match {
        Ok(())
    } else {
        Err(ValidationError::new(
            "decision.matched_rule_id",
            "does not match decision status",
        ))
    }
}

fn validate_receipt(receipt: &ControlReceiptV1) -> Result<(), ValidationError> {
    identifier("receipt.id", &receipt.id)?;
    identifier("receipt.decision_id", &receipt.decision_id)?;
    identifier("receipt.target_id", &receipt.target_id)?;
    identifier("receipt.result_code", &receipt.result_code)?;
    validate_timestamp("receipt.recorded_at", &receipt.recorded_at)?;
    if receipt.changed_state {
        Err(ValidationError::new(
            "receipt.changed_state",
            "must remain false",
        ))
    } else {
        Ok(())
    }
}

fn identifier(field: &'static str, value: &str) -> Result<(), ValidationError> {
    let valid = !value.is_empty()
        && value.len() <= 256
        && value.starts_with(|character: char| character.is_ascii_lowercase())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
        });
    if valid {
        Ok(())
    } else {
        Err(ValidationError::new(field, "must be a closed identifier"))
    }
}
