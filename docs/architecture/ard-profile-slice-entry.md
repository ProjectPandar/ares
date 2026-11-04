# ARD draft: profile-based slicing interface

Status: accepted. STL entry, bundle import and compatibility are merged; 3MF overlay entry
follows.

## Printer listing and the processed profile entry (2026-10-06)

`PresetCatalogHandle::printers()` lists selectable machine presets as
`SelectablePrinter {vendor, name, printer_model, printer_variant}` in
collection order (`Preset.cpp:3355`), grouped by their owning vendor
(user imports last, vendor `"user"`). Filtering follows the upstream
visibility rules the CLI observes: load-time visibility is
`instantiation != "false"` (`Preset.cpp:1663`) and printers with
unresolvable parents stay out (`Preset.cpp:1685-1691`).

**Install-matrix correspondence:** the GUI derives per-model/variant
visibility from the AppConfig install matrix
(`Preset::set_visible_from_appconfig`, `Preset.cpp:851-864`), and the preset
combo skips invisible entries (`PresetComboBoxes.cpp:1170-1186`). Ares has no
AppConfig; the caller's `add_vendor` set is the installed set and no
per-model/variant filtering is applied. This is a deliberate simplification
of GUI state: the CLI itself performs no visibility filtering
(`OrcaSlicer.cpp` slicing never consults `is_visible`), so the Ares entry
matches the CLI path it ports. `printer_model`/`printer_variant` are read
from each preset's parsed config here so TS never parses preset bodies
(ConfigWizard-style grouping stays client-side on returned data).

`slice_with_profiles_processed` mirrors `slice_project_plate_processed`:
the shared `slice_source(record_moves)` path returns `ProcessedGCode` with
byte-identical G-code to the plain entry; on a 3MF whose embedded config
equals the selected profiles the recorded moves equal
`slice_project_plate_processed` on the same archive.

## 3MF selected-profile overlay entry (2026-10-05)

Upstream semantics (verified in `OrcaSlicer.cpp` at `8500fcdc`):

- `:1624-1640` — a `.3mf` first input loads the embedded config as the
  full-config base (`LoadModel|LoadConfig|AddDefaultInstances|LoadAuxiliary`).
- `:2750-2812 update_full_config` — every key the loaded presets supply
  overwrites the embedded value; keys no preset supplies keep the embedded
  value. Machine applies first (`:2849-2867`), then process (`:2993-3011`),
  then filaments per loaded index (`:3115-3337`, `set_at(src, i-1, 0)` at
  `:3301`; growth fills with `values.front()`, `Config.hpp:427-447`).
- Metadata exclusions (`:2798-2804`): `compatible_print(s)`, `model_id`,
  `inherits`, `dev_model_name`, `name`, `from`, `type`, `version`,
  `setting_id`, `instantiation` never copy; embedded preset metadata stays.
- Settings IDs do copy: `printer_settings_id`/`print_settings_id` become the
  loaded names (`:2048,:2088`); `filament_settings_id` is written per loaded
  slot, tail slots keep embedded entries, and the loaded preset's own entry is
  erased (`:3155-3158`).
- `different_settings_to_system` clears the process slot, loaded filament
  slots and machine slot when all three kinds are loaded (`:2838,:2978,:3161`);
  group vectors are `filament_count + 2` (`:2817-2818`).
- `inherits_group` records per-slot system parents (loaded slots only).
- `print_compatible_printers` is overwritten unconditionally with the loaded
  process's `compatible_printers`, including absent → empty (`:2980-2984`).
- `:2275,:3680-3699` — a 3MF whose loaded filaments all map to one preset
  name disables the prime tower (smooth timelapse reverts) and records
  `enable_prime_tower` in the process slot of `different_settings_to_system`.
- Count mismatch: upstream `different_settings[filament_index]` writes past
  the `filament_count + 2` vector when more filaments are loaded than
  embedded (undefined behavior); Ares rejects that input explicitly.

