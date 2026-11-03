# Scarf default-speed cap — GREEN review evidence

Approved independent test-only RED:4098fde873f9fe0fd7eed475bf518300040d8c0e.
Rebased RED on main1a8c92f55:1d5aad000e403e82e6a8f6771d1647da52a2836d.
The approved spec and RED records are `gcode-scarf-default-speed-threshold.md`
and `gcode-scarf-default-speed-threshold-red-controls.md`.

## Exact implementation boundary

Only `crates/ares-core/src/project_slice/gcode_emit/motion/scarf.rs` changes:
remove its local mm-threshold speed early return and the `loop_length`
argument to `scarf_speed`. The length remains intact in `build` for slope
geometry. Role speed and configured absolute/percentage speed still resolve
through the existing match and `base.min(configured)`:
`OrcaSlicer/src/libslic3r/GCode.cpp:6515–6526`.

No change to upstream loop-only caller selection (`GCode.cpp:5808–5813`),
role eligibility, positive caller overrides, layers/ramps/cooling/flow,
precision, geometry, progress/time estimation, comparisons or test inputs.
The seven-line production diff has SHA256
9c526b536be38802917f8999eaa3f5098c5e3208f930129cd87f7ba09824a791,
identical before/after the zero-conflict rebase to1a8c92f55.

## Fresh paired evidence on1a8c92f55

All runs use isolated targets/caches and16 threads. The base worktree has
the approved ignored upstream symlink and verified pin8500fcdccaa10b5099ac20d252af3a7c560046f1.
No missing-resource/submodule fallback is used.

- Full seeded: base660/660, candidate664/664 PASS; both failure sets empty.
- Directed:13/13 PASS, new4 and all prior small-loop9; actual Native
  CPU0/seed0, runs1, fresh per-case caches, per-case shared300s deadlines,
  build completed outside the deadline, zero timeouts.13/13 Slice receipts
  bind exact input and reference outputs. Boundary now matches complete
  bytes and outer F3600/F7200; old positive small-loop ramp remains PASS.
- Fresh official subset `test(scarf) | test(small_loop)`: base10/10,
  candidate14/14 PASS;650 unrelated tests skipped by selector in each.
  This is deliberately NOT a full official run or a claim about unrelated
  raw failures.24/24 Slice receipts bind input/output streams.
- Full seeded stream invariance:518 paired identical ZIP-member payload
  inputs →518 identical whole Ares outputs after generated-by-only
  normalization; candidate has four new full-byte PASS streams. No changed
  pre-existing stream was hidden by weaker comparison or input changes.
- RED/GREEN: all five reviewed input payloads unchanged, all five complete
  Native outputs identical after generated-by-only normalization. The four
  new fixtures are exact byte-identical. The existing generated small-loop
  input differs only in ZIP container metadata; member payloads match.
- strict Clippy `--workspace --all-targets --all-features -- -D warnings`,
  rustfmt check and WASM check: exit0. Workspace nextest result is bound in
  the final REVIEW-REQUEST rather than anticipated here.

## Artifacts and reproduction

Retained round root: `${TMP_ROOT}/ares-scarf-threshold/green-1a8/`.

- `seeded-comparison.json`, base/candidate seeded logs, source/head captures,
  fresh artifacts/cache trees.
- `directed-seeded/`, `directed-audit.json`, per-case process/deadline records
  and speed-witness JSON.
- `official-comparison.json`, base/candidate official-audit JSON, logs and
  fresh artifact/cache trees.
- `same-payload-ares-output-audit.json` and `red-green-input-invariance.json`.
- `run_seeded.sh`, `run_directed.py`, `run_official.sh`, `run_gates.sh`,
  build logs, gate logs/exit files. Ordinary project tests use nextest;
  parity execution requires the configured Native/resources/artifact env.

No fixture tuning, ignored failing test, source instrumentation, epsilon,
numerical-domain extension, time-estimator repair or legacy fallback.
Exact-hash review is required before nonforce fast-forward push; later main
movement must be rebased and validated per coordinator instructions.
