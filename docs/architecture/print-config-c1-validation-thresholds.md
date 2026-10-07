# Spec: C1 PrintConfig validation thresholds — independent rejection/byte RED

## Source-owned boundary

Parent record integrated as `e5fe2bba89bd55579afbcee47a858ea627cbd878`
(approved record-only a366909bc, rebased onto4140e7e7a).
Upstream OrcaSlicer: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Port owner: `libslic3r/PrintConfig.cpp::validate(const FullPrintConfig&, bool)`,
10340–10457, plus `OrcaSlicer.cpp:3647–3653` diagnostic transport. This is not
`Print.cpp::Print::validate` (-51), G-code path validation (-102), or an
Ares-owned pipeline feature. Include only the ten C1 keys in the parent record.

Rust boundary: existing `ares-core/src/options/validation/{basic,flow,
dimensions,aggregate}.rs`, consumed by public project validation/validated
slicing. RED destination: a dedicated module under
`ares-cli/tests/orca_parity/print_validation/`, not private validator unit tests.
Existing generic metadata/sweep omissions remain a compatibility shell; this
slice does not automatically change650-domain summary status.

First deliverable is this spec, then independently committed RED. No production
changes in either. Later GREEN requires review of observed RED and a separate
bounded source-alignment change. Do not alter unrelated validators, normalize
bad requested values into valid defaults, repair an input after observing a
failure, emulate UB, relax the comparator or add fallback behavior.

## Predeclared sample matrix (22 cases)

Values below are JSON override strings; brackets are single-filament arrays.
Each case changes only its named key from one immutable, valid control project.
All ten definitions have source min0. Most runtime guards reject<=0; filament
diameter instead rejects<1, so its schema minimum and runtime boundary differ.

“Just across” means the explicitly fixed **0.001 delta**, not nextafter/ULP or
full-neighborhood coverage. For layer height this is1000 coordinate quanta
(SCALING_FACTOR0.000001), a positive practical sample rather than a huge
million-layer experiment. Do not substitute another delta after failure.

| Key (definition line) | Runtime guard | Samples → predicted Native result |
| --- | --- | --- |
| layer_height (771) |<=0, plus existing quantization check,10340–10347|"0"→reject; "0.001"→accept|
| initial_layer_print_height (3384) |<=0,10350–10352|"0"→reject; "0.001"→accept|
| bridge_flow (1327) |<=0,10422–10425|"0"→reject; "0.001"→accept|
| internal_bridge_flow (1354) |<=0,10428–10431|"0"→reject; "0.001"→accept|
| extruder_clearance_radius (2234) |<=0,10434–10437|"0"→reject; "0.001"→accept|
| extruder_clearance_height_to_rod (2215) |<=0,10438–10441|"0"→reject; "0.001"→accept|
| extruder_clearance_height_to_lid (2225) |<=0,10442–10445|"0"→reject; "0.001"→accept|
| nozzle_height (2242) |<=0,10446–10447|"0"→reject; "0.001"→accept|
| filament_flow_ratio (2315) |any element<=0,10450–10454|["0"]→reject; ["0.001"]→accept|
| filament_diameter (2606) |any element<1,10355–10359|["0"]→reject; ["0.999"]→reject; ["1"]→accept; ["1.001"]→accept|

Thus **11 predicted rejections /11 predicted acceptances**, plus one unchanged
control comparison. Predictions are source-derived expectations, not observed
Native outcomes. If Native differs, preserve the original observation and mark
the case unexpected; do not relabel it PASS, retry or silently adjust fixtures.
The full accepted stream still must match even when Native rejects an
unexpected later stage: that later failure is a blocker, not numerical parity.

## Exact rejection contract

C++ scalar diagnostics use `std::to_string` (six fractional digits), while
filament vectors use the option serializer. Expected payloads:

- Eight scalar-zero keys: `<key>: invalid value 0.000000`.
- `filament_flow_ratio=["0"]`: `filament_flow_ratio: invalid value 0`.
- `filament_diameter=["0"]`: `filament_diameter: invalid value 0`.
- `filament_diameter=["0.999"]`: `filament_diameter: invalid value 0.999`.

