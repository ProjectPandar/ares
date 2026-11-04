# Arachne extra-overhang perimeter application — source rewrite SPEC

Historical discovery used main5317e1c09cbcee822df3ee12b99e9e265aab4643.
Formal RED must precede the missing-call/visibility-only GREEN.

## 1. Upstream boundary and Rust destination

This is a bounded rewrite of OrcaSlicer2.4.2
`src/libslic3r/PerimeterGenerator.cpp`:

- `generate_extra_perimeters_over_overhangs`, approximately885–1084:
  **existing** Rust geometry implementation to reuse, not redesign.
- `PerimeterGenerator::apply_extra_perimeters`,1087–1113: activation,
  prepend generated paths to the current island, subtract their coverage
  from accumulated fill surfaces.
- `PerimeterGenerator::process_arachne`,2093 onward, specifically
  **2519–2521**: append ordinary `infill_exp`, then call the operation,
  before computing no-overlap fill polygons at2523 onward.
- Classic's existing call is **1672**, inside `process_classic`1144 onward.
  Earlier inbox shorthand reversed these engine/call-site labels; this
  document uses the actual function headers and correct line numbers.

Rust destination:
`crates/ares-core/src/project_slice/perimeters/arachne/materialize.rs`,
inside `materialize_record`, after ordinary island fill is appended.
The existing `classic/extra_perimeters/{mod,apply,ordering}.rs` implements
reusable geometry/materialization around Classic-specific context lookup.
Share the existing upstream operation with a small Arachne input adapter;
do not duplicate its geometry or create a new Ares slicing pipeline.
Extract only the private helper boundary required to share that operation.
The Classic adapter and its behavior remain unchanged.

The current Arachne scaffold stops at the ordinary wall/fill boundary,
with no equivalent of the2521 call. It is a temporary incomplete
compatibility shell around `process_arachne`, not an independent design
whose missing behavior should be preserved.

## 2. Reachable Native-exported witness

Geometry comes from the existing
`tests/parity/arachne-alternate-extra-wall/stepped-top-off.3mf` model,
rotated180 degrees about X: build diagonal `(1,-1,-1)`, translation
`(110,110,0.6)`. This is an ordinary GUI model rotation, producing a
stepped overhang, not malformed geometry or a synthetic option type.
Every retained fixture below is a fresh **official CLI-exported3MF**,
then sliced from that export with exit0.

Common isolation settings: Arachne, three walls, Aligned seam,
stagger OFF, scarf NONE, wipe OFF, detect-overhang-wall ON,
sparse-infill density0%, alternate-extra-wall OFF, only-one-wall-top OFF.
No seam/wipe/travel or fill-grouping code is part of this investigation.

Primary pair:

| Case | SHA-256 of complete exported3MF |
|---|---|
| `arachne-control` | `0f6cac9cd2e66be4b28ca80edfeab5bcc713609d99ddebd08dbedb364ed6d8cc` |
| `arachne-positive` | `77968f1ac5b7497c7f7207bf43153a954d1328a32faf89eff1c8d2268fcb037f` |

Their ONLY changed ZIP payload is `Metadata/project_settings.config`;
its ONLY changed key is `extra_perimeters_on_overhangs`, "0" → "1".
The complete payload hash map and sole-key assertion are in `verified.json`.

Positive:1086 complete normalized line differences. First differing real
movement, index53 (diagnostic alignment only):

- Native raw143: `G1 X117.223 Y117.223 Z1`
- Ares raw149: `G1 X123.939 Y123.939 Z1`

M73 estimates differ earlier because the planned path differs. Neither
M73 nor any other non-generator line is removed in acceptance checks.

## 3. Repeatability and controls

Fresh official
repeat and seeded runs of the positive and four controls:10/10 expected
outcomes. Native official, repeated official and seeded COMPLETE outputs
match after only the established single generator identity/date line is
removed. The positive reproduces1086 differences in both modes; controls
match Ares completely in both modes:

| Control | Exported3MF SHA-256 |
|---|---|
| extra OFF | `0f6cac9cd2e66be4b28ca80edfeab5bcc713609d99ddebd08dbedb364ed6d8cc` |
| Arachne detect OFF, extra OFF | `1c0b73fabffd108411557558eadd34db539bcb9cd1db6a3a265460579f44a545` |
| Arachne detect OFF, extra ON | `550e19473adf26e61e0861b2736784179808a7035fdac58168b521bd79ccbdf5` |
| Classic detect ON, extra ON | `85e93cb93363cd43ce7d6007db39162871d6e9d6539679bb974926617c414fce` |

The old stagger binary6c3038b32 and the local role-only probe based on
main5317 produce identical complete normalized streams for these cases.
The role-only probe changes only empty-flat scarf clipping; scarf is OFF
here.
These are discovery CLI observations, NOT formal public-API test evidence.

## 4. Read-only GDB: first unequal intermediate

