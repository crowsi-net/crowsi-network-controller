use std::collections::BTreeMap;

use crate::{
    ControlPolicyV1, ControlReportV1, ControlRequestV1, Validate, ValidationError, report::report,
    validation::validate_timestamp,
};

pub const MAX_LEDGER_ENTRIES: usize = 256;
pub const MAX_CONFLICT_ENTRIES: usize = 256;

#[derive(Debug, Clone)]
pub(crate) struct SeenEvaluation {
    pub(crate) fingerprint: EvaluationFingerprint,
    pub(crate) report: ControlReportV1,
}

/// Collision-free structural identity for an evaluated request and policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EvaluationFingerprint {
    pub(crate) request: ControlRequestV1,
    pub(crate) policy: ControlPolicyV1,
}

impl EvaluationFingerprint {
    pub(crate) fn new(request: &ControlRequestV1, policy: &ControlPolicyV1) -> Self {
        Self {
            request: request.clone(),
            policy: policy.clone(),
        }
    }
}

/// Stateful idempotency boundary for policy-only dry-run evaluation.
#[derive(Debug, Default)]
pub struct PolicyEvaluator {
    pub(crate) seen: BTreeMap<String, SeenEvaluation>,
    pub(crate) conflicts: Vec<SeenEvaluation>,
    pub(crate) next_conflict_sequence: u64,
    pub(crate) watermark: Option<String>,
}

impl PolicyEvaluator {
    /// Evaluates a request without applying any network change.
    ///
    /// # Errors
    ///
    /// Returns validation errors before evaluating authorization.
    pub fn evaluate(
        &mut self,
        request: &ControlRequestV1,
        policy: &ControlPolicyV1,
        evaluated_at: &str,
    ) -> Result<ControlReportV1, ValidationError> {
        request.validate()?;
        policy.validate()?;
        validate_timestamp("evaluated_at", evaluated_at)?;
        if self
            .watermark
            .as_deref()
            .is_some_and(|watermark| evaluated_at < watermark)
        {
            return Err(ValidationError::new(
                "evaluated_at",
                "must not move backwards",
            ));
        }
        self.watermark = Some(evaluated_at.to_owned());
        self.seen
            .retain(|_, seen| evaluated_at < seen.fingerprint.request.expires_at.as_str());
        self.conflicts
            .retain(|seen| evaluated_at < seen.fingerprint.request.expires_at.as_str());
        let fingerprint = EvaluationFingerprint::new(request, policy);
        if let Some(seen) = self.seen.get(&request.idempotency_key) {
            if seen.fingerprint == fingerprint {
                return Ok(seen.report.clone());
            }
            return self.evaluate_conflict(request, policy, evaluated_at, fingerprint);
        }
        if self.seen.len() >= MAX_LEDGER_ENTRIES {
            let result = report(
                request,
                policy,
                evaluated_at,
                None,
                "idempotency-ledger-full",
                None,
            );
            result.validate()?;
            return Ok(result);
        }
        let matched = if policy.enabled && in_window(request, evaluated_at) {
            policy.rules.iter().find(|rule| {
                rule.target_id == request.target_id
                    && rule.action == request.action
                    && rule.scope == request.scope
                    && rule.requesters.contains(&request.requested_by)
            })
        } else {
            None
        };
        let reason = if !policy.enabled {
            "policy-disabled"
        } else if !in_window(request, evaluated_at) {
            "request-outside-window"
        } else if matched.is_some() {
            "allowlist-match"
        } else {
            "default-deny"
        };
        let result = report(
            request,
            policy,
            evaluated_at,
            matched.map(|rule| rule.id.as_str()),
            reason,
            None,
        );
        result.validate()?;
        self.seen.insert(
            request.idempotency_key.clone(),
            SeenEvaluation {
                fingerprint,
                report: result.clone(),
            },
        );
        Ok(result)
    }
}

fn in_window(request: &ControlRequestV1, evaluated_at: &str) -> bool {
    request.requested_at.as_str() <= evaluated_at && evaluated_at < request.expires_at.as_str()
}
