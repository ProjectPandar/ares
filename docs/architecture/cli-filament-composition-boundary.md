# CLI filament composition boundary (B44 replacement)

## Decision

Replace the rejected B44 `8e1df6df6` and `f76c46db1` slices with a patch based on `dc127b48c`. Do not normalize a merged project as if it were a preset. `Preset.cpp:1908-1914` normalizes the individual `preset.config` after inheritance/default completion; the CLI never calls `Preset::normalize` on `m_print_config`. No additional preset normalization is ported in this slice. The existing machine-preset normalization is unchanged.

## Upstream ownership and Rust destination

- `OrcaSlicer.cpp:3739-3780` composes each loaded filament's vectors into `m_print_config`. Non-variant vectors use `set_at(source, filament_index - 1, 0)`.
- `Config.hpp:451-459` grows that destination to `i + 1` with its own first element before replacing slot `i` with the source element. The source front is not the padding value.
- Ares destination: `profiles/project_refresh/filament/slot.rs::apply`. Preserve the existing vector/variant dispatch, excluded keys, default creation, and protected-project-key behavior; correct only the non-variant destination padding. This includes `filament_is_support` through ordinary vector dispatch, without a literal-key repair.
- `OrcaSlicer.cpp:1893,2528-2529,3855-3866` uses CLI `filament_count`, derived from project filament selection or loaded filament count, for palette growth. An override of `filament_diameter` is not a new preset selection.
- Ares destination: `options/flush_matrix.rs::prepare`. Use the typed filament selection count (the selected identities are already composed by the profile entry), not the diameter vector length, for this CLI preparation step. Retain the existing matrix resize/recomputation.

The raw/typed Ares configuration is a temporary compatibility shell for the cited CLI config boundary, not a new pipeline. Full Preset normalization, changes to variant composition, and GUI `PresetBundle` matrix preparation are deferred. `PrintConfigDef::filament_option_keys` is unchanged.

## Failure model and public verification

Potential failures: source padding incorrectly leaks into skipped slots; an oversized diameter override spuriously grows the palette and triggers a support-count error; preset growth is accidentally lost; variant and protected keys are changed; output drifts from the `dc127b48c` NP baseline.

Use the existing public applied-map pair, six processor/reproducibility tests, full `print_validation` family in official and seeded modes, exact test-ID verdict comparison, and the master-extruder NP output witness. Retain artifacts and original failed attempts. Do not change expectations or add private tests.
