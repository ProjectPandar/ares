# Embedded coStrings import/export parity boundary

Status: prerequisite to the range-region port. The original
range fixture and strict comparison remain unchanged.

## Source-owned conclusion

PrintConfig.cpp:4193 declares inherits_group as coStrings. Config.hpp
ConfigOptionStrings::serialize delegates to Config.cpp:72-120
escape_strings_cstyle: semicolon-separated slots; quote any slot containing
space/tab/backslash/quote/CR/LF, and quote a single empty slot; escape backslash,
quote, CR and LF inside quotes. This is configuration serialization, not a
G-code numerical difference or an optional normalization rule.

Ares registry is already correct (`registry/definitions/table/middle_independent.rs`
Strings). Existing `config_export/value/strings.rs::escape_string_vector_element`
and OrcaStrings serializer already own the source rule. The defect is the
embedded raw export path: ProjectSettingsRaw::normalize_option_scalars decodes
scalar vectors but leaves raw JSON array strings as unescaped text; raw_settings
iter/token joins them without Strings semantics; config_export/writer.rs echoes
unmodeled inherits_group from that generic token. Profile-generated bookkeeping
already uses insert_string_slots and stored export-escaped tokens. Do not change
registry kind or manually patch quoted G-code lines.

## Minimal Rust destination

Normalize embedded JSON arrays and scalar vector wire inputs through one
registry-driven coStrings boundary in the existing raw/decode modules
(`options/json_deserialize.rs`, `project/raw_settings.rs` and its deserializer).
Every option whose registry kind is Strings must get the same
escape_strings_cstyle export representation, not an inherits_group special case.
Reuse the existing coStrings serializer; no hand-written quote rule in writer.

Raw List stores logical string elements and the option registry supplies the
type discriminator. Central raw rendering (`ProjectSettingsRaw::iter` and the
active-slot token-prefix accessor) derives coStrings wire tokens only on export.
Typed builders still receive unquoted logical text. Preserve raw element access and
slot count/order required by profile composition. Constructors for generated
profile bookkeeping must enter that same representation rather than pre-escape
slots and then pass them through normalization again. Writer/profile merging
must not perform a second escape or decode a semicolon embedded inside a slot.
Scalar wire decode and JSON-array decode must agree after normalization; generic
numeric/vector raw joining retains its current semantics.

This is a coStrings type-boundary correction, not a broad option-framework
refactor. Do not change the correct registry mapping or introduce a separate
writer option-name exception. External malformed strings remain validated at
parse/decode; internal normalized values are trusted.

## Affected registry options

The current registry declares these 34 Strings options, all covered by the same
boundary (not all are necessarily emitted or changed for a particular fixture):

adaptive_pressure_advance_model, compatible_machine_expression_group,
compatible_printers, compatible_prints, compatible_process_expression_group,
default_filament_colour, default_filament_profile, different_settings_to_system,
extruder_ams_count, extruder_colour, extruder_variant_list,
filament_change_extrusion_role_gcode, filament_colour, filament_colour_type,
filament_end_gcode, filament_extruder_variant, filament_ids,
filament_multi_colour, filament_notes, filament_ramming_parameters,
filament_settings_id, filament_start_gcode, filament_type, filament_vendor,
inherits_group, parallel_printheads_bed_exclude_areas, post_process,
preset_names, print_compatible_printers, print_extruder_variant,
printer_extruder_variant, small_area_infill_flow_compensation_model,
upward_compatible_machine, volumetric_speed_coefficients.

Existing typed options already serialize coStrings correctly; their logical
values must remain identical, not acquire literal quotes/backslashes. Changes
to raw-only exported coStrings are expected only when upstream quoting demands
them. Banned config keys remain banned, empty vectors and singleton/multi-empty
slots remain source-defined. No Stage2 region diff, numerical slicing change or
new G-code placeholder semantics belongs to this prerequisite.

## Empty coStrings profile JSON

`Preset::save` delegates to `ConfigBase::save_to_json`
(Preset.cpp:729-735). For vector options, Config.cpp:1483-1496 uses
`vec->vserialize()` and constructs a JSON array without dropping empty vectors.
`ConfigOptionStrings::vserialize` returns logical `values` unchanged
(Config.hpp:1140-1151), so an empty coStrings option is JSON `[]`, not scalar
`""`. This also applies to upward_compatible_machine generated for CLI import.

The loader accepts both representations: Config.cpp:927-929 sends a scalar
string to set_deserialize, while :950-1000 parses a Strings array into semicolon
wire form before set_deserialize. An empty array produces empty wire text;
ConfigOptionStrings::deserialize clears values (Config.hpp:1154-1159), and
unescape_strings_cstyle returns on empty input (Config.cpp:146-149). Both `""`
and `[]` therefore read as the same empty vector, even though profile JSON
export must use the canonical array shape. Do not preserve the scalar export
shape as a compatibility fallback.

## Verification conclusions required before acceptance

Use the existing unmodified project-array.3mf (10mm cube) and runtime OrcaRunner
Native. Independent baseline Native succeeds; actual inherits_group vector
contains a profile name with spaces and two empty slots. The public
slice_project_plate_validated output must fail complete generator-only equality
before production and pass afterward, including motion/M73 and every config line.
Runtime Native receipts remain outside the repository; historical CLI diagnostics
are not substitutes for a same-build runtime RED.

Before the applicable production change, runtime Native E2E must also establish
legal coStrings escaping cases (tab/backslash/quote/CR/LF, single empty slot and
multi-empty slots) at this same public seam, without weakening comparison or
modifying the original baseline fixture. Profile-entry controls must rule out
double quoting. Test source/input only enter the repository; Native bytes,
receipts, diagnostics and process records stay runtime-cache/artifact storage.

Acceptance requires fresh exact-main base/candidate seeded plus workspace
nextest, targeted E2E, clippy -D warnings/fmt and platform checks as requested.
Bind each changed option/output stream individually to runtime Native complete-
byte equality; full seeded differences in any of the other 33 coStrings options
must be explained per option and proved Native-correct, not called incidental
or hidden by a hash multiset summary. Unchanged motion/M73 and typed logical
string values remain invariant. No retry-to-PASS or fixture exceptions.
The region-range port then resumes its unchanged whole-output comparator;
fixing this prerequisite is not Stage2 GREEN.
