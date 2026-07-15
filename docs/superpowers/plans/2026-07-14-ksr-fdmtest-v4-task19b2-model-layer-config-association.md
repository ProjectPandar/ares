# Task 19B.2 Model and Layer Configuration Association Implementation Plan

> **For agentic workers:** REQUIRED WORKFLOW: use `sdd-workflow` and
> Superpowers Subagent-Driven Development. Every implementation slice is owned
> by a fresh Agent and begins with a genuine behavior RED. Independent reviewers
> are read-only and must not be the implementer. Do not begin production code
> until this exact plan has literal `APPROVE` verdicts from an independent Agent
> and OpenCode.

**Goal:** Port the fixed OrcaSlicer 2.4.2 model/layer `ModelConfig` import and
association boundary into typed, in-memory `ares-core` project ownership so all
3MF configuration is classified from the archive bytes before Task 19B.3
performs effective FDM normalization.

**Architecture:** Close the fixed 751-key registry, replace the current opaque
model-option retention with one canonical typed classifier, retain only fixed
structural provenance in the typed import document, and attach sparse object,
region, volume, and layer-range options directly to project-domain owners.
Model XML supplies only typed name, `pid`, and production-color data required by
the fixed no-settings fallback. The optional layer-range resource is read
through the bounded archive and associated by one-based final project ordinal.

**Tech stack:** Rust 1.91.0, edition 2024, `serde`, `quick-xml`, existing typed
option builders and `ProjectArchive`, Cargo Nextest, rustfmt, Clippy, wasm32,
wasm-bindgen browser tests, PowerShell, independent Agent and OpenCode review
gates.

---

## Reviewed specification and base state

- Reviewed specification:
  `docs/superpowers/specs/2026-07-14-ksr-fdmtest-v4-task19b2-model-layer-config-association.md`
- Frozen specification SHA-256:
  `1CF61C39ACC560AC54A262B1D88B1B4AC0A462EA7BBA9BDE53BEB5410441E67D`
- Independent Agent verdict: `APPROVE`
- OpenCode verdict: `APPROVE`
- Fixed OrcaSlicer commit/tag:
  `8500fcdccaa10b5099ac20d252af3a7c560046f1` / `v2.4.2`
- Implementation base commit:
  `8e09be79881c6365100fac06ed064f487c75fb85`

Before dispatching an implementer, verify the immutable inputs:

```powershell
(Get-FileHash docs/superpowers/specs/2026-07-14-ksr-fdmtest-v4-task19b2-model-layer-config-association.md -Algorithm SHA256).Hash
git rev-parse HEAD
git status --short
git -C OrcaSlicer cat-file -t 8500fcdccaa10b5099ac20d252af3a7c560046f1
```

Expected: the exact spec hash, the base commit above before Slice 1, and only
the approved untracked spec/plan before implementation. The mutable
`OrcaSlicer` checkout HEAD is not the source boundary; source checks use
`git -C OrcaSlicer show 8500fcdc:<path>`.

Any edit to the reviewed specification invalidates both spec approvals. Any
edit to this plan after its review invalidates both plan approvals.

## Fixed upstream rewrite boundary

This plan ports only these fixed OrcaSlicer slices:

- `src/libslic3r/PrintConfig.cpp:663-8031`, generated registration loops, and
  `Config.cpp:258-318,573-685` own the canonical registry, empty/default
  construction, legacy normalization, lookup, and concrete lexical decode.
- `src/libslic3r/Config.hpp` owns scalar, nullable, vector, enum, point, and
  point-group wire grammars.
- `src/libslic3r/Format/bbs_3mf.cpp:234-235,1785-1815,1896-1904,
  2043-2168,2886-2940,3364-3366,3575-3605,3719-3735,4263-4400,
  4894-5126,5425-5455,7517-7545` owns material-extension colors,
  object/part metadata, default volume selection, naming, optional layer
  ranges, and association.
- `src/libslic3r/Model.hpp:354-370,865-918`, `Model.cpp:2717-2747`, and
  `Slicing.hpp:150-151` own the destination model/volume configuration, volume
  vocabulary, and sorted raw layer ranges.
- `src/libslic3r/PrintApply.cpp:342-383` owns later range normalization and is
  explicitly deferred to Task 19B.3.

The Rust destination is limited to `ares-core::options` model-config decode and
`ares-core::project` parsing/domain assembly. The existing typed
`ObjectOptionOverrides`, `RegionOptionOverrides`, and `ProjectSettingsBuilder`
are reused. No new pipeline, dynamic config, erased option value, native I/O,
Orca invocation, fixture branch, or G-code behavior is authorized.

