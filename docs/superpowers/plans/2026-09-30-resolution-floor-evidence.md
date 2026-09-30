# Plan: Resolution floor application evidence

Base 9692fe6e, clean reused worktree `/tmp/ares-obsolete-silent.M6IJE6`.
Reuse the agreed original-export application proof and project-to-complete
G-code seams. Source/Rust boundaries and exclusions: ARD-0053.

1. Reproduce selected sweep; add proof regression RED.
2. Add only the source-owned floor rule; rerun RED to GREEN.
3. Direct original exports/full outputs at boundaries and generated probes.
4. Workspace Nextest, fmt/Clippy, Tier-1 compiler checks; independent review.
5. Repair/re-review then commit/push code and documentation together.

Selected baseline RED: exit 100, 2.472 s, requested zero exports as 0.001;
seeded case reaches full comparison (`/tmp/ares-resolution-red.log`,
`/tmp/ares-resolution-red.soxAJ9`). Source normalization is explicit, and
the existing Rust stage implements it. No production change is indicated.
Proof RED: 0.013 s, requested zero rejected against 0.001; GREEN: 0.010 s
(`/tmp/ares-resolution-proof-{red,green}.log`). Coverage-claim RED: 0.485 s,
the existing plan incorrectly marks its unprobed implicit ceiling as covered
(`/tmp/ares-resolution-ceiling-red.log`). The repair preserves generated
values and marks only resolution INCOMPLETE with a named ceiling omission.

Expanded direct run: 4 PASS, 4.634 s, including six first-reference complete
ordered output comparisons (`/tmp/ares-resolution-direct.log`,
`/tmp/ares-resolution-direct.ctNhI5`). The ordinary selected sweep now
strictly compares both generated cases, zero rejected; its exit 100 is the
expected INCOMPLETE ceiling gate, not an output/application failure (6.541 s,
`/tmp/ares-resolution-sweep.log`, `/tmp/ares-resolution-sweep.z2d1qd`).
All eight manifests' input/output sizes and SHA-256 hashes were verified.
The invoked official v2.4.2 binary SHA-256 is
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Invocation/binary identity are separate evidence; cache manifests alone
do not independently prove producer identity. No reference resampling is
used in the six direct cases; only generator identity/time normalization
is permitted by the unchanged ordered comparator.

WASM adapter, Windows GNU workspace and macOS aarch64 workspace compiler
checks pass (`/tmp/ares-resolution-{wasm,windows,macos}.log`); these are not
browser/native runtime tests. Workspace Nextest passes 7415 tests, 9 skipped,
252.299 s (`/tmp/ares-resolution-workspace.log`). Default and explicit parity-test
Clippy pass with the existing warnings; fmt and diff checks pass
(`/tmp/ares-resolution-{clippy,test-clippy,fmt}.log`). Independent six-axis
read-only review approved this nine-file slice with no required repairs.
