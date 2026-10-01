use crowsi_network_controller::{PolicyEvaluator, parse_policy_json, parse_request_json};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = parse_request_json(include_bytes!("control.request.json"))?;
    let policy = parse_policy_json(include_bytes!("control.policy.json"))?;
    let report =
        PolicyEvaluator::default().evaluate(&request, &policy, "2026-08-01T00:05:00.000Z")?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
