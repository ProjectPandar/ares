# PresetCollection built-in default prefix: Rust rewrite slice

Ares base: `b065c0e856011f468e1d88c192d17330ef228f64`.
Orca release source: `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

## Architectural decision

Port the initialized built-in default entities of `libslic3r::PresetCollection` into the existing `ares-core` catalog. Process, filament and machine share `Catalog`/`Record`/`ProfileKind`, so this slice covers their three FFF defaults together. A string substitution or independently designed preset pipeline cannot represent their configuration and ownership.

Keep the explicit-query API boundary. Existing printer/current-process/current-filament arguments supply the selection context. Compatibility, candidate visibility and default suppression are separate concepts. Queries do not retain an editor's selected index, edited configuration or visibility between calls.

Catalog storage, resolution and query methods form a temporary compatibility shell around the cited Native collections. Empty selections cannot stand in for an initialized default entity. Persistent Native GUI collection state remains outside this boundary.

Core remains portable, memory-only and UI-neutral across browser WASM, Windows, macOS and Linux. This slice adds no unsafe code, filesystem discovery, mutable global state, dependency, crate, public selection-state object, suppression toggle or legacy fallback.

## Upstream ownership

All paths below are relative to the pinned Orca source root.

| Source boundary | Included behavior | Rust destination |
|---|---|---|
| `src/libslic3r/Preset.hpp`, `PresetCollection` constructor declaration at 573–579 | Exact default names, including constructor default `Default Setting` | `profiles/catalog/defaults.rs` |
| `Preset.cpp:1766–1824`, constructor / `add_default_preset` / `reset` | Loaded default entity, configuration and prefix separate from ordinary entries | `profiles/catalog.rs`, `catalog/defaults.rs`, `catalog/store.rs` |
| `PresetBundle.cpp:192,484–554`, placeholder literal / constructor | FFF names, kind-specific configuration, metadata and nullable overrides | Catalog default initialization |
| `Config.cpp:322–342,484–530,1725–1745`, `create_default_option` / `apply_only` / `DynamicConfig::optptr` | Create definition defaults for declared keys before copying available static values, preserving vector/enum/nil shape | Existing registry/default-value owners used by catalog construction |
| `Preset.hpp:834–858`, `first_compatible_idx` | Suppressed default prefix is excluded from ordinary ranking. No winner resolves index zero | `profiles/compatible/selection.rs`, `compatible/print_selection.rs` |
| `Preset.hpp:1048–1071`, `Preset.cpp:3894–3909`, internal/public lookup | Search ordinary presets first, then default entries, independently of visibility | `catalog/chain.rs`, `catalog/store.rs`, `preset_values` |
| `Preset.cpp:4013–4066`, `Preset.hpp` default suppression field | Initial suppression and context-dependent default visibility | Compatibility candidate and selection handling |
| `Preset.cpp:4272–4295`, `select_preset` | A selected default has resolvable configuration and appears in the selection result | Explicit-query selection projection |

## Included FFF default entities

Names come from the constructor, not translated UI captions.

| `ProfileKind` | Native collection | Name |
|---|---|---|
| Process | `prints` | `Default Setting` |
| Filament | `filaments` | `Default Filament` |
| Machine | `printers` | `Default Printer` |

Each has explicit built-in/default identity, loaded configuration and no vendor/user-import ownership. A built-in is not a `from: system` resource, imported JSON, installed filament, vendor model or renamed alias. `Generic PLA @System` remains an independent real preset.

The three defaults share storage but remain kind-indexed entities. Each kind's logical default prefix is distinct from its sorted ordinary suffix. Prefix entries do not shift vendor indexes, reorder ordinary candidates, count as supplied CLI files or increment imported/vendor/model inventory counts.

Both the public handle constructor and one-shot catalog JSON construction contain the prefix. Vendor/import additions and reconstruction preserve it. The catalog exposes no public reset/delete API. Prefix preservation during construction does not introduce one.

### Initialized configuration

Construct each entity from the corresponding Native option defaults and apply its constructor initialization. Reuse the existing option registry, typed defaults and `project_refresh::default_preset_fields`. Do not create a second defaults table or a name-specific response in `preset_values`.

`PresetBundle.cpp:484–554` passes the `PrintRegionConfig` view of `FullPrintConfig::defaults()` and each kind's option list into its FFF collection. `ConfigBase::apply_only` creates each destination option before checking whether the source contains it. `DynamicConfig::optptr` delegates to `ConfigOptionDef::create_default_option`, which clones the definition default, including enum-vector mapping. An absent static source member therefore does not imply an absent or empty destination option. The registry's `full_config_default` flag does not filter preset membership.

- Process contains every declared process key. The constructor also touches `print_settings_id`, compatibility condition and `inherits`.
- Filament contains every declared filament key. The constructor also sets `filament_settings_id = [""]` and touches compatibility condition and `inherits`.
- Only nullable filament extruder override options become `nil`, according to `is_filament_extruder_override_key`. There are 24 such keys, including the five explicit predicate additions. Other nullable options retain their defaults. Nil is not zero.
- Printer contains the declared printer, machine-limit and nozzle keys. The constructor also touches settings ID, model, variant, thumbnails, default print profile, default filament profile and inherits. `optptr(key, true)` retains an existing option rather than emptying it.
- Scalar IDs/conditions/inherits are empty strings. Filament settings ID is one empty string. Default filament profile is an empty vector. `thumbnails` retains `48x48/PNG,300x300/PNG`.
- An empty wire value represents an empty vector. The two quote bytes (`""`) represent a one-element string vector containing an empty string. These shapes are distinct.
- SLA-specific keys, SLA printer presets and project configuration are excluded from FFF defaults. FFF printer options such as `extruder_variant_list` and `default_nozzle_volume_type` remain included when the FFF option list owns them.

Default values retain their scalar/vector/nil semantics at the public resolver boundary. The entity owns its configuration even when immutable defaults can be referenced without copying an entire table. Resolution uses ordinary catalog lookup, not caller-side special names or a first-real-preset substitution. Unknown-key errors remain errors.

Built-in raw fields are not prepended to imported/vendor presence views. Inheritance, direct typed-file composition, 3MF embedded refresh and raw-key presence retain their own behavior. Missing fields on ordinary records are not globally materialized or coerced.

## Supported option membership and reference limits

At this base, the registry has 881 macro declarations and the public definition export contains 805 keys. All keys in the FFF membership below have registered owners. Membership and scalar/vector/nullable shape do not establish equality of every initialized configuration value.

| Kind | Declared occurrences | Distinct declared keys | Including constructor keys | Unmapped | Filament constructor nil overrides |
|---|---:|---:|---:|---:|---:|
| Process | 394 | 394 | 395 | 0 | 0 |
| Filament | 152 | 152 | 153 | 0 | 24 |
| Machine | 219 | 208 | 209 | 0 | 0 |

Machine occurrences follow `Preset::printer_options()` concatenation of printer, machine-limit and nozzle lists. Repeated declarations are not additional entities or vector slots. Existing `default_is_nil` and native wire-default owners determine initialization. Public display-default strings do not determine boolean spelling, vector cardinality, percentage encoding or nil.

The existing `native_settings()` / `exported_definition()` channel in `crates/ares-cli/tests/orca_parity/runner/config_block_roundtrip/definition_defaults.rs` compares original-project settings with public PrintConfigDef values. Original-project settings have project identity and real-preset IDs, and may include project overrides. They are not initialized built-in preset entities. For example, an empty project `thumbnails` value cannot clear the constructor default, and a Creality preset ID cannot become a built-in ID.

Default names come from `PresetBundle.cpp:192,484–489` and the `PresetCollection` constructor argument. Default/loaded identity comes from the collection constructor. These are source-derived expectations, distinct from runtime configuration observations. Project settings and definition spot checks do not expose Native collection selection or GUI state, or establish complete initialized-configuration or slicing parity.

## Lookup and selection contract

### Lookup

1. Resolve an ordinary selectable preset of the requested kind first.
2. If absent, search that kind's built-in default entity.
3. Hidden or suppressed defaults remain name-resolvable.
4. Unrelated missing-parent, unknown-name and unknown-key diagnostics remain unchanged.
5. `Generic PLA @System` is not an alias, another kind's default cannot satisfy lookup, and a missing real vendor preset is not materialized.

`preset_values(Filament, ["Default Filament"], keys)` resolves without vendor/import records and after adding compatible or incompatible records. Process/printer queries reach their own initialized entities. Parent resolution searches ordinary entries before the matching default.

### Filament and process selection

Retain a resolvable compatible current selection independently of installed visibility, including an explicitly current default. A missing or incompatible ordinary slot ranks eligible visible non-default candidates using existing strict-best/tie/order rules. The initially suppressed default prefix does not enter that competition.

With no winner, selection resolves the kind's index-zero default entity and returns its name. An empty string does not represent that selection. Filament fallback occurs at the existing one-slot and multiple-slot replacement points, after hardware minimum/padding. Process fallback returns the initialized `Default Setting`, not an arbitrary real process. Without a loaded current process, preference scoring uses that entity's initialized layer height.

A four-nozzle non-SEMM printer retains four required slots. A SEMM printer retains its source-defined minimum. Surplus slots, last-slot padding, flush layout, colors and filament maps remain unchanged. Defaults do not truncate slots, collapse four slots to one or change ranking preferences.

### Suppression, compatibility and visibility

Native initializes `m_default_suppressed = true`. `first_compatible_idx` skips the default prefix while ranking under suppression but returns index zero with no winner, independently of prior default visibility.

`update_compatible_internal` skips defaults in its ordinary compatibility loop. Its `some_compatible` count measures printer compatibility before process compatibility is AND-ed, without filtering on installed visibility. A count of visible printer+process-compatible candidates cannot replace it.

Native default visibility also depends on whether a default is selected. `set_default_suppressed` uses that selection when the flag changes. `select_preset(0)` makes the selected default visible. Rust queries project selection from current/slots arguments without retaining editor state between calls.

- Initial suppression excludes defaults from ordinary default-versus-real ranking.
- Lookup is independent of suppression and installed-filament membership.
- Selection returns a resolvable default, not a missing hidden system row.
- Ordinary compatible-current retention and system/user candidate visibility remain unchanged.
- Compatibility-only filtering is distinct from a selected-state-dependent GUI visibility list.

## Deferred behavior

- Named/forced deletion and `delete_current_preset` are outside this slice. Default-aware lookup and `force=true` bypassing the overwrite guard do not establish prefix preservation during deletion.
- `Tab.cpp` / `UnsavedChangesDialog.cpp` Always/OnlyIfWasCompatible/Never policy and editor event sequencing.
- The Native profile validator's independent rejection of default fallback.
- Persistent selected/edited/dirty/saved state, complete GUI visibility transitions and a public suppression-switch API.
- SLA print/material defaults and SLA printer behavior.
- Resource/profile discovery, vendor installation, filesystem access, GUI/OpenGL or terminal behavior in core.
- Browser preset selection, presentation and transport changes.
- Changes to 3MF selected-versus-embedded count admission, relative-E validation, error order, slicing algorithms or unrelated default-bed/temperature behavior.

## Failure modes

1. A returned default name has no record, so values lookup or composition fails.
2. `Generic PLA @System` is used as an alias for the built-in filament.
3. Prefix entries displace a preferred compatible ordinary preset during ranking.
4. Vendor/import additions erase defaults, or lookup reaches another kind's default.
5. Built-in ownership corrupts vendor indexes, supplied-file cardinality or inventory counts.
6. A compatible hidden current preset is replaced unnecessarily.
7. Configuration has incorrect vector/nil shapes, identity keys or constructor overrides despite similar display text.
8. All-incompatible or all-hidden catalogs produce empty slots, or lose the four-slot minimum.
9. An unrelated missing parent/name/key is silently replaced by a default.
10. Built-in raw presence changes ordinary CLI composition or project refresh output.
11. Compatibility-only lists are treated as persistent GUI visibility, or process compatibility replaces the printer-stage count.
12. Original-project values replace constructor defaults, or source-derived identities are presented as observed Native selection state.

## Public API requirements

These behaviors are observable through `PresetCatalogHandle` construction, import and query methods.

| Case | Required observation |
|---|---|
| Fresh catalog, default lookup | Three exact kind-specific names resolve initialized configuration |
| Fresh catalog, Generic PLA @System lookup | Missing unless the real preset is supplied, never an alias |
| Valid single/fixed4 machine, no ordinary filament | One/four Default Filament slots |
| Valid process and only process-incompatible material | Default Filament fallback with unrelated compatibility diagnostics preserved |
| Compatible but uninstalled system material, no retained current | No visible ordinary winner, so selection resolves the built-in |
| Compatible hidden current real material | Current selection survives |
| Compatible ordinary candidates/preferences/ties | Existing winner and order remain unchanged |
| Explicitly current Default Filament with ordinary records | Default remains compatible and resolvable |
| No compatible ordinary process | Default Setting resolves with its own configuration |
| Surplus slots and SEMM | No truncation or hardware-minimum change |
| Vendor/import addition and reconstruction | Defaults persist without inventory/index pollution |
| Wrong kind, unknown name/key, unresolved parent | Existing strict diagnostics, without catch-all default resolution |
| Constructor configuration | Source-defined IDs, strings/vectors, nil overrides and supported defaults |