Native must report exactly the one expected key/message, not merely contain a
substring. CLI stderr framing is exactly:
`Param values in 3mf/config error: \n<key>: <message>\n`.
Expected exit is `CLI_INVALID_VALUES_IN_3MF=-18` (`Utils.hpp`),238 on Unix.
No output G-code is allowed. Keep stdout/stderr/status and any partial output.
Extra keys, wrong text, another stage, timeout or missing diagnostics are not
matching rejections.

Ares is invoked **independently with the same original project and override**,
not inferred from Native or from a future exported file that rejection prevents.
Use public `validate_project_plate` and
`slice_project_plate_validated`; both must reject with `SliceError::InvalidInput`
whose payload is exactly `<key>: <message>`. Keep successful Ares bytes if it
unexpectedly accepts. Record both outcomes before asserting so a Native mismatch
does not erase the independent Ares result. Ares display wrapper text is not
substituted for the error payload. Do not use the historical rejection adapter
as proof: it recognizes O1/-51, whereas these errors are PrintConfig/-18.

## Input and accepted-output contract

Use the existing stock-selection/export shell for **Creality Ender-3 S1 Plus
0.4 nozzle**, its normal defaults and the immutable repository
`tests/parity/cube08.stl` (**0.8×0.8×0.8mm**, centered cube). This small existing
model bounds0.001mm-layer cost without rescaling or repairing a fixture.
Record source/profile hashes, exact exported project bytes and SHA256.
The neighboring0.2mm-nozzle O4 fixture is intentionally invalid and must not
be used or repaired for this slice. Baseline must be accepted and completely
equal; if not, stop rather than tune it.

All directed runs use those same immutable bytes; one override only, expressed
identically in Native CLI and Ares ConfigOverrides. Array cardinality stays one.
Capture actual applied/exported values as available, without requiring rejected
Native export to exist. No swallowed override, default substitution or implicit
geometry mutation may count as accepted coverage.

For accepted runs retain Native G-code, Ares G-code and complete strict-byte
comparison, excluding **only generated-by identity** using the existing shared
comparator. Keep M73/footer/times/statistics; no standalone timing or semantic
comparison can replace this gate. No active bridge-path or other full consumer
coverage is implied by passing this validation-focused small-cube fixture.

## Execution/evidence gates

- Browser START blocks new Native/build work; wait for END. The current END
  releases the temporary window but future START notices remain authoritative.
  Nix builds, if needed, use `--cores 8`.
- Explicit seeded Native binary and resources identities; fresh per-case cache,
  `ARES_ORCA_RUNS=1`, CPU-pinned Ares and Native with explicit ORCA_REPLAY_CPU.
- Whole-case300s deadline covers preparation through comparison; terminate
  owned groups/descendants and preserve partial evidence. TIMEOUT overrides any
  earlier PASS marker. No retries-until-success.
- Manifest binds original input hash, override, upstream/source/read-base hash,
  Native binary/resources identities, command/output hashes, deadline result,
  Native diagnostics and both Ares public outcomes. Bind Slice receipts where
  present; raw rejection commands are command-bound, not invented receipts.
- Run each independently named case via `cargo nextest run`; keep RED exit,
  exact failing names and controls. Do not claim an unexecuted test is RED.
- After Rust test edits: rustfmt and Clippy; retain initial failures/corrections.
  Final acceptance later needs matching-base failure-name comparison and existing
  workspace/WASM/browser gates as authorized, not blanket650-domain GREEN.

## Failure inventory and exits

Tests must expose: zero accepted/clamped by Ares; scalar formatting mismatch;
bridge/internal-bridge guards accidentally coupled or an extra error key;
filament diameter using schema0 instead of runtime1; wrong vector shape;
Native CLI rejection mistaken for Print::validate; public validation and
validated slice disagreeing; accepted output divergence; lost/partial input or
producer provenance; and a deadline mistaken for PASS.

RED exit:22 independently identified cases plus the control exist, artifacts
preserve both producers before assertions, source expectations remain fixed,
observed failure names are recorded, and production diff is empty. No automatic
PASS promotion or claim of complete legal-range coverage. A later reviewed
GREEN should make the same cases pass without changing inputs, expectations,
messages, diagnostics, deadlines or comparators.
