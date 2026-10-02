# crowsi-network-controller

Evaluate a proposed network change against a deny-by-default policy before applying it.

## What you can do

- Validate a network-change request.
- Explain the policy result using a bounded response.

## Current scope

The current command evaluates requests. It does not mutate operating-system or cloud networking.

Package distribution is not activated by this documentation. Use the checked-in source and the declared dependency versions; published availability must be verified separately.

## Getting started

Install Rust 1.97 or newer and make the declared dependencies available. Use the configured private registry when a dependency is not distributed publicly. Run from this repository:

```sh
cargo test --locked
```

## Examples and interface details

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

## UI contract

Nuxt consumers depend only on the schemas in `schemas/`. Incompatible changes
require a new URI. Decision and receipt arrays are separate so an operations UI
can show intended policy outcomes and immutable audit facts independently.
Rust output fields are private, and deserialization validates the same safety
invariants before a report can be re-serialized.

Version 1 receipts identify the policy and matched rule but are not durable
signed audit artifacts. A future execution boundary must add a signed policy
revision digest under a new contract URI before enabling mutation.

## Documentation and source

[Interface reference](docs/interface-reference.md)

[Usage guide](docs/getting-started.md)

[Examples](examples) · [Schemas](schemas) · [Implementation and public interfaces](src) · [Verification cases](tests) · [Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [License](LICENSE) · [Attribution notices](NOTICE)
