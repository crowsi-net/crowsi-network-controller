# Security policy

## Non-mutation guarantee

This repository contains no network mutation adapter. Do not add direct
invocations of firewall, routing, interface, process, device, cloud, or remote
administration APIs. A future executor must live in another security boundary
and consume an explicitly authorized, signed contract after separate review.

## Request boundary

Requests and policies are closed JSON documents with bounded input. Values are
opaque identifiers, not commands, addresses, URLs, or scripts. Do not add
credential material, arbitrary environment variables, shell fragments, packet
data, or unbounded parameter maps.

The evaluator is deny-by-default. Authorization requires an enabled policy and
one exact target/action/scope/requester match. Failure to parse, validate, load
state, or resolve a rule must never become an allow decision.

## Audit

Receipts record policy outcomes only. They must never claim that state changed,
and must not contain secrets or raw operating-system errors. Allowed decisions
record the exact matched rule. In-memory replays require an identical request
and policy; conflicting reuse fails closed with fixed, non-user-derived codes.
Durable hosts must enforce the same rule before accepting a report.

## Private vulnerability reporting

Report vulnerabilities through this repository's GitHub private vulnerability reporting form. Do not put credentials, personal or customer data, or production certificate material in public issues or pull requests.