## Locked architecture and file policy

- `options/model_config_deserialize.rs` is the only model/layer option
  classifier. The old `options/typed_legacy/model.rs` is deleted when the new
  boundary is connected.
- Project-settings fields that do not belong to object/region projection are
  parsed into a temporary concrete builder and discarded. Registry-only fields
  are concretely lexical-validated and discarded.
- `ModelSettings` remains a typed provenance/import document, but domain
  configuration is never rejoined from it after assembly.
- `ProjectObject` owns object name/module, object overrides, region overrides,
  and raw layer ranges. `ProjectVolume` owns name, type, and region overrides.
- Model XML retains typed `name`, `pid`, and material-extension color groups
  only; do not add a generic XML resource map. Existing `p:*` object/component
  attributes remain in the production namespace
  `http://schemas.microsoft.com/3dmanufacturing/production/2015/06`.
  `m:colorgroup` and `m:color` are instead bound to the distinct 3MF material
  namespace `http://schemas.microsoft.com/3dmanufacturing/material/2015/02`.
- `graph.models` is root-first. Fixed color semantics require submodels in graph
  order with insert-only duplicate IDs, followed by root replacement.
- `MetadataIndex.part_transforms` must be removed. Its `BTreeMap` destroys
  source order and rejects duplicate part IDs, so it cannot represent fixed
  same-index/first-match selection.
- New/changed Rust source files must stay below 400 physical lines. Do not add
  to `project/tests/model/invalid.rs`, currently 396 lines. Split `project/xml.rs`
  before it reaches 400.
- Do not modify architecture/roadmap documentation until the whole
  implementation receives all required approvals.
- Do not stage or commit individual slices. The user requires one commit after
  implementation approval, documentation approval, and release verification.

## Dispatch, TDD, and review rules

- Execute Slices 1-7 in order. All production mutations are serialized in the
  shared checkout. Read-only audits and reviews may run in parallel.
- Use a fresh bounded implementer Agent for each of Slices 1-6. Slice 7 is a
  test/verification slice and receives an implementer only for its tests or a
  reviewed defect.
- Before every implementation slice, record the exact current manifest and
  dispatch only the approved spec, this slice, owned files, and commands.
- Each implementer first adds the complete slice test, runs it, and records a
  genuine RED that fails for missing/wrong behavior. Syntax, import, filter,
  environment, and unrelated failures do not count.
- After GREEN, the primary Agent inspects the diff and reruns the focused
  command. A different read-only Agent then returns literal
  `VERDICT: APPROVE` or `VERDICT: REVISE` for spec compliance and code quality.
- A `REVISE` verdict stops dependent slices. Use a fresh fix implementer,
  reverify, freeze new bytes, and repeat review until `APPROVE`.
- Record RED/GREEN run IDs, reviewer verdicts, hashes, and commands in ignored
  `.superpowers/sdd` evidence. Do not claim verification from stale output.

---

## Slice 1: Close the fixed 751-key registry

**Upstream:** `PrintConfig.cpp:674-932,1285-1349,2195-2213,3106-3137,
4127,5011-5030,5781-5808,6629`; `Config.cpp:258-318`;
`Config.hpp:954-981`.

**Production files:**

- Modify `crates/ares-core/src/options/registry/definitions/table/early.rs`
- Modify
  `crates/ares-core/src/options/registry/definitions/table/pre_middle_process.rs`
- Modify `crates/ares-core/src/options/registry/definitions/table/middle.rs`
- Modify
  `crates/ares-core/src/options/registry/definitions/table/middle_independent.rs`
- Modify `crates/ares-core/src/options/registry/definitions/table/middle_tail.rs`
- Modify `crates/ares-core/src/options/registry/definitions/table/late.rs`
- Modify
  `crates/ares-core/src/options/registry/definitions/table/late_tail_after_material.rs`
- Modify
  `crates/ares-core/src/options/registry/definitions/table/late_tail_after_pad.rs`
- Modify `crates/ares-core/src/options/registry/definitions/table/tail_raft.rs`
- Modify `crates/ares-core/src/options/registry/definitions/table/tail_final.rs`
- Modify `crates/ares-core/src/options/registry/definitions/table/tail_support.rs`
- Modify
  `crates/ares-core/src/options/registry/definitions/table/tail_terminal_suffix.rs`

**Test files:**

