# A1c: multi-filament Always project-preset selection

## Upstream boundary

This slice ports the external multi-filament branch of OrcaSlicer's
`PresetBundle::load_project` in the pinned source
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, specifically
`OrcaSlicer/src/libslic3r/PresetBundle.cpp:6363-6428`.

The Rust destination is `crates/ares-core/src/profiles/project_selection.rs`
and its `external.rs` submodule, which port the ordinary FFF metadata split and
single-filament `Always` path from the same `PresetBundle` flow. The existing
project selection collection and `PresetConfig` remain the compatibility shell
around the upstream preset-loading concepts.

## Included behavior

For an external ordinary FFF project with more than one filament slot, the
selection path will:

- create one filament config per slot from the filament default preset;
- copy scalar filament options to every slot;
- scatter vector options with `set_at(other, 0, i)`;
- for process-multi-extruder projects and keys in
  `filament_options_with_variant`, validate the source variant and self-index
  vector counts, scan self-index ranges in filament-id order, resize non-empty
  vectors by repeating their first value, and slice each filament's variant
  range;
- skip `compatible_printers` and `compatible_prints` while scattering;
- split `compatible_printers_condition[i + 1]`,
  `compatible_prints_condition[i]`, `inherits[i + 1]`,
  `different_settings_to_system[i + 1]`, and `filament_ids[i]`;
- apply `convert_filament_preset_name` before each external filament load;
- load external filament presets in reverse order, selecting slot zero with
  `Always` and every later slot with `OnlyIfModified`; return the upstream
  modified flag only for the `Always` found-preset path, and use the `Never`
  early-return rule when an unmodified matching preset must not be selected;

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

The Native-backed synthetic controls are fixtures for:

1. ordinary two-filament external Always selection;
2. an H2D-style dual-extruder project with variant-index slicing;
3. a single-filament control that remains unchanged;
4. three external filaments exercising modified and unmodified intermediate
   selections with distinct loaded names;
5. a modified non-first system filament that receives a new project-embedded
   `(file.3mf)` name;
6. variant-list input with no self-index vector, completed before validation;
7. equal variant and filament counts using the non-multi-extruder `set_at` path.

The controls must fail before the production port because the current Rust
entry point rejects multi-filament Always selection. The implementation must
preserve the existing single-filament behavior and all malformed-input and
resolver premises.
