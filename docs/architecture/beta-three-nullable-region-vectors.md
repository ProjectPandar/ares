# Beta three nullable region vectors

## Upstream boundary and Rust destination

Pinned libslic3r revision345efec64c7b95a05ce5cf96df0dde66a158835c.
This is one rewrite slice for three existing options, not a new Ares pipeline:

- PrintConfig.cpp1959-1965 / PrintConfig.hpp1567:
  enable_overhang_speed, coBools/nullable, default{true}.
- PrintConfig.cpp1967-1990 / PrintConfig.hpp1601:
  slowdown_for_curled_perimeters, coBools/nullable, default{false}.
- PrintConfig.cpp2702-2710 / PrintConfig.hpp1470:
  outer_wall_speed, coFloats/nullable, default{60}, min1.
- PrintConfig.cpp10103-10117: all three belong to print_options_with_variant.
- Config.hpp676-682: get_at selects indexed element, or the first for an
  out-of-range index; it is not unconditional first-element projection.
  Numeric/bool nullable classes own nil parsing/serialization/is_nil.
- GCode.cpp2157,5055-5061,6969,9276+,9553-9622: current filament's mapped
  nozzle-config index selects motion speed and Boolean behavior.
- GCode.cpp5080-5095: precomputation uses any_enabled across the vector,
  distinct from per-current-nozzle motion selection.

Rust destination: options registry/selected metadata, config codec and nullable
vector types; ProcessRegionSourceOptions and region override/merge paths;
existing selected-region/motion consumers and existing nozzle variant resolver.
The OrcaFloat/OrcaBool scalar compatibility shells in region_fields.rs and their
registry kinds are replaced with nullable vectors, not preserved behind
scalar/vector equality relaxations.

## Included and deferred behavior

Include public canonical vector DTO shape, source singleton defaults, nullable
wire/JSON semantics without loss of order/length, project import and effective
application, and existing runtime source selection for these three keys.
Trace nil inheritance/default application before implementing it; neither false
nor60 is an invented generic substitute for a nil slot. Distinguish global
any_enabled precomputation from selected current-nozzle flags.

Defer the other40existing scalar-to-vector migrations, active multi-nozzle
solver/grouping, fitting/overhang algorithm rewrites and unrelated metadata
refreshes. Only the three owning metadata records may change for this slice.
The old strict requested/effective shape guard remains authoritative. No
flattening, array/scalar equivalence, fixture exception, tolerance widening,
Native output table or new production fallback.

## Public verification seams

First tracer: Native --export-settings on a runtime project variant
feeds all three canonical values to public ProcessRegionSourceOptions,
SliceOptions, print_config_def_json and decode/encode APIs. Compare the full
Native wire-string arrays and type/default/nil declarations. The public codec's
JSON is deliberately typed bool/number/null, whereas Native settings JSON
contains vserialize strings. Compare complete typed codec arrays derived from
Native values and require byte-exact encoding back to the Native wire; do not
compare those two JSON representations directly or change codec output to
wire strings. DTO and project-import checks still compare complete Native
wire-string arrays. Report all three gaps in one artifact before asserting.
Do not call this a legacy-import leaf-DTO seam.

Subsequent public witnesses use load_project for actual project/default/nil
application and slice_project_plate_validated output for mapped-nozzle effects,
with independently qualified Native output on the identical input. A nozzle
witness must establish selection of a nonzero index with distinct vector
entries and stable Native replay; merely observing exported arrays or reading
an internal resolver cannot qualify runtime selection.

Use existing config_block_roundtrip grouped registration; no ROOT registration
or resurrected B12 tests. Keep runtime input/output/producer/JSON diagnostics as
repeatable external artifacts. No Native .gcode/goldens committed.

## Failure inventory before isolated DTO/default/nil assertions

- Scalar registry or DTO rejects Native arrays, or silently reduces them.
- Incorrect singleton defaults or metadata nil/type flags.
- Nil becomes false/zero, disappears from mixed vectors or loses position.
- Canonical vectors change length/order on encode/import/export.
- Nil override incorrectly clobbers a parent/default or is always filled.
- Runtime selects an unconditional first or logical-filament slot instead of
  the source-selected nozzle slot.
