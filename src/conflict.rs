use crate::{
    ControlPolicyV1, ControlReportV1, ControlRequestV1, PolicyEvaluator, Validate, ValidationError,
    evaluator::{EvaluationFingerprint, MAX_CONFLICT_ENTRIES, SeenEvaluation},
    report::report,
};

impl PolicyEvaluator {
    pub(crate) fn evaluate_conflict(
        &mut self,
        request: &ControlRequestV1,
        policy: &ControlPolicyV1,
        evaluated_at: &str,
        fingerprint: EvaluationFingerprint,
    ) -> Result<ControlReportV1, ValidationError> {
        if let Some(conflict) = self
            .conflicts
            .iter()
            .find(|conflict| conflict.fingerprint == fingerprint)
        {
            return Ok(conflict.report.clone());
        }
        if self.conflicts.len() >= MAX_CONFLICT_ENTRIES {
            return Err(ValidationError::new(
                "idempotency_key",
                "conflict ledger is full",
            ));
        }
        let sequence = self.next_conflict_sequence;
        self.next_conflict_sequence = sequence.checked_add(1).ok_or_else(|| {
            ValidationError::new("idempotency_key", "conflict sequence exhausted")
        })?;
        let suffix = format!("conflict-{sequence:016x}-{}", request.request_id);
        let result = report(
            request,
            policy,
            evaluated_at,
            None,
            "idempotency-conflict",
            Some(&suffix),
        );
        result.validate()?;
        self.conflicts.push(SeenEvaluation {
            fingerprint,
            report: result.clone(),
        });
        Ok(result)
    }
}