- Create `crates/ares-core/src/options/registry/tests/task19b2.rs`
- Modify `crates/ares-core/src/options/registry/tests.rs`
- Modify the sorted ledgers under
  `crates/ares-core/src/options/registry/tests/keys/{first,second,third}.rs`
- Modify `crates/ares-core/src/options/tests/registry_helpers/known_count.rs`:
  its fixture includes all three removed legacy-only registry names and none
  of the 18 additions, so the expected known count changes from 677 to 674
  while its total retained input count remains 678.

- [ ] **Step 1: Add registry RED**

Add `task19b2_registry_has_fixed_inventory_and_histogram` asserting:

- exactly 751 sorted unique definitions;
- the complete approved type histogram;
- all 18 additions with exact type/default;
- the three legacy-only source names are absent;
- `extruder` serializes its effective registry default as `0` without
  materializing a sparse missing field.

Run:

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/(task19b2_registry|option_definitions|known_definition_count)/)'
```

Expected RED: the inventory is still 736, new rows are missing, and legacy rows
are still canonical.

- [ ] **Step 2: Make the registry GREEN**

Insert the 18 approved definitions in sorted shards and remove only
`solid_infill_filament`, `sparse_infill_filament`, and `wall_filament` from the
registry. Do not change their Task 19A legacy rules. Update the three frozen key
ledgers. Do not rewrite the separate 653-row fixture inventory or
`options-v242.json`.

- [ ] **Step 3: Verify Slice 1**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/(task19b2_registry|option_definitions|known_definition_count|legacy)/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

- [ ] **Step 4: Independent Slice 1 review**

Freeze the Slice 1 paths and require literal `VERDICT: APPROVE` for counts,
defaults, sortedness, legacy-only removal, and absence of unrelated inventory
churn.

---

## Slice 2: Metadata-wire validation and discard destinations

**Upstream:** `Config.hpp:820-1257,1452-1702,1809-2201`;
`Config.cpp:573-685`; `PrintConfig.cpp:402-419,481-485`.

**Production files:**

- Create `crates/ares-core/src/options/model_config_deserialize.rs`
- Create `crates/ares-core/src/options/model_config_deserialize/wire.rs`
- Modify `crates/ares-core/src/options.rs`

**Test files:**

- Create `crates/ares-core/src/options/tests/model_config_deserialize.rs`
- Create
  `crates/ares-core/src/options/tests/model_config_deserialize/wire.rs`
- Create
  `crates/ares-core/src/options/tests/model_config_deserialize/scalar_enums.rs`
- Create
  `crates/ares-core/src/options/tests/model_config_deserialize/vector_enums.rs`
- Create
  `crates/ares-core/src/options/tests/model_config_deserialize/registry_kinds.rs`
- Modify `crates/ares-core/src/options/tests.rs`

- [ ] **Step 1: Add complete metadata-wire RED**

Add tests for:

- each `OptionValueKind` lexical branch through either a typed project field or
  the registry-only complement;
- all 18 typed scalar discard enum domains, every accepted token and invalid
  case, exact/case-sensitive/untrimmed;
- all nine enum-vector domains, comma splitting, per-element trim, empty and
  trailing comma, invalid elements, and `nil` only for the three approved
  nullable vectors;
- all five ownerless enum ledgers and defaults;
- the exact 650 typed-owner / 101 registry-only partition;
- the eight registry-only complement kinds and bounded keyed errors;
- no retained `ProjectSettings` state.

Run:

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/^options::tests::model_config_deserialize::/)'
```

Expected RED: the private adapter/classifier API does not exist.

- [ ] **Step 2: Implement concrete wire adapters**

Add a crate-private metadata value validator that:

- chooses a scalar `StringDeserializer` or concrete `SeqDeserializer` from the
  canonical definition kind;
- preserves fixed comma, semicolon/C-style string, nullable element, point,
  and point-group grammar;
- creates a fresh `ProjectSettingsBuilder` per typed destination, calls
  `deserialize_known_value`, and discards it without `resolve`;
- concretely validates the 101-key complement without `serde_json::Value`,
  `BTreeMap<String, ...>`, type erasure, or retained dynamic state;
- uses one private sorted five-entry enum token ledger only for the ownerless
  enums.

The new API may have a temporary dead-code allowance only until Slice 3 connects
the importer. No fallback values or forward-compatible enum substitution are
allowed.

- [ ] **Step 3: Verify Slice 2**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/^options::tests::model_config_deserialize::/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(project_settings|registry)/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

- [ ] **Step 4: Independent Slice 2 review**