Native: seeded binary with its DWARF, source mapping and seed0.
Break at `PerimeterGenerator::apply_extra_perimeters` entry and observe
return using a finish breakpoint. Ares: break at
`arachne/materialize.rs:258`, after ordinary island materialization.
No inferior calls, register writes or memory writes are used.

Scripts, logs and raw JSON:
`{arachne-control,arachne-positive}-{native,ares}-pg.{gdb,log,json}`;
readers: `native-pg-capture.py`, `ares-pg-capture.py`;
comparison: `gdb-comparison.json`, all under the artifact root.

Both control and positive have six layer records. For ALL12 records,
Native pre-application versus Ares materialization inputs match exactly:

- layer/slice height and wall count;
- both activation settings;
- complete scaled infill contour/hole arrays, without canonicalization;
- complete lower-support polygon arrays;
- flow width, height and spacing;
- three ordinary perimeter entities per island.

First divergence: positive layer_id2, slice_z0.5, print_z0.6:

| Intermediate | Native | Ares |
|---|---:|---:|
| ordinary island entity count |3|3|
| after extra application |**4**|**3**|

All other positive layers and ALL control layers remain3→3.
Native's caller is explicitly `PerimeterGenerator::process_arachne`.
Native prepends one additional entity and removes its filled coverage.
For example, the common input contour is:

    (13837614,13837614),(-13837612,13837614),
    (-13837612,-13837612),(13837614,-13837612)

The common lower contour has coordinates±7700000. Native's resulting
internal fill contour shrinks to corners with coordinates
`7053115/-7053111`; Ares retains the original infill contour.
Common flow inputs: width0.38214603066444397,
height0.20000000298023224, spacing0.3392256498336792.

Every complete G-code output produced under these four GDB runs matches
its corresponding non-GDB output after only the usual generator line.
This locates the missing PerimeterGenerator application BEFORE downstream
fill grouping, seam placement and travel emission; those are not fixes.

## 5. Included behavior and non-negotiable invariants

Port the missing Arachne application at the cited source position:

1. Preserve the upstream guard: not spiral, lower slices present,
   detect-overhang and extra-overhang enabled, wall count>0,
   layer_id>raft_layers. No engine-specific invented fallback.
2. Reuse the existing source-ported generator and current overhang flow,
   lower polygons, wall count and coordinate scale. No numeric rewrite,
   resampling, tolerance changes or independent path algorithm.
3. When generated paths are nonempty, prepend their native-equivalent
   entities to the CURRENT island, preserving existing wall entities and
   the source collection-ordering semantics.
4. Subtract generated filled coverage from accumulated `fill_surfaces`,
   retaining surface types. Do not merely append extra G-code.
5. Preserve Native's empty-path behavior: no entity replacement or fill
   subtraction if the generated paths are empty.
6. `infill_exp` itself and the subsequent no-overlap calculation retain
   their existing Native meaning; do not replace them with the cut fill
   surface result or redesign downstream grouping.

No new public slicing API, file I/O, terminal behavior, platform-specific
runtime or dependency belongs in core. Preserve WASM/Windows/macOS/Linux
portability. New/extracted Rust files stay below400 lines.

## 6. Deferred and explicitly excluded

- Classic detect-OFF discovery is independent follow-up. At density0%,
  BOTH extra-OFF and extra-ON have the same26 Native/Ares differences.
  Their only OFF/ON full-stream change is the faithfully serialized
  configuration footer line, separately in Native and Ares. Do not call
  this an extra-application failure or assume the Arachne cause fixes it.
  Its later read-only handoff proved all6 ordered PG wall records equal;
  Do not reopen that parked candidate or change seam/travel here.
- No changes to Classic geometry, medial-axis/bridge-direction numeric
  behavior, arc fitting or unrelated existing helper limitations.
- No infill-combination, `Fill.cpp::group_fills`, `SurfaceFillParams`,
  infill pattern changes, or grouping work.
- No seam, fake-wipe, inward, travel/retraction lifecycle work.
- No enum/random/scarf changes changes.

## 7. Formal implementation and acceptance plan

Plan:

1. Use main's merged `runner/native_api_cli.rs::NativeCase` public API/CLI
   and complete Native stream seam. Do not hand-write
   another runner. Native availability is the only environment condition,
   like main's NativeCase users; no borrowed/private reference gate.
2. Commit formal RED: these real exported fixtures, manifest/provenance,
   both reference modes, public-API bytes and CLI complete-stream checks.
   The positive must fail before production edits; four controls pass.
   Diagnostic movement alignment is never acceptance filtering.
3. Commit scoped GREEN: missing Arachne application and only the minimal
   existing-helper sharing needed for the cited upstream operation.
4. Re-run all five complete cases in official and seeded modes through
   the shared API/CLI harness; reproduce the equal input GDB snapshot,
   now Native/Ares4-entity output and cut fill contour at layer2.
5. Run fmt, workspace/all-target clippy, targeted nextest and wasm check.
