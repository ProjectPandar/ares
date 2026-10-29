# Project-settings unknown enum values substitute defaults (EnableSilent)

## Upstream boundary

`Model::read_from_archive` / 3mf project loading defaults to a
silent forward-compatibility substitution context
(`Model.cpp:259` — `ConfigSubstitutionContext temp_config_substitutions_context(
ForwardCompatibilitySubstitutionRule::EnableSilent)`), which
`_BBS_3MF_Importer::_extract_project_config_from_archive`
(`Format/bbs_3mf.cpp:2632-2654`) passes into
`ConfigBase::load_from_json` (`Config.cpp:820+`). For enum options whose
string value is not in the option's key map
(`ConfigOptionEnum::from_string`, `Config.hpp:2032-2040`),
`ConfigBase::set_deserialize_raw` (`Config.cpp:657-676`) substitutes the
option's default value, marks the option substituted, and continues —
the load succeeds. The valid `seam_position` keys are `nearest`,
`aligned`, `aligned_back`, `back`, `random` (`PrintConfig.cpp:353-359`);
any other string (e.g. `rear`) is not an error but a silent default
substitution.

## Ares boundary

`crates/ares-core/src/project/load.rs:119-122` deserializes
`Metadata/project_settings.config` straight into `ProjectSettingsRaw`
via serde; `ProcessSeamPosition`
(`options/process_options/object_source/enums.rs:67-81`) has no unknown-
variant fallback, so `seam_position: "rear"` aborts the whole slice with
`InvalidInput("… unknown variant `rear` …")`. Unknown option *keys* are
already tolerated (remaining map); unknown enum *values* are not.

## Witness (RED candidate 2026-10-19)

Fixture `target-rearseam.3mf` (from the crosshatch-ordering control
template; subdivided box, `seam_position: "rear"`,
`gcode_label_objects: "0"`; sha256
86bd5b216652980a27bac87f0b234ee449b5b0dd4d8f8e16c987eb040a669dd5)：

- Native (official 2.4.2, 3 re-runs): slices successfully, silently
  substitutes the default — the emitted config reads
  `; seam_position = aligned` and the G-code is byte-identical to the
  same fixture with `seam_position: "aligned"`.
- Ares at aaf208139: `Error: InvalidInput("invalid project settings
  JSON: invalid Orca option seam_position: unknown variant `rear` …")`
  — no output at all.

## Included / deferred

### Type-map audit (2026-10-20)

Every ares coEnum/coEnums option's typed Rust enum variants were
compared against its substitution check-set (`enum_type_keys` when
present, else the option's `enum_values` whitelist). Options where the
whitelist is narrower than the type map and therefore carry explicit
`enum_type_keys`:

- `s_keys_map_InfillPattern` (PrintConfig.cpp:226-256):
  `sparse_infill_pattern`, `internal_solid_infill_pattern`,
  `top_surface_pattern`, `bottom_surface_pattern`, `ironing_pattern`,
  `support_ironing_pattern` (differences: monotonic/monotonicline et
  al. valid for the type, rejected per-option by `Slic3r::validate`).
- `s_keys_map_GCodeFlavor`: `gcode_flavor` (typed legacy keys
  makerware/sailfish/… parse; the admission checks reject them).
- Typed `ProjectBedType`: `curr_bed_type` (whitelist plus the
  ares-tracked "Default Plate" sentinel).

All other typed enum options (35 checked: seam_position, brim_type,
wall_sequence, z_hop_types/nozzle_type/extruder_type/
retract_lift_enforce/nozzle_volume_type/thumbnails_format vectors,
etc.) have check-set ⊇ typed variants with no difference. The remaining
enum-kind registry options deserialize through string/generic paths
(SLA-only or remaining-map options) where the per-option whitelist is
native's own key map, so whitelist-based substitution is exact.

Included: the `EnableSilent` coEnum **and coEnums** substitution of
`ConfigBase::set_deserialize_raw` (`Config.cpp:657-676`) at Ares'
project-settings deserialization boundary (`project/load.rs`): when an
enum option's value fails its variant parse and the load rule is not
`Disable`, the whole option is set to the definition default
(`opt->set(optdef->default_value)` — for coEnums the entire vector is
replaced, not just the failing element). Non-enum option types keep
failing the load. The substitution is generic over option type; no
per-option special cases. The 3mf project load uses the silent rule
because `Model.cpp:259` defaults the substitution context to
`ForwardCompatibilitySubstitutionRule::EnableSilent` when the caller
passes none. The native default for `seam_position` comes from
`PrintConfig.cpp` (`seam_position` def, default `spAligned`/`"aligned"`).
The test pins the native-observable behavior: slice succeeds and the
emitted config reports the substituted default.

Deferred: coBool substitution (`Config.cpp:641-652`,
`ConfigHelpers::enum_looks_like_true_value`), the recorded-substitution
report channel (`substitutions_ctxt.substitutions`), unknown-value
handling in preset documents and CLI overrides, and non-enum option type
mismatches. Those need their own witnesses.

## Secondary observation (recorded, not in scope)

`spRandom` + `staggered_inner_seams` on the same box shows a single-line
E-precision divergence (`E.0392` native vs `E.03921` ares, one line of
~9.5k) — candidate for a separate float-semantics investigation
(`pick_random_seam_point` position arithmetic, SeamPlacer.cpp:1005-1007
`edge.normalized() * picked_len` vs ares ratio form). The random hash
constants differ textually (43758.547 vs 43758.5453) but round to the
same f32; not a divergence.
