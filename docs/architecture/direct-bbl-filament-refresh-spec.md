# Direct BBL 3MF filament refresh specification

## Status and scope

This is the B1 specification only. It is based on Ares tip `0a2e546bcd3aaa1189b3770f7c7e039c3d39d3ae` and the pinned OrcaSlicer 2.5.0 source `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`. B1 covers the platform-neutral core seam and the native CLI adapter. It does not add implementation, tests, or resource files in this change.

B2 is the later WASM/web adapter slice. Its required interface is specified here so B1 does not create a core API that cannot be supplied by a browser.

The missing behavior is the Native BBL/Orca 3MF filament refresh for a direct project slice. The three existing hull witnesses first differ in the CONFIG block at `activate_air_filtration_during_print`: Ares emits one value and Native emits one value per logical filament. The existing selected-profile variant overlay is not the owner of this behavior and must not be invoked a second time.

## Upstream source boundary

The rewrite boundary is the Native direct project load/refresh path in `OrcaSlicer/src/OrcaSlicer.cpp`:

- `1880–1893`: derive `current_filaments_system_name` from embedded filament names and inheritance;
- `2786–2817`, especially `2793–2817`: for `is_bbl_3mf && new_printer_name.empty()`, resolve each embedded filament system preset and insert successful resolutions into `load_filaments_config`, with `load_filaments_refresh=true`;
- `3510–3545`: validate/regenerate `filament_self_index` and normalize variant-aware values;
- `3639–3701` and `3753–3759`: apply retained and newly supplied per-filament variant segments;
- `3823–3845`: rebuild `filament_self_index` from the resulting variant counts;
- `2805–2807`: unresolved system preset log and keep-slot behavior;
- `libslic3r/PrintBase.hpp:66–75`: the slicing-notification enum, which has no unresolved-system-preset variant.

The exact unresolved-preset source behavior is:

```text
OrcaSlicer.cpp:2805  resolve_system_preset(...)
OrcaSlicer.cpp:2806  warning: system filament preset ... not resolved ... filament ... keeps its values
OrcaSlicer.cpp:2807  continue
```

Ares destination boundaries are:

- direct API construction in `crates/ares-core/src/project_slice.rs:112–137` and `project_slice/validated.rs:39–76`, which currently set `profile: None` and `refresh: None`;
- direct project loading in `crates/ares-core/src/project_slice/source.rs:75–113` and `crates/ares-core/src/project_slice/state.rs:31–90`;
- embedded 3MF metadata/config loading in `crates/ares-core/src/project/load.rs`, which already detects `bbl_family_3mf` and normalizes embedded settings;
- reusable lower-level profile/preset operations under `crates/ares-core/src/profiles/project_refresh/`, without invoking the selected-profile `ProjectPresetRefresh` orchestration.

## Core API contract

Direct project slicing must receive an optional parsed system bundle:

```text
Option<&PresetCatalogHandle>
```

`PresetCatalogHandle` is the existing core-owned, filesystem-free representation of vendor indexes, profile records, inheritance, and imported records (`crates/ares-core/src/profiles/catalog.rs`). The direct API and its validated/processed variants must carry this value through the existing `ProjectSource`/direct-load seam. The exact Rust parameter placement may be a small request struct if that avoids adding another argument to every public function; it must remain a public API input and must not hide a process-global catalog.

The direct loader applies the refresh only when all of the following hold:

1. the input is a 3MF project;
2. the loaded document is BBL/Orca family (`bbl_family_3mf`, corresponding to Native `is_bbl_3mf`);
3. the embedded project has the system-filament identity information needed by the Native branch;
4. the caller supplied `Some(&PresetCatalogHandle)`.

Non-BBL 3MF input, STL input, and `ProjectInput::Model` behavior are unchanged.

A BBL/Orca 3MF with `None` is not allowed to silently use the old direct path. It returns a dedicated missing-system-bundle `SliceError` before effective-config resolution. This makes the missing dependency explicit at the public boundary and prevents a false parity result. The error must identify that a system preset bundle is required for direct BBL/Orca 3MF slicing; it must not be a generic preset-not-found error.

With `Some(bundle)`, each embedded filament system name is resolved in slot order. A named preset that is absent from the supplied bundle follows Native `OrcaSlicer.cpp:2805–2807`: preserve that slot's existing values and continue resolving later slots. Native emits `BOOST_LOG_TRIVIAL(warning)` at line 2806; this is a diagnostic log, not a slicing notification or an exception. No slot compaction or substitution with another preset is allowed.

