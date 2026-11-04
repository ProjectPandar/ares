# Arachne alternate extra wall: proposed source slice

Status: accepted.

## Upstream owner and source identity

Port OrcaSlicer v2.4.2 `src/libslic3r/PerimeterGenerator.cpp`,
`PerimeterGenerator::process_arachne`, specifically lines 2128–2175.
Source file SHA256:
`45e0a6c56df0acf4e66ae82ae7a4aa4792dc558fb263a12b1211438de47b7de6`.
The worktree's ignored OrcaSlicer link resolves to the local read-only upstream
source used to check these citations. This is a libslic3r rewrite slice.

- Lines 2131–2134: initialize the zero-indexed loop number with configured
  wall count plus this surface's extra perimeters; convert sparse density to
  `int`; increment once when alternate extra wall is enabled, layer ID is odd,
  spiral vase is inactive and the converted density is positive.
- Lines 2136–2144: bottom-layer and topmost single-wall overrides follow that
  increment. Bottom is relative to `object_config->raft_layers`, not always 0.
- Lines 2163–2175: top-surface regeneration captures the resulting inner count
  before the primary pass is reduced to one wall; `loop_number + 1` feeds
  `Arachne::WallToolPaths`, which supplies walls and the inner infill contour.
- Lines 2224–2244: existing top-surface regeneration constructs remaining
  inner walls or the no-single-wall replacement using those derived counts.
- `PrintConfig.hpp:1103` declares sparse density as `ConfigOptionPercent`;
  preserve the local integer conversion in `process_arachne`, not a float
  positivity shortcut. In particular, density 0.5 does not enable the increment.

## Rust destination and existing scaffold

Production destination is
`crates/ares-core/src/project_slice/perimeters/arachne/config.rs` and
`.../arachne/walls.rs`; pass existing record context through
`.../arachne/materialize.rs` only if necessary for the upstream spiral predicate.
Carry the per-record decision into the per-surface loop count, before existing
first/top single-wall overrides and top-surface regeneration.

Remove the `alternate_extra_wall` rejection in `config::validate_record` after
the behavior has an admitted public RED. Preserve the existing fuzzy rejection
for its separate slice; do not replace the removed rejection with a different
UnsupportedProjectFeature guard, success with skipped geometry, or Classic
fallback. Existing Arachne wall generation, variable-width conversion, ordering
and infill-boundary generation remain the consumers of the corrected count.

`project_slice/perimeters/context.rs:97–104` already records effective spiral
mode and dispatches effective spiral layers through the existing Classic owner.
Keep that source boundary intact; use the effective per-record predicate rather
than treating the global spiral option as active on every bottom layer.
`perimeters/wall_loops.rs::resolve_wall_loops` is a separate public adapter with
a floating density predicate and zero-wall early return. It is temporary
compatibility scaffolding outside this project slice, not a substitute for
`process_arachne`. Do not refactor or route project Arachne through it.

## Included and deferred behavior

Include the complete alternate-extra-wall predicate, layer parity, density
integer conversion, surface extra-count composition, override ordering, and
propagation into existing primary/regenerated wall counts and infill contour.
Wall count zero retains the upstream arithmetic: odd positive-density layers
may request one wall; do not add the public adapter's zero-count shortcut.
Exercise the existing effective spiral dispatch as an unchanged control.
Apply per record and per island; do not read a first region as representative.

Defer fuzzy skin (both config and traversal) and overhang detection/segmentation
to separate source-cited specs. Also defer unrelated raft generation,
new multi-material policies, Classic/public adapter repairs, Arachne beading,
outline or stitch changes. Existing accepted surface/top processing is reused,
not claimed as a newly complete upstream implementation.

## Public E2E and failure inventory

Author deterministic input-only 3MF fixtures and a focused
`ares-cli/tests/orca_parity/arachne_alternate_extra_wall.rs` module, split helpers
as needed below 400 LOC. Compare Native official and seeded with both validated
ares-core byte API and real CLI output using the existing strict complete
generator-stream comparator. Assert Native admission and effective option
values; save commands, input/binary hashes, exits, raw output hashes and complete
streams outside Git. Native artifacts are never committed.

Finite fixture pairs must witness:

1. Enabled/disabled alternate walls on a multi-layer wide prism with positive
   sparse infill: even layers unchanged and odd layers gain the inner wall.
2. Enabled option with zero density and with sub-integer density 0.5: inert
   whole-output controls; density 1 activates the odd-layer behavior.
3. Odd topmost layer with `only_one_wall_top`: override wins; a stepped top
   surface additionally witnesses the existing regenerated inner-wall count.
4. `only_one_wall_first_layer`: bottom output remains one wall while later
   odd layers still exercise the extra wall.
5. Configured zero walls with alternate on/off: preserve upstream odd-layer
   increment and the resulting infill boundary, rather than silently skipping.
6. A two-island input and an effective spiral-mode control: count independently
   for each island; preserve the current non-Arachne spiral dispatch.

Failures to distinguish are rejection before emission, wrong layer parity,
float density instead of integer conversion, zero-wall short-circuit, adding
after single-wall overrides, stale regenerated counts or infill contours,
global instead of effective spiral gating, and cross-island/region leakage.
Use public emitted wall/infill witnesses in addition to full equality; no
private-function tests or implementation-derived goldens. A Native refusal is
reported as admission evidence, never counted as an Ares parity PASS or skipped.

## Test-first gates and evidence

Commit fixtures/tests and genuine Native-backed RED separately, before
editing production. Preserve assertions and inputs through GREEN.
Keep existing Arachne open-wall, only-one-wall/top-surface and ordinary option
controls; record unchanged Ares raw bytes for disabled/inert controls.
Run official and seeded directed tests, each with a fresh cache and
artifact root.

GREEN requires full output equality for admitted finite cases in both modes,
directed both modes and strict clippy/fmt/WASM32.
Each changed file stays at most 400 lines.

## Slice sequence

This SPEC covers alternate extra wall only. Then propose
Arachne fuzzy skin, then overhang detection/segmentation, each with its own
source boundary, public RED commit and production change.

Public fixture RED and source-derived preflight corrections are in
[RED evidence](arachne-alternate-extra-wall-red-evidence.md). An existing unit
assertion that requires the removed rejection needs the separately proposed
[test retirement record](arachne-alternate-extra-wall-test-retirement.md),
committed as 76b8fbe43. [GREEN evidence and remaining
gates](arachne-alternate-extra-wall-green-evidence.md) bind source bytes and
unchanged off/inert controls; no full/workspace claim is made here.
