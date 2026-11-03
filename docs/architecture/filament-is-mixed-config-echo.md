# Mixed-filament metadata cardinality and full-config echo

This is a source-cited rewrite of the complete upstream
`resize_mixed_filament_metadata` helper, not mixed-filament slicing or expansion.
Upstream is pinned to `345efec64c7b95a05ce5cf96df0dde66a158835c`.

## Owning upstream boundary

`src/libslic3r/PrintConfig.cpp:3545–3597` declares the seven members:

| Key | Type | Registered default |
|---|---|---|
| filament_is_mixed | coBools | `[false]` |
| filament_mixed_components | coStrings | `[""]` |
| filament_mixed_sublayer_ratios | coStrings | `[""]` |
| filament_mixed_gradient | coBools | `[false]` |
| filament_mixed_gradient_range | coStrings | `[""]` |
| filament_mixed_gradient_curve | coStrings | `[""]` |
| filament_mixed_gradient_per_part | coBools | `[false]` |

`PrintConfig.cpp:11475–11488`, `resize_mixed_filament_metadata`, creates/accesses
all seven typed options and applies one shared resize algorithm to each: first
truncate to `min(old_slot_count, existing_length)`, then resize to
`new_slot_count`. New bool elements are false; new string elements are logical
empty strings. This is not get_at/front-on-miss broadcast: `[true]` grown to two
is `[true,false]`, not `[true,true]`. Existing prefixes and ordering survive.
The helper is the rewrite unit; a flag-only projection is an incomplete shell.

The relevant caller is `src/OrcaSlicer.cpp:4225–4244`, `CLI::run`: after
`m_print_config.normalize_fdm()` and the out-of-range mixed-slot check, if
`filament_count > 0` it calls the helper with both counts equal to the CLI's
loaded logical `filament_count`. This precedes FullPrintConfig synchronization
(`:4249–4259`) and filament-variant normalization. The count is not used tools,
first-layer tools, physical nozzles or expanded variants. The CLI establishes
it from project/preset inputs (`:1893`, `:2528–2529`), not model activity.

Other callers are `PresetBundle.cpp:4361` (slot-list resizing) and `:6403`
(project-config loading). Their GUI/preset lifecycles are not included here.

`GCode.cpp:8244`, `GCode::append_full_config`, starts from
`print.full_print_config()`. `:8338–8351` iterates sorted keys, omits banned/nil
options and calls `cfg.opt_serialize(key)`; `:4884–4889` writes the non-BBL
CONFIG_BLOCK. `Config.hpp:1994–2003,2051–2059` serializes complete bool vectors
as comma-separated `0`/`1`. A two-slot default is `0,0`, even if only one slot
actually prints.

## Rust destination and scaffold replacement

The seven declarations already exist in
`crates/ares-core/src/options/registry/definitions/table/pre_middle_filament.rs`.
Leave their singleton defaults, kinds, metadata and definition count unchanged;
do not regenerate the registry or manufacture per-project defaults.

Replace the unresized metadata passthrough with one private source-named
`resize_mixed_filament_metadata` helper in `project/domain/profile_input.rs`,
called by the existing `Project::initialize_cli_loaded_project_config` shell
with the source old/new counts and positive-count gate. One algorithm visits
all seven keys; do not add seven independent echo patches. Missing values come
from the registered type-owned default; already normalized raw vectors retain
logical typed elements. Truncate by old count, then grow by new count with the
correct bool/string default-constructed element.

Reuse the caller's loaded logical CLI count. Preserve normalization ordering
with profile/command overrides and variant application; existing slice/validate
callers are `project_slice/state.rs` and `print/validate.rs`. No lifecycle
redesign, extra count policy or broader pipeline is included.

`options/config_export/full_config.rs` must consume the applied resized values
from `ProjectSettingsRaw`, not an unresized registry/raw fallback. Retain the
existing generic bool/coStrings serializers and sorted writer. Do not replace
G-code substrings, append delimiters to rendered tokens, alter banned keys or
add later special-case echoes that leave conflicting applied values. Source
normalization ordering is unchanged. Core remains portable to browser WASM,
Windows, macOS and Linux.

## Included, deferred and non-negotiable behavior

Include all seven members' applied resize and exact config echo together.
Preserve type-owned defaults, complete vector order/length/prefixes, false and
empty-string growth, and generic coStrings quoting. No scalar/vector equivalence,
forced default, first-element broadcast, Native-export-derived Ares input,
comparator change or rendered-token workaround is permitted.

Defer mixed-slot validation/diagnostics, virtual-filament resolution,
`expand_mixed_filaments`, interpretation of components/ratios/gradients, sublayer
or mixing geometry, wipe-tower mixing, and every member outside this exact
helper. Metadata resizing does not implement any of those behaviors. Do not
expand the first-layer-used set or change M140. The M140 sequential per-object
first-layer union and mixed-expansion producers remain deferred. Other callers'
GUI/preset lifecycles, broader STL/load-settings count discovery, configuration
ownership redesign and unrelated migrations are excluded. `filament_volume_map`
is a separate Print filament/nozzle volume-map mechanism, owned by
`PrintConfig.cpp:3118,11909–11928` and `OrcaSlicer.cpp:6829–6866,7126`, not this
seven-member metadata resize helper.

## Failure inventory and labelled public acceptance

1. Registered singleton defaults are changed instead of applied resizing.
2. Only the flag is resized; a companion vector retains stale cardinality.
3. Used/physical/variant count replaces the loaded logical slot count.
4. True/nonempty first values broadcast into new slots, tails survive shrinking,
   or prefix/order/type is lost; coStrings become scalar strings or double-escape.
5. A correct lone-empty quote is removed globally to hide an unresized vector.
6. Repeated preparation or profile overrides undo normalization, or only CLI
   output is repaired while the core API differs.
7. A section/field PASS hides an earlier or next strict difference, or metadata
   echo work silently implements deferred mixed behavior or changes M140/body.

Use exactly the unchanged original authored archives, never Native exports:

- `tests/parity/m140-initial-bed/nonzero-initial-tool.3mf`, SHA256
  `44338634ac93567c8cc10255a9825d229416c996b264c19c4dab75f78738496a`.
- `tests/parity/m140-initial-bed/unused-hotter-filament.3mf`, SHA256
  `893689e2ea06b9f6de4704fc21ca6743618ac483e4e02d00eb1e50419431f6b0`.

These are non-sequential, non-mixed two-slot inputs with actual Native model
activity only on logical filament1. Qualified comparisons require successful,
stable Native first/replay premises, API success, actual CLI exit0 and raw
API=CLI. M140's labelled start-temperature assertions remain separate.

The helper's public oracle is labelled **RESIZE_MIXED_FILAMENT_METADATA_ECHO_7**.
It selects exactly one complete CONFIG_BLOCK line for each of the seven keys
listed above and requires exact Native/API/CLI line equality. Both logical
slots are represented: bools serialize as `0,0` and two empty strings as `;`.
No scalar/vector equivalence, reordered prefix, token normalization or
empty-element omission is allowed. The oracle does not include other config
members, motion, temperature, filament expansion or volume-map behavior.

Complete strict Native/API and Native/CLI comparisons still run unchanged with
only their existing generator normalization. Their exact first differences
and original bytes are retained independently, even when the seven-line oracle
passes. A labelled helper-section PASS does not imply whole-output parity or
closure of a later first difference. Artifacts identify the seven selected
lines, comparison scope, original input hashes, Native commands/statuses,
API/CLI raw equality and complete strict diagnostics as repeatable evidence.
