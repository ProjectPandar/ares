# CLI override dirty columns: source-owned comparison and preset ownership

## Upstream boundary

Pinned source: `345efec64c7b95a05ce5cf96df0dde66a158835c`.

- `libslic3r/PrintConfig.cpp:2255–2258`: `different_settings_to_system` is `coStrings`, default empty, not a CLI-settable option.
- `libslic3r/PresetBundle.cpp:5498–5564`: `full_fff_config` collects process dirty keys against the system parent; filament collection follows in that function.
- `libslic3r/PresetBundle.cpp:5741–5749`: printer dirty-key collection; `:5804` publishes the collected columns in full config.
- `OrcaSlicer.cpp:4157–4223`: the CLI saves pre-override values of explicitly supplied keys, applies overrides, compares the resulting values, and records changed keys according to `Preset::print_options`, `filament_options`, and `printer_options` membership. This is the adapter call-site sequence, not ownership of a second comparison algorithm.

This slice rewrites the existing dirty-settings concept; it does not create an Ares-owned slicing stage or infer dirty keys from rendered G-code. The current empty-column emission in `ares-core/src/options/config_export/full_config.rs:213–227` is only a temporary compatibility shell for absent bookkeeping, not an implementation of override tracking.

## Non-negotiable Rust boundary

**Comparison, preset ownership and column mutation belong to ares-core as a pure API.** The destination is a small module under `crates/ares-core/src/options/overrides/`, exposed through the existing public `ConfigOverrides`/project preparation API. It operates on core-owned configuration values and presence information, without filesystem, process, terminal, clock, UI or native-only dependencies. It must work unchanged on browser WASM, Windows, macOS and Linux.

The API contract separates capture of only the explicitly overridden keys' loaded values from recording their post-application changes. It must preserve absent versus present values, scalar versus vector kinds, and serialized element values. Snapshot only those option values, never clone the project mesh or whole archive. A caller captures the loaded, profile-resolved state immediately before the overriding write and records against the resulting config before later normalization/materialization changes widths. Existing public byte-input/API slicing with `ConfigOverrides` must reach this same core contract; direct API and CLI results may not diverge.

`crates/ares-cli/src/main.rs` / `project_presets.rs` own only command-line parsing and orchestration at the equivalent `OrcaSlicer.cpp` call site: request the core before-values capture, apply validated overrides through core, invoke the core recorder, then continue the existing flow. CLI must not contain its own equality routine, key-family lists, dirty-column routing, deduplication or dirty-string formatting. Where existing preparation already delegates to core, keep that delegation rather than duplicate the algorithm in the adapter. Do not expose the private raw-settings representation merely to let CLI inspect it.

The existing core loader and preset-refresh override application sites (`project/load.rs`, `project/domain/preset_refresh.rs`) are compatibility entry points around the upstream configuration operation. They must share one pure recording implementation and correct loaded-before/overridden-after ordering, not grow independent dirty-state pipelines. Detailed public method spelling is not an ABI commitment in this SPEC; the module, data ownership, operation order and one-algorithm rule are binding.

## Included behavior

1. Three preset column families in the existing `filament_count + 2` layout: process at0, filament slots1 through count, printer last. A changed filament-owned override is recorded in every filament column, as the upstream caller does. Ownership comes from the source-defined preset option families, not string prefixes or an eleven-key lookup table.
2. Record only explicitly requested override keys whose effective value changed. Preserve existing column entries and their order; append a new key once, using the existing source-derived C-style string-list encoding. A key absent from the loaded config counts as changed even when the override equals a built-in default. An unowned key does not gain a fabricated owner.
3. Same value means no dirty addition. Scalars compare the source serialization; scalar/vector kind mismatch differs. Vectors compare serialized elements through the longer length, reading a missing element as the first element (`get_at` semantics), not by raw vector length. Thus `245` and `245,245,245` are equivalent in the same vector kind. Two empty vectors are equal; empty/nonempty differ. No epsilon, numeric coercion, nil-to-zero conversion or arbitrary broadcasting.
4. The actual overridden values, validation errors, preset refresh order and subsequent materialization remain unchanged. Dirty metadata is configuration state and is serialized normally; do not patch the footer text.
5. Public Native/API/CLI coverage for the existing eleven positive C1 IDs, including process, filament and printer ownership. Each ID supplies an independent literal expected footer line and asserts the complete `; different_settings_to_system = ...` line equals Native, API and CLI.

## Deferred / excluded

- Full GUI edited-preset/system-parent diff reconstruction, preset saving/reopening, user-preset discovery and new catalog/refresh behavior beyond preserving existing dirty columns.
- New preset ownership categories, generic change-tracking/event frameworks, a new profile model or broad exposure of private raw configuration types.
- Pre-cooling, nullable placeholder storage/language, single-variant activation, pressure advance, volume maps, fan/nozzle-temperature-range, planner/time/M73, geometry and rendering changes.
- Changes to legacy normalization, archive fixtures, Native producer flags, validation rulings, comparator normalization, skip/allow policy or whole-output acceptance.

## Failure modes to exclude

Comparison must not confuse width-only vector representation changes with value changes, read default values as if present in an old document, compare after a profile refresh has discarded the loaded value, mark internally generated defaults as user overrides, route filament keys by spelling, overwrite an existing dirty list, duplicate a key on repeated preparation, or update only CLI while public API keeps empty columns. Missing bookkeeping is not permission to edit rendered tokens.
