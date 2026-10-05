# ARD draft: profile-based slicing interface

Status: **interface approved (2026-10-05); native RED before implementation**.
Coordinator confirmed source-defined expression error=true with diagnostics and
required loadScene filename. Update web callers in the same candidate. Record a
real BBL filtering E2E duration including catalog parsing; do not add handle caches.
Commit this ARD with the implementation.

Base before rebase: `16c2dc90`; review base: `19e24fa8`, branch `pi/profile-slice-entry`.
The M853 rebase preserves its optional processed-move result: profile/ordinary
slicing request G-code only; `slice_project_plate_processed` retains move recording.
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Current bounded review slice

This candidate implements the STL selected-profile entry and five-argument WASM
transport, not completion of task24. Selected-profile 3MF remains rejected by
`load_model` rather than flattened or routed through a legacy fallback. Compatibility,
archive import, loadScene/web filename migration and BBL filtering timing remain
planned; the milestone exit criteria below are not yet met.

The catalog feeds the existing typed metadata and `ProjectSettingsBuilder`
deserializers directly from its parsed option tree. It does not encode that tree
as JSON to parse it again. Canonical option output stays with the existing typed
G-code config exporter. The raw view preserves input presence, nullable vector
slots and dynamic options; it is not a second serialized copy of resolved defaults.
The profile typed-shell audit remains unchanged.

## Non-negotiable ownership

This is a Rust rewrite slice of `libslic3r` preset loading and compatibility,
connected to the CLI's selected-profile loading/merge/export path, not a new Ares
slicing pipeline. The native E2E calls `--load-settings` / `--load-filaments`, so
`OrcaSlicer.cpp` owns this entry's composition semantics. `ares-core` owns
archive/JSON parsing, inheritance, compatibility, composition and model import.
`ares-wasm` only adapts bytes/JSON and errors. TS loads/stores/transports data and
renders choices; it does not resolve inheritance or evaluate compatibility.
No filesystem, clock, UI or preset data embedded in core/WASM.

## Inspected source boundaries

- `PresetBundle.cpp:514-547`: system/user loading followed by compatibility and selection.
- `PresetBundle.cpp:4850-4969`: parent/base-bundle lookup, parent then child overlay,
  inherited filament IDs, non-instantiable inheritance-only presets and normalization.
- `PresetBundle.cpp:1304-1447,1450-1549`: JSON/ZIP imports, bundle metadata,
  preset type detection from settings IDs, parent overlay and missing-parent handling.
  Port the byte-oriented import behavior, not temporary directories or persistence.
- `Preset.cpp:771-851`: print/printer compatibility, explicit-list precedence,
  user-printer immediate-parent matching, cross-vendor allowance, filament-library
  exclusions, and `printer_preset` / `num_extruders` expression inputs.
- `OrcaSlicer.cpp:1953-2097`: `load_config_file`, `Config::load_from_json`,
  `normalize_fdm`, selected IDs and system/user identity; `2750-2812`
  `update_full_config` copies every loaded config option except its explicit
  metadata exclusions. `2849-2867,2993-3011` apply machine then process;
  `3257-3290` applies filament vectors and routes filament scalar compatibility
  conditions to indexed groups (printer-condition group has filament_count + 2
  slots). Ordinary machine/process scalar conditions are not erased.
- `OrcaSlicer.cpp:3613-3630,5979-6040`: final full-config/default and plate application;
  `GCode.cpp:5591-5656` emits sorted non-banned, non-nil config keys.
  `PrintConfig.hpp` defines static FullPrintConfig membership: unlike `thumbnails`,
  `upward_compatible_machine` is dynamic-only. STL exports it when supplied;
  3MF import also materializes its default through the CLI's create-on-read
  `config.option(..., true)` (`OrcaSlicer.cpp:1769`). Preserve that 3MF behavior
  at the import-to-slice boundary, not by declaring the key globally static.
- `PrintConfig.cpp:7122-7127,8292-8316`: the absent thumbnail default is
  `48x48/PNG,300x300/PNG`; explicitly loaded thumbnails undergo composite
  normalization, whose separator is comma-space. Do not normalize absent defaults.
- `PresetBundle.cpp:3858-3863,3884-4170` remains context for the existing bounded
  composition helpers, not the CLI oracle. Its GUI-only scalar compatibility
  erasure (`4112-4117`) and GUI full-config lifecycle are **deferred**. Do not copy
  GUI erasure into this entry or invent a footer key allowlist to force parity.
- `Config.cpp:897-986` and `PrintConfigDef::handle_legacy`: normalize each preset
  before parent/child overlay. Reuse the existing typed project-config deserializer,
  preserving absent builder fields instead of prematurely filling defaults. The
  strict standalone `ProfileFragment` parser remains a bounded compatibility shell,
  not the complete vendor-file loader. Upstream-obsolete/unknown preset options
  follow this loader's policy; this does not relax `options_json`'s deny-unknown
  transport contract.
- `Format/STL.cpp:17-40`: absent explicit object name, use filename **including extension**.
- Existing `crates/ares-core/src/profiles/{fragment,inheritance,composition}.rs`:
  reuse typed fragments, same-kind chain resolution and composition. These are
  partial upstream ports, not evidence that the whole full-config lifecycle is done.
  In particular grouped metadata is separate from `ComposedProfile::into_settings`;
  it must not be silently dropped when connecting composition to slicing.

## Proposed WASM exports

