use std::error::Error;
use std::fmt;

use crate::{
    CONTROL_POLICIES_SCHEMA_V1, CONTROL_REQUESTS_SCHEMA_V1, ControlPolicyV1, ControlRequestV1,
    timestamp::normalized_utc, timestamp_math::unix_millis,
};

pub const MAX_POLICY_RULES: usize = 32;
pub const MAX_RULE_REQUESTERS: usize = 32;
pub const MAX_REQUEST_WINDOW_MILLIS: i64 = 3_600_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    field: &'static str,
    reason: &'static str,
}

impl ValidationError {
    #[must_use]
    pub const fn new(field: &'static str, reason: &'static str) -> Self {
        Self { field, reason }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.field, self.reason)
    }
}

impl Error for ValidationError {}

pub trait Validate {
    /// Verifies the closed contract before policy evaluation.
    ///
    /// # Errors
    ///
    /// Returns the first stable field-level validation error.
    fn validate(&self) -> Result<(), ValidationError>;
}

impl Validate for ControlRequestV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema("schema", &self.schema, CONTROL_REQUESTS_SCHEMA_V1)?;
        identifier("request_id", &self.request_id)?;
        identifier("idempotency_key", &self.idempotency_key)?;
        identifier("target_id", &self.target_id)?;
        identifier("scope", &self.scope)?;
        identifier("requested_by", &self.requested_by)?;
        validate_timestamp("requested_at", &self.requested_at)?;
        validate_timestamp("expires_at", &self.expires_at)?;
        if self.requested_at >= self.expires_at {
            return Err(ValidationError::new(
                "expires_at",
                "must be later than requested_at",
            ));
        }
        let window = unix_millis(&self.expires_at)
            .zip(unix_millis(&self.requested_at))
            .map(|(expires, requested)| expires - requested);
        if window.is_none_or(|millis| millis > MAX_REQUEST_WINDOW_MILLIS) {
            return Err(ValidationError::new(
                "expires_at",
                "request window must not exceed one hour",
            ));
        }
        Ok(())
    }
}

impl Validate for ControlPolicyV1 {
    fn validate(&self) -> Result<(), ValidationError> {
        schema("schema", &self.schema, CONTROL_POLICIES_SCHEMA_V1)?;
        identifier("policy_id", &self.policy_id)?;
        if self.rules.len() > MAX_POLICY_RULES {
            return Err(ValidationError::new(
                "rules",
                "exceeds the policy rule limit",
            ));
        }
        let mut rule_ids = std::collections::BTreeSet::new();
        for rule in &self.rules {
            identifier("rule.id", &rule.id)?;
            identifier("rule.target_id", &rule.target_id)?;
            identifier("rule.scope", &rule.scope)?;
            if rule.requesters.is_empty() || rule.requesters.len() > MAX_RULE_REQUESTERS {
                return Err(ValidationError::new(
                    "rule.requesters",
                    "must contain between 1 and 32 requesters",
                ));
            }
            let mut requesters = std::collections::BTreeSet::new();
            for requester in &rule.requesters {
                identifier("rule.requesters", requester)?;
                if !requesters.insert(requester) {
                    return Err(ValidationError::new("rule.requesters", "must be unique"));
                }
            }
            if !rule_ids.insert(&rule.id) {
                return Err(ValidationError::new("rule.id", "must be unique"));
            }
        }
        Ok(())
    }
}

fn schema(
    field: &'static str,
    actual: &str,
    expected: &'static str,
) -> Result<(), ValidationError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ValidationError::new(field, "unsupported schema"))
    }
}

fn identifier(field: &'static str, value: &str) -> Result<(), ValidationError> {
    let valid = !value.is_empty()
        && value.len() <= 128
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

pub(crate) fn validate_timestamp(field: &'static str, value: &str) -> Result<(), ValidationError> {
    if normalized_utc(value) {
        Ok(())
    } else {
        Err(ValidationError::new(
            field,
            "must use normalized UTC milliseconds",
        ))
    }
}
