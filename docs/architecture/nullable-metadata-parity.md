# Nullable metadata parity for loaded preset vectors

## Status

This slice ports the pinned OrcaSlicer v2.5.0 nullable metadata contract to the
Ares registry. It is source-cited to the `PrintConfig.cpp` option declarations
and the `Config.cpp`/`Config.hpp` load path; it does not introduce a general
preset substitution mechanism.

Pinned upstream: OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
Base: `8b28296357e58d5f418c4540aa92c27d8b10c4df5`.
Destination: `crates/ares-core/src/options/registry/definitions/metadata.rs`.
Public verification seam: `ares_core::option_definition` and
`ares_core::option_metadata`.

## Upstream behavior

`bridge_speed` is registered as `coFloats` at
`OrcaSlicer/src/libslic3r/PrintConfig.cpp:2043`, and the same declaration sets
`def->nullable = true` at `:2053` with default
`ConfigOptionFloatsNullable{25}`. The option's enum-like wire type is therefore
`coFloats`, but its concrete option is nullable.

The construction rule is in `Config.cpp:285-316`: a `coFloats` definition with
`nullable` set creates `ConfigOptionFloatsNullable`. Its vector deserializer in
`Config.hpp:932-958`, including the `nil` branch at `:944-948`, stores a nil
sentinel instead of throwing. The final value is therefore the authored
nil-bearing vector, not the default and not a partially substituted vector.

The `load_settings` path is:

1. `OrcaSlicer.cpp:1373-1381` collects `--load-settings` files and fixes the
   compatibility rule to `ForwardCompatibilitySubstitutionRule::Enable`.
2. `OrcaSlicer.cpp:2211-2217` calls
   `ConfigBase::load_from_json(file, config_substitution_rule, ...)`.
3. `Config.cpp:866-872` creates the substitution context and returns its
   recorded substitutions.
4. `Config.cpp:927-929` routes a JSON string value to `set_deserialize`; a JSON
   string array is joined by `parse_str_arr` and reaches the same setter at
   `:1004-1060`.
5. `Config.cpp:618-624` throws only when `set_deserialize_nothrow` reports an
   invalid value. `Config.cpp:630-707` performs the generic decode and only
   enables default substitution for `coEnum`, `coEnums`, and `coBool` (plus the
   dedicated boolean-vector path). A float-vector nil is not handled by that
   substitution branch.
6. `Config.cpp:1146-1168` catches `std::exception` from the JSON load and writes
   `reason`; `OrcaSlicer.cpp:2219-2223` treats a non-empty reason as a config
   file error. This is the path for a genuinely non-nullable float, not the
   `bridge_speed` path because its concrete option is nullable.
7. On successful loading, `OrcaSlicer.cpp:2258-2264` reports only recorded
   `ConfigSubstitutions`. `OrcaSlicer.cpp:521-534` emits the stderr warning and
   `config_value_substituted` result warning for each entry.

The Native nil-bearing witnesses contain no substitution warning. Mixed vectors
retain `bridge_speed = nil,50` or `bridge_speed = 50,nil` in the exported
configuration footer; an all-nil vector is omitted there by the upstream
`is_nil()` policy. `result.json` likewise contains no `config_value_substituted`
warning for these cases. Ares already has the substitution transport
(`crates/ares-cli/src/failure.rs:72-126`), but this slice must not use it for
`bridge_speed`.

## Ares mismatch and minimal fix

The Ares typed definition already uses `FloatsNullable`:
`crates/ares-core/src/options/registry/definitions/table/early.rs:69`.
However, its independently maintained presentation metadata marks
`bridge_speed` as `nullable: false` at
`crates/ares-core/src/options/registry/definitions/metadata.rs`. The loaded
range validator mirrors upstream `PrintConfig.cpp:12622-12664` in
`crates/ares-core/src/options/validation/loaded.rs:23-36`; a float nil is valid
only when this metadata flag is true. The false flag produces the observed
`bridge_speed: ... not in range` rejection before slicing.

The fix marks every direct `PrintConfig.cpp` declaration with
`def->nullable = true` as nullable, as well as `small_support_perimeter_speed`
whose cloned default admits nil despite the missing definition flag. The config
export collector passes the active filament count only to the filament
group; printer/process/project vectors use their complete nullable vector when
computing `is_nil`, matching `ConfigOptionFloatsTempl::is_nil()` in
`OrcaSlicer/src/libslic3r/Config.hpp:932-948`. This preserves mixed vectors
such as `nil,50` in the full-config footer instead of incorrectly suppressing
them. No vector decoder, default value, substitution warning, or G-code
comparison waiver is added. The representative `bridge_speed` E2E witness
covers the whole-file stream, exit/stdout/stderr, and complete `result.json`
payload.

