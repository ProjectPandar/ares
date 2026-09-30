# Spec: Resolution floor application evidence

Follow ARD-0053's PrintConfig normalization boundary and existing typed
Rust destination. Both the zero-minimum generated probe and the seeded probe
must reach original full-output comparison; keep their raw input evidence.
Additional direct cases cover below, at and above the 0.001 floor.

Acceptance: wrong exports still fail, unrelated numeric options keep strict
equality, and direct original complete outputs match without resampling.
The selected sweep must remain INCOMPLETE because its implicit FLT_MAX
ceiling is unprobed, rather than report an artificially green completion gate.
Workspace Nextest, fmt/Clippy, Tier-1 compiler checks and independent six-axis
review are required before code+docs commit/push.
