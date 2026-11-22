# Keep empty entries in saved-project filament replacement lists

In a saved 3MF, an empty `--load-filaments` entry preserves the embedded filament at that position. A later file replaces its own slot, not the first slot left vacant by an earlier empty entry. This applies to both a selected plate and all plates.

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):

- `src/OrcaSlicer.cpp`, `CLI::run:2368-2378,2418-2472`: the index advances before the empty-file check. Non-empty files are stored with their original one-based index. With `use_first_fila_as_default` off, empty entries contribute no config. At `:2528-2529` the list length supplies the filament count only when no count is already known.
- `CLI::run:3570-3604,3715-3728,3748-3772` applies each loaded `DynamicPrintConfig` (`src/libslic3r/PrintConfig.hpp`) to that indexed slot: preset identity, inheritance and changed settings, compatibility expressions, ordinary vector values, and variant segments. `src/libslic3r/Config.hpp:451-472`, `ConfigOptionVector::set_at`, owns the indexed vector write. The device-options pass at `CLI::run:3815-3823` also uses the original slot indices and checks actual coverage before replacing shared values.

## Rust boundary

The CLI file reader already preserves placeholders on both plate routes. In `ares-core/src/profiles/load_settings.rs`, `compose_with_project` remembers the original indices as it reads non-empty files, but the saved-project path drops those indices when it hands off the composed presets. The dense selection used by the catalog describes the loaded presets; it is not the saved-project slot layout.

Carry the original indices into the saved-3MF CLI overlay in `ares-core/src/project/domain/profile_input/{three_mf,overlay}.rs`. There, use the same mapping for filament values, IDs, compatibility expressions, inheritance and changed-setting columns, and variant segments. Keep the embedded values for empty slots, and base device-option coverage on the slots actually loaded. Keep the embedded slot count for these replacements. Catalog composition remains the existing adapter for resolving the supplied preset files; do not make holes into selectable presets or fill them with a different preset.

This slice covers leading, interior and trailing omissions within an existing saved-project selection, with at least one supplied filament file. It does not change all-empty list handling, growth beyond the saved filament count, the first-file default mode (`:2379-2417,2460-2468`), preset refresh, the mapped prime tower predicate, or default normalization unrelated to slot placement. STL and catalog-selected project paths retain their own selection rules.

## Verification

Use synthetic saved projects exported by the reference CLI. Run identical slice arguments on Native and Ares for each plate route. On an embedded `A;B` selection, `;A` yields `A;A`, whereas `A;` keeps `A;B`. On `B;B;A`, `A;;B` yields `A;B;B`, not `A;B;A` or `A;A;B`. Supply the matching machine and process files for the whole-file comparison to isolate the indexed overlay from retained-preset default normalization. Check complete G-code through the existing comparator in both official and seeded modes, followed by paired CLI, preset and project comparisons.
