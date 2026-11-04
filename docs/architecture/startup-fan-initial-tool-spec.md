# Startup fan-off gate: initial filament indexing — SPEC

## Actual discrepancy established

An actual GUI-stock-edit whole-output difference has been observed; this is NOT
a null-driven coverage proposal.

## Upstream owner and Rust destination

This is a source-cited Rust rewrite slice of OrcaSlicer libslic3r, source pin
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, not a new Ares pipeline feature.
Library paths below are under `OrcaSlicer/src/libslic3r/`; the GUI row is under
`OrcaSlicer/src/`.

| Upstream owner | Included behavior |
|---|---|
| `GCode.cpp:2822–2826`, `GCode::_do_export` (starts2461) | Auxiliary-fan capability AND `close_fan_the_first_x_layers.get_at(initial_extruder_id)` gates the ordered main/additional fan-off pair. Read the INITIAL filament's value, not logical filament0. |
| `GCode.cpp:2715–2801` | Existing initial-extruder identity is the input to that consumer; do not rewrite tool selection, physical mapping or tower policy. |
| `Config.hpp:624–627` | Existing indexed option access; this witness has full two-entry vectors, not an out-of-range fallback probe. |
| `GCodeWriter.cpp:1095–1155` | Existing `set_fan(0)` and `set_additional_fan(0)` bytes, ordering, comments and trailing whitespace for this admitted stock flavor. No new writer behavior. |
| `slic3r/GUI/Tab.cpp:4575,4079,4301,2671–2677` | Printer auxiliary accessory; per-filament first no-cooling layers; additional-fan visibility; ordinary feature filament selectors. |

Rust owner is
`crates/ares-core/src/project_slice/gcode_emit/machine.rs::append_limits`
(currently lines40–84). Its unconditional `.first()` for this option is the
temporary compatibility shell being replaced by the cited initial-filament-indexed
consumer. The future narrow implementation must consume the already-resolved
`machine/first_filaments.rs::InitialFilaments::initial()` identity at the existing
`orchestration.rs:218` startup call. It must not use the writer's default active ID:
no extruder has yet been selected at this stage. No new identity resolution,
registry, template placeholder, fallback scan or public API layer is needed.

Keep existing startup position (after machine envelope and before custom startup),
capability gate and serialization. Reuse established option-access semantics; do
not globally change get_at or other fan consumers. A later small argument handoff
at the shared orchestration call does not take ownership of ToolOrdering/priming.

## Frozen stock witnesses and GUI reachability

Source archive: accepted stock `chamber-t1-equal.3mf`, SHA256
`29064c553748e6b59f52041359ed123964570304517215ac86053fc9edd384cc`.
Original mesh, placement, presets, two-filament cardinality and every non-settings
ZIP member are unchanged. All six model-feature selectors still select T1.
Chamber capability/activation/target and additional-fan target are zero, decoupling
other startup fan writers. Support, tower, beam and painted fuzzy are off.
Machine-start is unchanged and contains no M141/M191; this does not exercise the
custom-temperature scanner. GUI reachability is source-constructive through the
listed settings, not a claim of interactive GUI saving. No physical printer runs.

| Input | Auxiliary | Close first layers T0/T1 | SHA256 |
|---|---|---|---|
| `aux-on-both-open.3mf` |on|0/0|`75fb14cec1cdab283cc62db7cfe0141093950930a03826f64bc8881b46c7069f`|
| `aux-on-initial-t1-closed.3mf` |on|0/1|`8305d6fcd9f2f9f04199cc17070a4b167b419ab9b1e8d5ffef213b23d44d11d6`|
| `aux-off-initial-t1-closed.3mf` |off|0/1|`c8b08779a0204b8cf2809c04d34c2bab6f171fcacb0deb6ad6f7b3ac3622164f`|

Inputs/settings/deltas/ZIP-member hashes were frozen BEFORE first Native execution;
no result-driven search or replacement witness was used. The two orthogonal controls distinguish the initial-slot read
from unconditional fan emission or merely having a nonzero close-layer value.

## Observed successful-output difference

Twelve fresh Native slices (three inputs × two modes × reference/replay) completed
globally before public execution. All six replay pairs are stable under the
unchanged repository generator-only comparator. Settings readback passes; every
Native/public output has positive MODEL XY extrusion only on T1, not merely a
configured, skirt-only or prime-only tool. All six actual validated API calls and
six actual CLI calls succeed, with all six raw API==CLI equalities.

| Input | Official whole Native/API + Native/CLI | Seeded whole Native/API + Native/CLI |
|---|---|---|
|aux-on-both-open|EQUAL / EQUAL|EQUAL / EQUAL|
|aux-on-initial-t1-closed|DIFFERENT / DIFFERENT|DIFFERENT / DIFFERENT|
|aux-off-initial-t1-closed|EQUAL / EQUAL|EQUAL / EQUAL|

The target's first difference is generator-normalized byte585, line22: Native
emits `M106 S0\nM106 P2 S0\n` immediately after machine-envelope limits, whereas
Ares proceeds directly to `;TYPE:Custom`. Normalized lengths are166644/166625.
In all four target comparisons, diagnostic deletion of ONLY this one19-byte
Native pair makes ALL remaining ordered bytes equal. That localization is NOT a
new allowance: the whole-stream gates remain DIFFERENT, raw outputs unchanged.
Metadata and estimates participate. No error/guard is counted as a semantic RED.

No compilation or runtime retry, production instrumentation or source edit occurred
in this probe.

## Proposed test-only RED seam and failure inventory

Copy exactly these three archives, hashes and settings
manifest; reuse existing `NativeCase`, `native_stream_case::qualify/check`, shared
model activity and repository whole-stream comparator. Plain registry entries only;
no duplicate runner/scaffolding, hardcoded Native command counts/expected bytes,
mocks, private-helper tests, expected-error PASS or tolerated differences.

Before public tests, require fresh Native-only preflight/replay for ALL inputs in
BOTH modes globally. The activity callback proves model-only T1 and stable replay
without exact command counts; input/readback checks establish the differing option
slots and auxiliary flag. Use actual validated API and CLI, retaining successful
raw streams and raw equality even when the whole comparator fails first.

- Target succeeds but fails whole comparison in both modes: genuine semantic RED;
  both controls must pass the same whole comparator.
- Wrong hash/settings, absent or wrong model tool, unstable Native, unsupported or
  failed API/CLI, missing backend or skip/guard: STOP, not RED.
- Wrong initial identity or writer-default0 substitution: target remains RED;
  do not "fix" it by changing tool-ordering or weakening activity requirements.
- Changed comments/flavor formatting, output order, metadata or estimates: whole
  comparison still owns all bytes; no diagnostic deletion in the test oracle.

Use the unchanged canonical parity control set for directed qualification,
strict fmt/check/workspace-all-target Clippy, WASM compile-only, nextest list and
the exact selected preflight/public tests. Preserve all errors; no chance retry.

## Included/deferred boundaries

Sign/layer-tag/FanMover work is adjacent, not this consumer. Skirt/brim,
tower/ToolOrdering, template/bed/processor and temperature scanner/labels remain
out of scope.

Included: this initial-filament-indexed startup gate in the full-vector stock
domain and its existing emitted pair. Deferred: tool-selection/physical mapping,
tower priming, full fan writer flavor/PWM policy, layer cooling, FanMover/speedup,
additional-fan scheduling, chamber/exhaust, generic get_at fallback, negative/raw
out-of-domain options, templates/scanner, WASM/Windows/macOS/browser runtime parity.
Fuzzy integration remains a separate slice.
