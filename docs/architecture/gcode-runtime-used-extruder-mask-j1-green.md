# J1 mask + J1b + J1c: GREEN and combined review record

Status: exact-hash combined REVIEW requested; not pushed.

## Approved boundaries and implementation

- Mask spec `b7b5b9a277b266bdb640abea75716082c0e5ffb7` and scoped RED
  `4f943a7eda2b71cc7c96a3ac6352be656e92efd1`: by-layer runtime usage is
  `tool_ordering.all_extruders()` (`GCode.cpp:2901–2904`), distinct from
  writer registration. By-object mask remains deferred and unchanged.
- J1b spec `5f33aaacc2d91d60a892652a92e3b10aaf24ea89`: preserve the initial
  current tool before writer selection (`GCode.cpp:2856, 7950–8007`), not
  `current=next` at every change. Existing cooling marker stays unchanged.
- J1c spec `ce01c3f999f11c894b6cda25d32bb6b3438b1149` and independently
  approved lifecycle RED `e9267f16ff689b6b7b08ebb2770feb1ac0ab4432`:
  consume mature Top fill surfaces (`AvoidCrossingPerimeters.cpp:1116–1130`)
  after `prepare_fill_surfaces` retagging (`LayerRegion.cpp:949–955`).
  `project_slice.rs` now extracts after combine-infill from the retained
  `PreparedPostHorizontalShellPropagation`. Only 4 additions/6 deletions;
  398 LOC. No edits to worker2's route/build functions or to M73, seam,
  wipe, cooling, shortest-direction or numerical intersection rules.

All upstream citations use pin8500fcdccaa10b5099ac20d252af3a7c560046f1.
Production commits remain separate after rebases. Current main base is
`96577fff7bb36b250d7237ba657cf27c85cdb5df`; current production commits:

| Slice | Current commit | Original production commit |
|---|---|---|
| mask | 9e634e6bc | df56e12f0fe3fad0e071cbb717f830ff9dbd8ca6 |
| J1b | ee404108c | 7b8dbd4877148b78a660ce4ce1e5b4d3062450b1 |
| J1c | 8a0005be2 | 9a807923d4a8ac33295db482172eb5a39d11cf9c |

Combined production diff SHA256:
`acf2aa7e9eb2635fefe07f52c889df8825725683bc7551c44f509640605a4f61`.

## Directed outcome — no primary relaxation

Original input SHA256 remains
`7d796432a448d46adc30e9ea91b4c729a6044058d3dd61bb4f5c13e64ea615f4`.
The primary now **passes the complete ordered-byte comparison**, retaining
M73, stats/footer and all motion, with generated-by identity as the sole
normalization. Before J1c it was RED at byte3810/line222. The extra stale
Top holes caused later real route changes and moved time/cooling thresholds;
fixing their lifecycle eliminates the complete-file residual without a
processor timing patch.

Six fresh directed checks PASS, zero timeouts:

1. Original role2 whole-file primary.
2. Role1 startup usage/initial/tool+flush witness.
3. Both-used startup witness.
4. By-object unchanged startup witness.
5. Zero-top final-layer travel lifecycle RED→GREEN.
6. Separately frozen nonzero-top final-layer travel control remains PASS.

The three startup controls and the two lifecycle travel controls are
**witness-only**, not claims of their whole-file parity. Their full streams
are retained. Original tests and frozen expectations were not relaxed.
Nonzero-top control SHA256
`83d0c1908087d796008602671bdc3a6ef43f12fdd7423b0d3c96db2d423af310`;
only `top_shell_layers:0→1` differs from the original configuration, with
other config/ZIP payloads unchanged. Control was frozen before Native.

Each directed call used public slicing, fresh Native cache/artifacts,
CPU0/seed0/runs1, prebuild outside its shared300s deadline and owned-descendant
cleanup. Six receipt bindings and witness data are in
`${TMP_ROOT}/ares-j1-mask/final-3c1/validation-audit.json`.

## Fresh full paired gates

Same-round base `3c1a7994164082964407c7a0179694c5c25cb5b8` versus candidate
`e9919a56af8d3ba011c6ab4bd8df4956486cc8f2`; 16 threads, one heavy at a time,
separate targets/worktrees and new per-variant/per-mode Native caches.
All configured full control-reference flags were enabled; **no test selector
narrowed either matrix**.