Require literal `VERDICT: APPROVE` for exact wire semantics, complete enum
domains, concrete builder reuse, the 650/101 proof, bounded errors, and absence
of dynamic/erased storage.

---

## Slice 3: Canonical classifier, legacy completion, and exact metadata scopes

**Upstream:** `Config.cpp:573-685`; `PrintConfig.cpp:8033-8338`;
`bbs_3mf.cpp:2067-2168,4263-4400,5081-5116`;
`PrintConfig.hpp:2034-2128`.

**Production files:**

- Complete `crates/ares-core/src/options/model_config_deserialize.rs`
- Delete `crates/ares-core/src/options/typed_legacy/model.rs`
- Modify `crates/ares-core/src/options/typed_legacy.rs`
- Modify `crates/ares-core/src/options.rs`
- Modify `crates/ares-core/src/project/model_settings.rs`
- Modify
  `crates/ares-core/src/project/model_settings/object_metadata.rs`
- Modify `crates/ares-core/src/project/model_settings/part_metadata.rs`

**Test files:**

- Create `crates/ares-core/src/options/tests/model_config_deserialize/owners.rs`
- Create
  `crates/ares-core/src/options/tests/model_config_deserialize/legacy.rs`
- Create
  `crates/ares-core/src/project/tests/documents/object_settings_metadata/classifier.rs`
- Create
  `crates/ares-core/src/project/tests/documents/object_settings_metadata/structural_scope.rs`
- Modify only the small parent module registrations and tests that currently
  expect unknown metadata to remain opaque.

- [ ] **Step 1: Add owner/scope/legacy RED**

Prove the five-action classifier order from the approved spec at object, part,
and layer scopes. Cover all 126 object fields and all 149 region fields plus
`extruder`, no overlap, last-write-wins, unknown and `perimeter_feed_rate`
errors, and all four formerly deferred profile rules. Prove object structural
keys are only `name`/`module`; part-only structural keys fail at object scope;
object-only structural keys fail at part scope.

Run:

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/(model_config_deserialize|object_settings_metadata)/)'
```

Expected RED: unknown values are retained, the broad structural allow-list
silently accepts wrong scopes, and the four profile rules still report
deferred.

- [ ] **Step 2: Connect the canonical classifier**

Implement exactly:

1. model-path Task 19A legacy normalization;
2. object then region typed sparse assignment at object scope, or region only
   at part/layer scope;
3. typed project destination validation/discard;
4. registry-only concrete validation/discard;
5. bounded rejection using the original source name.

Complete the three cumulative renames and canonical
`different_settings_to_system` only for this model/layer boundary. Leave
top-level JSON profile behavior unchanged. Remove the old typed-legacy model
classifier and the temporary dead-code allowance from Slice 2.

- [ ] **Step 3: Make structural document parsing precise**

Retain typed ordered object name/module and part name/type/matrix,
`mesh_shared`, `source_*`, and mesh statistics. Do not retain accepted config
as opaque key/value entries. A structural-looking key in the wrong scope goes
through the classifier. Preserve source order and last-write-wins for all
configuration owners.

- [ ] **Step 4: Verify Slice 3**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/(model_config_deserialize|object_settings_metadata)/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(object_options|region_options|typed_legacy)/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

- [ ] **Step 5: Independent Slice 3 review**

Require literal `VERDICT: APPROVE` for classifier order, exact scopes, legacy
completion, no opaque config retention, last-write-wins, and unchanged
top-level project JSON behavior.

---

## Slice 4: Typed model XML name, pid, and material colors

**Upstream:** `bbs_3mf.cpp:234-235,1785-1815,3364-3366,3575-3605,
3719-3735,5425-5455`; 3MF material namespace
`http://schemas.microsoft.com/3dmanufacturing/material/2015/02`.

**Production files:**

- Modify `crates/ares-core/src/project/model_xml.rs`
- Modify `crates/ares-core/src/project/xml.rs`
- Create `crates/ares-core/src/project/xml/element.rs` and move the existing
  role-aware non-root element namespace check into it before extending the
  fixed production-element vocabulary
- Modify `crates/ares-core/src/project/xml/model.rs`
- Modify `crates/ares-core/src/project/xml/attribute.rs`

**Test files:**

- Create `crates/ares-core/src/project/tests/model/production.rs`
- Modify `crates/ares-core/src/project/tests/model.rs`

- [ ] **Step 1: Add typed production XML RED**

Use synthetic model XML to prove:

- object `name` defaults empty and retains exact text;
- missing/unparsable `pid` becomes `0`;
- `pindex` is accepted but not retained as slicing state;
- `m:colorgroup` and `m:color` in the exact material namespace retain numeric
  group `id`, ordered colors, and the unprefixed `color` attribute;
- the last color in a group is observable for later mapping;
- `requiredextensions="p m"` validates each prefix against its distinct exact
  namespace; the wrong URI, an unbound required prefix, material elements in
  the production/core namespace, and production elements in the material
  namespace are strict bounded errors;
- only the fixed production/material elements and required attributes are
  accepted; unrelated extension elements remain strict errors.

Do not add to the 396-line `project/tests/model/invalid.rs`.

Run:

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/^project::tests::model::production::/)'
```

Expected RED: `ModelObject` lacks name/pid, `Resources` lacks color groups, and
the XML validator rejects the production elements.

- [ ] **Step 2: Add only the typed model XML fields**

Add a private `MATERIAL_NAMESPACE` constant beside the existing
`PRODUCTION_NAMESPACE`. Extend typed model records with name, permissive fixed
`pid`, and ordered material color-group/color records. Update required-extension,
element, and attribute validation only for the two exact source-cited
vocabularies. Do not add a generic extension map or effective extruder mapping
in this slice.

- [ ] **Step 3: Verify Slice 4**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/^project::tests::model::production::/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/^project::tests::(model|xml_limits)::/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

- [ ] **Step 4: Independent Slice 4 review**

Require literal `VERDICT: APPROVE` for namespace strictness, typed-only data,
fixed `pid` behavior, source order, distinct production/material vocabularies,
required-extension validation, and no generic resource mechanism.

---

## Slice 5: Project-domain ownership and model/volume association

**Upstream:** `bbs_3mf.cpp:2043-2168,4894-4954,5081-5126`;
`Model.hpp:354-370,865-918`; `Model.cpp:2717-2747`.

**Production files:**

- Modify `crates/ares-core/src/project/domain.rs`
- Modify `crates/ares-core/src/project.rs`
- Modify `crates/ares-core/src/project/load/assemble.rs`
- Modify `crates/ares-core/src/project/load/metadata.rs`
- Create `crates/ares-core/src/project/load/colors.rs`
- Create `crates/ares-core/src/project/load/volume_metadata.rs`
- Modify `crates/ares-core/src/project/load.rs` for module registration and
  assembly arguments only.

**Test files:**

- Create `crates/ares-core/src/project/tests/model/config_association.rs`
- Create `crates/ares-core/src/project/tests/model/volume_defaults.rs`
- Create `crates/ares-core/src/project/tests/model/production_colors.rs`
- Modify `crates/ares-core/src/project/tests/model.rs`
- Modify `crates/ares-core/src/project/tests/model/fixture.rs` only with small,
  reusable synthetic archive mutations.

- [ ] **Step 1: Add configured-domain RED**

Prove object settings order differs from final build order and part XML order
differs from BFS leaf order. Assert object name/module/object overrides/region
overrides and volume name/type/region overrides on the correct public domain
owner. Cover all five volume type spellings, subtype initialization, ordered
`volume_type`/`part_type` replacement, and strict unknown types.

- [ ] **Step 2: Add default-part and naming RED**

Prove:

- no object-settings record is valid;
- missing/unmatched parts create default `ModelPart` volumes;
- same-index matching wins even for a later repeated part ID;
- fallback scanning chooses the first source-ordered matching ID;
- extra unmatched parts do not create volumes;
- the crate-private selected-volume metadata record reports identity source
  transform, empty/default source provenance, and zero/default statistics for
  an absent/unmatched part while component transform and mesh remain unchanged;
- the default selection does not insert a synthetic `PartSettings` into the
  retained source `ModelSettings` document;
- object XML name / `Object_{ordinal}` fallback;
- unnamed volumes receive object name, `_2`, `_3`; explicitly named volumes do
  not consume/reset the counter.

- [ ] **Step 3: Add no-settings color fallback RED**

Prove last color per group, submodel graph-order first-ID merge, root
replacement, numeric group ordering, exact color deduplication, one-based
extruders, `pid=0`, unmapped groups, ignored `pindex`, and suppression by any
matching model-settings record even when it omits `extruder`.

Run all Slice 5 RED tests:

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/^project::tests::model::(config_association|volume_defaults|production_colors)::/)'
```

Expected RED: domain owners lack fields; missing object/part and repeated part
IDs are rejected; color data is not lowered.

- [ ] **Step 4: Implement source-ordered association**

