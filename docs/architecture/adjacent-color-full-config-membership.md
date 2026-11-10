# Adjacent-color preset composition boundary

## Rewrite boundary

This change preserves the OrcaSlicer v2.5.0-beta applied-configuration
boundary at revision `345efec64c7b95a05ce5cf96df0dde66a158835c`. It concerns
only parity-test input composition; it does not add print-host behavior,
custom-G-code behavior, or a second configuration serializer.

## Native source contract

- `OrcaSlicer.cpp:3099-3173` (`update_full_config`) copies selected printer,
  process, and filament values into the applied dynamic configuration.
- `OrcaSlicer.cpp:3167-3200` refreshes retained project presets from the
  selected system presets.
- `Preset.cpp:1059-1089`, `:1592`, and `:1787-1792` define printer preset
  loading, printer option membership, and default-preset materialization.
- `PrintConfig.hpp:2181-2184` and `PrintConfig.cpp:12716-12722` define and
  initialize the static `FullPrintConfig` composition. ConfigDef registration
  alone does not establish static membership.
- `GCode.cpp:8244-8246` copies the applied dynamic snapshot, and
  `GCode.cpp:8338-8352` serializes that snapshot's actual keys subject to the
  upstream banned and nil rules.

## Mechanism

The adjacent-color tests compared two different input-composition paths.

The processed side directly called
`ares_core::slice_project_plate_processed`. This core API has no project-preset
catalog argument, so it sliced the embedded project configuration without the
catalog refresh.

The unprocessed side called
`runner::application::slice_project_plate`. Its catalog branch called
`ares_core::slice_with_profiles` with the project-derived selection. That path
entered `profiles/entry.rs` and `ProjectPresetRefresh`, then materialized
printer-preset values through `project_refresh.rs` and
`Project::apply_project_preset_refresh`.

As a result, the two sides reached the same output comparison with different
applied dynamic key sets. The 15-key difference was a consequence of the
composition mismatch, including the effect of `d1b8a399e` on the no-catalog
membership path. It was not a missing serializer whitelist and was not caused
by `record_moves` or processor output rewriting.

## Ares boundary correction

`adjacent_color.rs` now routes every processed slice through the existing
`runner::application::slice_project_plate_processed` adapter. The unprocessed
side continues to use the matching application adapter. Both sides therefore
compose the same project-preset catalog before entering the shared slicing
path.

The core configuration exporter remains unchanged. It continues to serialize
the applied snapshot using its existing membership and nil rules. No literal
key list, fallback, normalization, assertion change, or comparator change is
introduced.
