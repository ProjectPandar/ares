# Per-instance skirt/brim group assembly and emission SPEC

Status: **SPEC REVIEW REQUEST — docs-only; implementation awaits approval**.
All citations verified against the beta authority
`345efec64c7b95a05ce5cf96df0dde66a158835c` (OrcaSlicer 2.5.0-beta),
paths under `src/libslic3r`.
Base: branch `processor/b19-cross-owner-brim-subtraction` @ `f4534628e`
(integrated main `6bba2bdda`). This is the disposition of the 10 honest
divergences left standing after the frozen-count premises were removed
from the witness tests; it is a source-cited rewrite slice of
`Print::_make_skirt` grouping and `GCode` group emission, not an
Ares-owned pipeline feature.

## Motivation: the 10 standing divergences

All ten fail the whole-output comparison (byte equality after generator-line
normalization) with the Native oracle emitting **per-instance** skirt groups /
brim blocks where ares emits an owner-merged or single-hull variant.
Evidence: the brim/skirt family after-run log and per-test artifacts
(`comparison.json`, `launch.json`) under `ARES_PARITY_ARTIFACT_ROOT`.

| Test (cases) | Native vs ares first difference |
|---|---|
| `per_object_skirt_copies` | byte 1089 L39 `M73 P3 R9` vs `P4 R9`; 183341 vs 179832 B — per-instance skirt groups missing |
| `per_object_skirt_min_length_copies_0` | same shape as above (identical fixture family) |
| `per_object_skirt_min_length_copies_20` | byte 694 L25 `M73 P0 R10` vs `R9`; 199420 vs 181598 B |
| `per_object_skirt_combine_multi_instance_far` | byte 2092 L83 Native continues skirt motion vs `M73 P6 R10`; 288769 vs 284314 B |
| `byobject_owner_brim_combined_target` | byte 5341 L177 `M73 P8 R12` vs `;WIPE_START`; 285845 vs 285740 B |
| `byobject_owner_brim_no_skirt_target` | byte 1612 L67 first brim target `X42.008 Y98.284` vs `X114.334 Y100.314` |
| `byobject_perobject_multi_copy_brim_target` | byte 1612 L67 first skirt point `X40.221 Y100.569` vs `X51.855 Y98.434`; 294485 vs 289967 B |
| `byobject_perobject_multi_copy_brim_combined_control` | byte 5341 L177 `M73 P8 R12` vs `;WIPE_START`; 285845 vs 285740 B |
| `byobject_perobject_multi_copy_brim_width0_control` | byte 1612 L67 first skirt point `X40.221 Y100.569` vs `X51.855 Y98.434`; 188031 vs 186279 B |
| `global_combined_brim_target` | byte 2048 L82 `M73 P10 R7` vs skirt motion; 198249 vs 197405 B — the suspended combined-brim contact gap |

`byobject_perobject_skirt_target` (Option::unwrap panic) is NOT counted
here; it is expected to share the by-object emission boundary but keeps its
own triage first.

## Owning upstream boundary (beta 345efec64)

| Source | Behavior being ported |
|---|---|
| `Print.cpp:4025-4078` `_make_skirt` preamble, `ObjectSkirtHull` | Per-object local hull from object contours + support points up to `skirt_height_z`. Already ported (local-hull part of ares `occupied::items`); constrains the rewrite, unchanged. |
| `Print.cpp:4110-4161` `append_skirt_loops_for_hull` | Per-hull loop generation, `min_skirt_length` extension, `m_skirt_convex_hull` collection. Already ported; reused per group. |
| **`Print.cpp:4169-4209` group items** | **One `SkirtBrimGroupItem` per instance**: object hull translated by the instance shift (:4188-4193) plus that instance's brim-area contours from `m_objectBrimAreasByInstance` keyed `ObjectInstanceID` (:4195-4202); wipe tower is an obstacle-only item (`emits_skirt=false`, :4205-4208). |
| `Print.cpp:4227-4229` | `stCombined` unites all items into one group before any merge pass. |
| `Print.cpp:4231-4247` `build_skirt_brim_groups` | Union-find roots → per-group occupied points, instance list, `emits_skirt`. |
| `Print.cpp:4249-4276` | `stPerObject` fixed-point merge: expand emitting groups by `grouping_offset = skirt_distance + loops*spacing`, merge intersecting groups, repeat until stable; obstacles never expand. |
| **`Print.cpp:4297-4389` `make_brims_for_skirt_brim_group`** | Per-group brim assembly from `m_brimMapByInstance`: `brim_instances` = instances with non-empty per-instance brim; `combine_brims && size()>1` (:4326) → contact union-find with `offset_ex(area, 2*scaled_spacing, jtRound, SCALED_RESOLUTION)` against the other instance's area (:4352-4363), nozzle-compatibility gate, per-cluster area union + cleanup + bed-exclusion clipping; else per-instance brims (:4327-4336). Absorbs the previously suspended combined-brim contact gap. |
| `Print.cpp:4408-4434` group assembly | Per emitting group: skirt loops from the group hull, `m_skirt_brim_groups.push_back({skirt, instances, brims})` (:4432), `m_has_shared_per_object_skirt` (:4418). |
| `GCode.cpp:5956-5967` `find_skirt_brim_group_idx` | `ObjectInstanceID` → owning group lookup (linear scan). |
| `GCode.cpp:5969-5989` `generate_object_skirt_group` | `stPerObject` skirt emission at an object chunk, deduped per group via `m_skirt_group_done`. |
| **`GCode.cpp:5991-6075` `generate_object_brim`** | Per-instance brim emission: the group's `brims` entries whose `instances` contain the `ObjectInstanceID` (:6065-6071); `m_objsWithBrim` dedup; owner-filament wait. |
| `GCode.cpp:7422-7447, 7588-7591, 8130` wiring | `m_skirt_group_done` sizing; by-object per-instance `generate_object_brim`/`generate_object_skirt_group` call sites; by-layer call site. |
| `Brim.cpp:984-996` | `m_objectBrimAreasByInstance` production (islands shift + skirt-grouping footprints) — consumed at `Print.cpp:4195-4202`; per-instance areas already ported with the instance-keyed brim map, reused as-is. |

