# Task 19B.2: Model and Layer Configuration Association

## Status

Draft for independent specification review. Implementation must not begin until
this document and its subsequent implementation plan each receive literal
`APPROVE` verdicts from an independent Agent and OpenCode.

## Goal

Port the fixed OrcaSlicer v2.4.2 model-configuration ownership and optional
layer-range import boundary into `ares-core` so that every configuration value
loaded from `Metadata/model_settings.config` or the optional
`Metadata/layer_config_ranges.xml` is legacy-normalized, classified, decoded,
and associated with the correct project object or volume before Task 19B.3
performs effective FDM normalization.

This is a source-cited Rust rewrite of `libslic3r::ModelConfig`, the BBS 3MF
importer, and their static object/region projection boundary. It is not a new
Ares slicing pipeline. The existing typed `ObjectOptionOverrides` and
`RegionOptionOverrides` are the Rust lowering of the subset of a fixed
`DynamicPrintConfig` that later `PrintObjectConfig` and `PrintRegionConfig`
actually consume.

The current KSR fixture has one model-settings object with source ID `2`, one
part with source ID `1`, object-level `extruder=1`, and no layer-range resource.
Those facts are test observations only. Production behavior must be derived
solely from the supplied 3MF bytes and must work for other IDs, object orders,
options, and range documents.

## Fixed upstream boundary

All citations refer to OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

### Canonical option lookup and lexical decode

- `src/libslic3r/PrintConfig.cpp:663-672`: `PrintConfigDef` construction.
- `src/libslic3r/PrintConfig.cpp:674-932`: common option definitions.
- `src/libslic3r/PrintConfig.cpp:934-7328`: FFF option definitions.
- `src/libslic3r/PrintConfig.cpp:7395-8031`: SLA option definitions that are
  still part of the global `PrintConfigDef` accepted by `ModelConfig`.
- `src/libslic3r/PrintConfig.cpp:4504-4567`: generated XYZE machine-limit
  registrations.
- `src/libslic3r/PrintConfig.cpp:63-84,7288-7326`: generated nullable filament
  override registrations.
- `src/libslic3r/Config.cpp:573-685`:
  `set_deserialize_nothrow`, `set_deserialize`, and `set_deserialize_raw`
  legacy handling, canonical lookup, alias/shortcut dispatch, typed option
  creation, and concrete lexical decode.
- `src/libslic3r/Config.cpp:258-318` and `Config.hpp:954-972`:
  empty/default option construction and the implicit zero value serialized by
  an `coInt` definition with no explicit default object.
- `src/libslic3r/Config.hpp`: scalar, nullable, vector, string, enum, point, and
  point-group `ConfigOption*::deserialize` wire grammars.
- `src/libslic3r/PrintConfig.cpp:402-419,481-485`: exact lexical token maps for
  the five canonical enums that have no object/region owner.
- `src/libslic3r/PrintConfig.cpp:8033-8338`: fixed legacy handling already
  ported by Task 19A.

Within the fixed `PrintConfigDef` constructor/init methods and their generated
axis/filament registration loops cited above, the Task 19A canonical boundary
contains exactly 751 unique keys. This count does not include `this->add`
calls owned by other config definitions later in `PrintConfig.cpp`. The current
Ares registry has 736 unique keys: 733 fixed canonical keys and three
legacy-only inputs. This task closes that boundary by adding these 18 canonical
definitions with their exact Ares serialized defaults:

All 16 generated nullable filament override keys are already among those 733
current fixed canonical rows, including `filament_retract_before_wipe`,
`filament_long_retractions_when_cut`, and
`filament_retraction_distances_when_cut` at `PrintConfig.cpp:81-83`; they do not
add three more rows to the Task 19B.2 delta.