Do not create an unresolved-system-preset `PrintWarning`. `PrintBase.hpp:66–75` defines notification values 0 through 6; value 3 belongs to `SlicingEmptyGcodeLayers`. Its enum has no unresolved-preset variant. Keep `SlicedPlate.warnings` for source-defined slicing notifications and leave direct byte API return shapes unchanged. The public behavior to verify on a resolver miss is the preserved slot, successful continuation and complete Native-equivalent output, not an invented warning transport.

The refresh must run before direct effective-config resolution and CONFIG block serialization, after the embedded project has been loaded and metadata classified. Caller overrides retain their current ordering. The implementation may share the lower-level preset-chain and filament variant replacement primitives with `ProjectPresetRefresh`; it must not construct or call the selected-profile refresh orchestration, because that path already owns project-derived selection and would apply a second refresh.

## CLI adapter contract (B1)

The direct CLI route is `crates/ares-cli/src/main.rs:87–181` when there is no `--project-preset-catalog`, `--load-settings`, or `--load-filaments`. Its current `compatible_machine::load` call only reads `profiles/BBL/machine_full/<embedded-printer>.json` from `--resources-dir`; that file supports the upward-compatible-machine report and is not a complete system bundle.

B1 must adapt this route as follows:

1. Keep `--project-preset-catalog` on its existing selected-profile/project-refresh route. Do not route it through direct BBL refresh or apply both paths.
2. When direct 3MF slicing receives `--resources-dir`, load the complete vendor profile bundle from that resource root into `PresetCatalogHandle`: vendor index plus the machine/process/filament records and inheritance files required by the embedded system names. The resource loader is CLI-owned filesystem code; core receives parsed bytes/records only.
3. Reuse the catalog loading, digest/shape validation, and staged-system-tree checks already established by `crates/ares-cli/src/project_presets.rs` and the parity support loader, rather than inventing a second catalog format. The resource-root loader may produce the same vendor envelope consumed by `PresetCatalogHandle::add_vendor`.
4. Pass `Some(&catalog)` to the direct core request when the complete bundle was loaded from the explicitly supplied `--resources-dir`. `args.rs:32` defines `resources_dir` as an `Option`; there is no executable-adjacent default for this bundle. When `--resources-dir` is absent, a BBL/Orca 3MF direct slice passes `None` and receives the dedicated missing-bundle error. It must not guess an executable-adjacent directory and must never silently run the old direct path.
5. Preserve `compatible_machine::load` and its `machine_full` behavior separately. The upward report and the filament system bundle have different Native source responsibilities and must not be conflated.

The direct CLI route therefore becomes source-cited parity with Native's complete bundle requirement, while `--load-settings`/`--load-filaments` and `--project-preset-catalog` remain their existing supplied-profile routes.

System resource JSON accepts a leading UTF-8 byte-order mark for both vendor indexes and indexed preset documents, matching Native's JSON reader (`PresetBundle.cpp:8231–8234,8432–8434`). The CLI normalizes that byte prefix at its filesystem boundary; core receives the existing parsed catalog transport. The CLI regression test supplies a BOM-bearing index and referenced filament document and retains a repeatable G-code artifact.

## B2 WASM/web interface contract

The browser changes share the core/CLI branch; these interfaces remain explicit:

- `ares-wasm::sliceProject`, `sliceProjectPlate`, and processed direct variants currently call direct core APIs without a catalog (`crates/ares-wasm/src/lib.rs:49–105`). B2 must add a catalog-bearing API or a catalog JSON argument. The browser supplies vendor envelopes/imports as bytes or JSON; core performs no filesystem access.
- Validated core direct calls carry `plate`, `overrides`, and `system_catalog` in `DirectProjectSliceRequest`; the validated byte/processed counterparts accept that request plus metadata and context. This is an argument carrier for the existing source-loading boundary, not a separate slicing orchestration.
- The existing `PresetCatalog`/catalog JSON transport in `crates/ares-wasm/src/profiles/catalog.rs` and `profiles.rs` is the compatible transport. B2 should either pass its underlying parsed handle to the direct request or add a direct slice method accepting the same catalog JSON. It must not create a second incompatible bundle type.
- `web/packages/ares-wasm-js/src/index.ts:87–90`, its worker/protocol types, README API contract, and direct consumers need coordinated signature/type updates. The exact direct fixtures are:
  - `web/e2e/consumer/main.ts` is the generic file-upload consumer; `web/e2e/package.spec.ts:15` supplies `tests/parity/arachne/default-prisms.3mf`. That fixture has `Application=BambuStudio-02.06.00.51` and `OrcaSlicer=2.4.2`, so it is BBL/Orca and will require a bundle under B1.
  - `web/e2e/ksr-project-page/main.ts` slices `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf` as set by `web/e2e/ksrProject.spec.ts:16`. Its metadata is also `Application=BambuStudio-02.06.00.51` and `OrcaSlicer=2.4.2`, so it is BBL/Orca and will require a bundle.
  - `web/e2e/wasmMoves.mjs` runs two direct fixtures at `:28`: `tests/parity/orca_cli_ender3.3mf` (`ender3`) and `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf` (`ksr`). Both are BBL/Orca 3MF fixtures; both will require the catalog-bearing B2 API.