Add and re-export `ProjectVolumeType`. Attach configuration directly to domain
objects/volumes. Remove `MetadataIndex.part_transforms`; preserve the
source-ordered `ObjectSettings.parts` vector. For each BFS leaf, select the
same-index part if its ID matches, otherwise the first source-order ID match,
otherwise typed default metadata. Continue rejecting duplicate object-settings
IDs.

Implement selection in `project/load/volume_metadata.rs` as a crate-private
typed association record used by assembly for both matched and defaulted
volumes. It exposes, to crate tests only, the selected part identity plus fixed
name/type/config, source transform, source provenance, and mesh-stat defaults.
The default branch constructs this association record in memory and never
mutates or appends to `ModelSettings`. Public projection of non-slicing
`source_*`, `mesh_shared`, and mesh statistics remains deferred as specified;
the internal record is the observable test seam proving the fixed default
branch without widening `ProjectVolume`.

Implement fixed naming after metadata application. Retain `ModelSettings` only
as import/provenance; no later production config rejoin is allowed.

- [ ] **Step 5: Implement deterministic color mapping**

Because `graph.models` is root-first, explicitly iterate submodels
`models[1..]` in graph order with insert-only group IDs, then replace from
`models[0]`. Iterate the final numeric group map ascending and deduplicate exact
color strings to positive one-based extruder IDs. Apply only in the
no-model-settings branch.

- [ ] **Step 6: Verify Slice 5**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/^project::tests::model::(config_association|volume_defaults|production_colors)::/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/^project::tests::model::/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(object_options|region_options|object_settings_metadata)/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

- [ ] **Step 7: Independent Slice 5 review**

Require literal `VERDICT: APPROVE` for public ownership, source order,
duplicate/default behavior, the internal default-metadata test seam without
public provenance projection, naming, transform separation, color semantics,
retained source-document integrity, and absence of a later settings rejoin.

---

## Slice 6: Optional layer configuration ranges

**Upstream:** `bbs_3mf.cpp:209-216,1896-1904,2087-2095,2886-2940,
7517-7545`; `Slicing.hpp:150-151`; deferred `PrintApply.cpp:342-383`.

**Production files:**

- Create `crates/ares-core/src/project/layer_config_ranges.rs`
- Modify `crates/ares-core/src/project.rs`
- Modify `crates/ares-core/src/project/domain.rs`
- Modify `crates/ares-core/src/project/load.rs`
- Modify `crates/ares-core/src/project/xml/role.rs`
- Modify `crates/ares-core/src/project/xml/attribute.rs`

**Test files:**

- Create `crates/ares-core/src/project/tests/layer_config_ranges.rs`
- Create `crates/ares-core/src/project/tests/layer_config_ranges/archive.rs`
- Create
  `crates/ares-core/src/project/tests/layer_config_ranges/association.rs`
- Create `crates/ares-core/src/project/tests/layer_config_ranges/invalid.rs`
- Modify the test module registration in `crates/ares-core/src/project.rs`

- [ ] **Step 1: Add archive/path RED**

Prove absent optional entry yields empty ranges, one ASCII case variant is read
by its exact validated `PackagePath`, and multiple case variants produce one
bounded ambiguous-input error. Preserve normalized path and backslash
rejection; do not add raw zip-name lookup.

- [ ] **Step 2: Add parsing/association RED**

Prove one-based final object ordinals independent of source object IDs; sorted
range output; later complete assignment for exact duplicate ranges; later
assignment for duplicate options; empty groups add no state; finite negative,
reversed, gapped, and overlapping ranges remain unnormalized.

Prove bounded errors for zero/out-of-range/duplicate ordinals, missing or
non-finite bounds, malformed XML, unknown keys, invalid values, and missing
attributes. Layer options use the Slice 3 region-only classifier.