| Key | Fixed type | Serialized default |
| --- | --- | --- |
| `bottom_surface_filament_id` | `coInt` | `0` |
| `bridge_line_width` | `coFloatOrPercent` | `100%` |
| `chamber_minimal_temperature` | `coInts` | `0` |
| `extruder` | `coInt` | `0` |
| `flashforge_serial_number` | `coString` | empty string |
| `inner_wall_filament_id` | `coInt` | `0` |
| `internal_solid_filament_id` | `coInt` | `0` |
| `lightning_overhang_angle` | `coFloat` | `45` |
| `lightning_prune_angle` | `coFloat` | `45` |
| `lightning_straightening_angle` | `coFloat` | `45` |
| `outer_wall_filament_id` | `coInt` | `0` |
| `parallel_printheads_bed_exclude_areas` | `coStrings` | empty string |
| `parallel_printheads_count` | `coInt` | `1` |
| `relative_bridge_angle` | `coBool` | `false` |
| `sparse_infill_filament_id` | `coInt` | `0` |
| `support_parallel_printheads` | `coBool` | `false` |
| `top_surface_filament_id` | `coInt` | `0` |
| `use_3mf` | `coBool` | `false` |

Fixed `extruder` has no explicit `set_default_value` call at
`PrintConfig.cpp:2200-2213`; `ConfigOptionDef::create_default_option` therefore
creates an empty `ConfigOptionInt`, whose constructor value and serialization
are `0`. Ares records that effective fixed serialized default rather than
inventing a sentinel for the absent C++ pointer. This registry default does not
materialize a missing sparse model key: absent `extruder` remains absent, while
an explicitly decoded `0` is stored and means inherit/default filament.

It removes `solid_infill_filament`, `sparse_infill_filament`, and
`wall_filament` from the canonical registry. They remain accepted only through
the Task 19A legacy-rule ledger and lower to their fixed `*_filament_id`
targets. The resulting registry has exactly 751 sorted, unique canonical keys
and the fixed type histogram:

| Type | Count | Type | Count |
| --- | ---: | --- | ---: |
| `coBool` | 117 | `coBools` | 22 |
| `coEnum` | 49 | `coEnums` | 9 |
| `coFloat` | 210 | `coFloatOrPercent` | 36 |
| `coFloats` | 92 | `coInt` | 47 |
| `coInts` | 45 | `coPercent` | 26 |
| `coPercents` | 5 | `coPoint` | 4 |
| `coPoints` | 6 | `coPointsGroups` | 1 |
| `coString` | 48 | `coStrings` | 34 |

The fixed definition has one registered alias,
`perimeter_feed_rate -> inner_wall_speed`; Orca reaches it through the alias
scan in `Config.cpp:603-626`. There are no fixed registered shortcuts. Ares'
no-legacy-fallback rule intentionally leaves that alias unported in this slice:
Task 19B.2 must continue to reject `perimeter_feed_rate` with its exact source
name and must not add a generic alias or shortcut mechanism.

Object/region sparse ownership is narrower than the five typed
project-settings groups. After an object/region owner miss, 18 scalar enum keys
still have these existing concrete typed project-settings destinations:

| Typed destination | Scalar enum keys |
| --- | --- |
| `printer.gcode` | `bed_temperature_formula`, `enable_power_loss_recovery`, `gcode_flavor`, `printer_structure`, `wipe_tower_type` |
| `printer.machine` | `input_shaping_type` |
| `printer.remaining` | `host_type`, `printer_technology`, `printhost_authorization_type`, `thumbnails_format` |
| `process.print` | `draft_shield`, `print_order`, `print_sequence`, `skirt_type`, `timelapse_type`, `wipe_tower_wall_type` |
| `project.gcode` | `filament_map_mode` |
| `project.print` | `curr_bed_type` |

All nine fixed `coEnums` keys likewise miss object/region ownership but have
existing concrete typed project-settings destinations:

| Typed destination | Enum-vector keys |
| --- | --- |
| `printer.gcode` | `extruder_type`, `nozzle_type`, `retract_lift_enforce`, `z_hop_types` |
| `printer.remaining` | `default_nozzle_volume_type` |
| `filament.print` | `overhang_fan_threshold` |
| `filament.retract_overrides` | `filament_retract_lift_enforce`, `filament_z_hop_types` |
| `project.gcode` | `nozzle_volume_type` |

Model/layer classification must convert the fixed metadata string wire form to
the concrete scalar or sequence deserializer and call the existing
`ProjectSettingsBuilder::deserialize_known_value` path for these keys. The
temporary builder is discarded without `resolve`; this reuses every existing
typed enum token domain and nullable-vector codec without creating a second
ledger or production owner. It also provides the first discard validation
choice for every other project-settings-owned key that has no object/region
owner.