- The normal web app project-slice path already calls `plater.sliceWithProfiles(..., FROM_PROJECT, ...)` in `web/app/src/state/actions.ts:151`; `web/app/src/state/projectPresets.ts:8–10` builds a catalog from `FILAMENT_LIBRARY` and installed vendors. That path should not receive a second refresh. WEB may keep this path as-is or route the catalog-bearing direct API through it, but the choice must be explicit.
- A browser caller that supplies no catalog for a BBL/Orca 3MF receives the same explicit missing-bundle error as the core direct API. Non-BBL projects remain usable without a bundle.
- **Batch constraint:** B1 and B2 must land in the same batch. B1 must not be landed alone while these existing direct WASM/web consumers still call the no-catalog API with BBL fixtures; B2 must first provide the catalog-bearing browser interface and update these consumers (or explicitly change their fixture/expectation under the same reviewed batch).

The core/CLI and browser changes form one landing batch and are reviewed together.

## Existing tests affected by B1

The direct-call inventory and fixture contexts are listed below. Every direct 3MF test must make the bundle dependency explicit. Tests using STL or a non-BBL project remain unchanged and are listed to prevent accidental scope expansion.

| Existing test/caller | B1 disposition | Reason |
|---|---|---|
| `orca_parity::runner::native_exclude_outline_migration` (three hull cases) | Pass a complete public catalog/bundle to the direct public API | These are the three BBL/Orca 3MF witnesses; their first diff is the missing filament refresh. |
| `orca_parity::runner::native_bbl_boundary_migration` | Update the `runner::application::slice_project_plate` adapter's `None` branch to pass a complete bundle or assert the explicit missing-bundle error; keep its supplied-profile branch on `slice_with_profiles` | Since W2 commit `317f774c0`, this file no longer calls the direct API itself, but its application adapter `None` branch still reaches direct slicing. Both `bbl-object-boundary.3mf` and `bbl-toolchange-boundary.3mf` are BBL/Orca projects. |
| `cli_progress.rs` | Pass a bundle for its BBL 3MF fixture; retain the progress assertion | It enters the direct 3MF API and must not become a missing-bundle failure merely because the test is about progress. |
| `orca_parity::cli_retained_exit.rs` | Pass a bundle for the 3MF direct call | It directly slices a project and checks CLI retention/exit behavior; the new dependency must be supplied by its test harness. |
| `orca_parity::empty_selected_plate_controls.rs` and `empty_selected_plate.rs` | Pass a bundle | These are direct 3MF plate-selection calls; refresh must happen before plate selection output without changing empty-plate semantics. |
| `orca_parity::option_coverage::print_extruder_identity.rs` | Pass a bundle for the generated 3MF direct call | It exports/reloads a project and then uses the direct project API; the generated BBL metadata must use the same bundle as Native. |
| `orca_parity::plater_delete_instance_import.rs` | Pass a bundle | It directly imports/slices a BBL-family 3MF and must retain its object/instance assertions after refresh. |
| `orca_parity::print_volume_printable.rs` | Pass a bundle | Its direct 3MF cases exercise project loading and print-volume output; refresh is a load-stage prerequisite, not a geometry change. |
| `orca_parity::runner::adjacent_color.rs`, `processor_migration::stops.rs`, and `user_overrides.rs` | Pass a bundle for their direct 3MF branches; leave any STL branch unchanged | These helpers combine direct 3MF and/or STL coverage. Only BBL/Orca 3MF inputs require the bundle; STL must not acquire a catalog requirement. |
| `orca_parity::runner::native_jerk_semm_migration.rs` | Update the `runner::application::slice_project_plate` adapter's `None` branch to pass a complete bundle or assert the explicit missing-bundle error; keep its supplied-profile branch on `slice_with_profiles` | Since W2 commit `317f774c0`, this file no longer calls the direct API itself, but its application adapter `None` branch still reaches direct slicing. Its BBL 3MF fixtures still require the system bundle. |
| `orca_parity::runner::native_plate_origin_migration::{first_layer_print_space.rs, mod.rs}` | Pass a bundle | These direct 3MF plate-selection witnesses must reach the same effective config and plate-origin checks after the refresh. |
| `orca_parity::print_validation::bed_compatibility_controls.rs` | Leave STL-only calls unchanged; pass a bundle only for any BBL 3MF branch | Its direct API coverage includes STL validation. The B1 predicate is input-format and metadata based, not caller-name based. |
| `orca_parity::runner::application::project_presets.rs` and `support/project_preset_catalog.rs` | Keep the supplied-catalog/profile branch; change the `None` direct branch to pass the bundle or assert missing-bundle explicitly | This helper intentionally has two modes. The project-supply mode already uses `slice_with_profiles`; it must not receive a second refresh. |