Ares implementation: `Project::from_3mf_with_profiles`
(`crates/ares-core/src/project/domain/profile_input.rs`) loads the project
normally, overlays preset keys per the rules above in the raw presence view,
re-resolves `ProjectSettings` through the presence-aware
`ProjectSettingsBuilder`, and re-enters the existing `prepare_loaded_project_slice`
lifecycle. `Catalog::compose` additionally returns one merged raw per selected
filament for the per-slot overlay. STL behavior is unchanged;
`sliceWithProfiles` accepts 3MF bytes without a new export. Multi-variant
filament migration (`set_with_restore_2`) is deferred: selections whose
filament variant groups differ from the embedded groups are not supported.
The slice entry performs no machine-switch compatibility check, matching the
CLI, whose `--load-settings` path does not gate on it; compatibility stays
with `filterCompatiblePresets`. E2E: `profile_entry_3mf_load_settings`
exports a native 3MF and compares full ordered output against the native
`3mf --load-settings/--load-filaments` slice; unit tests pin retention,
per-slot tail preservation, count rejection, unconditional
`print_compatible_printers`, and the single-preset prime-tower rule.

Interface decision (2026-10-05): native RED before implementation.
Source-defined expression error=true with diagnostics and
required loadScene filename. Update web callers in the same candidate. Record a
real BBL filtering E2E duration including catalog parsing; do not add handle caches.
Commit this ARD with the implementation.

Base before rebase: `16c2dc90`; rebased onto `19e24fa8`.
The M853 rebase preserves its optional processed-move result: profile/ordinary
slicing request G-code only; `slice_project_plate_processed` retains move recording.
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Current bounded slice

This slice implements the STL selected-profile entry and five-argument WASM
transport. Selected-profile 3MF remains rejected by
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
The five-parameter slicing export follows an exact-version
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

**Decision revision (2026-10-06, stateful `PresetCatalog` handle):** the
no-handle rule above applied to the stateless transport design, where every
call re-sent the catalog. With real vendor envelopes (multi-megabyte) the
per-call stringify/parse cost is unworkable for filtering interactions, so
ares-wasm now exposes a per-page `PresetCatalog` class
(`new`/`addVendor(bytes)`/`addImport(bytes, filename)`/`filterCompatible`)
wrapping `ares_core::PresetCatalogHandle`. This is still not a global
registry: each instance is independent, owned by the caller, and garbage
collected with it. Upstream boundary: `PresetCollection::load_presets`
(`Preset.cpp:1573+`) holds loaded vendors in the collection and
`PresetBundle::import_json_presets` (`PresetBundle.cpp:1450+`) adds user
imports; filtering reuses the held records. A preset whose non-empty
`inherits` cannot be resolved is skipped at load upstream
(`Preset.cpp:1685-1691`, `PresetBundle.cpp:1521-1525`, `Preset.cpp:2300-2305`),
so it never enters filtering results; the handle keeps the record and reports
it in the structured `missing_parents` list instead, and it re-enters once
the parent is added (incremental loads may arrive after children, which
upstream avoids by dependency-ordered loading). One-shot
`filterCompatiblePresets` shares this semantics through the same catalog
code. `sliceWithProfiles` remains stateless (deferred).

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

## Model placement (2026-10-06, `placement` option)

`options_json` gains `placement: "none" | "center"` (default `none`).
`center` ports the CLI `--center` transform: `Model.cpp:704-722`
`center_instances_around_point` (world-bbox center over model-part volumes,
`EPSILON = 1e-4` no-op guard from `libslic3r.h:52`) followed by the Z ground
drop (`OrcaSlicer.cpp:4382-4392`), at `BuildVolume::bed_center()`
(`BuildVolume.hpp:83`, the printable-area bbox center). The native oracle
cannot exercise `--center` directly — the option is commented out in this
Orca build (`PrintConfig.cpp:10775`) — so the E2E compares against a
pre-positioned reference cube (bbox center at bed center, grounded) sliced
through the proven pipeline; identical world geometry must give identical
G-code.

**Known deviation:** upstream STL inputs default to full libnest2d arrange
(`need_arrange` starts true, `OrcaSlicer.cpp:1438`; non-3MF inputs re-set it,
`:1797`; the arrange block runs at `:4890`; the `--arrange` flag's own default
of 0 (`PrintConfig.cpp:10780-10786`) only applies when passed explicitly,
`:7190-7195`). Ares defaults to `none` (model keeps its own coordinates) until
the arrange slice ports the packing; the `placement` API is the seam that
slice will use to converge the default.

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
and full official/seeded controls. Follow
latest-main revalidation before fast-forward integration.