For the nine enum-vector keys, the metadata-wire adapter follows fixed
`ConfigOptionEnumsGenericTempl::deserialize`: split on commas, trim each
element, treat empty input as an empty vector, do not append an element for a
trailing comma, and accept `nil` only for the three nullable definitions
`nozzle_type`, `filament_retract_lift_enforce`, and
`filament_z_hop_types`. After element trimming, enum tokens remain exact and
case-sensitive.

Only the 101 canonical keys with no typed project-settings owner fall through
to registry-kind validation. That complement contains five `coEnum` keys, so
add one private, sorted, compile-time lexical ledger used only by model/layer
classification; do not widen the public `OptionDefinition` API or create an
erased option value. The exact entries are:

| Key | Accepted tokens | Default |
| --- | --- | --- |
| `display_orientation` | `landscape`, `portrait` | `portrait` |
| `first_layer_sequence_choice` | `Auto`, `Customize` | `Auto` |
| `material_print_speed` | `slow`, `fast` | `fast` |
| `other_layers_sequence_choice` | `Auto`, `Customize` | `Auto` |
| `support_pillar_connection_mode` | `zigzag`, `cross`, `dynamic` | `dynamic` |

Tokens are case-sensitive, are not trimmed, and do not accept integer ordinals
or UI-label variants, matching `ConfigOptionEnumGeneric::deserialize`. An
invalid token is a strict bounded input error naming the option and value; Ares
does not apply Orca's forward-compatibility substitution at this external
boundary. Therefore all 23 scalar discard-only enum domains are concrete: 18
reuse typed project-settings deserializers and five use this fixed ledger. All
nine enum-vector domains reuse typed project-settings deserializers.

### Model settings import and identity

- `src/libslic3r/Format/bbs_3mf.cpp:744-764,3575-3672`: model geometry identity
  is `(model path, object id)`.
- `src/libslic3r/Format/bbs_3mf.cpp:3893-3908,4136-4165`: final
  `ModelObject` order is build-first-occurrence order.
- `src/libslic3r/Format/bbs_3mf.cpp:828-832,867-927,3440-3513`: transient
  object/part configuration document state.
- `src/libslic3r/Format/bbs_3mf.cpp:4263-4400`: object and part IDs plus
  source-ordered metadata collection.
- `src/libslic3r/Format/bbs_3mf.cpp:2067-2168`: object metadata association;
  `name` and `module` are structural and all other keys enter `ModelConfig`.
- `src/libslic3r/Format/bbs_3mf.cpp:2043-2056,2156-2160,3597-3605,3719-3735`:
  no-settings object `pid`, production color-group collection, and the derived
  one-based object `extruder` fallback.
- `src/libslic3r/Format/bbs_3mf.cpp:4894-4954,5081-5126`: part association by
  mesh subobject ID, default metadata for unmatched leaf meshes, volume
  type/name/source metadata, remaining `ModelConfig` entries, and unnamed
  fallback.
- `src/libslic3r/Model.hpp:354-370,865-918`: `ModelObject` and `ModelVolume`
  own their model configurations.
- `src/libslic3r/Model.cpp:2717-2747`: the five supported model-volume types.
- `src/libslic3r/PrintConfig.hpp:2034-2128`: `ModelConfig` ownership around a
  `DynamicPrintConfig`.

Model settings use a bare source object ID. Parts attach by leaf mesh object ID,
not by their XML array position. Ares must preserve its existing stricter
external-boundary rejection when one bare object ID ambiguously maps to
different model paths.

The fixed volume-type vocabulary lowers to one project-domain enum:

```rust
pub enum ProjectVolumeType {
    ModelPart,
    NegativeVolume,
    ParameterModifier,
    SupportEnforcer,
    SupportBlocker,
}
```

The wire spellings are `normal_part`, `negative_part`, `modifier_part`,
`support_enforcer`, and `support_blocker`. Orca falls back from unknown strings
to model part; Ares intentionally rejects an unknown spelling because this
repository requires no legacy fallback. The error must name the offending
`subtype`, `volume_type`, or `part_type` key/value.