| Mode | Base | Candidate | Exit / failure-set result |
|---|---|---|---|
| Full seeded | 676/676 PASS, 0 skips | 682/682 PASS, 0 skips | both exit0; no failures/timeouts |
| Full official | 666 PASS /10 FAIL, 0 skips | 672 PASS /10 FAIL, 0 skips | both exit100; identical ten named failures; 0TIMEOUT |
| Directed | — | 6/6 PASS | exit0, 0TIMEOUT |
| Strict clippy | — | workspace/all-targets/all-features `-D warnings` | exit0 |
| rustfmt | — | all/check | exit0 |
| WASM | — | ares-wasm wasm32-unknown-unknown | exit0 |

Official is **not** all GREEN. Its ten unchanged failures are listed in full
in `validation-audit.json`; candidate-only and base-only failure sets are
empty. No timeout, numerical exception, fixture tuning or retry-until-success
was waived. A workspace nextest suite was not run in this round; do not infer
one from the full parity matrices.

Comparator-manifest stream/input audit (not the entire historical inventory):

- Seeded: 520 exact same-input ZIP-payload/label groups, 519 unchanged groups,
  534 unchanged bound full-stream pairs; one expected changed group.
- Official: 509 exact same-input groups, 508 unchanged groups, 523 unchanged
  bound full-stream pairs; the same expected changed group.
- Changed group is `compatible-regions/printing-tool-template-lifecycle`:
  base's retained whole-file manifest was DIVERGENT despite its scoped test
  PASS; candidate manifest becomes PASS. The byte change removes the false
  initial `Change T0 -> T1` block and preserves the cooling marker, directly
  matching J1b's approved initial-current restoration. This is not an
  unrelated output change. Do not equate all nextest PASS with every retained
  manifest being complete-byte PASS.
- One candidate-only comparator group is the newly added original J1 primary.

## Rebase/evidence history

- First GREEN directed round on ff40: six PASS; source-only J1c commit9a807923.
- Rebased dc8 (`-100` error type/CLI exit-only). Partial dc8 round completed
  six directed checks and strict clippy/fmt. Browser hold stopped the shell
  driver before matrices; its WASM child completed but the stopped parent
  had not collected exit. That driver was explicitly terminated after
  rebasing both source worktrees. Its partial evidence and checkpoint remain
  `${TMP_ROOT}/ares-j1-mask/final-dc8/BROWSER-HOLD-CHECKPOINT.json`; it is not the
  paired acceptance round and not a parity timeout.
- Coordinator required fresh base/candidate on3c1: main's Arachne open-wall
  scan changes six narrow-neck cases FAIL→PASS, already included in both
  members of the successful fresh seeded round above. No old paired evidence
  was substituted.
- After round completion, rebase to8264 web-only plus96577 docs-only.
  `git diff 3c1a79941..96577fff7 -- crates Cargo.toml Cargo.lock scripts tests
  .config` is empty. Candidate `crates` tree is identical before/after:
  `3c00dccd161a03d47cf678cfef4cffce81ff0bb3`. Combined production diff hash
  is identical. Carrying this paired evidence follows coordinator's
  web/docs-only ruling; no fresh Rust behavior changed after the round.

## Durable review artifacts and reproduction

Root `${TMP_ROOT}/ares-j1-mask/final-3c1/`:
`validation-audit.json`, `stream-invariance.json`, paired mode/variant
`head.txt`, `exit`, `nextest.log`, full artifacts/cache receipts,
`production.diff`, `rebased-production.diff`, directed process records,
`clippy.log`, `fmt.log`, `wasm.log` and gate exits.

Drivers: `run_all.sh`, `run_directed.py`, `audit.py`, `audit_streams.py`.
Their output directories are intentionally create-once: reproduction must
use new roots/caches, not overwrite or retry old attempts. Public test
command is `cargo nextest run -p ares-cli --test orca_parity` with the pinned
reference environment; full-mode helper `${TMP_ROOT}/ares-parity-control-set-v2.sh`
provides all current reference flags and artifact routing.

No push before independent exact-hash combined review and coordinator
approval. Any later Rust rebase requires a fresh matched round; a subsequent
explicitly approved docs/web-only rebase may carry the recorded evidence.
