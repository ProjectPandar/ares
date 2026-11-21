# Type2 wipe-tower finish filament

At OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, `src/libslic3r/GCode/WipeTower2.cpp:2405–2431` (`first_toolchange_to_nonsoluble_nonsupport`) chooses which tool prints the finish extrusions: sparse fill, wall, and brim. A return of `-1` places them before the first change, with the layer's incoming tool; a nonnegative index places them after that change. `WipeTower2::set_extruder` (`:1132–1145`) reads the support flag by filament ID. In forced mode it sets the tower-local soluble flag to false only on the chosen filament, without overwriting the print's material settings.

- An empty change list returns `-1`.
- Forced filament (`wipe_tower_filament > 0`): select the first change whose new filament is not marked soluble in the tower parameters, regardless of its support flag. If none is found, use `-1`.
- Automatic filament: select the first change whose new filament is neither soluble nor support. If none is found and the first change's old filament is neither, use `-1`. Otherwise use index `0` so the saving and minimum purge still apply on a layer with only support or soluble filaments.

## Rust boundary and call sites

In `crates/ares-core/src/project_slice/gcode_emit/wipe_tower`, retain the tower-local soluble assignment in `filament.rs::Filament::from_config` and carry the configured support flag alongside it. Replace `generator.rs::first_nonsoluble_change` with the upstream selector, representing `-1` as `None`. Forced and empty-list results already match; automatic mode currently always returns `Some(0)` for a nonempty list and can assign the shell to the wrong filament.

The upstream Type2 method has two call sites:

| Upstream | Rust | Boundary |
|---|---|---|
| `save_on_last_wipe`, `WipeTower2.cpp:2353` | `generate.rs::save_layer_on_last_wipe` | Both use the selected change for finish-extrusion volume saving. With no selected change they emit finish before the loop, then apply the existing non-finishing minimum clamp where needed. The selector is the mismatch; the saving and purge-floor formulas are unchanged. |
| `generate`, `WipeTower2.cpp:2552` | `generate.rs::generate_layer_changes` | Both append finish extrusions to the selected result, or emit finish first and merge the first change into it with forced travel. If there are no changes they emit the finish result by itself. The selector is the mismatch; the result assembly is unchanged. |

The similarly named Type1 method in `GCode/WipeTower.cpp:3349` and its call sites in `save_on_last_wipe` (`:3308`) and `generate` (`:5195`) are separate and unchanged. This document covers the Type2 selector excluded from [minimum-purge accounting](filament-minimal-purge-on-wipe-tower.md). It does not change role routing, flush allocation, material remapping, validation, or tower geometry formulas.

## Behavioral verification

Derive projects at runtime from `tests/parity/semm-ramming/R1.3mf`, the Creality two-filament Type2 control with supports disabled. In separate automatic cases, mark the second material as support or soluble; compare complete Native replay, public API, and CLI output with the generator-line-only comparator. The case must print a tower and use both tools; the control and forced selector stay strict. Before the port, Ares enters the first tower toolchange instead of printing the finish extrusions with the incoming filament in these automatic cases. Run the same Type2 and purge-accounting cases before and after to detect regressions without masking unrelated output.
