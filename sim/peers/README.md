# Peers

This directory defines peer roles and per-peer configuration expectations.

Recommended first peer roles:

- `peer-seed`
- `peer-reader-a`
- `peer-reader-b`
- `peer-fault`

Minimum per-peer state:

- `node_id`
- key material reference
- transport endpoint or logical bus address
- capabilities
- bootstrap peers
- object store path or logical store name

Suggested future additions:

- peer role schema
- sample peer configs
- fault-injection toggles