Required direct coverage includes the three hull cases run through the public direct API with a supplied bundle, plus a BBL/Orca direct call with `None` asserting the dedicated missing-bundle error. A supplied-bundle unresolved-name case must exercise Native `2805–2807`, assert slot preservation and compare complete output. It must not require a `PrintWarning` for Native's diagnostic-only log.

### Core and WASM direct-call inventory

These callers also cross the direct load boundary (`OrcaSlicer.cpp:2786–2817`). Their existing assertions remain unchanged; a supplied catalog is an explicit load input, not a replacement expected output. Invalid-input and non-BBL controls keep their original inputs when the refresh predicate is not reached. Each owning test binary is included in both base/tip modes.

| Test file | Input disposition | Existing behavior retained |
|---|---|---|
| `crates/ares-core/tests/adaptive_isolation/mod.rs` | Supply a catalog for BBL direct cases; retain non-BBL inputs | Adaptive slicing remains isolated from unrelated config. |
| `crates/ares-core/tests/gcode_thumbnail_image_free.rs` | Supply a catalog for BBL direct cases | Image-free thumbnail export assertions. |
| `crates/ares-core/tests/gcode_thumbnail_images/support.rs` | Supply a catalog in its direct BBL helper | All thumbnail image assertions in the owning `gcode_thumbnail_images` binary. |
| `crates/ares-core/tests/multi_plate_instances_keep_global_coordinates.rs` | Supply a catalog for direct BBL slicing | Global coordinates and selected-plate instance behavior. |
| `crates/ares-core/tests/painted_attribute_in_wrong_namespace_is_rejected.rs` | Keep malformed-input rejection; supply a catalog only if a valid BBL control reaches slicing | Namespace rejection, not a missing-bundle substitution. |
| `crates/ares-core/tests/plateless_instances_hit_no_suitable_objects_gate.rs` | Supply a catalog when its BBL input reaches the direct load stage | Original no-suitable-objects rejection. |
| `crates/ares-core/tests/post_process_rejection.rs` | Supply a catalog for BBL direct cases; keep STL inputs | Post-process validation and export rejection. |
| `crates/ares-core/tests/processed_filament_config.rs` | Explicit empty catalog through `processed_after_system_preset_miss`; original fixtures and assertions unchanged | Resolver miss (`OrcaSlicer.cpp:2805–2807`) leaves missing density for the `PrintConfig.cpp:2864–2870` default, rather than filling it from an installed preset. |
| `crates/ares-core/tests/profile_entry_processed.rs` | Keep profile-selected requests; supply a catalog only to direct BBL comparison calls | Processed-versus-profile output comparisons, without a second profile refresh. |
| `crates/ares-core/tests/slice_bed_type_initialization.rs` | Supply a catalog for BBL direct cases; keep model-only cases | Bed-type initialization and validation. |
| `crates/ares-core/tests/slicing_status.rs` | Supply a catalog for BBL direct cases | Status reporting assertions. |
| `crates/ares-core/tests/template_errors_surface_upstream_format.rs` | Supply a catalog for BBL direct cases | Upstream template error text and attribution. |
| `crates/ares-core/tests/version_identity.rs` | Supply a catalog for BBL direct cases | Exact generated-by version identity. |
| `crates/ares-core/tests/profile_filament_self_index.rs` | Keep explicit missing-bundle control and supplied-catalog cases | Variant/self-index behavior, successful supplied input, preserved unresolved slot and no synthetic print warning. |
| `crates/ares-cli/tests/orca_parity/option_coverage/direct_bbl_missing_system_preset.rs` | Change only the embedded filament name at runtime | Qualify Native's unresolved name and retained density, flow and diameter, then compare complete Native/API/CLI output. |
| `crates/ares-wasm/tests/custom_gcode_overview.rs` | Supply catalog JSON to processed direct BBL calls | Custom G-code overview rows. |
| `crates/ares-wasm/tests/estimation.rs` | Supply a catalog through the public core processed API used to construct adapter input | Estimation data. |
| `crates/ares-wasm/tests/marker_time_tagged.rs` | Supply a catalog through the public core processed API used to construct adapter input | Marker timing and move tags. |
| `crates/ares-wasm/tests/role_filament_usage.rs` | Supply a catalog through the public core processed API used to construct adapter input | Per-role filament usage. |
| `crates/ares-wasm/tests/direct_project_catalog.rs` | New explicit no-catalog rejection and supplied-catalog success | Public WASM byte API contract and repeatable G-code artifact; absent from the base. |

