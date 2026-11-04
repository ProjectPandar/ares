# SPEC: per-group minimum skirt filament length

This extends one deferred `libslic3r` boundary, not an Ares pipeline design.

## Non-negotiable decisions / bounded scope

- Rewrite `Print::_make_skirt`'s **per-group** minimum filament extension for
  positive-loop per-object skirts, one filament, by-layer order, no brim,
  support, raft, tower or draft; uniform skirt angle. Include the already
  source-owned disjoint/overlap/transitive/copies membership model.
- A minimum is filament length (mm), not path length or volume. Its accumulator
  resets for each group, not each PrintObject, each copy, or the global preview
  loop list. Requested skirt_loops remains a minimum loop count.
- Preserve the upstream nominal grouping reach
  `skirt_distance + configured_skirt_loops * spacing`. Do not regroup by the
  extra loops' eventual reach or invent new collision validation.
- Only narrow the existing minimum-length rejection term for this no-brim
  slice. Preserve `resolved.objects.len()>1` and every other deferred term,
  including the existing singleton/uncombined outer-brim predicate.
  Positive minimum **combined with brim** remains deferred within the retained
  minimum-length term; zero-minimum brim controls stay admitted unchanged.
  Do not introduce a new guard/error or combined-skirt fallback.
- Native-first public RED precedes GREEN. Keep all original
  inputs/assertions; no comparator, verify_value, matrix classification,
  skip/fallback or Native-output commits. Changed files <=400 LOC; split
  mechanically before any semantic growth beyond that bound.

## Owning upstream source boundary

Vendored `OrcaSlicer/src/libslic3r`:

- `PrintConfig.cpp:5751-5760`, `min_skirt_length`: min 0, default 0, units
  filament mm. Tooltip explicitly says final loop count is not accounted for
  when arranging/validating object distance. This is not a new placement rule.
- `Print.cpp:2703-2721`, `Print::_make_skirt`: fixed initial skirt flow/spacing
  and `Extruder::e_per_mm(mm3_per_mm)` conversion.
- `Extruder.hpp:38-41`: conversion includes filament flow ratio and filament
  cross-sectional area, not arbitrary print-flow or bridge modifiers.
- `Print.cpp:2728-2778`, `append_skirt_loops_for_hull`: initial distance
  `skirt_distance-spacing/2`, outward spacing steps; local extruded-length
  vector initialized inside the call. Add each simplified loop's full length
  times extruder conversion; when remaining count reaches 1 and filament
  target is unmet, extend the loop counter. Track final occupied skirt reach.
- `Print.cpp:2784-2900`, group items/union-find/fixed-point envelopes: nominal
  grouping_offset is defined at 2795 from configured loop count, not from
  minimum-length-induced extra loops. Keep ownership/copy membership intact.
- `Print.cpp:2972-2996`: call append_skirt_loops_for_hull independently for
  every emitting group; reverse each group's loops; retain object IDs in
  SkirtBrimGroup. Flat `m_skirt` is the preview/extents mirror only.
- `GCode.cpp:4260-4299`, Skirt loop allocation: first-layer export uses the
  actual skirt entity count; one printing filament gets all those entities.
- `GCode.cpp:4391-4452`, `generate_skirt`: materialized loop range, layer-height
  flow and seam/clipping; minimum-length budget is not recomputed at export.
- `GCode.cpp:4456-4484`, `generate_object_skirt_group`, and `5325-5336`:
  owner-group lookup and group-done state, once before first member content.

Multiextruder budget switching in the append loop is outside this slice;
only the single-filament branch is ported/admitted.

## Exact Rust destination / current real gap

`crates/ares-core/src/project_slice/gcode_emit/`:

1. `skirt/generate.rs:30`, existing `print.min_skirt_length.0 != 0.0` term
   rejects these multi-source public projects before any group loop generation.
   Only narrow that term for no-brim inputs; all other admission remains as-is.
2. `skirt/generate.rs` already derives `e_per_path_mm` from initial flow,
   filament ratio/diameter and passes `LoopParameters` separately per group.
   Audit its float/lattice ordering against the cited source, do not redesign
   a configurable extrusion budget API.
3. `skirt/loop_generation.rs::make_loops` already has a per-call filament
   accumulator and condition `count<requested || length<minimum`. It is an
   existing compatibility shell around append_skirt_loops_for_hull. Public
   strict RED/GREEN decides whether that shell is equivalent or needs the
   smallest source-cited precision/counter repair; admission deletion alone
   is not assumed parity evidence.
4. `skirt/member_groups.rs`, `skirt/occupied.rs`, `SkirtPlan` group loop ranges,
   `layers/object_skirt.rs`: retain source-owned identities, nominal fixed-point
   grouping and once-per-group/layer dispatch. Additional generated loops
   belong to the group, not to the flat preview list or every member/copy.