Volume type ordering is source ordered: the `<part subtype>` initializes the
type; each later `volume_type` or `part_type` metadata assignment replaces it;
the last assignment wins. Part `name` follows the same last-write-wins rule.
Part `matrix`, `mesh_shared`, and the fixed `source_*` metadata remain
structural provenance rather than options. At object scope only `name` and
`module` are structural. A structural-looking key used at the wrong scope must
go through option classification and fail if it is not canonical; the former
shared broad structural allow-list must be removed.

Object and volume naming also preserve the fixed importer fallback. When a
matching model-settings object exists, its ordered `name` metadata supplies the
object name and an absent name leaves it empty. When no matching model-settings
object exists, the object uses the model XML name when non-empty, otherwise
`Object_{ordinal}` with the one-based final object ordinal. After all metadata
for a volume is processed, an empty volume name becomes the final object name.
Only unnamed volumes increment the fallback counter: the first receives the
bare object name, the second receives `{object_name}_2`, then `_3`, and so on;
explicitly named volumes neither consume nor reset that counter. This behavior
is applied during domain assembly, where final object order and volume metadata
are both available, and does not require a later document re-join.

Part metadata is optional per leaf mesh. Fixed Orca first accepts the part at
the same leaf-volume index when its subobject ID matches, otherwise searches
the source-ordered part list from the beginning for the first matching leaf ID.
Repeated part IDs are not rejected: the same-index fast path may select a later
duplicate, while the fallback search selects the first duplicate. If no part
matches, create default volume metadata for that leaf: `ModelPart`, identity
source transform, empty metadata/config/name, zero mesh statistics, no
text/emboss data, and default source provenance (`input_file=""`, object and
volume indices `-1`, zero offset, and false flags). The accumulated component
transform and mesh remain unchanged. This default applies both when the object
has no model-settings record and when a present object record omits or does not
match a leaf part. Extra unmatched part records do not create volumes.
Defaulted volumes then participate in the same unnamed-volume counter described
above. Replace the current Ares missing-object, missing-part, and duplicate-part
assembly errors with this source behavior; do not synthesize a part from
fixture facts.

When an object has no model-settings record, derive its optional object-level
`extruder` override from the model XML object's `pid` and production-extension
color groups (`bbs_3mf.cpp:2043-2056,2156-2160,3597-3605,3719-3735`). Within
one group the last color wins. Merge submodel group maps in graph/importer order
with the first duplicate group ID retained, then let the root-model group map
replace any same-ID entry. Iterate group IDs in numeric ascending order; assign
one-based extruder IDs to distinct exact color strings in first-seen order, and
reuse an ID for an exactly equal color. A missing or unparsable object `pid`
acts as `0`; an unmapped group yields no override. Ignore `pindex`. A matching
model-settings object suppresses this fallback even when that record omits an
`extruder` option. Store a mapped positive result in the object's canonical
`RegionOptionOverrides.extruder` field.

### Optional layer configuration ranges

- `src/libslic3r/Format/bbs_3mf.cpp:209-216,1896-1904`: canonical optional
  archive path `Metadata/layer_config_ranges.xml`.
- `src/libslic3r/Format/bbs_3mf.cpp:2886-2940`: XML import of
  `<objects>/<object id>/<range min_z max_z>/<option opt_key>`.
- `src/libslic3r/Slicing.hpp:150-151`: sorted `(min_z,max_z)` range map.
- `src/libslic3r/Format/bbs_3mf.cpp:2087-2095`: association to final model
  objects.
- `src/libslic3r/Format/bbs_3mf.cpp:7517-7545`: exporter proof that layer
  object IDs are one-based final `ModelObject` ordinals.
- `src/libslic3r/PrintApply.cpp:342-383`: later gap/overlap normalization,
  explicitly deferred to Task 19B.3.

The optional resource is read only through the existing bounded in-memory
`ProjectArchive`. Match `Metadata/layer_config_ranges.xml` with ASCII
case-insensitive path equality, mirroring fixed `boost::algorithm::iequals`.
Zero matches produce no error and an empty associated range set; one match is
read through its exact validated `PackagePath`; multiple case variants are a
strict ambiguous-input error naming the canonical document path. The latter is
an intentional deterministic Ares deviation from Orca's central-directory
ordered partial merge: do not select a `BTreeMap` first match. Preserve the
existing `PackagePath` normalization and backslash rejection; do not add a raw
archive-name fallback for this optional resource. No native filesystem path,
UI, terminal, FFI, or platform-specific API may be added to `ares-core`.

