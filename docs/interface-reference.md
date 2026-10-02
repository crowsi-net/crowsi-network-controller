# crowsi-network-controller interface reference

Use the [usage guide](getting-started.md) for the first steps. This reference preserves the current interface details and operational limits. Run command examples from the repository root, after preparing the exact declared dependencies and registered configuration.

## Commands

```bash
cargo run -- sample
cargo run -- evaluate examples/control.request.json examples/control.policy.json
cargo test
```

Both commands emit `crowsi://network/control-receipts/v1`. `sample` uses fixed
input and time. `evaluate` reads closed, size-bounded JSON documents and uses
the current UTC execution time when checking the request window. Input paths
must resolve to regular files.

## Policy behavior

- Requests must use `mode: "dry-run"`.
- Public-edge maintenance actions produce policy evidence only; they never
  publish a status file, deploy a site, or change CDN/DNS state.
- Request authorization windows cannot exceed one hour.
- Rules are exact allowlist entries; wildcard targets and scopes do not exist.
- Requester identities, targets, actions, and scopes must all match.
- Unknown fields and non-identifier values are rejected.
- An idempotency key can refer to only one request and policy.
- Replays compare the full request and policy; conflicts fail closed.
- Missing, disabled, expired, or conflicting requests are denied.
- Allowed decisions include the exact matched rule identifier as audit evidence.

`PolicyEvaluator` uses an in-memory ledger suitable for embedding and tests.
Policies accept at most 32 rules and 32 requesters per rule. Expired entries
are purged before evaluation, and the ledger fails closed at 256 live entries.
A separate 256-entry conflict ledger preserves immutable conflict replays and
uses process-unique bounded receipt IDs. Evaluation time cannot move backwards.
A host that spans process restarts must provide serialized
request processing around a durable idempotency boundary before invoking this
library. It must still keep mutation execution outside this repository.

## UI contract

Nuxt consumers depend only on the schemas in `schemas/`. Incompatible changes
require a new URI. Decision and receipt arrays are separate so an operations UI
can show intended policy outcomes and immutable audit facts independently.
Rust output fields are private, and deserialization validates the same safety
invariants before a report can be re-serialized.

Version 1 receipts identify the policy and matched rule but are not durable
signed audit artifacts. A future execution boundary must add a signed policy
revision digest under a new contract URI before enabling mutation.
