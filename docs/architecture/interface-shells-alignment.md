# interface_shells Alignment Slice — ARD/SPEC

Scope: remove the last `interface_shells` rejection and align with upstream
OrcaSlicer 2.4.2 semantics for multi-region (multi-part) layers.

## Current Ares state and rejection point

- Rejection: `crates/ares-core/src/project_slice/prepare_infill/surface_type_detection/preflight.rs:9-17`
  — `validate()` returns `UnsupportedProjectFeature("interface_shells")` when any
  object has `interface_shells=true` and any layer has `model_parts.len() > 1`
  (multi-material / multi-region layers). Single-part layers already pass through.
- Already aligned (kept, no change):
  - `perimeters/classic/top_split.rs:80` and `perimeters/arachne/top_surface.rs:26-77`
    consume `interface_shells` for top-surface perimeter inputs
    (`upper_same_region_surfaces`, `perimeters/types.rs:246-252`).
  - `prepare_infill/vertical_shells/stage.rs:37`
    `let shared = multiple && !options.interface_shells.0;` already selects the
    per-region vertical-shell path.
- Gap: `surface_type_detection/stage.rs:classify_slices`
  (`crates/ares-core/src/project_slice/prepare_infill/surface_type_detection/stage.rs:196-241`)
  classifies top/bottom against **whole-layer** upper/lower slices
  (`perimeters/types.rs:229-244`), matching only the upstream
  `!interface_shells` path. The per-region variant does not exist.

## Upstream owner (OrcaSlicer 2.4.2, `src/libslic3r/PrintObject.cpp`)

1. `PrintObject::detect_surfaces_type()` — `PrintObject.cpp:1530-1723`:
   - `:1530` `interface_shells = !spiral_mode && m_config.interface_shells.value`
   - `:1543` per-region `surfaces_new` cache (`:1721` move-back) for thread safety
   - `:1589` top surfaces: with interface_shells, diff against the **same region's**
     upper slices (`upper_layer->m_regions[region_id]->slices.surfaces`) instead of
     whole-layer `upper_layer->lslices`
   - `:1606-1624` bottom surfaces: the `#if 0` block at `:1603-1610` is dead
     code upstream and is NOT ported. Only the `#else` path is ported: the
     default bridge-diff stays whole-layer (`stBottomBridge`); with
     interface_shells an **extra** pass adds `stBottom` for parts of this
     region lying on material of *other* regions:
     `opening(diff(intersection(current_region, lower_layer->lslices),
     lower_layer->m_regions[region_id]->slices.surfaces), offset)`
   - `:1691-1698` internal surfaces from `surfaces_prev` (region slices when
     interface_shells)
2. `PrintObject::discover_vertical_shells()` — `PrintObject.cpp:2018-2135`:
   - `:2024` `top_bottom_surfaces_all_regions = num_printing_regions() > 1 &&
     !interface_shells` — with interface_shells the vertical-shell walk is
     per-region (`:2121` branch comment). Ares `vertical_shells/stage.rs:36-37`
     already implements this split; this slice only **verifies** parity, no planned
     change.
3. `PrintObject::clip_fill_surfaces()` — `PrintObject.cpp:3869-3948`: clips fill
   surfaces per layer/region against layer slices; no `interface_shells`
   branching (whole-layer `lslices` clip applies equally). Ares
   `surface_type_detection/stage.rs:clipped_fill` mirrors this — no change.
4. Config invalidation lists `opt_key == "interface_shells"` at `PrintObject.cpp:1288`
   (already wired through Ares options registry).

## Rust destination boundary

All inside `crates/ares-core/src/project_slice`:

1. `prepare_infill/surface_type_detection/preflight.rs` — delete the multi-part
   rejection (`:9-17`); `bottom_kind` stays.
2. `perimeters/types.rs` — add `lower_same_region_surfaces(record)` mirroring
   `upper_same_region_surfaces` (`:246-252`); the post-perimeter record set must
   carry the same-region lower index exactly as it carries `upper_same_region`.
3. `prepare_infill/surface_type_detection/stage.rs` — extend
   `classify_slices` (or a sibling `classify_slices_interface_shells`) to take,
   per record: same-region upper surfaces (for the `:1589` top diff), whole-layer
   lower (for `:1606` bridge diff) and same-region lower (for the `:1622`
   extra-`stBottom` pass). Output kinds reuse `RegionSurfaceKind::{Top, Bottom,
   BottomBridge, Internal}`. Spiral mode keeps disabling interface shells
   (upstream `:1530`).
4. No change: vertical_shells stage (already keyed), clipped_fill, perimeter
   top_split/arachne paths, options registry.

## Included behavior

- Multi-region layers with `interface_shells=true` classify per
  upstream `:1589/:1606-1624/:1691-1698`.
- The extra `stBottom` (non-bridging bottom on other regions' material) surfaces
  flow into the existing bottom-surface consumers (fill/infill roles) unchanged.
- Spiral-mode interaction (`interface_shells` ignored under spiral) preserved.

## Deferred behavior

- `discover_vertical_shells` per-region deep verification beyond the existing
  `shared` split (parity rides the E2E; a dedicated audit is a follow-up if diffs
  appear).
- `clip_fill_surfaces` is interface-independent upstream; no work.
- Support-material interactions beyond existing bottom_kind logic (upstream
  `bottom_is_fully_supported` already ported in `preflight::bottom_kind`).
- Any UI/viewer exposure.

## Public Native-backed RED plan (TDD)

- New public E2E in `crates/ares-cli/tests/orca_parity/` (option_coverage-style
  group `interface_shells_alignment`): a multi-region fixture (one object, two
  extruders/parts overlapping in Z so some layers carry both) sliced with
  `interface_shells=true`.
- RED (before this slice): the run must produce the *documented failure shape* —
  `UnsupportedProjectFeature("interface_shells")` — asserted by the test as the
  pre-change expectation marker (test asserts full parity, so against main it is
  red exactly at that error).
- GREEN: full pipeline byte-parity vs the determinized official Native oracle
  (setarch wrapper, 47d636ea5) for official, and the seeded oracle for the
  seeded leg.
- Directed legs use a fresh cache/artifact per run.

## Gates

- strict clippy / fmt / wasm32 (ares-core stays platform-neutral; no I/O added).
- Files stay ≤ 400 LOC (`stage.rs` is 274; the new classification sibling keeps
  within the limit, splitting into its own file if needed).
- No `#[ignore]`, no allow-lints, no `dbg!`/`println!`, no weakened assertions,
  no UnsupportedProjectFeature guard left for the admitted input class.