Unlike model settings, `<object id>` in this document is not a source 3MF ID.
It is a one-based index into final build-created `Project.objects()` order.
Ordinal `1` must therefore attach to the sole object even when that object's
source ID is `2` or `42`.

Options within a range are processed in XML source order and later duplicate
assignments win. Ranges are retained in lexicographic `(min_z,max_z)` order.
When the exact same finite pair occurs more than once, the later complete range
configuration replaces the earlier one. Empty object range groups produce no
stored state. Negative bounds, reversed bounds, gaps, and overlaps remain raw;
Task 19B.2 must not normalize or reject them.

At the untrusted 3MF boundary Ares intentionally uses one strict error model
where fixed Orca inconsistently logs, skips, or throws. Malformed XML, missing
attributes, zero or out-of-range ordinals, duplicate object ordinals,
non-finite bounds, unknown option names, and malformed option values return a
bounded `SliceError::InvalidInput` naming the document and relevant key or
ordinal. This strictness is not described as byte-for-byte reproduction of
Orca's recovery behavior.

## Rust destination and ownership

### Canonical model-option classifier

Add a private `options/model_config_deserialize.rs` boundary used by object,
part/volume, and layer-range imports. It runs Task 19A model-path legacy
handling first, then performs exactly one of these actions:

1. At object scope, assign a canonical object key to
   `ObjectOptionOverrides`; if it is not an object key, assign a canonical
   region key to `RegionOptionOverrides`.
2. At part or layer scope, assign only a canonical region key to
   `RegionOptionOverrides`.
3. If a canonical key has a concrete typed project-settings owner but no owner
   in this model scope, validate it through a temporary
   `ProjectSettingsBuilder` field deserializer and discard it.
4. If a canonical key is in the complete 751-key registry but has none of the
   typed owners above, validate its concrete `OptionValueKind` lexical form,
   using the five-entry enum ledger where applicable, and discard it. Actions 3
   and 4 match fixed downstream static projection: accepted `ModelConfig`
   entries outside `PrintObjectConfig`/`PrintRegionConfig` do not affect FDM
   object or region state.
5. Reject a still-unknown key and include the exact original source name.

The classifier must not construct or retain `serde_json::Value`, an erased
option enum, `BTreeMap<String, ...>`, a generic dynamic config, or a fixture
special case. A private metadata-wire adapter may feed a `StringDeserializer`
or `SeqDeserializer` into the typed project-settings builder; registry-kind
validation may instantiate a concrete typed value temporarily and discard it.
The five registry-only enum keys additionally consult the fixed lexical ledger
above. Storage remains only in the existing typed sparse owners.

The five typed project-settings groups cover 650 canonical keys. The remaining
101 canonical keys must still be recognized by the classifier and validated
even though most are irrelevant to FDM object/region projection. This 650/101
partition is independent of object/region ownership. In particular, canonical
`extruder` is added to the registry and continues to use the existing special
typed region field.

Task 19A previously deferred four profile/UI rules. At this final model-config
boundary their importer behavior is completed without adding profile state:

- `inherits_cummulative` lowers to canonical `inherits_group`;
- `compatible_printers_condition_cummulative` lowers to canonical
  `compatible_machine_expression_group`;
- `compatible_prints_condition_cummulative` lowers to canonical
  `compatible_process_expression_group`;
- canonical `different_settings_to_system` is concrete-lexically validated.

All four canonical targets are then discarded because neither fixed static
object nor region projection consumes them. Task 19A's top-level project JSON
profile/UI behavior remains unchanged. Other obsolete, consumed, executable,
and explicitly invalid legacy rules retain the already approved Task 19A
semantics. After this completion there are no remaining
`DeferredProfileBookkeeping` rules in the model/layer classifier.

Within one owner, canonical and legacy spellings share the final canonical
field and XML source order is authoritative: a later assignment replaces an
earlier one. Model XML does not adopt Task 18's duplicate-key rejection because
fixed `ModelConfig::set_deserialize` is assignment based.

### Project-domain ownership

Move production model configuration ownership into the source-faithful domain
objects:

- The typed model XML object record retains its `name` attribute, defaulting to
  empty, so no-settings object fallback has the fixed source value available
  during assembly.
