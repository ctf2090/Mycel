# Mycel over Tor Profile v0.1

Status: profile draft

This profile defines a narrow deployment model for running Mycel over Tor-oriented transport.

This profile is transport-focused.

It does not claim to provide complete anonymity by itself.

Instead, it narrows:

- transport expectations
- peer addressing rules
- peer discovery behavior
- metadata handling
- role-separation expectations

## 0. Scope

This profile assumes the implementation already supports:

- the Mycel core protocol
- the v0.1 wire protocol
- proxyable outbound transport
- configurable peer addressing

This profile applies to:

- Tor-routed peer transport
- onion-first peer addressing
- anonymity-aware deployments that want a narrow first operating profile

## 1. Profile Goals

The goals are:

1. reduce direct IP exposure
2. reduce peer-address leakage
3. keep transport assumptions explicit
4. make anonymity limitations visible instead of implied

## 2. Transport Requirements

A conforming node in this profile must satisfy the following:

1. all outbound peer traffic MUST be routed through Tor or an equivalent local Tor proxy
2. direct clearnet peer dialing MUST be disabled by default
3. peer addresses SHOULD prefer onion endpoints when available
4. transport failures MUST NOT silently fall back to direct clearnet transport

This profile treats direct fallback as a profile violation unless explicitly overridden outside the profile.

## 3. Peer Addressing

This profile prefers onion-first addressing.

Recommended address classes:

- `tor:onion-v3`
- `tor:intro-ref`
- local bootstrap aliases that resolve only through Tor-aware configuration

This profile does not require a public clearnet address.

A conforming implementation should:

- store onion endpoints separately from any optional clearnet endpoints
- prefer onion endpoints for outbound connection attempts
- avoid publishing unnecessary clearnet alternatives in profile-conforming mode

## 4. Peer Discovery

This profile intentionally narrows peer discovery.

Allowed discovery sources:

- explicit local bootstrap list
- accepted peer manifests obtained over Tor-routed transport
- out-of-band trusted peer introductions

Disallowed as default behavior in this profile:

- public clearnet seed probing
- opportunistic direct-network scanning
- silent clearnet fallback discovery

The implementation may still support manual peer addition, but profile-conforming operation should keep discovery bounded and explicit.

## 5. Message Metadata Handling

Tor transport alone is not enough.

A conforming implementation should also reduce unnecessary linkable metadata.

Recommended rules:

- avoid long-lived local node labels in user-facing transport displays
- avoid exposing non-required sender metadata outside the wire envelope
- do not add deployment-specific routing hints to replicated objects
- keep timestamp precision no higher than needed by the active profile

This profile does not redefine the wire envelope, but it constrains deployment behavior around how much extra metadata is attached or retained.

## 6. Local Logging and Caching

Local behavior can easily undermine transport anonymity.

A conforming implementation should:

- avoid persistent logging of raw peer IPs
- avoid storing clearnet fallback attempts in normal operation
- separate anonymous-reading caches from signer or payment-operation caches
- support cache cleanup and local compartment separation

This profile does not forbid local diagnostics, but diagnostics that preserve direct network identity should be disabled by default.

## 7. Role Separation

This profile recommends operational role separation.

Recommended separations:

- anonymous reader nodes separate from governance-maintainer nodes
- governance-maintainer nodes separate from signer nodes
- signer nodes separate from effect or payment runtimes

This does not make public governance activity anonymous, but it reduces unnecessary cross-role correlation.

## 8. Wire Behavior Constraints

A profile-conforming node should preserve normal wire compatibility while constraining transport use.

Recommended rules:

- `HELLO` and other wire messages still follow the v0.1 wire envelope
- peer sessions should be established only over Tor-routed transport
- repeated failed connection attempts should not trigger direct-network fallback
- node operators should be able to inspect whether a session was profile-conforming

This profile does not modify message shapes.

It constrains how sessions are established.

## 9. Replication Strategy

This profile recommends narrower replication behavior for anonymity-sensitive deployments.

Recommended controls:

- fetch only object families needed by the active role
- avoid universal mirroring of all sensitive app-layer records on reader nodes
- preserve accepted-state verification without forcing full artifact replication everywhere

This profile favors bounded replication over maximal visibility.

## 10. Operational Warnings

This profile should surface the following warnings clearly:

- Tor transport does not hide stable governance keys
- public signing activity remains linkable
- payment or effect runtimes may deanonymize related activity
- local mixing of anonymous reading and identified operations weakens anonymity

## 11. Minimal Conforming Flow

The minimal conforming flow is:

1. load an onion-first bootstrap list
2. establish outbound sessions only through Tor-routed transport
3. exchange normal v0.1 wire messages
4. fetch only required objects for the local role
5. avoid clearnet fallback
6. preserve local state in a compartment appropriate to the role

## 12. Non-goals

This profile does not define:

- perfect anonymity
- immunity to global traffic analysis
- anonymous public governance signing
- anonymous payment settlement
- a new wire-message format

## 13. Minimal First-client Requirements

For a first interoperable client, I recommend:

- one Tor proxy configuration path
- onion-first peer list support
- no automatic clearnet fallback
- explicit UI warning when a non-profile transport is used
- separate local profiles for anonymous reading versus identified operations

## 14. Open Questions

- Should a later version require onion service publication for all publicly reachable peers?
- Should timestamp precision be reduced further in an anonymity-specific profile?
- Should peer discovery be split into separate public-anonymous and restricted-anonymous profiles?