Run:

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/^project::tests::layer_config_ranges::/)'
```

Expected RED: the document role, parser, optional lookup, public raw range type,
and association do not exist.

- [ ] **Step 3: Implement bounded optional read and raw typed ranges**

Add `XmlRole::LayerConfigRanges` for contextual errors. Scan the already
validated archive path set with `eq_ignore_ascii_case`. Parse finite bounds and
region overrides. Use a vector for `(f64,f64)` keys: replace pairs under normal
floating equality so `-0.0` and `0.0` are equivalent, then sort finite bounds
lexicographically with `total_cmp`. Do not use `BTreeMap<(f64,f64), _>` and do
not normalize ranges.

Associate only after final build-created object order is known. Store the
typed sorted records on `ProjectObject`.

- [ ] **Step 4: Verify Slice 6**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/^project::tests::layer_config_ranges::/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(model_config_deserialize|project::tests::model|layer_config_ranges)/)'
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

- [ ] **Step 5: Independent Slice 6 review**

Require literal `VERDICT: APPROVE` for bounded archive use, path ambiguity,
ordinal identity, duplicate semantics, region-only classification, finite
sorting, raw gap/overlap preservation, and errors.

---

## Slice 7: Real KSR and public incomplete-boundary regression

**Test file:**

- Create `crates/ares-core/src/project/tests/model/task19b2_fixture.rs`
- Modify `crates/ares-core/src/project/tests/model.rs`

- [ ] **Step 1: Add the complete real-fixture RED**

Through public `load_project`, assert from the committed 3MF bytes:

- final object source ID `2` and volume source ID `1`;
- object typed region `extruder=1`;
- volume type `ModelPart`, fixture-derived name, and typed volume config;
- object name/module/config ownership;
- empty optional layer ranges;
- retained typed documents remain provenance only.

The test may reference the fixture only in test code and must not read the
reference G-code. If all assertions are already GREEN because earlier slices
covered them, record that Slice 7 is verification-only; do not invent a RED.

- [ ] **Step 2: Verify public core/CLI/browser boundaries**

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/^project::tests::model::task19b2_fixture::/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/(model_config_deserialize|project::tests::model|layer_config_ranges)/)'
cargo +1.91.0 nextest run -p ares-cli --test ksr_fdmtest_v4
cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
npm --prefix crates/ares-wasm/tests/browser test -- project-slice.spec.mjs
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets -- -D warnings
git diff --check
```

Expected: valid project loading reaches the richer typed domain, public slicing
still returns exactly `ProjectSlicingIncomplete`, and the complete CLI golden
remains configured skipped. This task does not claim G-code parity.

- [ ] **Step 3: Independent Slice 7 review**

Require literal `VERDICT: APPROVE` for fixture-derived assertions, no production
hardcoding, no reference-G-code read, and unchanged incomplete boundaries.

---

## Whole implementation approval gate

- [ ] Freeze a deterministic implementation manifest for every changed
  production/test path, representing deletions as `DELETED`. Exclude spec,
  plan, and later docs-only paths.
- [ ] Record the complete fresh pre-review matrix required by the specification.
  Generate the browser binding from the reviewed implementation bytes before
  running the browser test; do not reuse an older `target/wasm-browser` output:

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/(task19b2|model_config_deserialize|object_settings_metadata|project::tests::model|layer_config_ranges)/)'
cargo +1.91.0 nextest run -p ares-cli --test ksr_fdmtest_v4
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
cargo +1.91.0 check -p ares-core
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
npm --prefix crates/ares-wasm/tests/browser ci
npm --prefix crates/ares-wasm/tests/browser test
git diff --check
```

Also run the fixture hashes, forbidden production scans, deleted-old-classifier
check, and changed-Rust LOC audit listed under **Mandatory audits** below before
freezing reviewer evidence. The post-documentation release matrix repeats all
of this from fresh final bytes; it does not substitute for this pre-review gate.

- [ ] Dispatch a fresh independent spec-compliance reviewer against the exact
  spec, approved plan, manifest, diff, and verification. Require exactly:

```text
VERDICT: APPROVE | REVISE
SPEC_COVERAGE:
- [implemented requirement or missing requirement]
BLOCKERS:
- [blocking gap or None]
REQUIRED_CHANGES:
- [change or None]
ROLE: SPEC COMPLIANCE
```

- [ ] Dispatch a different fresh code-quality reviewer against identical bytes.
  It checks correctness, source faithfulness, ownership, bounded input parsing,
  performance, Rust idioms, LOC, tests, and removal of obsolete scaffolding.
  Require literal `VERDICT: APPROVE` and `ROLE: CODE QUALITY`.
- [ ] Run the same bounded whole-diff review through OpenCode's default model.
  Require literal `VERDICT: APPROVE`.
- [ ] Any `REVISE` unfreezes implementation. Fix with a fresh Agent, rerun
  focused and cumulative verification, freeze a new manifest, and rerun all
  three reviewers. Do not update tracked architecture/roadmap docs until all
  approvals apply to identical implementation bytes.

## Post-approval documentation gate

After whole implementation approval, update only:

- `docs/architecture/option-parity-v4.md` with the fixed 751-key boundary,
  model metadata classifier, domain ownership, default-part/name/color rules,
  raw optional layer ranges, portability boundary, and explicit 19B.3/19C/20E
  deferrals;
- `docs/roadmap.md` with Task 19B.2 completion evidence and Task 19B.3 next;
- ignored `.superpowers/sdd` progress/release evidence with RED/GREEN/review
  run IDs and hashes.

Freeze the two tracked docs and dispatch a fresh documentation reviewer. It
must validate every claim against the approved implementation and fixed source,
then return literal `VERDICT: APPROVE` with `ROLE: DOCUMENTATION`. A tracked doc
edit invalidates only the docs approval.

## Fresh release matrix

After implementation and documentation approvals, run from the frozen tree:

```powershell
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 nextest run -p ares-core -E 'test(/(task19b2|model_config_deserialize|object_settings_metadata|project::tests::model|layer_config_ranges)/)'
cargo +1.91.0 nextest run -p ares-cli --test ksr_fdmtest_v4
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
cargo +1.91.0 clippy --workspace --all-targets -- -D warnings
cargo +1.91.0 check -p ares-core
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
cargo +1.91.0 build -p ares-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm --target web --out-dir target/wasm-browser
npm --prefix crates/ares-wasm/tests/browser ci
npm --prefix crates/ares-wasm/tests/browser test
git diff --check -- . ':(exclude)tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode'
```

Mandatory audits:

```powershell
# Fixture bytes remain exact.
Get-FileHash tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf -Algorithm SHA256
# 698F40F13C9075B818ABEDD3D10F022FBB5D8200AED48FBDDE651F6BFB21B8A9
Get-FileHash tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode -Algorithm SHA256
# 10AEC9A156849F59929B578429A764A61453996A5834056F600C0ADBB5D6A1B3

