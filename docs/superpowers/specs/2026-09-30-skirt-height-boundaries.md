# Skirt height boundary coverage

## Source boundary and destination

Cover `PrintConfig.cpp:5692`'s `skirt_height` maximum 10000 and
`PrintConfig.cpp:10422-10424`'s validation minimum zero,
`Print.cpp:575-585::has_infinite_skirt/has_skirt`, and
`Print.cpp:2663-2666`'s object-layer cap. The current Rust rewrite boundary
is `project_slice/gcode_emit/skirt.rs::SkirtPlan`; this slice changes only
CLI parity coverage in `option_coverage/domains/probes.rs`, `activation.rs`
`domains.rs` and a separate public-output test. Existing skirt planning remains a
temporary compatibility shell around upstream Print skirt behavior.

Replace fixed 1/2/10000 authored probes with the source-owned effective
minimum/maximum and deterministic random interior. Set one skirt loop and
disable draft shield so height controls the result. Add public output cases
for one and two layers alongside generated minimum/maximum/interior cases;
test upstream negative-input rejection separately. Zero is a validation
minimum, not an explicitly declared schema minimum.

Deferred: multi-object/raft skirt geometry, infinite draft shields, other
option gaps, default-printer parity. No source pinning, weaker comparison
or core algorithm changes are planned.

## Acceptance

The old domain gate must fail before correction; afterwards the generated
domain and five complete output cases must pass against official Orca,
and negative input must produce an attributable validation rejection.
Verify effective options and emitted skirt-layer counts, not just config
comments. Run workspace Nextest, fmt and Clippy, independent six-aspect
review, then commit/push code and documentation together.
