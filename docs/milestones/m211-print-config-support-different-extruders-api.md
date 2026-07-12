# M211: DynamicPrintConfig support different extruders API

## Source boundary

This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8744-8766`, with `PrintConfig.hpp:661` declaration context, `Config.hpp:624-630` vector `get_at` fallback semantics, `PrintConfig.cpp:5239-5244` `extruder_variant_list` option context, and existing `nozzle_diameter` option context already ported in the registry. It adds only a read-only `SliceOptions::support_different_extruders()` helper returning the source boolean plus the source out-parameter extruder count. It does not port `get_index_for_extruder`, generated variant IDs, preset bundle materialization, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior.

## Exit criteria

- Missing `nozzle_diameter` uses the existing Ares single-nozzle default and returns `supported = false`, `extruder_count = 1`.
- Present `nozzle_diameter` sets `extruder_count` to the resolved nozzle vector length.
- Missing `extruder_variant_list` returns `supported = false` while preserving the nozzle-derived count.
- Variant strings are split with Orca `boost::split(..., token_compress_on)` comma semantics, preserving boundary empty tokens and compressing repeated separators.
- Unique variant tokens across all nozzle-indexed `extruder_variant_list.get_at(index)` values determine `supported`: more than one unique token returns `true`; zero or one unique token returns `false`.
- `extruder_variant_list.get_at` fallback uses the first value for out-of-range indices.
- Invalid non-vector/non-string boundary values, empty present variant vectors when a nozzle count requires access, and invalid nozzle diameter values return `SliceError::InvalidInput` instead of panicking.
- No `get_index_for_extruder`, generated variant ID, preset/profile composition, slicing, extrusion, G-code, crate, or dependency changes.