## Cloned defaults and nullable metadata audit

`DynamicConfig::optptr` (`Config.cpp:1743`) instantiates a missing option via
`ConfigOptionDef::create_default_option` (`Config.cpp:322-341`). When a default
object exists, its concrete class determines deserialization: the code clones
that object (reattaching the enum map when necessary) rather than using
`def->nullable` to select an empty option. The subsequent range check is
separate: `PrintConfig.cpp:12622-12664` validates `coFloats` elements using
`Config.cpp:344-349`, which rejects NaN if the definition flag is false. The
range switch does not include `coFloatsOrPercents`.

Auditing the concrete default types and definition flags in the pinned
`PrintConfigDef` found 73 direct nullable defaults: 70 with the flag set and
exactly three without it. Solid-plate loads in both Native modes distinguish
deserialization from slice admission:

| Option and declaration | Nil input | Mixed input |
| --- | --- | --- |
| `extruder_printable_height`, `PrintConfig.cpp:1151-1158` | Range rejection (238) | `nil,100`: range rejection (238) |
| `small_support_perimeter_speed`, `PrintConfig.cpp:2735-2745` | Success, omitted from `CONFIG_BLOCK` | `nil,50%`: success, exported verbatim |
| `small_support_perimeter_threshold`, `PrintConfig.cpp:2747-2754` | Range rejection (238) | `nil,10`: range rejection (238) |

Only the accepted support speed gets nullable metadata. The two `coFloats`
rows keep their false flags so loaded range validation still rejects nil. The
19 dynamic filament override rows are unchanged; their concrete default types
are assigned in the `add_nullable` factory at `PrintConfig.cpp:8788-8810`.
The audit found 31 direct rows already correct and 40 to correct:

```text
bridge_acceleration
bridge_speed
default_acceleration
default_jerk
default_junction_deviation
gap_infill_speed
infill_jerk
initial_layer_acceleration
initial_layer_infill_speed
initial_layer_jerk
initial_layer_speed
initial_layer_travel_acceleration
initial_layer_travel_jerk
initial_layer_travel_speed
inner_wall_acceleration
inner_wall_jerk
internal_bridge_speed
internal_solid_infill_acceleration
internal_solid_infill_speed
outer_wall_acceleration
outer_wall_jerk
overhang_1_4_speed
overhang_2_4_speed
overhang_3_4_speed
overhang_4_4_speed
small_perimeter_speed
small_perimeter_threshold
small_support_perimeter_speed
sparse_infill_acceleration
sparse_infill_speed
support_interface_speed
support_speed
top_solid_infill_flow_ratio
top_surface_acceleration
top_surface_jerk
top_surface_speed
travel_acceleration
travel_jerk
travel_speed
travel_speed_z
```

The public metadata consistency test enumerates the 71 direct upstream keys in scope,
asserts that each has an `option_definition`, and asserts through
`option_metadata(key).nullable` that every one is nullable. It is intentionally
not a private-table or generated-file-only test.

## E2E witness contract

The test uses the stock Creality process preset, the
`tests/parity/counterbore-partiallybridge/solid-plate.stl` fixture, and changes
one process option per case. For `bridge_speed`, it covers the following
values in both official and seeded Native lanes:

- `nil`;
- `nil,50`;
- `50,nil`;
- a known nullable vector with no nil token as the control.

For `small_support_perimeter_speed`, it covers `nil` and `nil,50%`. The plate
does not generate support, so this exercises load and export without choosing
a nil support speed at runtime.

Each case must compare complete ordered Native/Ares G-code after removing only
the existing guarded generator line, plus exit code, stdout, stderr, and
`result.json` while ignoring only documented timing fields. The expected Native
semantics are preserved nil sentinels and no substitution warning. The Ares
success path must expose the same serialized values and an empty substitution
list.

Deferred: genuinely non-nullable float-wire `nil` behavior, new substitution
classes, default-value substitution, inheritance completion, and unrelated
nullable option runtime behavior. No fallback or warning is added for a value
that upstream accepts as a nullable sentinel.
