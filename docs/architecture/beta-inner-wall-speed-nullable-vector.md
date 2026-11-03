# Nullable-vector boundary: nullable inner-wall speed and its indexed readers

This source boundary continues the deferred nullable-vector inventory after
`beta-three-nullable-region-vectors.md`, for exactly **inner_wall_speed**.
It is a source-cited libslic3r rewrite slice, not a new slicing pipeline.

## Owning upstream boundary

Pinned OrcaSlicer revision: `345efec64c7b95a05ce5cf96df0dde66a158835c`.

- `src/libslic3r/PrintConfig.cpp:5957–5966`, `PrintConfigDef`'s
  `inner_wall_speed` declaration: `coFloats`, nullable, minimum1, default
  `ConfigOptionFloatsNullable{60}`. Preserve the declared alias
  `perimeter_feed_rate`; do not invent aliases or change the default.
- `PrintConfig.hpp:1546`, `PrintRegionConfig`:
  `ConfigOptionFloatsNullable inner_wall_speed`.
- `PrintConfig.cpp:10103–10107`, `print_options_with_variant`: the field is a
  process/nozzle variant, not a scalar or a logical-filament vector.
- `Config.hpp`, `ConfigOptionVector::get_at`, nullable float parsing,
  serialization, `is_nil` and equality; `Config.cpp`,
  `ConfigBase` methods `load_from_json` / `set_deserialize`: preserve wire strings,
  complete arrays, nil positions and source indexed/front-on-miss behavior.
- `PrintApply.cpp`, process variant materialization; `PrintConfig.cpp`,
  `set_variant_override`; `PrintObject.cpp`, region override application;
  `Config.hpp`, `apply_override`: use original process variant indices and
  source non-nil assignment / singleton broadcast rules.
- `GCode.cpp:2155–2157`, `NOZZLE_CONFIG`, and `:9393–9397`, `:9557–9561`:
  inner-perimeter speed and overhang reference speed use the current
  filament's selected nozzle-config index. `:3123`'s precomputation is a
  separate read: do not silently turn it into the current-tool selector.
- `Layer.cpp:167`, `is_perimeter_compatible`: compare selected numeric values
  exactly, using the authored outer-wall filament argument passed to
  `Print::get_extruder_id`. Do not substitute the inner-wall ID or subtract1.
- `Model.cpp:3212–3213,3473–3474`: the brim speed table explicitly reads
  nullable float column0. It must not inherit the current motion selector.
- `GCode/WipeTower2.cpp:1062,1086–1087`, `WipeTower2::WipeTower2`:
  snapshot perimeter speed using `get_extruder_index(config, initial_tool)`;
  retain the source zero-to80 path. This is **initial-tool** state, not an
  unconditional column0 or a value recomputed from the changing current tool.

## Rust destination and scaffold replacement

Replace the temporary `OrcaFloat(60)` field in
`crates/ares-core/src/options/region_fields.rs` and its `Float` registry kind
in `options/registry/definitions/table/middle_independent.rs` with the existing
`OrcaFloatsNullable` representation and canonical singleton default `[60]`.
Update only this field's owning metadata record. Keep definition count and all
other records, including the three bed-exclusion records, unchanged; do not
regenerate the registry wholesale.

Use the existing public DTO/codec, project import and process-variant paths:
`options/process_options/region_source/wire.rs`, `options/process_options/wire/`,
`options/project_variants/`, `options/region_options/{overrides,merge}.rs` and
`project/effective_config/`. Extend the existing field-specific nullable merge
and source-equality handling only for this key. A non-nil singleton broadcasts;
a singleton nil inherits rather than replacing the target with NaN or60.
Do not recompute variant indices after vectors have already been reduced.

Update the owning scalar readers, not the surrounding algorithms:

| Rust destination | Upstream-owned projection |
|---|---|
| `project_slice/gcode_emit/motion/options/build/region.rs` | Existing selected nozzle-config index |
| `project_slice/perimeters/groups/compatibility.rs` | Same existing physical slot as outer-wall comparison, exact selected value equality |
| `project_slice/gcode_emit/brim/speed.rs` | Column0, including existing object speed-table arithmetic |
| `project_slice/gcode_emit/wipe_tower/generator.rs` | Initial-tool physical-index speed snapshot |
| `options/speed.rs` and any remaining direct field reader | Inventory the owning upstream scalar projection before changing it |

The derived `MotionOptions.inner_wall_speed: f64` remains a valid temporary
carrier for an already selected upstream numeric speed. Do not replace it with
an Ares-owned selection policy. Its existing scarf/overhang/feature consumers
receive that selected value without an algorithm rewrite. The wipe-tower state
may gain only the source-equivalent initial perimeter-speed snapshot; do not
redesign tower planning, change other speed fields or use a guessed index.

