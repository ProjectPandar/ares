# Spec: Filament-map wire-domain parity

Use ARD-0055's named ConfigOptionEnum key-map and existing typed Rust boundary.
Generate exactly the three canonical serialized values from upstream, verify
them against the public Rust enum, and compare each first official output in
full. Preserve the invalid Default export as a negative proof: its observed
default substitution must remain a mismatch, not accepted normalization.
Activate each concrete value in the generated fixture's plate metadata as
well as global config; verify both owners and the final config header.
Missing/stale plate metadata must fail evidence, not silently credit coverage.

Acceptance: domain regression RED/GREEN, three direct full comparisons,
negative Default application check, selected sweep, workspace Nextest,
fmt/Clippy, Tier-1 compiler checks and independent six-axis review. No source
line/hash/structure pinning assertions; behavioral token/output checks only.
Code and documents are committed/pushed together after verification.
