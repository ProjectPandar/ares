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
The present OrcaFloat/OrcaBool scalar fields in region_fields.rs and their
registry kinds are temporary compatibility shells to replace, not preserve
behind scalar/vector equality relaxations.

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
Native arrays and type/default/nil declarations; report all three gaps in one
artifact before asserting. Do not call this a legacy-import leaf-DTO seam.

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
- Runtime selects first/filament-id/physical-extruder instead of mapped slot.
- Out-of-range get_at or empty/default normalization differs from source.
- any_enabled precomputation is mistaken for selected-nozzle behavior.
- A later failure is relabelled PASS merely because shape admission unblocks.

First qualify Native and public RED, then commit RED. No production migration
before that artifact. Exact affected surviving ledger replays remain bounded
and separately account downstream residuals; no FULL.
