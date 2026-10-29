# Chamber writer-used extruder reduction — SPEC

## Authority and upstream boundary

Coordinator accepted chamber discovery and authorized docs-first SPEC then test-only RED on new branch `worker1/chamber-used-extruder`, based on frozen exhaust `53890b7805a50aded6b83bb9ef4ec41a722acc0e`. GREEN requires separate approval. Exhaust and chamber will rebase together only in the explicitly authorized integration window. No production edits, FULL, merge or push in this stage.

This is a source-cited Rust rewrite slice of OrcaSlicer libslic3r at `8500fcdccaa10b5099ac20d252af3a7c560046f1`, not a new Ares pipeline:

| Source under OrcaSlicer/src | Included contract |
|---|---|
| libslic3r/GCode.cpp:3008–3013 | OR activation and independently maximize chamber temperature over exactly m_writer.extruders(). Temperature max is not conditional on that filament's activation. |
| libslic3r/GCode.cpp:3037 | Publish the same maximum as overall_chamber_temperature. |
| libslic3r/GCode.cpp:3130–3133 | Preserve startup placement, active/positive-temperature and existing custom-start temperature-command gates. |
| libslic3r/GCode.cpp:3498–3499 | Preserve shutdown activation/positive-temperature gate and ordering. |
| libslic3r/GCodeWriter.cpp:192–212 | Preserve M191/M141 bytes and auxiliary-fan behavior. |
| libslic3r/GCode.cpp:2760,2791,5646–5648; GCodeWriter.cpp:63–80 | Existing writer identity provenance; reuse the established ToolOrdering/ExtruderRegistry handoff. |

Rust destination is `crates/ares-core/src/project_slice/gcode_emit/machine/temperature.rs`: chamber_settings and its overall_chamber_temperature, append_chamber_startup and append_chamber_shutdown consumers. Existing object-derived chamber membership is an incomplete temporary compatibility shell around the named upstream reduction. Future authorized GREEN must pass the existing `ExtruderRegistry::ids()` through the already established orchestration/machine boundary to all three chamber consumers. Do not create another registry, infer membership from model-positive extrusion, retain an object-scan fallback or add an API shim. Preserve current first-value option indexing, serialization, gates and placement; do not modify unrelated nozzle/bed callers of used_filaments.

## GUI reachability

`Tab.cpp:2671–2677` exposes six feature-filament selectors. `:4576` exposes printer chamber capability; `:4357` controls visibility; `:3955–3975` exposes chamber target/minimum/activation. `:4595–4604` exposes machine-start G-code. `ConfigManipulation.cpp:167–184` warns on target temperature without clamping it; minimum0 is below both targets. These are ordinary user settings, not malformed/private-helper witnesses. No physical printer is involved.

## Immutable inputs

Copy exactly these accepted discovery archives into `tests/parity/chamber-used-extruder/`. All four have printer chamber capability enabled, minimum0 and the same custom-start overall_chamber_temperature comment. Geometry and all other ZIP members are unchanged. Input values below are user settings, not Native expected output constants.

| Archive | Target T0/T1 | Activation T0/T1 | Actual model tool | RED role | SHA256 |
|---|---|---|---|---|---|
| chamber-t1-equal.3mf | 30/30 | on/on | T1 | equal control | 29064c553748e6b59f52041359ed123964570304517215ac86053fc9edd384cc |
| chamber-t1-unused-t0-hot.3mf | 80/30 | on/on | T1 | true RED: unused hot T0 | 644dc85eb009c0d11926461d8ab4eb7e000238a7e1e91ff3ee4dc018752f5e0f |
| chamber-t1-unused-t0-enabled.3mf | 80/30 | on/off | T1 | true RED: unused active T0 | dfd4b243129b813032b2828cb258e4e21e2d995e5894f80cc61f26197ba4643d |
| chamber-t0-hot-used-control.3mf | 80/30 | on/on | T0 | used-hot control | cce1688b2dc25507c351f6c6d003816710fa45928ee55d0df170e70d53ae5af4 |

Accepted evidence: `/home/indexyz/ares-evidence/worker1/chamber-get-at-discovery-53890b780/DISCOVERY-REPORT.md`, SHA256 `48f5b899c4923bf196dfd77012d30ad98ef5c9d14e6857c2b780a8e4a3680fb6`. Preserve its failures and get_at evidence unchanged.

## Agreed public seam and failure inventory before tests

Reuse unchanged `native_stream_case::qualify/check`, `NativeCase`, shared activity parser and `golden::compare_ordered_bytes_generator_only`. Four success-required public tests exercise actual Native → public API → actual CLI; require successful outputs, complete ordered comparisons including config/time estimates and raw API==CLI. No Native output constants, command counts, G-code goldens, mocks, should-panic, expected-error pass, extra normalization, tolerances or filtering. The callback proves positive model extrusion only on the selected input tool and replay activity equality, without exact activity counts. Test registry holds only input labels, hashes and selected tool identity.

Before public legs, a Native-only preflight for all four inputs in both official/seeded modes must pass with fresh replay artifacts, globally before any public Ares leg. Failure cases:

- Unused T0 affects max, activation, placeholder or shutdown: two target tests genuinely fail complete comparison; successful API/CLI outputs must still be retained and their raw equality audited externally if the comparator panics first.
- Equal temperatures or genuinely used hot T0: two controls pass the same full comparison.
- Wrong SHA/settings, unexpected model tool, absent positive extrusion, Native nondeterminism, missing backend, Unsupported/API/CLI error, affinity breach or guard: STOP, not semantic RED.
- Preserve all infrastructure/preparation failures; no retry-until-PASS or weakened comparator.

## Verification and exit

CPU18, jobs1/test-threads1/parity-workers1; independent target directory preserves old binaries. Run cargo fmt, workspace/all-target checks and strict Clippy, wasm32 compile-only, then nextest list and exact directed filters (never cargo test or FULL). Official and seeded each require one preflight PASS, two true public RED failures and two public control passes. Record filter exclusions separately from selected skips/internal guards; do not claim total Native threadcount1. Commit docs first, then test-only RED, audit exact-tip evidence and send RR. Stop before GREEN.

Deferred: get_at GUI reachability (no slice, evidence retained); support/custom-tool/wipe-tower/multiple-used runtime witnesses, registry changes, nozzle/bed semantics and other fans, unrelated worker scopes, Windows/macOS/browser runtime parity. Existing wasm32 qualification is compile-only. Generic membership parity is not inferred from this narrow matrix.