## Current ares state (temporary compatibility shells)

1. `gcode_emit/skirt/occupied.rs::items` — one item per **object**: all
   instances of the object are folded into one occupied point set
   (`for &member in &group.instance_indices`), and brim contribution is the
   whole-plan `covered_hull()`, not per-instance contours.
2. `gcode_emit/skirt/generate.rs` — member grouping is gated on
   `per_object && traversal.objects.len() > 1`; a single object with N copies
   skips grouping entirely and uses one all-instances hull.
3. `gcode_emit/brim/owner.rs::Area{per_instance, owner_collection}` —
   per-instance generation/emission exists only for uncombined by-layer
   printing; ByObject and `combine_brims` emit the owner-merged collection
   (documented in the struct comment citing the same upstream rows).

These shells are replaced by the upstream shape; the milestone does not
keep them as configuration.

## Included behavior

- Per-instance occupied items with per-instance brim-area contours and the
  obstacle-only tower item.
- `stCombined` unite-all and `stPerObject` fixed-point merge over per-instance
  items; group = skirt loops + instance list + brims.
- Per-group brim assembly, including the `combine_brims` contact union
  (2×spacing offset, jtRound, SCALED_RESOLUTION) and per-cluster area
  union/cleanup/clipping.
- Per-instance skirt/brim emission at by-object chunks
  (`generate_object_skirt_group`/`generate_object_brim` shapes) with
  group-done and objs-with-brim dedup; by-layer emission re-keyed onto the
  same group assembly.
- Emission ordering: groups and their members visited so the observable
  first-target selection matches the Native call sites.

## Deferred behavior

- Belt-printer skirt/brim paths (beta returns early; already out).
- Wipe-tower Type2 cone corner sampling and IMEX shifts (separate slices).
- `m_skirt`/`m_skirt_convex_hull` flat-collection consumers beyond emission.
- Nozzle-specific (`Shared` vs per-nozzle) bed-exclusion brim compatibility
  beyond what existing fixtures exercise.
- `byobject_perobject_skirt_target` unwrap triage (may flip with slice 3 but
  is not an exit criterion).

## Destination boundary

`crates/ares-core/src/project_slice/gcode_emit/`:
`skirt/occupied.rs` (per-instance items), `skirt/generate.rs` (gate removal,
group assembly), `skirt/member_groups.rs` (instance-keyed membership),
`skirt/groups.rs`, `brim/groups.rs` + `brim/combined.rs` (group brim
assembly incl. contact union), `brim/emit.rs`/`brim/owner.rs` (per-instance
emission un-gated from print sequence), and the by-object emission call
sites. No new crate, no new pipeline stage.

## Implementation slices (one RED→GREEN chain each)

1. **Per-instance items + merge parity** — flips the copies/min-length/far
   skirt content (4 tests).
2. **Group brim assembly incl. combine contact union** — flips
   `global_combined_brim_target` (1 test).
3. **By-object per-instance emission** — flips the byobject families
   (5 tests).

## Exit criteria

- The 10 named tests pass the whole-output comparison (byte equality after
  `normalize_one_generator_line`); no count premises reintroduced anywhere.
- `test(brim) | test(skirt)` family: ≥150 passing (current 140 + the 10), no
  regression among the currently passing 140.
- `cargo fmt` and `cargo clippy --workspace --all-targets -- -Dwarnings`
  green; run logs and artifacts persisted under `ARES_PARITY_ARTIFACT_ROOT`.
