# A1c: multi-filament Always project-preset selection

## Upstream boundary

This slice ports the external multi-filament branch of OrcaSlicer's
`PresetBundle::load_project` in the pinned source
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, specifically
`OrcaSlicer/src/libslic3r/PresetBundle.cpp:6363-6428`.

The Rust destination is `crates/ares-core/src/profiles/project_selection.rs`,
which currently ports the ordinary FFF metadata split and single-filament
`Always` path from the same `PresetBundle` flow. The existing project selection
collection and `PresetConfig` remain the compatibility shell around the
upstream preset-loading concepts.

## Included behavior

For an external ordinary FFF project with more than one filament slot, the
selection path will:

- create one filament config per slot from the filament default preset;
- copy scalar filament options to every slot;
- scatter vector options with `set_at(other, 0, i)`;
- for process-multi-extruder projects and keys in
  `filament_options_with_variant`, resize the source vector to the extruder
  variant count and slice each filament's variant range;
- skip `compatible_printers` and `compatible_prints` while scattering;
- split `compatible_printers_condition[i + 1]`,
  `compatible_prints_condition[i]`, `inherits[i + 1]`,
  `different_settings_to_system[i + 1]`, and `filament_ids[i]`;
- apply `convert_filament_preset_name` before each external filament load;
- load external filament presets in reverse order, selecting slot zero with
  `Always`, then using `Never` after a prior modification or `OnlyIfModified`
  otherwise.

The Native reference is the existing GUI Always preset-selection probe and its
pinned hooked gcroot. Native is used as an observer for authored controls; it
is not an independent second Rust implementation.

## Deferred behavior

This slice does not port published-project handling, SLA selection, normal
(non-Always) CLI refresh semantics, printer/process preset loading changes,
filament-map recommendation, G-code generation, or whole-project/archive
parity. `compatible_printers` and `compatible_prints` remain deliberately
excluded from the per-filament scatter, as in the cited upstream branch.

## Test-first contract

The RED controls are Native-backed synthetic fixtures for:

1. ordinary two-filament external Always selection;
2. an H2D-style dual-extruder project with variant-index slicing;
3. a single-filament control that remains unchanged.

The controls must fail before the production port because the current Rust
entry point rejects multi-filament Always selection. The implementation must
preserve the existing single-filament behavior and all malformed-input and
resolver premises.