Core remains byte/API-oriented and portable to browser WASM, Windows, macOS and
Linux. No filesystem, environment, resource search, terminal or viewer behavior
belongs in this migration.

## Included, deferred and non-negotiable behavior

Include this key's canonical DTO/wire/default/nil shape, import, original-index
variant gathering, region overrides and the explicitly listed readers.
Maintain source singleton/front-on-miss and nil semantics; never flatten an
array, erase nils, reorder values or accept scalar/vector output equivalence.
String-wire input remains accepted through the existing source-backed decoder;
serialization remains a complete canonical array.

Keep internal option-equality EPSILON semantics only where upstream uses them.
They do not permit tolerance in DTO, full CONFIG_BLOCK, selected numeric
perimeter compatibility or complete G-code comparisons. Preserve all existing
comparators, fixtures, Native preconditions and retired-test decisions.

Defer the other39 scalar/vector fields, broader metadata drift, machine variant
ordering work, active layered nozzle-group solving, adaptive-PA residuals,
model-ironing work and unrelated geometry/G-code algorithms. No exporter footer
patch, preset-name exception, legacy fallback or Native-export production data.
No change to the accepted project-refresh/report range or its generated wiring.

## Failure inventory before test implementation

1. Native arrays are rejected by the public DTO, or admitted by silently taking
   the first element; default kind/nullable/default-array declarations disagree.
2. Scalar wire passes import normalization but fails direct typed DTO parsing;
   mixed/all-nil arrays lose position, order, length or canonical wire tokens.
3. Nil clobbers a parent/default; singleton broadcast or absent-column retention
   differs from source; override mapping uses already reduced indices.
4. Motion chooses logical filament or column0 instead of the selected physical
   nozzle slot; overhang/scarf readers receive a different selection.
5. Perimeter compatibility uses approximate scalar equality or the wrong ID;
   the brim speed table incorrectly selects the current nozzle.
6. Wipe-tower speed follows a later tool rather than its initial-tool snapshot,
   or preserves the old scalar via an unqualified column0 projection.
7. A later unported-field/body discrepancy is misreported as whole-output GREEN
   merely because this field's shape or selected feedrate now matches.

## Native-first public qualification

The public tests use the
existing nested `config_block_roundtrip` registration and public slicing seams;
do not edit the ROOT registry or resurrect retired width/structure tests.

1. **Definition and full-array RED.** Qualify true Native defaults with the
   existing no-input settings-export seam. Compare the complete Native string
   array and type/default/nil declarations with public `print_config_def_json`,
   decode/encode and typed DTO output, including singleton, distinct multi-value,
   mixed/all-nil and scalar-wire inputs. Typed codec numeric/null JSON and Native
   wire-string arrays are different representations: derive their exact
   correspondence, never change the comparator to equate scalars with arrays.
2. **Original-input application.** Feed the same original scalar-authored and
   vector-authored 3MF bytes independently to Native and `load_project` /
   `ProjectSettings`; do not feed Native's re-export back into Ares. Removing a
   key from a project is not proof of registry defaults because preset/import
   defaults may apply. Preserve complete effective arrays and full CONFIG_BLOCK.
3. **Nonzero motion selection.** Start from the existing nozzle-flush input
   family used by the outer-wall vector witnesses. Prove distinct original
   process columns, physical variant gathering and nonzero selected slot in
   repeated successful Native output. Use distinct valid inner-wall speeds and
   demonstrate that cooling/volumetric/first-layer limits do not mask them.
   A Native export or internal resolver probe alone is not a runtime RED.
4. **Override and reader controls.** Qualify original-index remapping, non-nil
   singleton broadcast and nil inheritance. Include model-part/modifier/material/
   layer-range application as supported by the same source path, selected-value
   perimeter compatibility and column0 brim controls. Qualify the wipe-tower
   initial-tool selection independently, including a later tool change, before
   accepting its snapshot projection. Source-owned internal equality boundary
   controls must not weaken any external exact comparison.
5. **Public comparison contract.** Replay identical inputs against the
   original strict complete-output comparator where its whole-stream premise is
   satisfied, plus the complete DTO/wire/config and selected runtime channels.
   Channel-only assertions and inherited residuals are identified separately.

Successful Native preconditions are required for qualified public comparisons.
Failed setup or Native premises do not establish reader parity. If a nonzero nozzle or initial-tool witness cannot be independently
qualified, the corresponding parity is not established; do not invent a selector or
leave a permanent first-column shell just to compile.

Artifacts must contain original 3MF hashes/bytes, Native repeated commands and
statuses, oracle/CLI/test identities, source provenance, full requested/effective
arrays, comparison receipts and unmodified strict output diagnostics. Runtime
catalog supply uses only original pinned documents through the shared source-document
adapter; bare requests stay genuine, including genuine errors.