```rust
#[wasm_bindgen(js_name = parsePresetBundle)]
pub fn parse_preset_bundle(
    input: Vec<u8>,
    filename: String,
) -> Result<String, JsValue>;

#[wasm_bindgen(js_name = filterCompatiblePresets)]
pub fn filter_compatible_presets(
    catalog_json: String,
    printer: String,
    process: Option<String>,
) -> Result<String, JsValue>;

#[wasm_bindgen(js_name = sliceWithProfiles)]
pub async fn slice_with_profiles(
    input: Vec<u8>,
    filename: String,
    catalog_json: String,
    selection_json: String,
    options_json: String,
) -> Result<Vec<u8>, JsValue>;

// Change the existing export; no nameless-STL fallback.
#[wasm_bindgen(js_name = loadScene)]
pub fn load_scene(input: Vec<u8>, filename: String) -> Result<SceneData, JsValue>;
```

JS receives strings containing JSON, Uint8Array G-code, or the existing SceneData.
The slicing export returns a Promise; the other exports are synchronous.
The coordinator approved the five-parameter slicing export after an exact-version
wasm-bindgen probe confirmed the six-parameter draft violates the project's
Clippy arity limit. No lint suppression or threshold change is permitted.
`options_json` is `{plate?: number, seed: number}`: Rust deserializes it into a
struct with `#[serde(deny_unknown_fields)]`, `plate: Option<u32>` and `seed: u32`.
Missing seed, unknown fields and invalid value types are errors. Selection JSON
remains unchanged. The core entry groups borrowed input metadata in a typed
`ProfileSliceRequest`, separately from `GenerationMetadata` and `SlicerContext`.
All failures use the existing WASM error convention; no partial G-code success.

## JSON contract proposal

`catalog_json`:

```ts
{
  vendors: VendorProfiles[]; // each loadVendor() result unchanged: {index, files}
  imports: ParsedPresetBundle[]; // each parsePresetBundle() JSON result unchanged
}
```

Accepting the existing vendor envelope avoids making TS select profile files,
remove machine-model records, infer system/user origin, or discard vendor metadata
needed for filament-library exclusions. Callers supply dependency vendors too
(e.g. OrcaFilamentLibrary); Rust reports missing parents instead of guessing.
No global mutable preset registry or WASM object-handle lifecycle.

`parsePresetBundle` accepts an Orca preset JSON or `.orca_printer` / `.orca_filament`
archive. Proposed `ParsedPresetBundle` result:

```ts
{
  profiles: ProfileJson[]; // import-normalized name/type, original option values/inherits
  bundle: BundleMetadata | null; // source bundle metadata, when present
}
```

The parser extracts and validates records but does not flatten inheritance or
require external parents to be in the uploaded archive. Rust derives missing
profile type from the settings-ID fields, as upstream import does. Bundle naming
and internal references remain Rust-owned. This is a transport envelope, not a
new flattened profile format; concrete metadata fields will be pinned in the spec.

`selection_json` uses the current Rust `ProfileSelection` vocabulary:

```json
{"machine":"printer preset name","process":"process preset name","filaments":["filament preset name"]}
```

Filament order is significant and duplicate filament selections are valid.
Selectable names identify presets within their kind. Inheritance-only templates
are vendor-scoped: identical names in LH and OrcaFilamentLibrary are valid and
resolve local-vendor first, then the library (`PresetBundle.cpp:4885-4918`).
Conflicting duplicates within the same scope, or ambiguous selectable names, are
rejected rather than silently overwritten. Import replacement UI is deferred.

`filterCompatiblePresets` result:

```ts
{
  processes: string[];
  filaments: string[];
  diagnostics: Array<{kind: "process" | "filament"; name: string; message: string}>;
}
```

Return selectable preset names only; inheritance-only records remain available
for resolution. With no process, filter both lists against the printer. With a
process, additionally test filaments against that process. It must not silently
choose a different printer/process/filament or auto-repair the user's selection.

**Confirmed source policy:** upstream catches malformed
compatibility expressions and returns `true` (`Preset.cpp:781-787,830-836`).
Propose retaining that exact upstream result with a returned diagnostic, rather
than inventing strict-false behavior or hiding the error. Missing parents, cycles,
invalid JSON and invalid selections remain errors, not this source-defined case.

## Model/config behavior

- Existing `sliceProject` / `sliceProjectPlate` retain embedded-3MF behavior.
- `sliceWithProfiles` is an explicit selected-profile path. Compose in core and
  enter the existing project/Print lifecycle, not the older standalone STL scaffold.
- For 3MF, preserve geometry, instances, plate identity and scoped overrides;
  global selected-profile precedence must be source-cited and verified against
  native `--load-settings` before implementation. Do not rewrite a ZIP merely to
  feed the existing byte-only entry.
- `plate` keeps the existing `plater_id` meaning; STL has no plate selector.
- `seed` creates the existing explicit per-slice SlicerContext, preserving template
  and FuzzySkin stream isolation. WASM does not invent entropy.
- Browser `filename` is File.name (a basename, not a filesystem path). Use it for
  STL object/volume naming in both scene loading and slicing. 3MF stored names stay
  intact. No hardcoded `model` substitute and no stripping `.stl`.

## Included / deferred and confirmation gate

Included: FFF profile input, inheritance and full-config connection, compatibility,
JSON/archive parsing required by the import UI, thin bindings, STL filename names.
Deferred: SLA, cloud/account sync, on-disk install, overwrite prompts/persistence,
new placement/UI behavior, multi-transform membership/shared-centering task 23,
and GUI `PresetBundle::full_fff_config` lifecycle/compatible-field erasure.
No claim of full PresetBundle parity or completion of the historical stock sweep.

After interface confirmation: write the source-cited spec/plan and concrete failure
inventory, establish public native full-output RED (same model and system JSON via
`--load-settings`), then implement. Validate complete `ordered_bytes_generator_only`,
workspace Nextest, fmt, default Clippy warning delta, actual wasm32 core/WASM builds,
and full official/seeded controls. Follow exact-candidate review/approval and
latest-main revalidation before fast-forward integration.
