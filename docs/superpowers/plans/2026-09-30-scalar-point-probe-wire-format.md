# Plan: Scalar-point probe wire format

Base 22a6a511, clean isolated `/tmp/ares-enum-artifacts.XZ9HkF` advanced by
fast-forward after the prior reviewed slice. Source/Rust scope: ARD-0051.
Use the established CLI export/application and complete-output seams.

1. Reproduce one scalar-point failure and add the real export regression RED.
2. Change only the authored scalar wire strings to canonical commas.
3. Verify all four points and alternate spellings, then the ordinary sweep.
4. Nextest workspace, fmt/Clippy, platform compiler checks.
5. Independent six-axis review, repair/re-review, code+docs commit/push.

Prior four-option diagnosis: exit 100, 5.732 s (`/tmp/ares-point-repro.log`,
`/tmp/ares-point-repro.I3EZXY`). Minimal repeat on this base: only
`best_object_pos`, exit 100, 1.756 s (`/tmp/ares-point-minimal-red.log`,
`/tmp/ares-point-minimal-red.v0bbb4`), requested `0.25x0.75`, effective
`0.25,0.75`. Coordinates and owning preset survive; the cited serializer
explains the sole change, excluding preset replacement or numeric mutation.

Actual export regression RED: 0.782 s, exit 100
(`/tmp/ares-point-regression-red.log`, `/tmp/ares-point-regression-red.BlHZvl`),
the same `bed_mesh_min` spelling mismatch. Two authored strings and their
scalar-format comment changed; no numeric value or general verifier changed.
GREEN: 1 PASS, 5.243 s, eight direct first-reference complete comparisons
(`/tmp/ares-point-regression-green.log`, `/tmp/ares-point-regression-green.7IL6DR`).
The original four-option sweep now passes all four generated cases, 0 rejected,
4.200 s (`/tmp/ares-point-sweep.log`, `/tmp/ares-point-sweep.bKRjeY`).
Its inherited summary's domain-complete terminology is not proof of exhaustive
coordinate ranges: these are representative authored probes, as scoped above.
No historical whole-option report was replaced or merged.

All 12 retained comparison manifests report PASS; input/output sizes and
SHA-256 digests were verified, along with all four requested/effective proof
JSON files. The live invocation used the official 2.4.2 binary with SHA-256
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Cached-byte manifests do not themselves establish producer identity; invocation
and binary verification are separate provenance evidence.

Platform compiler checks exited 0 for wasm32-unknown-unknown (`ares-wasm`),
x86_64-pc-windows-gnu and aarch64-apple-darwin (workspace); logs
`/tmp/ares-point-{wasm,windows,macos}.log`. These are not native runtime tests.
Final isolated verification exited 0: `cargo nextest run --workspace`
(7408 PASS, 9 skipped, 251.047 s), `cargo clippy` (248 existing core warnings),
explicit parity-test Clippy (28 existing test warnings), `cargo fmt --all
-- --check` and `git diff --check`. Logs:
`/tmp/ares-point-{workspace,clippy,test-clippy,fmt}.log`.
Independent reviewer `review_completion_gate` approved the seven-file slice
after static and terminal-evidence six-axis review, with no required repairs.
Approval is limited to representative scalar-point export formatting, not
exhaustive coordinate ranges, template activation or global parity completion.