# No dynamic model config, fixture/source pinning, native I/O, or Orca runtime.
rg -n 'serde_json::Value|serde_json::Map|RawValue|BTreeMap<String' crates/ares-core/src/options/model_config_deserialize.rs crates/ares-core/src/options/model_config_deserialize
rg -n 'OrcaSlicer|8500fcdc|ksr_fdmtest_v4|include_(str|bytes)!|\.gcode|std::fs|File::|Command::' crates/ares-core/src/options/model_config_deserialize.rs crates/ares-core/src/options/model_config_deserialize crates/ares-core/src/project/layer_config_ranges.rs crates/ares-core/src/project/load crates/ares-core/src/project/domain.rs
rg -n 'part_transforms|typed_legacy::.*deserialize_(object|part)_model_field' crates/ares-core/src
Test-Path crates/ares-core/src/options/typed_legacy/model.rs

# All changed Rust files stay below 400 physical lines.
$rustPaths = git diff --name-only --diff-filter=ACMR -- '*.rs'
$rustPaths += git ls-files --others --exclude-standard -- '*.rs'
$rustPaths | Sort-Object -Unique | Where-Object { Test-Path $_ } | ForEach-Object {
    $lines = (Get-Content $_).Count
    if ($lines -ge 400) { throw "$_ has $lines lines" }
}
```

The first three `rg` commands are expected to return no forbidden production
result, and the deleted old classifier path must print `False`. Fixture/source
names are allowed only in reviewed tests/docs, never production.

## Commit, push, and exact-SHA Tier 1

- [ ] Recompute a final manifest containing spec, plan, approved implementation,
  approved tracked docs, and any reviewed baseline changes. Confirm `git status`
  contains only intended paths and index/worktree bytes match the manifest.
- [ ] Apply the Conventional Commits skill, stage only the frozen manifest, and
  create one commit:

```powershell
git commit -m "feat(project): associate model and layer config"
```

- [ ] Push the current branch:

```powershell
git push origin codex/ksr-fdmtest-v4-parity
```

- [ ] Verify local/remote equality and clean state:

```powershell
git rev-parse HEAD
git rev-parse origin/codex/ksr-fdmtest-v4-parity
git status --short
```

- [ ] Wait for the exact pushed SHA's Tier 1 workflow and require all five jobs
  green: `format`, `ubuntu-latest`, `wasm`, `macos-latest`, and
  `windows-latest`. Do not start Task 19B.3 or call Task 19B.2 released while
  that exact-SHA gate is pending.

Task 19B.2 completion still leaves Task 19B.3 normalization/effective
orchestration, Task 19C config serialization, Task 20E final dynamic consumer
removal, geometry slicing, toolpaths, G-code, metadata/post-processing, and the
complete normalized `ksr_fdmtest_v4` golden parity open.