- Typed model XML also retains the object's optional `pid` attribute and the
  production-extension color groups needed by the no-settings fallback above;
  it does not add a generic resource map.
- `ProjectObject` owns its model-settings `name`, `module`,
  `ObjectOptionOverrides`, object-level `RegionOptionOverrides`, and sorted
  typed layer ranges.
- `ProjectVolume` owns its part `name`, `ProjectVolumeType`, and volume-level
  `RegionOptionOverrides` in addition to existing mesh/path/transform state.
- Layer range records own finite `min_z`, finite `max_z`, and one
  `RegionOptionOverrides`.

Assembly associates object settings by bare source object ID and volume
settings by leaf mesh object ID, while preserving existing path-aware geometry
identity and ambiguity rejection. Repeated build instances continue to share
one configured `ProjectObject`.

Retain `ModelSettings` as the typed import/provenance document for non-slicing
`source_*`, `mesh_shared`, and `mesh_stat` data that are not yet projected into
public project-domain structures. Production normalization must not later
re-join configuration by walking `ProjectDocuments.model_settings`.
Configuration has one production owner after assembly: the corresponding
project-domain object, volume, or layer range. The retained import document is
not an effective-option source; retain the other already required documents
unchanged.

The minimal new project module is
`project/layer_config_ranges.rs`, split further only if a Rust source file would
otherwise reach 400 physical lines. Existing modules must also stay below 400
physical lines.

## Required behavior and tests

Implementation follows TDD. Each implementation slice begins with a genuine
RED caused by missing or incorrect production behavior, reaches focused GREEN,
then receives independent spec-compliance and code-quality review before the
next dependent slice.

### Registry and classifier proof

- Freeze the 751-key unique sorted registry and exact type histogram without
  reading an Orca checkout at test runtime.
- Prove all 18 missing canonical rows with the exact serialized defaults above
  and the absence of the three legacy-only rows.
- Exercise every `OptionValueKind` lexical branch with valid and invalid XML
  metadata forms, including nullable elements, escaped scalar/vector strings,
  points, and point groups, through an owner field or a focused private lexical
  validator test.
- Prove all 126 object fields and all 149 region fields plus `extruder` route to
  their existing concrete sparse owners with no unclassified overlap.
- Prove valid and invalid model/layer metadata for all 18 typed scalar
  discard-only enums and all nine typed enum-vector keys, including invalid
  vector elements and nullable `nil` forms where applicable. The assertions
  must cover each concrete type's complete accepted token domain, demonstrate
  dispatch through the existing concrete project-settings field types, and
  leave no retained project setting.
- Prove valid and invalid canonical non-owner values for each of the eight kinds
  actually present in the 101-key complement: `coBool`, `coEnum`, `coFloat`,
  `coFloats`, `coInt`, `coPercent`, `coString`, and `coStrings`. Prove all five
  non-owner enum token sets and defaults exactly; malformed values name their
  keys.
- Prove unknown keys and `perimeter_feed_rate` report bounded exact-name errors.
- Prove the three cumulative model-path rules and canonical
  `different_settings_to_system` complete as specified.
- Prove canonical/legacy duplicates are last-write-wins in XML order.

### Model settings and domain association proof

- Use synthetic in-memory 3MFs where model-settings object order differs from
  final build order and source object IDs differ from build ordinals.
- Use component graphs where part XML order differs from leaf mesh/volume order;
  associate volume config by leaf part ID.
- Prove object and volume sparse configs, names, module, and type attach to the
  correct domain owner.
- Prove volume subtype initialization, ordered `volume_type`/`part_type`
  replacement, all five type spellings, and strict unknown-type errors.
- Prove no-settings object-name fallback and the unnamed-volume sequence with
  named volumes interleaved: object name, `_2`, `_3`, with named volumes not
  incrementing the counter.
- Prove an absent object-settings record and a present record with an omitted or
  unmatched leaf part each create default `ModelPart` volumes with identity
  source transforms, empty overrides, default/zero provenance and statistics,
  preserved component transforms, and correct unnamed numbering. Prove that a
  same-index duplicate part ID wins at that index, while a fallback search uses
  the first source-ordered match. Extra unmatched part records do not create
  volumes, and repeated IDs are not rejected merely for being repeated.
