# Draft SPEC: AutoBrim object-config maximum speed lookup

Native-first discovery; no code in this document.
Docs base: `5317e1c09cbcee822df3ee12b99e9e265aab4643`.
Dependency: AutoBrim geometry/harness chain ending `cb2f0b4be`, with
optional adjacent admission chain ending `7d2783203`. Neither chain changed.
This is a source-cited `libslic3r` rewrite slice, not an Ares pipeline feature.

## Observed user-facing discrepancy

Starting from an existing exported 3MF, set a tall object's inner-wall speed to
60 mm/s while global inner/outer speeds remain40/80. Native increases AutoBrim
footprint using the global wall-speed ratio; current Ares leaves the footprint
at the global-only size. This is an actual first-layer toolpath difference, not
only a configuration/header difference or a non-observable internal formula.
The same object override with `no_brim` has matching Native/Ares whole output.

## Evidence before source localization

The source project is the repository's actual exported
`tests/parity/mesh-edge-pairing/cube-control.3mf`, SHA256
`3d28ef67d923a90e022b8c186ebf06c050d1473ecf1bcdc83e1768f470592533`.
It retains export metadata, object/volume identities, instance and printer
settings. Its mesh is10×10×2 mm (despite the `cube10` filename). Only build Z
scale is changed from1 to50, making10×10×100 mm; **mesh entry bytes unchanged**.
One printable object, one model-part volume, one PLA filament, ByLayer,
no support/raft/tower/skirt; `combine_brims=0`, brim width10, gap0.

All four cases have global speeds: inner40, outer80, sparse/solid/top/support10.
The local pair adds only object-level
`<metadata key="inner_wall_speed" value="60"/>` beside object extruder metadata;
no volume speed key is added. `auto_brim`/`no_brim` is the other paired axis.
All8 Native invocations (4 cases × official/seeded) exit0; all4 existing CLI
invocations exit0. Official/seeded Brim bodies agree byte-for-byte.

| Input | SHA256 | Native / Ares Brim XY extrusion segments | Whole output vs Ares, both modes |
|---|---|---|---|
| Auto, global only | `e862e3a09ab896c964f2caad70e974f880354c79e740238186cfc6ab609c27d5` | 448 /448 | equal |
| Auto, object inner60 | `5524e86cb65791afff41b03f9ea00a6866bbc43978367c19829fab120ed57fd2` | 780 /448 | **different** |
| No brim, global only | `59ac1c8bac3851349e5d4ddb0645b46d8896f0778fea13cd692b4c652b2a80a5` | 0 /0 | equal |
| No brim, object inner60 | `219e0806e2ca7e0c04f888c3129c9940692608317b784e8d5384dfc57be86978` | 0 /0 | equal |

The exploratory comparison changes only the generated-by line. It is not a
public API test or a replacement parity comparator; no API or nextest was run
in discovery. All equalities above are recorded exploratory CLI comparisons.

Brim segment coordinate bounds in mm (same X/Y ranges):

- Global-only Native and Ares:105.664..124.336.
- Local-inner Native:103.401..126.599; local-inner Ares:105.664..124.336.
- One nonempty Brim block in each Auto case; no Brim blocks in no-brim controls.

First normalized whole-output difference at byte1321 in both modes is the
travel entering the Brim block: Native `G1 X105.076 Y105.608`, Ares
`G1 X106.808 Y107.065`. Thus M73 is **not** the first discrepancy, and no time
or movement normalization was used to manufacture a signal. Changed ordinary
inner-wall speeds are already handled: the local no-brim control matches.

An initial Zscale10 run produced height20 mm and no Native Brim in any case;
those parent-directory artifacts are negative discovery, **not** an active
witness. Only `height100/` above is the final active input set.

## Source ownership and root cause

Read-only upstream checkout: `OrcaSlicer` commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, read from the original AutoBrim
worktree because this discovery worktree has no initialized submodule.
Paths below are `OrcaSlicer/src/libslic3r`:

1. `Model.cpp:2976-3010::Model::setPrintSpeedTable` initializes the **global**
   print-speed map/max from inner, outer, sparse infill, solid infill, top solid
   and support speeds. Do not infer this global map from a resolved local region.
   This routine does not assign global small-perimeter percent as a physical
   speed; preserve its actual source semantics, not an extra role maximum.
2. `Model.cpp:3222-3255::Model::findMaxSpeed(const ModelObject*)` starts from
   that global map. An object without config keys returns global max directly.
   For an object `inner_wall_speed` key, it sets internal speed and scales
   external speed by `global_external/global_internal * object_internal`.
   A separately present object outer-speed key subsequently overrides external
   speed. Sparse/solid/top/support/small-perimeter keys override their roles.
   It takes the role maximum and applies250 only on the cited nonpositive
   object-maximum branch, not indiscriminately to a zero global return.
