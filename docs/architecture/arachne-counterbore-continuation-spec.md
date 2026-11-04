# SPEC — Arachne non-None counterbore continuation on merged main

Status: proposed. Docs only, before RED refresh or production.
Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Binding decisions: [ARD](ard-arachne-counterbore-continuation.md).

## Remaining Native difference, prerequisite now resolved

The previous public matrix at `9af8025a1` established successful Native slicing
but eight exact Ares `UnsupportedProjectFeature("counterbore_hole_bridging")`
REDs: Bridges/Filled × stepped-hole, first-layer-hole, straight-hole, two-islands.
Those tests stay archived at `819f7e1a4` and were
not merged as permanently failing tests.

Its additional None baseline G2 mismatch was diagnosed to the wrong infill
resolution argument, not ArcFitter. The separate prerequisite is merged in this
main: `ArachneRecordConfig.scaled_resolution` reaches the normal infill boundary;
wall-input reduction remains distinct. The isolated public None test is now
registered in `runner/arachne_counterbore_none.rs`. Full official and
seeded each1217/1217 and self-hosted CI passed. Reuse that main-owned None control;
do not duplicate it, change its assertions or reimplement its correction.

On current main the two remaining production handoff omissions are unchanged:
1. `classic/preflight.rs::validate_record:105–113` rejects non-None Arachne holes.
2. `classic/counterbore.rs::process:35–36` skips all Arachne dispatch; common
   prelude therefore does not mutate its perimeter surface group or retain
   generated counterbore fill.
3. `arachne/materialize.rs::materialize_record:153` starts fill empty, replacing
   the classic slot; retained `counterbore_fill` must seed Arachne region fill.

## Upstream owner → exact Rust destination

All C++ paths below are under pinned `OrcaSlicer/src/libslic3r/`:

| Source slice | Included behavior / Rust handoff |
| --- | --- |
| `PrintConfig.hpp:401–403`; `PrintConfig.cpp:553–557,1547–1563` | None/Bridges/Filled enum mapping already implemented; no new option or alias. |
| `LayerRegion.cpp::make_perimeters:82–142` | Flow/slice inputs and dispatch. Preserve normal Arachne selection, never classic-wall fallback. |
| `PerimeterGenerator.cpp::process_arachne:2093–2128`, specifically `:2123` | Calls shared `process_no_bridge` on cloned surfaces **before** simplification and per-island WallToolPaths. Enable the existing shared prelude helper for Arachne, not a new stage. |
| `PerimeterGenerator.cpp::process_no_bridge:1728–1929` | Reuse detector, unsupported span/angle, Filled `:1777–1855`, Bridges `:1856–1891`, fill/remainder mutation `:1899–1929`. `classic/counterbore.rs` is the current temporary storage shell for this shared upstream concept, not a classic-only algorithm boundary. |
| `PerimeterGenerator.cpp::process_arachne:2148–2176`; `Arachne/WallToolPaths.cpp::{generate,getToolPaths,getInnerContour}:482,721,779` | Existing backend consumes the mutated surface group through `perimeter_surfaces`; no wall/beading rewrite. |
| `PerimeterGenerator.cpp::process_arachne:2475–2533` | Append normal island fill **after** shared counterbore fill. Seed existing `PreparedLayerRegionPerimeterRecord` in `arachne/materialize.rs`; preserve separate no-overlap output and merged resolution fix. |
| `Layer.cpp::make_perimeters:248–279`; `PrintObject.cpp::detect_surfaces_type:1579–1583,1698–1704,1912`; `LayerRegion.cpp::slices_to_fill_surfaces_clipped:63–80` | Reuse current recovery/redistribution and Filled comparison versus original/internal domain clipping; no edits to these owners. |
| `Feature/FuzzySkin/FuzzySkin.cpp::group_region_by_fuzzify:415–473` | Masks use original compatible-region slices before wall mutation. Existing `types.rs::compatible_fuzzy_regions` uses original member surfaces; counterbore updates merged perimeter view. Keep domains and random draws unchanged. |

Margin authority remains pinned constant `BRIDGE_INFILL_MARGIN=1` in
`libslic3r.h:89`, `process_no_bridge:1736`; anchor minimum at`:1774`.
Do not invent a user option to satisfy a fixture.