5. `skirt.rs::emit_group` / loop emitter: export the actual materialized range,
   reversed outside-to-inside order and existing seam/flow behavior. No new
   minimum budget in motion/export and no changes to unrelated motion policy.

These modules are temporary compatibility shells only where they do not
match the cited upstream behavior. Stop on producer identity/centering gaps
rather than fix resolution inside this slice.

## Native admission + real effect proof (completed; no Ares RED claim)

Official and seeded Native ran serially, each input with fresh cache
and artifacts; all **16/16** return_code/OS exits are 0.

Each pair retains the corresponding original per-object skirt input's mesh,
source object identity, transforms, instance membership and all options.
Only min_skirt_length changes 0 -> 20. JSON/ZIP are reserialized; exact bytes
and hashes are retained, not regenerated during a future test run.
Both modes have identical witness counts/bounds/extrusion sums:

| Geometry | Groups | Loops per group, 0 -> 20 | Positive XY relative-E mm per group, 0 -> 20 |
|---|---:|---|---|
| disjoint | 2 | 2 -> 8, **each** group | 4.90079 -> 21.36385, each |
| overlap | 1 | 2 -> 6 | 7.11314 -> 22.22306 |
| transitive | 1 | 2 -> 5 | 9.57134 -> 24.48214 |
| copies | 1 | 2 -> 4 | 12.27531 -> 24.84710 |

The G-code is explicitly relative E (M83, no M82); sums count only positive-E
XY `; skirt` motion. A loop witness counts distinct complete positive top-edge
horizontal Y levels of these known rectangular occupied hulls with southwest
seams. Bottom seam splits are not counted. An initial analysis incorrectly
assumed equal complete top/bottom edge counts for wide hulls; that parser
assertion failed, not Native admission. Raw outputs remain untouched; the
corrected top-edge witness and physical extrusion increase are recorded.
Do not convert this shape-specific discovery helper into a general slicer
algorithm or infer generic loop counts from TYPE blocks.

Native generation accounts the full loop length, while export seam clipping
can reduce emitted E. Therefore exported E >= target is not the definition
of the algorithm. Actual positive motion and its paired increase prove the
feature path here; strict full-output comparisons remain the parity gate.
Disjoint evidence proves the target is independently applied to both groups;
merged/copies evidence proves shared ownership does not multiply emissions.

### Exact pure input SHA256

| Case | SHA256 |
|---|---|
| disjoint-0 | `45ee9b8b8fdad2dc8a6c1e95f72d62e939e77743b78f10131fb26a384f918962` |
| disjoint-20 | `86b070b0b100cd0eadf60edc3e9ec1db3d21a10481f25ff51acd40bd195272fb` |
| overlap-0 | `4b7e02fd4fbf914957d72ac175b7f496d19bf078f68953295f3018c119194275` |
| overlap-20 | `7438cc1786c91530bf5a79f3e25e0c17214b456bb7b5d6081b0d59f897ebeba2` |
| transitive-0 | `9c06439795927d51659d92f4b852c177b4fd6014e391b0c661c6a623ad75a6b1` |
| transitive-20 | `fc341ae293d45b4d74a9d1fa66e8b192d73fb61704379d4e4b7dd8ff6083b30e` |
| copies-0 | `df8489398e315dc878290af18d97f04ac8e3745d21a75858f276378a5890b05c` |
| copies-20 | `7a6a8896755bd308289fd314a4cadb748c44f1df421467ebda2d2acdb07998f9` |

## Public RED / GREEN and exits

1. Commit these eight pure inputs/hashes and public API+CLI parity tests before
   production edits. Native-first fresh official/seeded runs must reproduce
   group ownership and extra positive skirt extrusion before Ares assertions.
   Expected baseline: 4 zero controls PASS, 4 positive cases FAIL at existing
   `UnsupportedProjectFeature("skirt_type per-object")`, both modes. This is
   source-predicted.
2. GREEN requires Native strict full ordered G-code comparisons using only
   existing generator-metadata treatment, raw API/CLI equality, actual extended
   group motion and unchanged zero-control bytes. No rejection-as-success,
   geometry normalization or hand-created expected G-code.
3. Retain original four no-brim grouping tests, already merged brim tests,
   combined/single/zero-loop controls and relevant option-domain controls.
   Positive-minimum+brim stays deferred; no widening through the independent
   existing brim term. Report first raw differences before repairs.
4. Both serial modes use fresh Native caches/artifacts.
5. Strict clippy/fmt/WASM checks; no core I/O/UI/platform leak. Stop if
   only narrowing the existing term cannot represent this boundary.

## Deferred adjacent source behavior

Positive minimum with brim (including currently admitted zero-minimum outer
brim), support/raft height interactions, tower obstacle groups, draft lifetime,
sequential shared-skirt validation, multiextruder budgets/tool ordering,
nonuniform group-member start angles. No option schema rewrite, placement
planner, source identity/transform changes or new Ares-owned pipeline.
