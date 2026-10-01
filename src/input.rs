use crate::{ControlPolicyV1, ControlRequestV1, Validate, ValidationError};

pub const MAX_DOCUMENT_BYTES: usize = 65_536;

/// Parses and validates one closed control request.
///
/// # Errors
///
/// Rejects oversized, malformed, unknown, or invalid fields.
pub fn parse_request_json(source: &[u8]) -> Result<ControlRequestV1, ValidationError> {
    bounded(source, "request")?;
    let request: ControlRequestV1 = serde_json::from_slice(source)
        .map_err(|_| ValidationError::new("request", "invalid closed JSON"))?;
    request.validate()?;
    Ok(request)
}

/// Parses and validates one closed control policy.
///
/// # Errors
///
/// Rejects oversized, malformed, unknown, or invalid fields.
pub fn parse_policy_json(source: &[u8]) -> Result<ControlPolicyV1, ValidationError> {
    bounded(source, "policy")?;
    let policy: ControlPolicyV1 = serde_json::from_slice(source)
        .map_err(|_| ValidationError::new("policy", "invalid closed JSON"))?;
    policy.validate()?;
    Ok(policy)
}

fn bounded(source: &[u8], field: &'static str) -> Result<(), ValidationError> {
    if source.len() > MAX_DOCUMENT_BYTES {
        Err(ValidationError::new(field, "document exceeds 65536 bytes"))
    } else {
        Ok(())
    }
}
