# Arachne alternate extra wall: proposed source slice

Status: accepted.

## Upstream owner and source identity

Port OrcaSlicer v2.5.0-beta (`345efec64`) `src/libslic3r/PerimeterGenerator.cpp`,
`PerimeterGenerator::process_arachne`, specifically lines 2702–2877.
Source file SHA256:
`48f48a856c9cb86457299e6c692a94ebb9231dcb75d5cb5c314c87cf515ea042`.
The worktree's ignored OrcaSlicer link resolves to the local read-only upstream
source used to check these citations. This is a libslic3r rewrite slice.

- Lines 2703–2706: initialize the zero-indexed loop number with configured
  wall count plus this surface's extra perimeters; convert sparse density to
  `int`; increment once when alternate extra wall is enabled, layer ID is odd,
  spiral vase is inactive and the converted density is positive.
- Lines 2708–2716: bottom-layer and topmost single-wall overrides follow that
  increment. Bottom is relative to `object_config->raft_layers`, not always 0.
- Lines 2681–2682, with 619–622 and 628–631: `only_one_wall_top` needs top shell
  layers and `only_one_wall_first_layer` needs bottom shell layers. With zero
  shell layers both options are inert for every use below, so a fixture only
  reaches the overrides with a positive shell count.
- Lines 2735–2744: top-surface regeneration captures the resulting inner count
  before the primary pass is reduced to one wall; `loop_number + 1` feeds
  `Arachne::WallToolPaths`, which supplies walls and the inner infill contour.
  Line 2738 also reduces the topmost layer to one wall, so removing the override
  at 2715–2716 alone changes no output.
- Lines 2751–2877: existing top-surface regeneration constructs remaining
  inner walls or the no-single-wall replacement using those derived counts.
- `PrintConfig.hpp:1480` declares sparse density as `ConfigOptionPercent`;
  preserve the local integer conversion in `process_arachne`, not a float
  positivity shortcut. In particular, density 0.5 does not enable the increment.

## Rust destination and existing scaffold

Production destination is
`crates/ares-core/src/project_slice/perimeters/arachne/config.rs` and
`.../arachne/walls.rs`; pass existing record context through
`.../arachne/materialize.rs` only if necessary for the upstream spiral predicate.
Carry the per-record decision into the per-surface loop count, before existing
first/top single-wall overrides and top-surface regeneration.

The `alternate_extra_wall` rejection in `config::validate_record` is removed; do
not replace it with a different UnsupportedProjectFeature guard, success with
skipped geometry, or Classic fallback. Existing Arachne wall generation, variable-width conversion, ordering
and infill-boundary generation remain the consumers of the corrected count.

`project_slice/perimeters/context.rs:105–112` already records effective spiral
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

Fuzzy skin and overhang handling belong to their own source-cited specs
([fuzzy skin](arachne-fuzzy-skin-spec.md),
[extra overhang perimeters](2026-10-20-arachne-extra-overhang-perimeters.md)).
Also defer a positive `top_surface_expansion`: `top_fill_replaces_inner_walls`
(lines 637–640, 2842–2855) and `PrintObject.cpp:2112-2136` are not covered here,
and Native and Ares differ on the stepped fixture with that option set. Also defer
unrelated raft generation,
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
3. Odd topmost layer with `only_one_wall_top` and one top shell layer: override
   wins; a stepped top surface additionally witnesses the existing regenerated
   inner-wall count. With zero top shell layers both shapes keep every wall.
4. `only_one_wall_first_layer` with one bottom shell layer: bottom output remains
   one wall while later odd layers still exercise the extra wall. With zero bottom
   shell layers the first layer keeps every wall.
5. Configured zero walls with alternate on/off: preserve upstream odd-layer
   increment and the resulting infill boundary, rather than silently skipping.
6. A two-island input and an effective spiral-mode control: count independently
   for each island; preserve the current non-Arachne spiral dispatch.

The positive shell cases rewrite the committed fixtures' settings at run time, so
the fixtures and their hashes stay as they are. Expected wall counts come from
each input's settings and the source lines above, never from a stored table.

Failures to distinguish are rejection before emission, wrong layer parity,
float density instead of integer conversion, zero-wall short-circuit, adding
after single-wall overrides, overrides that ignore the shell gate, stale regenerated counts or infill contours,
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

This SPEC covers alternate extra wall only. Arachne fuzzy skin and overhang
handling each have their own source boundary and production change.

Public fixture RED and source-derived preflight corrections are in
[RED evidence](arachne-alternate-extra-wall-red-evidence.md). An existing unit
assertion that requires the removed rejection needs the separately proposed
[test retirement record](arachne-alternate-extra-wall-test-retirement.md),
committed as 76b8fbe43. [GREEN evidence and remaining
gates](arachne-alternate-extra-wall-green-evidence.md) bind source bytes and
unchanged off/inert controls; no full/workspace claim is made here. Both documents
describe v2.4.2 behavior, where the overrides apply without the shell gate of
lines 2681–2682.
