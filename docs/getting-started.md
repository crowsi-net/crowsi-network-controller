# Using crowsi-network-controller

Evaluate a proposed network change against a deny-by-default policy before applying it.

## Before you start

The current command evaluates requests. It does not mutate operating-system or cloud networking.

## First steps

Run from the repository root:

```sh
cargo test --locked
```

## How to assess the result

- Validate a network-change request.
- Explain the policy result using a bounded response.

A passing source-level check establishes only what that check observes. Keep missing configuration, unavailable services and unverified deployment paths visible.

## Continue reading

[Repository overview](../README.md)
