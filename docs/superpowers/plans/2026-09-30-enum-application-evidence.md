# Plan: Enum application evidence filenames

Base 151ed023, isolated `/tmp/ares-enum-artifacts.XZ9HkF`.
Source and Rust boundaries: ARD-0050 and companion spec. Reuse the agreed
CLI option-sweep and exported-project/full-output testing seams.

1. Reproduce the actual wall-order sweep; add its evidence regression RED.
2. Project the case label to a stable filesystem-safe digest; preserve all
   original values, proof contents, comparison and completion gates.
3. Run all wall-order values and a width domain through original Orca;
   verify proof retention and stable filtered-case identity.
4. Run Nextest workspace, fmt, Clippy and Tier-1 compiler checks.
5. Independent six-axis review, repairs/re-review, code+docs commit/push.

## Diagnosis

The unchanged sweep reproduced Application/Io ENOENT for both slash-valued
cases; the third case passed full output comparison (exit 100, 2.614 s;
`/tmp/ares-wall-sequence-red.log`, `/tmp/ares-wall-sequence-red.Niz4d5`).
The same writable root and successful original project export exclude a
missing root and rejected enum value. Raw-label path construction in both
proof-write branches is the failure boundary.

Regression RED: 1.693 s, exit 100, the same two path errors
(`/tmp/ares-enum-regression-red.log`, `/tmp/ares-enum-regression-red.S1Kv6a`).
After projecting filenames to SHA-256 hex, GREEN: 1 PASS, 2.542 s,
three enum outputs plus one filtered rerun
(`/tmp/ares-enum-regression-green.log`, `/tmp/ares-enum-regression-green.vN1UUb`).
An initial compile attempt needed explicit bytewise hex formatting for the
workspace sha2 output type; no runtime behavior was changed by that repair.

The original sweep rerun with `wall_sequence,line_width` passed 2/2 domains,
8 generated/strictly compared, 0 rejected (`/tmp/ares-enum-width-sweep.log`,
`/tmp/ares-enum-width-sweep.8VT1LA`). All eight proof files retain equal
requested/effective values; all five width files retain the typed magnitude
and percent flag. The sweep's existing reference budget is unchanged, so
these are observed complete comparisons, not a first-reference or oracle
determinism guarantee. No historical all-option summary was replaced.

All 12 retained comparison manifests report PASS; input/output sizes and
SHA-256 digests were rechecked. The live invocation used the official 2.4.2
binary with SHA-256
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
The cached-byte manifests themselves do not prove producer identity;
the invocation and binary check provide separate provenance evidence.

Final isolated checks exited 0: `cargo nextest run --workspace` (7407 PASS,
9 skipped, 254.853 s), `cargo clippy` (248 existing core warnings), explicit
parity-test Clippy (28 existing test warnings), `cargo fmt --all -- --check`
and `git diff --check`. Logs: `/tmp/ares-enum-{workspace,clippy,test-clippy,fmt}.log`.
Compiler checks passed for wasm32-unknown-unknown (`ares-wasm`),
x86_64-pc-windows-gnu and aarch64-apple-darwin (workspace). Logs:
`/tmp/ares-enum-{wasm,windows,macos}.log`. These do not establish native
runtime coverage on the cross-compiled platforms.

Independent reviewer `review_completion_gate` approved the six-file slice
after static pre-review and terminal-evidence re-review. The first final
review attempt hit a service usage limit; the resumed review completed all
six axes with no required repairs. Approval covers only this filename and
evidence slice, not three-wall activation, the next coordinate slice or
global parity completion.