- Out-of-range get_at or empty/default normalization differs from source.
- any_enabled precomputation is mistaken for selected-nozzle behavior.
- A later failure is relabelled PASS merely because shape admission unblocks.

First qualify Native and public RED, then commit RED. No production migration
before that artifact. Exact affected surviving ledger replays remain bounded
and separately account downstream residuals; no FULL.

## Qualified runtime selection boundary

The bounded nozzle witness starts from unchanged nozzle-flush-dataset/D0.3mf,
uses Manual filament mapping in project and plate settings, disables the prime
tower and layer cooling, and authors distinct four-column process vectors.
PrintApply.cpp:1429 and PrintConfig.cpp:11661-11860 gather the physical columns:
outer speeds [17,21,33,47] become [17,33], Boolean vectors become [0,1].
Mapping the same logical filament to physical extruder 2 selects configuration
slot 1. The independently replayed Native outer-wall channel contains F1980
rather than physical-extruder-1 F1020; the original public RED emitted F1020
in both cases. This is a selected-feedrate witness, not a replacement
whole-G-code comparator or complete stream parity claim.

Default/nil witnesses use Native export with no input for true source defaults
[1]/[0]/[60], and runtime 3MF variants for mixed/all-nil DTO and load_project
retention. Removing a key from a historical project does not prove registry
defaults because the importer can seed profile-specific values. Source
PrintConfig.hpp:887-890 explicitly includes the nonzero nil Boolean sentinel
in any_enabled; Config.hpp:1992 does the same for const get_at. Preserve that
behavior rather than invent nil-to-false or float-nil-to-60 substitution.

## Bounded runtime and region-override rules

Print.cpp:5008-5015,5153-5155 defines the ordinary path without a layered
nozzle-group producer: the current logical filament maps to the physical
extruder column, and an out-of-range filament argument returns physical0.
Do not invent a canonical-variant search in that path. Existing printer/process
variant gathering remains owned by project_variants; active layered grouping
and its distinct get_config_index_base path remain deferred.

GCode.cpp:760-774,4034-4037 selects the physical extruder for the initial
outer_wall_volumetric_speed placeholder (and corresponding nozzle diameter),
not unconditional column0. Retain the existing initial-filament compatibility
shell; this does not introduce a new ToolOrdering engine. Model.cpp:3218,3485
explicitly selects column0 for the brim speed table. Layer.cpp:168 passes the
authored outer_wall_filament_id unchanged to Print::get_extruder_id; retain
that literal argument and Print's miss rule rather than silently subtracting1.

PrintApply.cpp:1413-1432 retains original process variant indices across
materialization. Carry that single selection into model-part, modifier,
material and layer-range resolution, rather than recomputing it from already
reduced vectors. PrintObject.cpp:4363-4374 and PrintConfig.cpp:11421-11429 apply
set_variant_override: a non-nil singleton broadcasts; other inputs use the
original indices. Config.hpp:612-629 resizes from the source front and only
assigns in-range non-nil source columns, retaining existing target columns
for nil or absent inputs. A singleton nil therefore inherits, not a NaN
motion speed or a generic false/default substitution. Apply this rule only
to the three migrated fields; other40migrations remain deferred.

Supplemental public sliced-output witnesses cover the mapped physical-nozzle
placeholder and object-region overrides (multi-column remapping, non-nil
singleton broadcast and singleton-nil inheritance). Native replay must be
stable and the API admitted on identical inputs; preserve strict output-channel
comparison without claiming complete G-code parity. A separate scalar-authored
input witness passes identical original 3MF bytes independently to Native and
load_project, never Native's re-export to Ares. This distinguishes genuine
input-parser parity from a historical test comparing an authored scalar request
to a canonical Native array. Any actual parser discrepancy remains a residual;
rewriting test requests to source-kind arrays cannot hide it.

A neighboring typed-printer change owns bed_exclude_volumes,
bed_exclude_volume_mode and extruder_bed_exclude_volumes. This slice does not
change their declarations; shared schema/export files must retain those
records when the printer prerequisite is integrated.
