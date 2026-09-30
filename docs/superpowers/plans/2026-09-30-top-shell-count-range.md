# Top-shell count range implementation plan

Source/destination and non-negotiable scope: ARD-0066. Base: bd951efa.

1. Require the three representable full-range probes and observe domain RED.
2. Generate the range and exercise original export, exact application, final
   config and first-reference complete comparison in the non-spiral fixture.
3. Diagnose any discrepancy from paired artifacts and the named upstream
   projection boundary before changing production. If the original boundary
   cannot support deterministic parity, record the blocker without declaring exit.
4. On actual GREEN, selected sweep, full checks, independent six-axis review,
   repairs and code+docs commit/push. The bd951efa full scan remains immutable.

Domain RED: `/tmp/ares-top-shell-red.log`, one FAIL (0.954s), old two
samples versus required three. Before the production change, full comparison
failed for INT_MAX and INT_MAX-1, but passed for 0, seeded 786872368 and
INT_MAX-100 (`/tmp/ares-top-shell-boundary.log`, artifacts
`/tmp/ares-top-shell-boundary.LuFNd3`). No failed cases were discarded.

Native disassembly and runtime breakpoint evidence:
`/tmp/ares-top-shell-projection-assembly.log`,
`/tmp/ares-top-shell-gdb.log`. The observed addition wraps at index 1;
the repeated original G1 stream is stable. G1-only diagnostics were not used
as the comparison gate. See ARD-0066 for the binary-specific UB qualification.

After the explicit wrapping expression, the enhanced direct regression and
range-label contract pass: three tests (4.043s),
`/tmp/ares-top-shell-final.log`, five first-reference complete paired outputs
in `/tmp/ares-top-shell-final.1F2lSd`. Exported requests, final count config,
non-spiral mode, ensure_all and thickness 0.8 are asserted. All five final
manifest file sizes and SHA256 hashes have been independently read back;
manifest producer identity remains unknown, not proven by these hashes.

Selected sweep: one PASS (12.286s), three generated/strict comparisons and
zero rejected, `/tmp/ares-top-shell-selected.log`; artifacts
`/tmp/ares-top-shell-selected.KSmy0q`. All three manifests have verified file
sizes/SHA256. An earlier mistyped test selector matched no tests (exit 4,
`/tmp/ares-top-shell-sweep.log`); it is not passing evidence.

Full workspace Nextest: 7444 PASS, 9 skipped (269.961s),
`/tmp/ares-top-shell-workspace.log`. Tier-1 compiler checks pass for
ares-core/ares-vgcode/ares-wasm on wasm32-unknown-unknown and the workspace
on x86_64-pc-windows-gnu / aarch64-apple-darwin, logs
`/tmp/ares-top-shell-{wasm,windows,macos}.log`. These are cross-compilation
checks, not runtime tests on those targets.

The immutable bd951efa full scan is separately terminal: 380 PASS,
268 INCOMPLETE, 2 FAIL; 1659 generated, 1501 strictly compared, 1 rejected
(1313.701s), `/tmp/ares-options-bd951efa.v1B9dv/option-coverage-summary.md`.
It predates this repair; do not splice the focused result into its aggregate.

`cargo clippy`, parity-test Clippy, `cargo fmt`, `cargo fmt --check` and
`git diff --check` pass; logs `/tmp/ares-top-shell-{clippy,test-clippy,fmt}.log`.
Existing 248 core / 28 parity warnings remain. No changed Rust file exceeds
400 lines. Final independent six-axis review approves the eight-file slice,
with no remaining required repairs. This finite
slice does not claim 650-domain or all-printer completion. Pause after this
slice's code+docs commit and push, as requested.
