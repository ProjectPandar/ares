# Profile-based slicing: upstream rewrite slice

Native RED precedes production edits.
Contract: [ARD](../../architecture/ard-profile-slice-entry.md).
Upstream revision: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Bounded slice

This slice covers STL profile slicing, typed five-argument
transport and CLI config export parity only. It does not satisfy this whole
specification. Selected-profile 3MF, compatibility/import APIs, loadScene/web
migration and BBL filtering measurement remain open. Existing embedded 3MF slicing
is a regression control; it is not evidence for selected-profile 3MF support.

## Owners and scope

Port `PresetBundle.cpp:4850-4969` loading/parent overlay and the CLI's
`OrcaSlicer.cpp:1953-2097,2750-3011,3257-3290` selected-profile loading/copy rules
into `ares-core::profiles`. `GCode.cpp:5591-5656` owns config export. GUI
`PresetBundle::full_fff_config` lifecycle and its `compatible_*` erasure are deferred;
copying that erasure would contradict the CLI oracle. See the ARD's exact mapping.
Reuse existing fragment/inheritance/composition modules; close their gaps rather
than introduce an independent pipeline. Feed the resulting in-memory Project into
the existing project-slicing lifecycle. Its byte-only ProjectSource is a temporary
compatibility shell to extend at the load seam, not a reason to manufacture ZIPs.
`Preset.cpp:771-851` owns compatibility, including expression errors returning
true with diagnostics. `PresetBundle.cpp:1304-1549` owns import extraction and
normalization; persistence, cloud sync, filesystem installs and UI prompts are not
ported. `Format/STL.cpp:17-40` owns filename-based object naming.
SLA and task-23 multi-transform/shared-centering remain deferred.

`OrcaSlicer.cpp:1953-2080` is the native CLI oracle boundary for explicit settings.
The CLI accepts flattened files, not separate unresolved parent files. Reference
preparation must therefore independently overlay the same raw system fragments;
it must not use production Rust composition to manufacture the expected output.
Retain raw inputs, flattened native inputs, arguments/status and full G-code.
No smoke overrides, fixed-template substitutions or weakened comparator.
The STL tracer positions the shared cube's vertices at LH's bed center before
passing identical bytes to both engines, then uses `--arrange 0`; GUI arrangement
is not part of this libslic3r slice. Root template names are vendor-scoped, with
local-before-library parent lookup; selectable-name ambiguity is a separate error.

## Bundle/compatibility slice contract

This candidate lands `parsePresetBundle` and `filterCompatiblePresets`.

`filterCompatiblePresets(catalog_json, printer, process?)` resolves the named
active printer (and optional process) inside the existing `Catalog`, merges its
inherits chain, then tests every selectable process and filament preset exactly
as `Preset.cpp:771-851`:

- `is_compatible_with_printer`: OrcaFilamentLibrary filament `m_excluded_from`
  (alias-matched concrete-vendor `compatible_printers` lists per
  `PresetCollection::update_library_profile_excluded_from`,
  `Preset.cpp:3705-3736`) reject first; an explicit non-empty
  `compatible_printers` list outranks any condition; otherwise a non-empty
  `compatible_printers_condition` is evaluated with
  `PlaceholderParser::evaluate_boolean_expression` against the merged active
  printer config plus `printer_preset`/`num_extruders`
  (`Preset.cpp:843-850`); a parse/eval error returns `true` with a diagnostic
  (`Preset.cpp:830-836`); fallthrough accepts `is_default`, empty active name,
  empty list, exact active-name match, or — for a non-system active printer —
  its immediate `inherits` parent match (`Preset.cpp:799-806,840`).
- `is_compatible_with_print` additionally gates filaments when a process is
  selected: same list-vs-condition structure, but the condition evaluates
  against the merged active *process* config (`Preset.cpp:771-792`),
  error→true with diagnostic.
- Returns `{processes, filaments, diagnostics}`; diagnostics carry kind, name
  and the upstream message for error→true cases only. Non-instantiable
  (`instantiation == "false"`) presets are excluded from both lists but stay
  resolvable as inheritance parents. Unknown printer/process names are errors.

`parsePresetBundle(input, filename)` ports `PresetBundle.cpp:1304-1549`
byte-oriented: a `.json` input parses one preset record; a ZIP input
(`.orca_printer`/`.orca_filament`/`.orca_bundle`/`.zip`, magic `PK\x03\x04`)
reads `bundle_structure.json` metadata plus each member `.json` record
in-memory — no temp dirs, overwrite prompts or persistence. Each record keeps
its original option values and `inherits`; kind is inferred from
`printer_settings_id`/`print_settings_id`/`filament_settings_id` presence
(`PresetBundle.cpp:1469-1482`), falling back to the record `type` for
plain-JSON presets. The result is `{profiles:[...], bundle:{...}|null}`; it
does not flatten inheritance and does not require external parents to be in
the same input. Malformed JSON/ZIP are errors.

`loadScene` filename, web caller migration, the real BBL filtering E2E timing
measurement and selected-profile 3MF remain open after this slice.

## Agreed public test seams and failure inventory

1. Core profile slicing / WASM sliceWithProfiles: same STL or 3MF and selected
   system profiles versus native `--load-settings` / `--load-filaments`.
   Failures: profile settings ignored; wrong defaults/overlay order; lost metadata,
   filament IDs/vector cardinality; old STL scaffold used; wrong filename;
   discarded 3MF instance/plate/scoped config; seed leakage; unwanted mutation of
   the existing embedded-3MF path. Compare complete ordered generator-only bytes.
2. Compatibility export: explicit list outranks condition; user printer's immediate
   parent match; process-dependent filament filtering; cross-vendor and library
   exclusions; inherited conditions; non-instantiable presets excluded; expression
   syntax errors return true plus diagnostics. Missing parents/cycles and invalid
   selections are errors, never expression-error compatibility.
3. Import export: JSON and both Orca archive kinds, settings-ID kind inference,
   bundle naming/references, malformed archive/JSON, duplicate records and external
   inheritance dependencies. Extraction is memory-only and does not flatten early.
4. Scene export through web worker: required filename reaches STL object and volume
   names, extension preserved; stored 3MF names unchanged. Update callers together.

Build each vertical slice's tests before its implementation. The first runtime
RED uses the existing public STL entry as an explicitly labelled pre-feature
baseline because profile slicing does not exist yet; switch only that invocation
to the new public entry during implementation, retaining oracle and comparison.
Do not claim the baseline call already exercises a nonexistent profile API.

## Performance and acceptance

No handle/cache API. Record one end-to-end filterCompatiblePresets duration for a
real BBL vendor catalog (including JSON decoding) in E2E artifacts with catalog
byte size and environment. This is measurement, not a timing waiver or flaky gate.

Exit: public complete-output E2E parity; workspace Nextest; cargo fmt; default
Clippy with no added warning set; actual wasm32 core/WASM build; full official and
seeded controls. Preserve historical stock failures and known official
coordinate-witness cases. No full-stock completion claim.
