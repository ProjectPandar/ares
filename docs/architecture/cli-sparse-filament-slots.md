# Sparse filament slots in direct STL slicing

Source: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

`src/OrcaSlicer.cpp:2373–2469` counts every entry of `--load-filaments`,
but appends a config only for a filename when `--load-defaultfila` is off.
It records the original one-based index, and `:2528–2529` uses the list
length for a fresh STL. The merge (`:3510–3575,3592–3622,3740–3800`)
updates the stored index, not the ordinal among loaded files. The developer
options are rebuilt only if every slot has a config (`:3815–3825`).

A skipped slot has no selected preset. `PresetBundle.cpp:485–518` constructs
the default filament from `FullPrintConfig::defaults()`, with an empty
settings ID and nil nullable overrides. The fresh CLI does not select that
preset by name: `DynamicConfig::optptr` (`Config.cpp:1725–1741`) materializes
individual option defaults, and the FFF completion (`OrcaSlicer.cpp:4254–4259`)
adds remaining defaults. `ConfigOptionVector::set_at`, `set_with_restore_2`
and `resize` (`Config.hpp:451–469,542–581,687–716`) can also extend vectors
from their current first value. For example, with authored temperatures 205
and 225, a leading gap stays at the default 200, while an interior gap
between those files gets 205. Names remain empty at skipped positions;
`filament_ids` grows from the first loaded ID (`OrcaSlicer.cpp:3592–3622`).
A trailing gap does not itself write a value: `Preset::normalize`
(`Preset.cpp:575–589`) and variant normalization can grow individual
vectors, while the flush calculation can reject an undersized support
vector first (`OrcaSlicer.cpp:3850–3867,3925–3934`). For a successful
trailing-gap flush rebuild the input must also cover the flush minima:
`src/slic3r/GUI/Plater.cpp:1252–1305` sizes the minima from the diameter
vector and `OrcaSlicer.cpp:4011` indexes them by palette slot.

The Rust destination is the direct-file composition boundary in
`ares-core/src/profiles/load_settings.rs` and `profiles/catalog/compose.rs`,
with the resulting raw config passed through the existing fresh STL
constructor. The compact list of loaded documents is a compatibility shell;
its ordinal is not the logical filament index. Preserve sparse positions
and per-option growth without synthesizing a loaded preset for a hole.
Compare fresh direct STL artifacts against live Native for leading,
interior and trailing gaps. Dense lists, the `--load-defaultfila` first-file
fallback, normal catalog selections and saved 3MF refresh are unchanged.
An entirely empty list with the flag off is outside this slice; do not
reproduce a Native crash. Authored filament ID parsing and unlisted-parent
admission are separate from preserving the destination index.