The shared integration-test drivers in `tests/support/{direct_catalog,slice_project,slice_project_plate,slice_project_plate_processed,slice_project_plate_validated}.rs` supply the catalog through public core APIs. Native-mode runs use the supplied `ARES_PARITY_PROJECT_PRESET_CATALOG`; standalone isolated runs supply an explicit empty catalog, exercising the unresolved-name/retained-slot path at `OrcaSlicer.cpp:2805–2807`, never the no-catalog legacy path. Dedicated missing-bundle controls remain unwrapped. The WASM overview test passes the same catalog JSON directly to its catalog-bearing byte API.

The 12 caller-rendered thumbnail Native comparisons have no available `ARES_GCODE_THUMBNAILS_BIN` supplier. Both modes retain the same dependency failure; these cases are **not verified** and are excluded from P/F comparison, not counted as passes. `project::ENDER3` is `tests/parity/orca_cli_ender3.3mf`, whose metadata is BambuStudio/OrcaSlicer, as is `project::BBL`. They are BBL inputs and are not outside the refresh predicate. The shared driver therefore supplies the catalog without changing either fixture's metadata. The image-free and empty-caller-image controls do not depend on that supplier and remain required base/tip comparisons with their original complete-stream/error assertions.

The table records the required disposition, not an unverified claim that every listed file needs an edit. The final inventory records which inputs actually reach the BBL predicate and the exact input-only adaptations made. No comparator, retained assertion, or expected output is weakened.

## Base/tip inventory acceptance

Every affected test file listed above must be included in a frozen base-versus-tip inventory at base `317f774c06033bdabbfaefa24d8fae7c9fd45106` and the combined B1/B2 implementation tip, in both official and seeded modes, with complete parity environments and the worker's CPU range 36–39. The inventory must prove P→F = 0. The base includes the application adapter's routing changes for the BBL boundary and jerk/SEMM witnesses; earlier `0a2e546bc` runs are invalid for this comparison. The direct BBL hull cases, native BBL boundary cases, CLI direct 3MF cases, and the B1 missing-bundle witness must be selected explicitly; STL/non-BBL controls remain in the inventory to prove they are unaffected.

A supplied bundle can legitimately change output for a previously passing Native comparison if the old Ares path omitted Native's refresh. That is still a P→F and is not admissible as an unexplained fixture update. Such a change must be attributed to the exact upstream refresh operation and lines (`OrcaSlicer.cpp:1880–1893`, `2786–2817`, and/or `3510–3845`), with the before/after first difference and the bundle identity recorded. A new failure without that source-cited explanation is a rejected B1 result.

The KSR browser Native comparison supplies `ARES_PARITY_PROJECT_PRESET_CATALOG` when that frozen input is provided, rather than silently substituting the freshly generated profiles package. The installed package currently has 73 vendors, whereas the frozen KSR catalog has 72; their identities are recorded separately. The package-consumer witness still supplies the installed profiles catalog matching its CLI resource input. No comparison rules are changed.

The B2 browser inventory must be part of the same batch acceptance: the BBL direct consumers listed above must run against the catalog-bearing API in both modes (where a Native oracle exists), and any changed expectation must be source-cited to the same Native refresh boundary. B1 cannot be landed or claimed batch-ready before B2 is prepared for these fixtures.

## Explicit non-goals and acceptance boundary

- No selected-profile `ProjectPresetRefresh` or `profile_input/overlay` changes.
- No second refresh on `slice_with_profiles`, `ProjectPresetRefresh`, `--load-settings`, or `--load-filaments` paths.
- No filesystem or resource lookup in `ares-core` or WASM.
- No fallback to the old direct path for BBL/Orca input when the required bundle is absent.
- No B2 web implementation in B1.

B1 is ready for implementation only after this specification is accepted. The implementation must use the pinned upstream lines above, keep direct non-BBL behavior unchanged, and prove both supplied-bundle hull parity and explicit missing-bundle behavior.