- Prove the no-settings `pid`/production-color fallback: per-group last color,
  submodel first-ID merge, root replacement, numeric group ordering, exact
  color deduplication, one-based extruders, missing/invalid/unmapped `pid`,
  ignored `pindex`, and suppression by any matching model-settings object.
- Prove object-only and part-only structural metadata scopes; wrong-scope
  pseudo-structural keys must not disappear silently.
- Preserve existing repeated-instance, distinct-path, and ambiguous bare-ID
  behavior.

### Layer-range proof

- Absence of the optional archive entry yields empty ranges.
- A single case-variant archive path is accepted; multiple case variants are a
  bounded ambiguous-input error.
- Ordinal `1` attaches to the first final project object even when its source ID
  is `42`; multiple ordinals follow final build-created order.
- Range output is lexicographically sorted; duplicate exact ranges and
  duplicate options use the later complete assignment.
- Negative, reversed, gapped, and overlapping finite ranges remain unnormalized.
- Empty object groups add no state.
- Invalid/duplicate/out-of-range ordinals, missing bounds, non-finite bounds,
  malformed XML, unknown options, and invalid values return bounded keyed
  errors.

### Real KSR and boundary regression proof

Load the real project through the public byte-oriented path and prove:

- one final object has source ID `2` and one volume has source ID `1`;
- object-level `extruder=1` is present in its typed region overrides;
- the volume is `ModelPart` with the fixture-derived name/configuration;
- the optional layer range set is empty;
- no production value is supplied by test code, the reference G-code, fixture
  name/hash recognition, or an Orca executable.

The existing public `slice_project` and generated browser WASM package must
still reach `ProjectSlicingIncomplete` only after the newly strict project load
succeeds. The existing complete CLI golden remains configured skipped. No test
may weaken its comparison contract or claim G-code parity.

## Error contract

Errors are compact and bounded. They must include the relevant original option
key, metadata key, range attribute, object/part ID, or layer ordinal, but must
not include an entire XML document, 3MF archive, or G-code buffer. Archive
errors remain wrapped through the existing project-document context, for
example `invalid project layer configuration ranges XML: ...`.

## Explicitly deferred

- `normalize_fdm`, `normalize_fdm_1`, `normalize_fdm_2`, active
  extruder/filament sizing, used-filament discovery, and source-ordered
  effective `FullPrintConfig` orchestration (Task 19B.3).
- Production `ObjectOptions::resolve` / `RegionOptions::resolve` calls and the
  object -> volume -> material -> layer effective merge (Task 19B.3). Existing
  pure typed precedence tests remain unchanged.
- Gap/overlap normalization of raw layer ranges (Task 19B.3).
- A material-config archive reader. Fixed BBS 3MF has no such document;
  optional material precedence remains a pure typed boundary.
- Projection of non-slicing `source_*`, `mesh_shared`, and `mesh_stat`
  provenance into public project-domain structures, plus shared-mesh storage
  reuse. The typed import document remains available and existing mesh/source
  transform behavior remains intact; these structural keys never enter option
  classification.
- Modifier-parent discovery, bounding-box intersection, painted/fuzzy region
  construction, and region deduplication.
- Config-block serialization (Task 19C).
- Dynamic `SliceOptions` consumer migration/removal (Tasks 20A-20E).
- Geometry slicing, toolpaths, G-code generation, and final golden parity.

## Verification and release gate

Before implementation approval, run focused Task 19B.2 tests plus the adjacent
typed model/object/region tests, all workspace Nextest tests, the dynamic-value
audit, rustfmt, warning-denying Clippy, native and WASM checks/builds,
`wasm-bindgen`, and the browser real-3MF test. Preserve both fixture SHA-256
hashes and verify no production code reads the reference G-code, recognizes
fixture identity, invokes Orca, adds native I/O, or adds erased/dynamic option
storage.

The frozen implementation diff must receive independent whole-specification
and whole-code-quality `APPROVE` verdicts and OpenCode `APPROVE`. Only then may
architecture and roadmap documentation be updated and independently approved.
After fresh release verification, create one Conventional Commit, push the
current branch, and require format, Linux, WASM, macOS, and Windows jobs green
for the exact pushed SHA before Task 19B.3 begins.