## Proposed production allowlist and minimal implementation

Only these files under `crates/ares-core/src/project_slice/perimeters/`:
- `classic/preflight.rs`: retire this specific compatibility rejection only
  after public RED and the real two geometry/fill handoffs are implemented.
- `classic/counterbore.rs`: allow the existing shared `process_no_bridge` port
  for Arachne, retaining None/lower/geometry no-op gates and both mode bodies.
- `arachne/materialize.rs`: seed counterbore fill from the shared prelude before
  normal island fill. Keep WallToolPaths, typing and `config.scaled_resolution`.

No extra state/API/module relocation, pipeline design, fallback, region[0], new
Unsupported or core I/O. Changed modules remain below400LOC. Any insufficient
scope requires an exact scope amendment before another production file is touched.

Read-only: `classic/prelude.rs`, groups/recovery/types, Arachne config/walls/fuzzy/
ordering, Fill/grid, geometry/ArcFitter, classification/interface_shells and
centering/placement. Browser WASM/Windows/macOS/Linux share core logic.

Included: Bridges/Filled common geometry, mutation/metadata/order, correct
Arachne wall generation, counterbore-fill survival, first-layer/no-unsupported
no-ops, None and classic regressions. Deferred: new nested-island and compatible
multiregion public reachability, support/spiral/raft/material scope expansion,
new wall/bridge/fuzzy/Fill algorithms. Existing recovery port is not proof that
all recovery branches are publicly reachable; two independent islands do not
close the nested-island gap.

## Public RED refresh plan

Reuse frozen STLs and parent-chain provenance in
`tests/parity/counterbore-partiallybridge/`: WEMAKE3D PhoenixProV1 0.4mm stock
machine/process and Generic PLA. Explicitly override wall generator to Arachne
and counterbore mode on both Ares public `slice_with_profiles` and independent
Native flattened settings. Do not claim stock process defaults to Arachne.
Placement::None, deterministic metadata/seed, enabled arcs and original pattern
remain unchanged; no centering/interface-shell/grid fixture adjustment.

Extract **only the ten non-None tests** from the archived RED matrix (two modes
× five original geometries); omit its None test because main owns the identical
control. Preserve eventual PASS assertions and original helper byte comparison.
No permanently failing branch is submitted for merge: these frozen REDs stay on
this feature branch until their GREEN is implemented. Keep tests/fixture commit
separate from production. Any assertion evolution needs exact rationale/diff
rationale rather than silent threshold or normalization changes.

Expected new-main directed verdict:
- Eight non-None hole cases: exact counterbore guard RED after Native success.
- Two non-None solid controls: complete-output PASS.
- Main-owned Arachne None: complete-output PASS, new raw SHA control baseline.
- Eleven current classic partial/None/Filled controls: PASS and raw Ares SHA
  baselines recorded before GREEN; all existing related Arachne controls reused.

Both primary modes must retain independent Native-vs-None witnesses normalizing
**both as Orca**; movement difference supplements full streams and excludes
config-footer-only activation. It is not an Ares acceptance filter or complete
branch-coverage claim. Preserve all inputs/settings hashes, producer/wrapper
identity, Native exit/log/timing, Native/Ares G-code or exact error, first byte
difference, metadata/context and cleanup. No retry-until-green or historical
Variant-A assumption; report deterministic mismatches.

Run with fresh Native cache/artifacts and mode-sequential execution; no
workspace or full test run during RED refresh.

## Ten old capability/lifecycle reject assertions

The pre-existing ten Arachne Filled rejection assertions still live on main.
This SPEC does **not** weaken/delete/migrate them. After
public RED, record each exact test diff and changed contract
in a separate test commit before GREEN. Preserve unrelated disposal,
zero-invocation and commit assertions; do not replace them with invented guards.

## Exit sequence

Fresh public RED → separately recorded exact lifecycle
assertion diffs → bounded source-cited GREEN → public official/seeded full-byte
parity, raw control invariance, strict/fmt/WASM.
Merge requires actual full/CI evidence. A passed
resolution prerequisite does not imply non-None counterbore is already supported.