3. `Brim.cpp:477-478` obtains adhesion and calls **object-level** findMaxSpeed
   once. `:500-501` passes that speed into
   `configBrimWidthByVolumeGroups`; `:260-305` consumes it in the moment-derived
   height-to-area term before the geometric cap/omission. These widths feed
   existing per-polygon quantization and Brim toolpath generation.

For the active pair, Native's object max includes
`80/40*60 =120`, not the resolved region's `max(outer80,inner60,...) =80`.
The 10×10 square moment is10000/12 mm⁴; at height100 the speed term is
5 mm for80 and7.5 mm for120 before existing lattice quantization. This explains
why a moderately tall project crosses real loop-width steps without changing
material/geometry decisions. These are source explanations, **not** expected
G-code values hardcoded into tests; Native is the output oracle.

## Exact Rust destination / replacement boundary

Under `crates/ares-core` after the queued dependency lands:

- `project_slice/gcode_emit/brim/auto.rs::width_mm`: replace the temporary
  max-of-resolved-region-speeds calculation. Keep all moment calculations,
  diagonal caps, omission rules, material terms and quantization unchanged.
- `brim.rs::BrimPlan::generate` / `brim/auto.rs::AutoGroupContext`: supply a
  source-object maximum computed once per PrintObject, not a separate role
  scan for each volume group. Current width context is a temporary compatibility
  shell around upstream `configBrimWidthByVolumeGroups`, not a new policy owner.
- Use the global `traversal.resolved.views.full.process.region` and global
  object support-speed source, plus the source `ProjectObject` accessed via
  PrintObject identity. `project/domain.rs::ProjectObject` already exposes
  typed object/region overrides that distinguish `Some` from absent values;
  `options/object_options.rs` and `options/region_options/overrides.rs` own
  those carriers. Do **not** use overlaid region candidates/volume modifiers
  as a substitute for raw object config-key presence.
- If typed carriers cannot preserve a required distinction in upstream
  `object->config.keys()` (including the empty-key path), identify the exact
  missing source carrier before proposing a minimal change to
  `options/model_config_deserialize.rs` / project metadata. No guessed key
  presence, broad new generic map, or independent speed-policy abstraction.

Port the cited findMaxSpeed role lookup as one bounded upstream function,
including key presence/outer override precedence and the nonpositive branch.
RED coverage must include the active ratio witness, its three controls, and
Native-first explicit outer-override/other-role cases before claiming general
lookup parity. Current discovery proves the ratio failure only, not every
edge case of findMaxSpeed. No implementation is included in this draft.

## Included / deferred behavior and stop conditions

Included: global-role map input, object-config role overrides, global wall
ratio for object inner speed, subsequent explicit outer override, upstream
maximum/empty-key/nonpositive behavior as consumed by AutoBrim. A volume-only
or layer-range speed override must not silently become an object key for this
lookup. Existing normal extrusion speed resolution is not changed.

Deferred and unchanged: object-wide material adhesion/first-layer extruder
eligibility (`Brim.cpp:120-148`), mixed-material grouping, AutoBrim geometry
formulas, perobject skirt admission/group merging, support/raft/tower, combined
brims, UI/filesystem behavior and M73 estimator differences. No new guard,
fallback, option flag, or UnsupportedProjectFeature is allowed to hide them.

Native rejection, inactive Brim, a failing no-brim/global Auto control, an
unrepresented config-key distinction, or a first mismatch outside this lookup
blocks implementation. Do not patch unrelated routing/speed resolution or
weaken existing assertions. Keep each file<=400 lines and Tier1/WASM boundaries.

## Proposed SPEC → RED → GREEN

1. Reconfirm final inputs and explicit outer-override/remaining-role controls
   Native-first.
2. Commit public Native/API/CLI tests before production. Freeze only referenced
   input3MFs, assert their hashes and active Brim motion, then compare full bytes
   with the unchanged `compare_ordered_bytes_generator_only` and API/CLI raw
   equality. Use the shared runner helper. Require the
   object-inner Auto target to FAIL while the three demonstrated controls PASS
   in both modes; log actual differences, not only max-speed calculations.
3. GREEN changes production only in the cited upstream lookup/carrier boundary;
   no tests/docs edits, constants to force the witnessed loop count, new guards,
   skip behavior, or committed Native output. Retain all AutoBrim/small-diagonal
   and per-object admission regressions unchanged.
4. Final directed dual-mode evidence, fmt, clippy, wasm32 and an exact-hash
   dual FULL. No FULL during discovery.
