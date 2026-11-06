# Preset extruder-count normalization

## Upstream boundary

This slice rewrites the non-single-extruder-multi-material branch of
`Preset::normalize` in `src/libslic3r/Preset.cpp:524-541`, at upstream revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`. It is not a new slicing pipeline.

The call order is:

1. `PresetBundle.cpp:7585-7666` constructs a system preset from its parent or
   collection defaults, applies included presets in order, applies the child's
   own fields, extends default lengths, and removes misplaced keys. A retained
   non-instantiated base returns before normalization. An instantiated preset
   calls `Preset::normalize(out.config)` at line 7666. The ordinary preset-file
   loader similarly overlays its parent/default at `Preset.cpp:1894-1912`, then
   calls `Preset::normalize` at line 1914. Normalizing every inherited fragment
   independently is not equivalent to this ordering.
2. `Preset.cpp:527-534`: a missing `single_extruder_multi_material` option or a
   true value takes the filament branch. It does not infer a physical-extruder
   count from the filament count. That branch is not replaced by this slice.
3. `Preset.cpp:535-541`: only an explicitly false value, with an existing
   `nozzle_diameter` float vector, calls `set_num_extruders(nozzle_diameter.size())`.
4. `PrintConfig.cpp:10669-10685` calls `extend_extruder_variant` first, then
   resizes the registered extruder-option vectors according to
   `get_parameter_size`. `default_filament_profile` is explicitly excluded.
5. `PrintConfig.cpp:10630-10667` creates an absent `extruder_variant_list` with
   one `Direct Drive Standard` entry per extruder, or resizes an existing list
   using its own first entry. It rebuilds `printer_extruder_id` and
   `printer_extruder_variant` with one-based physical IDs, walking each entry's
   comma-compressed variant tokens. Its final loop at 10661-10666 resizes
   `printer_options_with_variant_2` machine-limit vectors using the rebuilt
   variant count. `Config.hpp:687-718` owns vector growth and truncation.
6. The CLI project-preset refresh at `OrcaSlicer.cpp:3167-3200` consumes the
   resolved system preset through `Preset::load_external_config`
   (`Preset.cpp:1059-1090`). This latter function overlays the project on
   collection defaults and applies the existing dirty-key rules; it is not an
   additional unconditional `Preset::normalize` call on the combined project.
7. After refresh and overrides, `PrintApply.cpp:1423-1429` selects printer and
   process variants. `PrintConfig.cpp:11623-11660,11702-11730` synthesizes
   length-one process columns from the now-complete extruder variant list.
   Selected printer IDs may repeat in the final per-filament view; final
   `printer_extruder_id = 1,1` is not proof that preset normalization failed.
   The J1 process-column witness is `print_extruder_id = 1,2`.

## Rust destination and non-negotiable ordering

The destination is preset resolution, before project-preset refresh or explicit
profile composition exports its resolved machine settings:

- `crates/ares-core/src/profiles/project_refresh/preset.rs::system_config`:
  after defaults and resolved inheritance layers have been applied and wire
  vectors decoded, before the result becomes the refresh source.
- `crates/ares-core/src/profiles/catalog/compose.rs::Catalog::merge`: the
  corresponding completed machine-preset resolution for non-direct catalog
  selections. Do not normalize each inherited fragment. Direct CLI
  `--load-settings` files retain their own `OrcaSlicer.cpp:1953-2010` semantics;
  this is not permission to normalize them as installed presets.
- A small private preset-normalization module under `profiles/` owns the
  `Preset.cpp:535-541` dispatch. Existing `SliceOptions::set_num_extruders`
  (`options/extruder_count.rs`) and `extend_extruder_variant`
  (`options/extruder_variant.rs`) are the temporary dynamic-config shell for
  the cited upstream methods, not a second normalization algorithm. Decode
  only at the existing preset boundary and keep normalized raw/typed preset
  views consistent. Do not round-trip every project through a JSON string.

Those methods currently have no external caller. The existing extruder-count
method includes the registered-vector loop and default-filament exclusion.
The existing variant method has the list/token/ID work but lacks the cited
10661-10666 machine-limit loop. Connecting the caller must not silently claim
that omitted behavior is already implemented. Completing that exact resizing
loop is included in the proposed boundary; machine-envelope formulas and
planner behavior are not. Coordinate this shared file boundary before any
production edit.

No call belongs in global `Deserialize<ProjectSettings>`, the embedded 3MF
wire-substitution path, project variant selection, motion-option assembly, or
G-code emission. Existing selection and process-column synthesis remain
consumers of a normalized preset, not replacement sites for normalization.

## Included and deferred behavior

Included: the explicitly-false machine-preset branch, physical nozzle count,
missing/short/existing variant-list handling, compressed comma traversal and
one-based IDs, exact registered extruder-option resizing with
`default_filament_profile` excluded, and the source's variant-two limit
resizing. Reuse existing default/member and parameter-size logic. Preserve
existing explicit-project/dirty-key refresh precedence and later overrides.

Deferred: the true-or-absent filament branch and the remainder of
`Preset::normalize` (filament-preset vector normalization and SLA legacy
handling), GUI preset editing, direct-file semantics changes, region overlays,
filament-map recommendation, planner/envelope mathematics, and unrelated
footer differences. Existing Ares profile/refresh machinery remains a
compatibility shell around the cited preset boundary, not an Ares-owned
pipeline design. The acceleration/jerk materialization slice remains separate.

## Failure model and public witnesses

Potential failures before implementation:

- missing dispatch leaves a two-nozzle J1 with one extruder variant entry;
- process-column synthesis then keeps the second process ID at one;
- normalizing the full project or every ancestor overwrites authored/dirty
  values or changes inheritance precedence;
- confusing filament count with physical nozzle count grows a single-nozzle
  multi-material printer incorrectly;
- splitting a preset's supported variants into physical extruders corrupts a
  single nozzle that supports Standard and High Flow;
- list growth uses a registry default rather than the existing first entry;
- extruder-option/variant-two sizing runs before rebuilt metadata, truncates
  the wrong dimension, or resizes excluded default-filament choices;
- only the raw footer changes while the typed consumer still sees old metadata.

The public seam is runtime Native slicing of retained 3MF inputs, compared
with validated Ares API and CLI config lines. The initial witnesses use the
existing J1 endpoint-contact fixture, a single-nozzle Standard/High Flow
Ender-3 control, and a two-filament single-nozzle X1C control. J1 independently
checks the extruder list and the selected process IDs. Controls must continue
to pass. Run API and CLI before asserting either line equality so a failing
API comparison does not suppress the CLI observation. No expected Native
G-code is committed, no result JSON is written by the tests, and no comparator
or whole-output acceptance rule is changed.

The initial public witnesses do not exhaust growth/truncation, absent boolean,
missing nozzle-vector, distinct first-entry padding, default-filament exclusion,
or dirty-key ordering. Additional public controls for those effects must be
identified before claiming full normalization parity. Whole-output changes,
including the J1 cases with body differences, require separate Native-direction
comparison after implementation; footer equality alone is not whole-output
parity.
