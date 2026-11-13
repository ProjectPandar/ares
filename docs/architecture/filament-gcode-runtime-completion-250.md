# Loaded filament G-code runtime completion

## Upstream boundary

This slice follows OrcaSlicer source revision
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `libslic3r/Config.hpp:410–418,1189`: `ConfigOptionVector<std::string>::values`
  and `ConfigOptionStrings` hold the filament script slots.
- `OrcaSlicer.cpp:710–728`: `load_default_gcodes_to_config(TYPE_FILAMENT)`
  resizes an empty start or end script vector to one empty string. Existing
  nonempty vectors are retained. The CLI invokes this completion for a loaded
  filament at line 3590.
- `libslic3r/GCode.cpp:4851–4879`: final filament-script dispatch reads the
  active filament's end script through `get_at` in single-extruder
  multi-material mode, or iterates the available end scripts otherwise.
- `libslic3r/Print.cpp:2743–2766`: non-BBL relative-extrusion
  `MarlinLegacy`/`MarlinFirmware` input requires uppercase `G92 E0` in a
  before-layer or layer-change script.

## Required behavior

For a single typed filament document without inheritance, authored
`filament_start_gcode: []` and `filament_end_gcode: []` become one empty string
in both the composed configuration view and the options used during slicing.
A compatible machine/process/filament input must complete G-code generation
without indexing a zero-length end-script vector. Its configuration comments
retain `filament_start_gcode = ""` and `filament_end_gcode = ""`.

Completion does not replace existing nonempty script vectors or change the
registered one-space defaults. Vendor omission continues to expose `[" "]`;
the typed authored-empty case exposes `[""]`. The configuration-view tests
remain separate from the public slicing regression.

## Rust ownership

The destination is `ares-core::profiles::catalog::compose`, at the existing
`resize_empty_filament_gcodes` loaded-file completion. Whenever that operation
creates an empty raw slot, it must project the same slot into the merged
filament's `FilamentOptions.gcode` before `compose_resolved_profiles` collects
those options for slicing.
`project_slice::gcode_emit::finish` remains a consumer, not a second
normalization boundary. No finish-stage fallback or global default change is
part of this slice.

The separate raw overlay and typed merged options are a temporary compatibility
shell around upstream `DynamicPrintConfig`; they must not define different
values for this completion rule. Core remains byte-oriented and independent
of filesystem and terminal adapters.

## Failure modes

1. Completing only the raw overlay leaves the runtime end-script vector empty.
2. A finish-stage fallback conceals the incomplete loaded-file configuration.
3. Changing registered defaults conflates vendor omission with authored emptiness.
4. Clearing or truncating nonempty vectors discards authored scripts.
5. Updating only the end script leaves the paired start-script completion absent.
6. Relative-extrusion input without the required reset is rejected before slicing.

## Deferred behavior

Multi-filament slot collection, vendor default-length runtime propagation,
general sparse normalization, other script vectors, STL repair parity and
whole-output G-code parity are outside this slice. Existing behavior at those
adjacent boundaries is not redefined by the single-file empty-vector rule.
