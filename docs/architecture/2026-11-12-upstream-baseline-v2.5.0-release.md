# Upstream baseline migration: OrcaSlicer v2.5.0 release

Date: 2026-11-12. Status: **proposed implementation specification**.
The release target is chosen. This document does not switch producers or port
release behavior. Its profile-byte and oracle extension must be agreed on before
the corresponding tests and fixes. The beta prerequisite is already on `main` at
`2ec2be77f9b7aba723f0a77a800c4a68d9679f54`; the extension starts from that tree.

## Decision and immutable source identities

Use the official **v2.5.0 release**, not the beta, as the next source authority for
source-cited Rust rewrites of `libslic3r` and `libvgcode`.

| Input | Commit | `resources/profiles` tree |
|---|---|---|
| Previous v2.5.0-beta | `345efec64c7b95a05ce5cf96df0dde66a158835c` | `1a0b4442f231034b13984b227ac5e15e23b92541` |
| Target v2.5.0 | `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19` | `356aeec67e6eb11b0a756c48da789aa6c5f6c19a` |

The official tag resolves to the target commit. The beta is its ancestor.
All upstream paths and function names below refer to the
[release source tree](https://github.com/OrcaSlicer/OrcaSlicer/tree/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19).
The [beta baseline record](2026-10-30-upstream-baseline-v2.5.0-beta.md) remains a
historical decision and inventory. Its old counterpart classifications do not
assert the current implementation status.

A source pin, a verified Native producer, and an executed parity result are
separate identities. Existing beta binaries, resources, caches and PASS results
retain their beta identities. Do not relabel them, support two algorithms, add a
legacy fallback, relax comparators or repair fixtures to conceal release changes.
Existing official, seed-only and merged release producers retain their owners.
The two additional producers included here are segment-colors and initialized
KSR replay. Check existing release handoffs before building either; do not rebuild
unrelated Native variants. The separate version-identity change owns
`generation.rs`'s compatibility constant, the emitted header identity and
test-comparator version regexes. Source-citation comments remain in this change
only after checking their sources.

## Complete `libslic3r` changed-file inventory and ownership

Reproduce the endpoint comparison in an upstream repository containing both
commits. Rename detection is disabled so every path has one inventory entry.

```sh
beta=345efec64c7b95a05ce5cf96df0dde66a158835c
release=40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19
git diff --no-renames --name-status "$beta" "$release" -- src/libslic3r
git diff --no-renames --numstat "$beta" "$release" -- src/libslic3r
git diff "$beta" "$release" -- src/libslic3r
```

There are **61 changed files, all modified**, with no added, deleted or renamed
paths. Each appears once below. The primary source domains retain the beta
inventory's ownership partition. These are file-level ownership assignments, not
claims that all touched behavior is ported or a plan to port everything at once.

| Primary owner | Files |
|---|---:|
| Config/profiles | 13 |
| Geometry/mesh/import | 11 |
| Walls/seams | 1 |
| Fill/support/surfaces | 13 |
| Emission/tool/material planning | 21 |
| Processor/platform | 2 |
| **Total** | **61** |

Upstream filenames in the tables are relative to `src/libslic3r/`. Rust paths are
relative to `crates/ares-core/src/` unless another crate is named. Module roots
identify the existing destination boundary, not full class parity. Existing Ares
scaffolding that differs from these upstream concepts remains a temporary
compatibility shell. It is not an Ares-owned pipeline design to extend.

### Config/profiles owner

| Upstream file | Touched source boundary | Rust destination or explicit deferral |
|---|---|---|
| `AppConfig.cpp` | `AppConfig::set_defaults`, `AppConfig::load`, JSON checksum splitting | Desktop filesystem/application settings remain deferred. No file I/O in core. |
| `Config.cpp` | `ConfigBase::apply_only` | `options/config_types`, configuration application |
| `Preset.cpp` | default extension, user-preset resolution, embedded-preset loading and config diff | `profiles/catalog`, `profiles/compatible`, `profiles/project_refresh` |
| `Preset.hpp` | preset declarations | `profiles` types with the corresponding source slice |
| `PresetBundle.cpp` | AMS synchronization and `load_config_file_config` family | `profiles/composition`, `profiles/catalog`, `profiles/import`. Device synchronization remains deferred. |
| `PresetBundle.hpp` | `PublishedConfig` and bundle declarations | `profiles` data boundary. Publishing/device runtime remains deferred. |
| `PresetCacheFormat.cpp` | `CacheDictionary`, enum/config serialization | Native binary preset-cache persistence remains deferred. Do not add a second core catalog format. |
| `PresetCacheFormat.hpp` | cache dictionary/serialization declarations | Same deferred persistence boundary |
| `PrintApply.cpp` | `Print::apply` | `project/effective_config`, `options/project_variants` |
| `PrintConfig.cpp` | `PrintConfigDef::init_fff_params`, `normalize_fdm`, `normalize_fdm_1`, `set_num_filaments` | `options/registry`, typed options, FDM normalization and variant columns |
| `PrintConfig.hpp` | configuration declarations | Matching typed `options` definitions, not footer-only substitutions |
| `PublishSettings.cpp` | published-config filtering and material publication helpers | Source ownership stays here. Publishing is deferred from the byte-oriented slicing API. |
| `PublishSettings.hpp` | published material/config declarations | Same deferred publishing boundary |

The profile resources tree is a separate config-owned input. Provision the entire
release tree, including vendor/inheritance/default inputs, rather than replacing
only the profiles exercised by smoke tests.

### Geometry/mesh/import owner

| Upstream file | Touched source boundary | Rust destination or explicit deferral |
|---|---|---|
| `Arrange.cpp` | arrange inflation, bed margins and exclusion preloading | `plater`/placement boundary. General upstream auto-arranger parity is deferred from the pin change. |
| `BeltTransform.cpp` | `BeltTransformPipeline` transform construction | Belt execution remains deferred. Do not introduce a new core pipeline. |
| `BeltTransform.hpp` | belt transform declarations | Same deferred belt boundary |
| `ExclusionVolumeGeometry.cpp` | `bed_exclusion_physical_extruders` | `options/bed_exclusion`, `plater`/`print` validation with the material-planning owner |
| `Format/STEP.cpp` | `StepPreProcessor::preprocess` | STEP preprocessing remains deferred. Filesystem handling cannot enter core. |
| `Format/bbs_3mf.cpp` | `PlateData::parse_filament_info` and exporter metadata | `project/load`, `project/archive` for supported byte-oriented import. Native archive export is deferred. |
| `Format/bbs_3mf.hpp` | `PlateData` filament information declarations | Corresponding project data. Coordinate processor result representation with its owner. |
| `Format/svg.cpp` | `get_profile_area` | Native SVG/CAD boundary remains deferred. |
| `MeshBoolean.cpp` | `cgal_to_triangle_mesh` | `geometry` source boundary. No native CGAL dependency in core or WASM. |
| `calib.cpp` | `CalibPressureAdvance` and PA-line G-code generation | Native calibration workflows remain deferred. Shared writer changes belong to the emission owner. |
| `calib.hpp` | calibration options/declarations | Same deferred calibration boundary |

### Walls/seams owner

| Upstream file | Touched source boundary | Rust destination or explicit deferral |
|---|---|---|
| `ExtrusionEntity.cpp` | `ExtrusionLoop::is_smooth`, seam trimming, `ExtrusionLoopSloped` | `extrusion_entity` and perimeter/seam consumers. Coordinate loop emission with the emission owner. |

### Fill/support/surfaces owner

| Upstream file | Touched source boundary | Rust destination or explicit deferral |
|---|---|---|
| `BeltBrim.cpp` | `make_belt_brim`, belt band paths | Belt brim execution remains deferred with belt transforms. |
| `Fill/FillBase.cpp` | `insert_gap_fill` ordering constraints | `fill` and `project_slice/fill_entities` |
| `Fill/FillGyroid.cpp` | `FillGyroid::_fill_surface_single` | `fill/gyroid` |
| `Fill/FillSpiralInset.cpp` | `FillSpiralInset::_fill_surface_single` | [Spiral Inset for project surfaces and support interfaces](spiral-inset-fill.md). |
| `Fill/FillTpmsD.cpp` | `FillTpmsD::_fill_surface_single` | `fill/tpms_d` |
| `Fill/FillTpmsFK.cpp` | `FillTpmsFK::_fill_surface_single` | `fill/tpms_fk` |
| `Fill/Lightning/DistanceField.cpp` | `DistanceField` construction | `fill/lightning/distance_field` |
| `Fill/Lightning/DistanceField.hpp` | distance-field declarations | Same distance-field boundary |
| `PrintObject.cpp` | `PrintObject::update_slicing_parameters` | `project_slice/parameters` and object configuration |
| `PrintObjectSlice.cpp` | slicing-error repair | `project_slice/region_slices`, `project_slice/compensation` |
| `Slicing.cpp` | `SlicingParameters::create_from_config`, layer-height adjustment | `project_slice/planning`, `project_slice/parameters` |
| `Slicing.hpp` | `equal_layering` | Corresponding slicing-parameter comparisons |
| `Support/TreeSupport.cpp` | `TreeSupport::drop_nodes` | `project_slice/support_material/tree/non_organic` |

### Emission/tool/material-planning owner

| Upstream file | Touched source boundary | Rust destination or explicit deferral |
|---|---|---|
| `FilamentGroup.cpp` | grouping context, cost and result behavior | `project_slice/gcode_emit/tool_ordering/filament_group`, bounded source slices only |
| `FilamentGroup.hpp` | grouping data/declarations | Same grouping boundary |
| `FilamentGroupUtils.cpp` | grouping helpers | `options/filament_map_recommend`, tool-ordering helpers |
| `FilamentGroupUtils.hpp` | helper declarations | Same helper boundary |
| `FilamentMixer.cpp` | mixed-filament expansion, including `expand_mixed_filament_ids` | Material-planning ownership. General mixing is deferred, not simulated through ordinary filament grouping. |
| `FilamentMixer.hpp` | mixed-filament expansion declarations | Same mixed-filament boundary |
| `GCode.cpp` | export setup, loop/path emission, travel/retraction and belt helpers | `project_slice/gcode_emit`. Belt-specific execution remains deferred. |
| `GCode.hpp` | exporter state and declarations | Same exporter boundary |
| `GCode/AdaptivePAProcessor.cpp` | layer processing and model validation | `project_slice/gcode_emit/adaptive_pa` |
| `GCode/AdaptivePAProcessor.hpp` | adaptive PA declarations | Same adaptive PA boundary |
| `GCode/ToolOrdering.cpp` | filament-group context and recommended grouping | `project_slice/gcode_emit/tool_ordering` |
| `GCode/WipePathHelpers.cpp` | inward-support and wipe-path offsets | Wipe/seam emission boundary, shared with walls/seams. Helper ports remain separate source slices. |
| `GCode/WipePathHelpers.hpp` | wipe helper declarations | Same wipe boundary |
| `GCode/WipeTower.cpp` | type-1 tower tool change | `project_slice/gcode_emit/wipe_tower/type1` |
| `GCode/WipeTower2.cpp` | type-2 tower tool change/load/unload | `project_slice/gcode_emit/wipe_tower` |
| `GCodeWriter.cpp` | `GCodeWriter::lazy_lift` | `gcode_writer`, motion/lift emission |
| `PlaceholderParser.cpp` | placeholder expression grammar | `project_slice/gcode_emit/template`, `project_slice/gcode_emit/expression` |
| `Print.cpp` | exclusion collision, validation and physical unprintable filaments | `print/validate`, bounded `project_slice` ownership |
| `Print.hpp` | `Print` data/declarations | Same bounded print boundary, not a complete native `Print` class |
| `PrintBase.hpp` | print ownership declarations | `project_slice/state`, `slicer_context` |
| `ProjectTask.hpp` | project-task declarations | Native task runtime remains deferred. Do not duplicate core's async byte API. |

### Processor/platform owner

| Upstream file | Touched source boundary | Rust destination or explicit deferral |
|---|---|---|
| `GCode/GCodeProcessor.cpp` | multi-extruder validity, config application, move storage and object mass | `gcode_processor`, `project_slice/gcode_emit/processor` |
| `GCode/GCodeProcessor.hpp` | result/filament data and processor state | Core processor types. Rendering-neutral exposed data belongs to `ares-vgcode` without Native layout or viewer runtime. |

## Adjacent source boundaries verified separately

`git diff "$beta" "$release" -- src/libvgcode` is empty. Both endpoint trees are
**`dc9e02da638df56dc8ab1ed41a04de46fb3c9ceb`**. This establishes source identity
for `libvgcode`, not viewer execution or whole G-code-processor identity.

`git diff "$beta" "$release" -- src/OrcaSlicer.cpp` contains exactly one hunk,
at release lines **6653–6655** in `CLI::run`. The single-color limited-build-volume
branch collects `expand_mixed_filament_ids(vol->get_extruders(), m_print_config)`
instead of `vol->get_extruders()`. It inserts component filament IDs into the
unprintable sets. The material-planning owner owns the `FilamentMixer` helper.
The CLI adapter owns only the corresponding invocation/report boundary. No CLI
change to original-project preset refresh or early upward metadata occurs in
this endpoint diff. Porting mixed-material rejection is a separate source-cited
slice, not part of the repository-pin implementation.

`src/slic3r/GUI/GCodeViewer.cpp` **does change**. `GCodeViewer.hpp`, `IMSlider.hpp`,
`LibVGCode/LibVGCodeWrapper.cpp` and `libslic3r/Utils.hpp` retain identical blob
objects between these endpoints. Do not infer that every viewer extraction
manifest is unchanged from the `libvgcode` tree equality. Regenerate excerpts,
source hashes and line anchors from the release before claiming a release kernel.

## Proposed repository-pin implementation

First implement the web source refs and BOM readers below as a separate change.
The independent parity action, probe/caller cleanup and oracle builds follow later.
The release `Snapmaker/machine/fdm_U1.json` supplies `support_multi_bed_types = 1`.
Do not change `default_bed_type` enum parsing or Default Plate temperatures.

The implementation includes source/resource selection, original-byte profile
JSON readers, active probe provenance, source extraction manifests, the build
script and the two oracle producers below. It does not include the 61
behavioral ports, unrelated Native provisioning, new Rust dependencies or crates,
UI/OpenGL, filesystem behavior in core or bulk replacement of historical citations.

### Raw profile action

In `.github/actions/parity-profile-input/action.yml`, update the checkout ref,
HEAD assertion, profile-tree assertion and displayed identity together to the
release row above. Retain the sparse checkout, disabled persisted credentials and
explicit `ARES_PARITY_PROFILE_RESOURCES_DIR` binding. Check both identities before
exporting the directory. A release label with the beta profile tree is invalid.
This action is an independent parity input, not a consumer of the web source pin.
Its existing beta ref must not change merely because web refs change; migrate its
ref, HEAD and tree checks explicitly with matching release producer inputs.

### Original-byte profile JSON readers

The upstream reader boundary is `ConfigBase::load_from_json` in `Config.cpp:875–932`
and vendor JSON reading in `PresetBundle.cpp:8234,8302`. Both use nlohmann JSON;
`deps_src/nlohmann/detail/input/lexer.hpp:1478–1488,1504–1511` skips one `EF BB BF`
only at the beginning of input. It does not trim arbitrary text or fix a bad BOM.

Port that byte-entry behavior to `web/packages/ares-profiles/scripts/build-profiles.mjs`
and core's `profiles/fragment.rs` byte constructors, including original-machine
and up-to-date-file reads. Check `profiles/catalog.rs`, `profiles/catalog/store.rs`
and `profiles/import.rs` byte-entry readers too. Catalog/vendor envelopes remain
temporary transport shells around `PresetBundle`; do not invent a new grammar.
Keep metadata, typed option decoding, inheritance and error ordering unchanged.
Use a borrowed suffix in Rust; do not copy or rewrite caller buffers. Parsed-value
paths need no additional normalization. Core must not discover directories or
read files; Node and CLI adapters continue to own filesystem access.

The five original inputs under `resources/profiles/BenFab/process/` are:

| File | Bytes | SHA-256 |
|---|---:|---|
| `fdm_process_benfab_04020.json` | 552 | `2e5cb439e92d47a99965b593cdc62bd09a2446710867b40b00503037c6172aa7` |
| `fdm_process_benfab_06030.json` | 850 | `2f8aa6d9fd3229b7c7595f8de4f118bd952db027ee708d9375dcc60925370af6` |
| `fdm_process_benfab_08040.json` | 885 | `56d827ef87cf25bf6591b40d0e8532d27dcf9c3f282c2f07a5dceefc9d3bac35` |
| `fdm_process_benfab_10050.json` | 914 | `e08d2630ee29e6227f8d90588cfd85ca29cfc936368dbd1bc6443f18a2090ea0` |
| `fdm_process_benfab_common.json` | 3331 | `73ef97ce3de1039a7d67ba1b75dd5e211a46b720b1393595c882caf8ac9de198` |

Write Node build and public Core API tests before fixes. Exercise those unchanged
inputs, non-BOM process/filament/machine controls, vendor/catalog byte entries and
BOM inside JSON string values. Repeated/truncated BOMs, BOM after leading whitespace
and unsupported encodings must not become accepted by removing arbitrary bytes.
General decoder changes, XML/3MF handling and archive-format changes are deferred.
Save the complete generated catalog, input hashes and test results outside Git;
never repair the upstream JSON resources or commit generated Native outputs.

### Active `tools/chain-probe` pins and provenance

* Retarget the revision guards in `extract-marker-time.mjs`,
  `extract-actual-speed-profile.mjs`, `extract-custom-gcode-overview.mjs`,
  `extract-role-times.mjs`, `extract-segment-hw.mjs` and
  `extract-segment-colors.mjs`. The last still selects 2.4.2, not beta.
* Regenerate each corresponding extraction directory from the exact release
  checkout. Keep the existing adaptation boundary and checks. Do not edit only
  `upstreamRevision`, copy old whole-file hashes, bypass anchors or alter arithmetic.
* Bind active emitted identities in `marker-time-oracle.hpp`,
  `actual-speed-profile-oracle.hpp`, `custom-gcode-overview-oracle.hpp`,
  `role-times-oracle.hpp`, `segment-hw-oracle.hpp`, `segment-colors-oracle.hpp` and
  `color-change.mjs` to the source actually used by their rebuilt producers.
* Retarget the active `role-filament-map.py` source guard only with a newly
  verified producer. Its binary-layout assumptions need fresh verification
  because `GCodeProcessor.hpp` changes. A new revision string is not an ABI proof.
* Check `libslic3r_version.h` and rebuilt Nix variant version/source labels so
  active release probes do not inherit a misleading 2.4.2 package identity.
  Keep the existing nixpkgs/compiler/dependency inputs unless verified separately.
* Preserve fixed standalone `.inc` probe origins, historical store paths and
  the beta rationale in `marker-time-stable.mjs`. They are not release provenance.
  Update only instructions describing the new active producer contract.

### Release segment-colors producer

The boundary is `GCodeProcessorResult` through `libvgcode::convert` in
`LibVGCodeWrapper.cpp`, then the extracted `ViewerImpl::update_color_ranges`,
`get_vertex_color` and palette methods plus real `ColorRange`/`Layers` data.
`extract-segment-colors.mjs`, `segment-colors-oracle.hpp` and `segment-colors.nix`
retain that source-kernel boundary: the recipe applies `procgcode.patch` and
`segment-colors.patch` to the checked 40ecc66f7 source and builds on its 2.5.0 build
recipe, both given through the environment. This is not the whole Native
viewer or OpenGL execution, nor a new Rust viewer or slicing pipeline.

Regenerate the release extraction before building. Keep vertex order, position
bits, seven color views and the travel-visible speed view; do not alter arithmetic,
palettes, conversion or comparison rules. Record the release source, patches,
compiler/SDK inputs, wrapper/ELF hashes and a separate gcroot. Bind the actual
producer to `ARES_SEGMENT_COLOR_BIN` in the release environment and update active
instructions/known-failure notes. A merged smoke result or the old 2.4.2 gcroot
cannot establish release segment-colors output. Retain the old producer's identity.

### Initialized KSR release replay

Preserve the initialized authority described in
[the KSR decision](ard-ksr-initialized-runtime-authority.md), not a seed-only
producer. At release `Print.hpp:742`, `PrintObject::m_id` already starts at zero.
`Arachne/WallToolPaths.hpp:33,38` supplies `min_length_factor = 0.5f` and
`is_top_or_bottom_layer = false`; `FillConcentric.cpp:135` and
`FillConcentricInternal.cpp:47` default-construct those parameters. Keep these
upstream initializers rather than carrying duplicate assignments in the old patch.

`Feature/FuzzySkin/FuzzySkin.cpp:39–41` still seeds `random_value` nondeterministically.
Adapt `tools/orca-probes/ksr-initialized-source.patch` to the release context while
retaining its `ORCA_REPLAY_SEED`/MT19937 behavior. Do not add the seed-only patch's
`GCode.cpp` placeholder RNG change. Retarget `ksr-initialized.nix` and the build
script's KSR source/version together. The shared build-script branches must also
stop applying obsolete object-id/concentric initialization hunks now supplied
upstream; their existing seed-only policy remains separate and unchanged.

The consumers include `runner/stock_project_native_stream/producer.rs`,
`option_coverage/safc_reference.rs` and `web/e2e/ksrProject.spec.ts`. Keep their
initialized environment binding and authenticate the new wrapper/ELF hashes;
do not remove the old hash assertions without replacing them with verified release
identities. KSR labels, selectors and expected header literals must describe the
actual release producer; do not change version-identity constants or comparator regexes.

Use unchanged KSR project bytes and two fresh, serial seed-zero/single-core Native
captures. Record complete output hashes and require repeatability with only the
existing validated generator timestamp normalization. Run the complete ordered
G-code/config consumers against that producer, not a CONFIG_BLOCK-only substitute.
Keep original and initialized historical captures distinct. Source defaults,
patch applicability, compilation and repeatable execution are separate claims.

### Build script and missing stable-options call graph

`scripts/ci/build-oracle.sh:26` still downloads 2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Retarget its archive revision to the
release. Keep explicit per-variant patch composition, strict failures, bounded
build jobs and substitution-only cache-miss failure. Preflight the existing
patches against a fresh owned release tree before any build. Source-only checks
show the old KSR patch rejects release FuzzySkin/`Print.hpp`; `procgcode.patch`
applies and the following segment-colors patch checks successfully. Those checks
are not builds. The KSR section defines the retained initialization policy;
patch-context updates must not introduce other algorithm or policy changes.

The current inconsistency is concrete:

1. The `merged-oracle` branch applies nonexistent
   `tools/chain-probe/procgcode-stable-options.patch` between `procgcode.patch`
   and `procgcode-dynamics.patch`.
2. The generic dispatch accepts `procgcode-stable-options`, but its `.nix` recipe
   is also absent. It accepts `marker-time-stable-options`, whose existing recipe
   imports `marker-time.nix` and appends the same missing patch.
3. `.github/workflows/tier1.yml:81` schedules the absent procgcode variant.
   Its substitution loops at lines 270 and 377 request it again. The environment
   blocks at lines 293 and 404 export its nonexistent output as
   `ARES_PROCGCODE_STABLE_BIN`.
4. The same workflow's `ORCA_REF` at line 23 remains 2.4.2 and controls five
   upstream checkouts. Changing only extractor guards would leave CI inconsistent.

**Smallest caller repair:** retire the unavailable stable-options variants.
Remove the missing patch application, both stale dispatch names and
`marker-time-stable-options.nix`. Remove the procgcode
matrix entry, both substitution-loop entries and both stale environment exports.
Move `tier1.yml`'s `ORCA_REF` to the release in the same caller repair.
Do not manufacture a replacement patch, alias stable to raw, add a fallback or
remove unrelated oracle variants. Check that remaining consumers do not require
the retired environment variable before removing it.

Web's global resource source moves from `8500fcdccaa10b5099ac20d252af3a7c560046f1`
to the release: change `cloudflare-pages.yml:25,63` and `tier1.yml:23`. In Cloudflare,
change only the two checkout refs and their version comments; keep deployment
gates, sparse checkout, credentials, caching and build commands unchanged. Do not
create a submodule or redirect shared checkouts. The parity profile action above
remains an independent source binding. Official packaging in
`scripts/ci/fetch-orca-appimage.sh` remains with its owner; do not claim all
CI/deployment consumers are release-based.

## Verification and implementation limits

Agree on this extended source/ownership specification before tests and fixes.
Keep specification, failing tests and implementation in that order. Workflow
changes are limited to the caller repair and web source refs above. Inspect
existing release handoffs before proposing either Native build; do not
reuse a seed-only ELF as an initialized KSR producer. Rebuild only the two
specified oracles if needed, with four build jobs and one concurrent build;
substitution-only requests retain zero local build jobs.

For the repository-pin change, retain repeatable artifacts containing the Ares
commit, upstream commit/profile-tree IDs, generated extraction hashes, patch
application results and exact commands. Check that the profile action rejects a
wrong HEAD or profile tree. Run `bash -n` on the changed build script, syntax-check
changed JavaScript/Python, and run all six extraction `--check` commands against
the release checkout. Wrong-source rejection must remain strict. Audit every
remaining variant-to-recipe and patch reference, including workflow callers.

A Native build or E2E result additionally requires fixed release source, patches,
binary hashes, resources, invocation and environment. Use fresh release caches
and output directories. Missing producers or failed builds remain
blocked execution, not skipped parity PASS. Preserve full G-code/report comparison
rules and channel/exit checks. Each later behavioral rewrite first needs its
source-bounded specification and failing public E2E witness, followed by the
matching implementation and Rust formatting, Clippy, host and WASM checks.
Browser runtime results remain distinct from WASM compilation. Run formatting,
strict locked workspace/all-target Clippy, selected nextest, host and Core/adapter
WASM checks for changed Rust, plus web build/typecheck/lint for the byte-reader
change. Use a fresh strict-port 4184 browser server with a free-port check and
save the actual served assets and downloaded output. Do not borrow a server or
change source files during verification. Keep source/producer setup failures and
actual behavior failures separate; missing output is never equality.

This specification records source inventory, ownership, original-byte inspection
and patch applicability only. It contains no release build, runtime parity,
cross-platform or completed-port claim.
