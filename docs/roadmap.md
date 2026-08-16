# Ares roadmap

## Future milestone gate
Every future milestone must be a source-cited Rust rewrite slice of `OrcaSlicer/src/libslic3r` or `OrcaSlicer/src/libvgcode`. Specs and plans must name the upstream files/classes/functions being ported, define the Rust crate/module boundary, list included and deferred upstream behavior, and avoid adding Ares-owned pipeline abstractions except as temporary compatibility shells around named upstream concepts. Milestones that add slicing, G-code, configuration, or viewer-data behavior must start from the owning upstream boundary rather than from a new Ares pipeline design. See `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

This applies to every later milestone: the default design target is an upstream `libslic3r` or `libvgcode` rewrite slice, not an Ares-authored pipeline. Any milestone that cannot point to the upstream source boundary and the Rust destination boundary must be rejected or rewritten before work begins.

## Active program: OrcaSlicer v2.4.2 3MF project G-code parity

The active development program is the source-cited vertical rewrite defined by
`docs/architecture/ard-0023-3mf-project-gcode-parity.md` and
`docs/superpowers/specs/2026-07-10-ksr-fdmtest-v4-gcode-parity.md`. Its fixed
upstream baseline is OrcaSlicer `v2.4.2`, commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. The exit condition is normalized
byte equality between Ares output for
`tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf` and the supplied reference
G-code, where only the validated generator name/timestamp line differs.

Program order:

1. Add the bounded golden harness, hash-pinned fixtures, Tier-1 CI, dynamic
   serde-value migration audit, and remove obsolete source-line-only pinning
   tests/modules without removing runtime behavior.
2. Port the in-memory Bambu/Orca 3MF package, relationship, model, metadata,
   plate, slice-info, and filament-sequence loader from `Format/bbs_3mf.*` and
   `Model.*` into typed serde-backed `ares-core::project` structures.
3. Replace the raw dynamic option map with concrete typed printer, process,
   filament, object, region, and G-code option structs; then port effective
   normalization and config export from `Config.*`, `PrintConfig.*`, and
   `PrintApply.cpp`. This step is split into separately reviewed config-group
   increments.
4. Port scaled geometry, polygon operations, and mesh/layer slicing from
   `Geometry.*`, `TriangleMeshSlicer.*`, `ClipperUtils.*`, and
   `PrintObjectSlice.cpp`.
5. Port layer surfaces, classic perimeters, shell/fill/gap behavior, and brim
   generation from `Layer.*`, `Surface.*`, `PerimeterGenerator.*`, `Fill/*`,
   `Brim.*`, and `PrintObject.cpp`.
6. Port print ordering, extrusion, travel, retraction, speed, acceleration, fan,
   and writer state from `Print.*`, `ExtrusionEntity.*`, `GCode.*`, and
   `GCodeWriter.*`.
7. Port placeholder evaluation, custom G-code, and exact header/config/footer
   serialization from `PlaceholderParser.*`, `GCode.*`, `GCodeWriter.*`, and
   `Config.*`.
8. Port `GCodeProcessor` time estimation, statistics, and post-processing;
   remove the temporary golden-test ignore; verify original OrcaSlicer v2.4.2
   E2E provenance and final byte parity.

Every implementation increment uses `sdd-workflow` and
`superpowers:subagent-driven-development`, requires literal
`VERDICT: APPROVE` from both an independent Codex reviewer and OpenCode,
updates this roadmap with completed and deferred behavior, passes the specified
local/Tier-1 verification, and is committed and pushed before its dependent
increment starts.

One-source-line `PrintConfig.hpp`, `PrintConfig.cpp`, and staged `PrintApply`
milestones, their raw-line/token metadata, pinning tests, and milestone
documents have been removed. New work may not create raw-line metadata
modules, source-boundary pinning tests, or source-line-only roadmap entries.
Functional runtime work from earlier milestones remains subject to replacement
or reuse according to the exact destination map in the approved parity spec.

### 2026-07-10 Establish Tier-1 parity verification

Task 1A adds the parity program's committed verification matrix. Native jobs
run workspace nextest and warning-denying clippy on Windows, macOS, and Linux;
separate Linux jobs check rustfmt and compile `ares-core` plus `ares-wasm` for
`wasm32-unknown-unknown`. The local baseline is 15,916 passing nextest tests,
clean rustfmt/clippy, and successful checks for both WASM crates. The only
baseline portability correction gates Unix-only test helpers with
`#[cfg(unix)]`, preserving their Unix behavior while removing Windows-only
dead-code diagnostics. Browser JavaScript-to-WASM project slicing remains
deferred to Task 4, and the byte-exact golden harness, dynamic-value audit,
source-pinning cleanup, 3MF import, typed options, slicing, G-code, processor,
and final parity tasks remain incomplete.

### 2026-07-10 Establish the byte-exact project golden

Task 1B pins the project and reference hashes, the 15-entry package contract,
269,330 reference lines, 460 layer markers, and normalized reference SHA-256
`c61202df3fa26ffcb3064f2dbc02e06a89f95565b8325b31029ec4ed6cedcdc4`.
The test-only helper validates exactly one complete UTF-8 Orca/Ares generator
line, normalizes only that line, and reports mismatches with a bounded
three-line first-difference diagnostic. The full CLI golden remains explicitly
ignored: its required RED run currently exits because `slice` still requires
`--options <OPTIONS>`. Project import, embedded typed options, slicing, G-code
generation/post-processing, and activation of the exact CLI comparison remain
incomplete.

### 2026-07-12 Establish the syntax-aware dynamic-value migration audit

Task 1C protects the typed rewrite boundaries owned by upstream `Config.*`,
`PrintConfig.*`, and `Format/bbs_3mf.*` without adding slicing behavior. The
audit follows the reachable production module/include graphs rooted in
`ares-core`, `ares-cli`, and `ares-wasm`, excludes test-only items, resolves
the reviewed import/re-export and type-alias forms, and records 743 existing
dynamic-value occurrences as stable named-owner/occurrence fingerprints. The
committed baseline must match its bootstrap production scan and may only
shrink across every full-history parent edge. Tier-1 checkout now fetches full
history for that ratchet. The strict source-cited open-field allowlist has no
approved entries and rejects any allowed field used for bounded type or
slicing dispatch. Focused audit verification passes 22 tests with one
print-only test skipped; the reviewed workspace baseline is 6,564 passing
nextest tests with clean rustfmt, warning-denying clippy, and both WASM checks.
Project import, concrete typed options, slicing, G-code generation and
post-processing, baseline migration to empty, and final byte parity remain
deferred to their planned increments.

### 2026-07-12 Remove PrintConfig source-line pinning

Task 1D removes the non-runtime `OptionDefinition.source` field, the source
argument from all 736 option definitions, and the associated
`PrintConfig.hpp`/`PrintConfig.cpp` line-fragment assertions. The ordered
registry key, kind, and default-value tuples are unchanged, and the mixed
registry tests continue to verify those runtime contracts. The former
`print_config_hpp_modules` aggregate and its registrations remain absent.
Focused verification passes 1,302 Option tests and the 22-test syntax-aware
dynamic-value audit; independent Codex and OpenCode reviews both approve the
final tree. Typed option structs, removal of the 743-entry dynamic-value
baseline, embedded 3MF option composition, and end-to-end project slicing
remain deferred to their planned increments.

### 2026-07-12 Remove staged PrintApply source-token pinning

Task 1E removes copied C++ receiver/source/comment/action metadata, commented
non-actions, queried-key mirrors, and numeric-discriminant-only assertions from
mixed `PrintApply.cpp::Print::apply` state modules. It retains observable
filament-map branching and pruning, count changes, normalization, status
precedence, invalidation ordering, transforms, volume caches, regions, and
geometry behavior. Declarations and tests remain one-to-one for 37 production
state modules and 53 test modules. Focused verification passes 404 PrintApply
tests and the 22-test syntax-aware dynamic-value audit; independent Codex and
OpenCode reviews both approve the final tree. Public project wiring, concrete
typed `PrintApply` config ownership, replacement of the remaining staged
compatibility shell, and complete slicing/G-code parity remain deferred.

### 2026-07-12 Add the bounded in-memory 3MF package reader

Task 2 ports the archive-extraction and OPC package-path boundary from the
fixed OrcaSlicer v2.4.2 `Format/bbs_3mf.hpp::load_bbs_3mf` and
`Format/bbs_3mf.cpp::_BBS_3MF_Importer` helpers into crate-private
`ArchiveLimits`, `PackagePath`, and `ProjectArchive` types under
`ares-core::project`. A raw central-directory preflight now enforces the
4,096-entry, 256 MiB per-entry, 1 GiB total, and 1,000:1 expansion limits
before payload allocation; accepts only Stored and Deflated data; and rejects
encryption, exact or normalized duplicates, conflicting Unicode aliases,
central/local/ZIP32/ZIP64 descriptor mismatches, expanded-size mismatches, and
CRC failures. Entry reads are capped at declared size plus one and consume EOF
to force CRC verification. Host-independent OPC paths resolve package-root and
owner-relative targets with one normalization pass while rejecting drive/UNC,
authority, backslash, NUL, empty/dot, encoded-separator, query, and fragment
ambiguities. Focused verification passes 21 archive and 19 path tests; the
reviewed workspace passes 4,115 tests, the dynamic-value audit, rustfmt,
warning-denying Clippy, and both WASM checks. Independent Codex and OpenCode
reviews approve the final Task 2 tree. Typed content types, relationships,
project documents, model/domain loading, embedded options, and project slicing
remain deferred to the dependent tasks; this reader is not wired into the old
empty 3MF compatibility shell.

### 2026-07-12 Deserialize typed 3MF package metadata

Task 3 ports the fixed OrcaSlicer v2.4.2 `Format/bbs_3mf.cpp` content-type,
relationship, model-settings, slice-info, filament-sequence, and plate-metadata
wire boundaries into concrete crate-private serde structs under
`ares-core::project`. Namespace-aware XML validation now scans the complete
document before direct typed deserialization, limits documents and decoded text
to 64 MiB, nesting to 256 levels, and attributes to 1,024 per element, and
rejects DTD/entity expansion plus XML 1.0/1.1 illegal characters. JSON is
likewise size-bounded and deserialized directly without `Value`, a DOM, or an
erased catch-all. OPC relationship targets distinguish package-root and
owning-part relative resolution; content types enumerate every PNG part and
force archive size/CRC validation without decoding preview pixels. Focused
verification passes 5 typed-document and 14 hostile-document tests. Primary
review and a fresh independent Codex re-review approve the final tree after an
initial review found and the implementation closed the XML Legal Character
gap; the user-approved temporary OpenCode bypass applies to this increment.
The reviewed workspace passes 4,134 tests with 2 skipped, the 22-test
dynamic-value audit with 1 skipped, rustfmt, warning-denying Clippy, and both
WASM checks. The ignored CLI golden remains at its planned nonzero
required-`--options` boundary until the project API and adapters land.
Model XML, meshes/transforms, public project-domain loading, embedded project
options, and slicing remain deferred to dependent tasks, and this metadata
layer is not wired into the old empty 3MF compatibility shell.

### 2026-07-12 Load 3MF models into the public project domain

Task 4 ports the model, mesh, component, build-item, transform, and
volume-assembly boundary from fixed OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1` (`Format/bbs_3mf.cpp`) into
concrete `ares-core::project` types. `load_project` now starts at the OPC root
relationship, loads every reachable model part in memory, preserves `f64`
vertices and triangle indices, composes build and component transforms in Orca
order, retains part matrices as source provenance, and exposes path-qualified
objects, volumes, instances, plates, project-settings bytes, loaded label ID
133, and the original `printable` / `auto_drop` flags. The old empty 3MF branch
in the explicit-option STL loader is gone and returns a clear `load_project`
boundary error instead.

The public untrusted-input boundary validates XML typed-attribute namespace
ownership, OPC relationship-ID uniqueness, relationship-part ownership and
MIME, declared preview existence and PNG MIME without decoding pixels, exact
`(PackagePath, object_id)` component references for every loaded model, mesh
indices and finite coordinates, and the unavoidable ambiguity where Orca's
path-qualified build identity meets bare-ID model-settings, plate, and assemble
metadata. Optional unreferenced model parts and their canonical relationship
parts remain ignored. Review-discovered namespace spoofing, cross-model ID
collapse, missing preview/relationship ownership, duplicate relationship IDs,
and unused reachable component-reference gaps were each reproduced with RED
mutations and closed before the final independent Codex re-review approved the
tree; the user-approved temporary OpenCode bypass applies to this increment.

`GenerationMetadata`, `load_project`, and async `slice_project` are now stable
core APIs. Until the later project pipeline lands, a successfully loaded
project returns the typed `ProjectSlicingIncomplete` result. The stable WASM
`sliceProject` export supplies local `js_sys::Date` fields, calls only the core
API, and is exercised through generated `wasm-bindgen` JavaScript in real
headless Chromium with the committed fixture. Reviewed verification passes
4,178 workspace tests with 2 skipped, the 22-test dynamic-value audit with 1
skipped, rustfmt, warning-denying Clippy, both WASM checks, and the Playwright
browser boundary 1/1. Embedded typed option codecs/inventory, effective project
configuration, project geometry slicing, toolpaths, G-code serialization and
processing, CLI project-form activation, and removal of the golden-test ignore
remain deferred to Tasks 5 onward; the ignored CLI golden still fails at its
planned required-`--options` boundary.

### 2026-07-12 Establish typed option codecs and the fixed v2.4.2 inventory

Task 5 ports the serialization boundary of fixed OrcaSlicer v2.4.2
`Config.hpp/cpp::ConfigOption*`, option registration/default/nullable/legacy
metadata from `PrintConfig.hpp/cpp`, raw scopes from `Preset.cpp`, metadata
headers from `ConfigBase::save_to_json` plus `Preset.hpp`, and config-export
control flow from `GCode.cpp::append_full_config`. Ares now has concrete serde
codecs for Orca scalar, vector, nullable, point, enum, and fixture-specific
opaque wire forms, plus typed group dispatch support that does not deserialize
through an erased value.

The committed fixed-source inventory proves 653 sorted fixture keys, exact
scope/type/default/wire/nullable/owner/projection/legacy/export metadata, 31
nullable options, and export disposition of 615 canonical, 31 omit-when-nil,
three metadata, and four special entries. Its ignored provenance test
reconstructs every row from commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, verifies every citation, and runs
19 generator plus 13 independent Rust-side semantic mutations. The 650
non-metadata consumer citations intentionally identify the generic
`append_full_config` runtime guard; they are provenance for retention/export,
not claims that every option's slicing behavior is implemented.

Reviewed focused verification passes nine codec tests, two project-inventory
tests, the active inventory test, and the fixed-source ignored provenance test,
with clean rustfmt and warning-denying `ares-core` all-target Clippy. Two
independent Codex reviews approve the final implementation under the
user-approved temporary OpenCode bypass. The later Task 6 review corrected
qualified enum-default provenance for `InputShaperType::Default`, distinguished
`NozzleType::ntUndefine` from the nullable `nil` sentinel, and moved the 12
loop-generated axis definition citations from a variant list to their exact
`PrintConfig.hpp` typed declarations. Independent JavaScript and Rust checks
now bind the `AxisDefault` aggregate to its three registration blocks.

### 2026-07-12 Type the printer machine-envelope group

Task 6 ports fixed OrcaSlicer v2.4.2
`PrintConfig.hpp::MachineEnvelopeConfig`, the `PrintConfig.cpp::AxisDefault`
table and registration loop, the remaining literal machine-envelope defaults,
the 13-token `InputShaperType` map, and the fixed `Preset.cpp` printer-scope
lists. `ProjectSettings` now contains `PrinterOptions`, whose flat typed
dispatcher owns a completed `MachineEnvelopeOptions` child with exactly 28
fields and no untyped remainder.

The new group preserves upstream declaration order separately from
lexicographic 3MF serialization, retains all raw fixture vector values, applies
fixed defaults through a private typed builder, and rejects duplicate, unknown,
or invalid enum fields. All 28 fields remain `retained-only`: the focused
behavior test observes changed typed state, while strict top-level project
deserialization, effective config composition, normalization, and typed-path
G-code consumption remain deferred.

Reviewed verification passes seven focused machine-envelope tests, the active
inventory tests, the fixed-source ignored provenance gate, 19 generator
mutations, the 22-test dynamic-value audit, clean rustfmt, warning-denying
`ares-core` all-target Clippy, and both `ares-core` and `ares-wasm` WASM checks.
Two fresh independent Codex reviews approve the implementation under the
user-approved temporary OpenCode bypass. Task 7 is next: type the exact 62-key
printer/`GCodeConfig` intersection without pulling adjacent `PrintConfig` point
groups into that boundary.

### 2026-07-12 Type the printer G-code-source group

Task 7 ports the exact fixed OrcaSlicer v2.4.2 intersection of the 132 printer
raw keys with `PrintConfig.hpp::GCodeConfig`: 62 concrete fields, their literal
`PrintConfig.cpp` defaults, nine fixed enum maps, nullable integer/enum vector
semantics, and declaration order. `PrinterOptions` now flat-dispatches both its
28-field machine-envelope child and its 62-field G-code-source child without a
dynamic remainder.

The group preserves fixed declaration order separately from lexicographic 3MF
serialization, all fixture vector cardinalities, the five multiline G-code
values byte-for-byte, all 13 `GCodeFlavor` tokens, and the distinction between
`NozzleType::Undefine` and nullable `nil`. Its sole point field is
`wrapping_exclude_area`. The adjacent `extruder_printable_area`,
`extruder_offset`, and `bed_shape` examples from the original plan are outside
this exact intersection and were explicitly excluded rather than pulled into
Task 7.

Reviewed verification passes 11 focused tests, the workspace nextest suite,
warning-denying `ares-core` all-target Clippy, both WASM checks, the 22-test
dynamic-value audit, rustfmt, and the diff whitespace gate. Independent review
approves the implementation under the user-approved temporary OpenCode bypass.
All 62 fields remain typed `retained-only`; effective config composition,
normalization, runtime consumers, and G-code output remain deferred. Task 8 is
next: complete the remaining 42 printer raw keys at their `PrintConfig` and
runtime ownership boundaries.

### 2026-07-12 Complete the typed printer raw scope

Task 8 ports the final 42 fixed OrcaSlicer v2.4.2 printer raw rows: 27 declared
by `PrintConfig.hpp::PrintConfig` and 15 classified `unowned` by the committed
FFF raw inventory with defaults and enum maps registered in `PrintConfig.cpp`.
`PrinterOptions` now has one remaining child and completes the exact disjoint
`28 + 62 + 42 = 132` printer set. Its direct parent serializer emits all 132
keys in global lexicographic order without `flatten`, buffering, an erased map,
or a dynamic remainder.

The implementation preserves both element-nullable float vectors, all point,
point-list, and point-group shapes, explicit empty area arrays, the fixture's
physical-extruder and expanded-variant cardinalities, and the raw bytes of
`extruder_variant_list` and `thumbnails`. Variant normalization and exact
thumbnail composite parsing remain deferred instead of reusing behavior that
does not match the fixed source. The original Task 8 plan incorrectly named
`extruder_ams_count`; inventory and source review proved it is a residual key,
so `PrinterOptions` rejects it and Task 14 retains ownership.

Reviewed verification passes 13 focused printer-option tests, 4,222 workspace
tests with three configured skips, warning-denying workspace all-target Clippy,
rustfmt, both `ares-core` and `ares-wasm` WASM checks, the dynamic-value audit,
and the diff whitespace gate. Two independent frozen-byte Codex reviews and
the primary-agent review approve the implementation under the user-approved
temporary OpenCode bypass. All 42 fields remain typed `retained-only`;
effective config composition, normalization, slicing consumers, config export,
and G-code output remain deferred. Task 9 is next: type the exact 126
process/`PrintObjectConfig` raw rows.

### 2026-07-12 Type the process object-source group

Task 9 ports the 126 active fixed OrcaSlicer v2.4.2
`PrintConfig.hpp::PrintObjectConfig` declarations and their exact
`PrintConfig.cpp` defaults and canonical enum maps. The two commented tuple
lines are excluded. Every field is a scalar string: 22 bool, 12 enum, 63 float,
six float-or-percent, 13 int, and ten percent fields. `ProcessOptions` now owns
one flat typed `object` child and `ProjectSettings` exposes it; effective
`ObjectOptions` projection is not created early.

The source review corrected the original Task 9 examples: first-layer height
and resolution belong to Task 11, while wall count, sparse fill, and top/bottom
shell fields belong to Task 10. The raw support-ironing enum uses all 28 fixed
`InfillPattern` tokens rather than the two UI entries. The shared typed builder
now preserves the Option key in value-decoding errors. Direct tests exercise a
non-default value for all 126 fields, because only 18 fixture values differ
from upstream defaults.

The current dynamic behavior pipeline contains literal consumers for 108 of
the 126 names; Task 9 records those collisions without migrating them. Task 15
now establishes effective object projection and ordered sparse overrides,
while typed migration and removal of those dynamic consumers remain owned by
Tasks 20A-20E. Reviewed verification passes 12 focused tests, 4,234 workspace
tests with three configured skips, warning-denying workspace all-target Clippy,
rustfmt, both WASM checks, the dynamic-value audit, and the diff whitespace
gate. The Task 9 raw group remains `retained-only`. Task 10 is next: type the
exact 149 process/`PrintRegionConfig` raw rows.

### 2026-07-12 Type the process region-source group

Task 10 ports the exact 149 retained process rows owned by fixed OrcaSlicer
v2.4.2 `PrintRegionConfig`, from 155 active HPP tuples. It excludes four
filament-scope nullable ironing overrides and the two legacy-only shells
`ironing_direction` and `wall_infill_order`. The type histogram is 31 bool,
14 enum, 49 float, 24 float-or-percent, 15 int, one integer vector, 11 percent,
three string, and one string-vector field.

The two vectors retain arbitrary valid lengths rather than encoding the
fixture's four-extruder cardinality. Five pattern fields reuse the full fixed
28-token raw infill map, while nine dedicated raw enums preserve the remaining
canonical domains. `ProcessOptions` now decodes both process children from one
flat map and directly streams their global lexical 275-key union. Task 10
introduced all fields as retained raw source state; UI/legacy conversion
remains deferred to Task 19A, while Task 16 now reuses the same 149-field
inventory for effective region projection.

The existing dynamic behavior pipeline has literal consumers for 109 of the
149 names. Task 10 records that boundary without migrating consumers. Focused
tests cover all 149 direct dispatch arms, exact source/default/fixture shape,
the 30 fixture overrides, enum maps, vector shapes, and exact 149-child and
275-parent bytes. Reviewed gates pass 24 object/region focused tests, 4,246
workspace tests with three configured skips, warning-denying workspace
all-target Clippy, rustfmt, both WASM checks, the dynamic-value audit, and the
diff whitespace gate. Task 11 is next: type the remaining 77 process raw
options owned by `GCodeConfig`, `PrintConfig`, and the one unowned row.

### 2026-07-12 Complete typed process raw ownership

Task 11 ports the remaining exact 77 process raw options from fixed OrcaSlicer
v2.4.2: 17 filtered `GCodeConfig` declarations, 59 FFF `PrintConfig`
declarations, and the one unowned `ironing_expansion` definition. Together
with the 126 object-source and 149 region-source fields, `ProcessOptions` now
owns all 352 process raw keys exactly once. The remaining histogram is 25
bool, six enum, 24 float, six float-or-percent, one float vector, six int,
four percent, three string, and two string-vector fields; all 77 are
non-nullable.

The two new typed children preserve the fixed HPP declaration boundaries and
strict canonical enum domains, while `ironing_expansion` remains a direct
parent scalar rather than an invented effective group. The three raw arrays
`post_process`, `small_area_infill_flow_compensation_model`, and
`wiping_volumes_extruders` preserve arbitrary valid lengths, including empty
arrays. `ProcessOptions` directly streams its flat 352-key union in global
lexical order across the 115/124/113-entry parent wire helpers. All fields
remain `retained-only`; legacy/UI conversions remain Task 19A work,
full-print normalization remains Task 19B, behavioral-consumer migration
remains Tasks 20A-20D, and compatibility parser removal remains Task 20E.

TDD began with an E0432 failure for the missing `ProcessGCodeSourceOptions`.
Focused tests cover exact 77/352 ownership, the three arrays, all six enum
domains, the exact 15 fixture overrides, every field's valid non-default
child/parent dispatch, all 77 keyed null failures, all 74 scalar array/object
shape failures, direct-scalar duplicate handling, and exact standalone-child
and 352-parent bytes. Reviewed local gates pass 14 focused tests, 24 adjacent
process tests, 4,260 workspace tests with three configured skips,
warning-denying workspace all-target Clippy, rustfmt, both WASM checks, the
22-test dynamic-value audit with one configured skip, the diff whitespace
gate, and the physical-LOC audit for every changed production and test module.

### 2026-07-13 Type the filament G-code-source group

Task 12 ports 52 live filament preset names in the fixed OrcaSlicer v2.4.2
`Preset.cpp:1309-1346` / `PrintConfig.hpp:1299-1476` `GCodeConfig`
intersection plus the separately project-owned `filament_colour` from
`PresetBundle.cpp:43-58,2652-2658,2795-2802`. The resulting exact 53
declarations are fixed at `PrintConfig.hpp:1308-1464`, with
definitions/defaults in `PrintConfig.cpp` and raw array/nullable serde
behavior in `Config.hpp` and `Config.cpp`. The exact histogram is eight bool
vectors, 27 float vectors, seven int vectors, and 11 string vectors. All 53
wire values are arrays and none is a Task 12 enum.

The seven nullable-element fields use direct `Vec<Nullable<T>>` storage. The
four raw semantic string-vector wrappers are `CsvTable`, `VariantStride`,
`RammingParameters`, and `SpaceTuple`; they retain bytes without parsing or
normalization. Exact upstream defaults remain singleton vectors. The fixture
retains 43 two-element vectors plus the ten fixed variant-stride eight-element
vectors, and exactly 17 payloads differ from defaults after cardinality is
ignored. Arbitrary valid raw lengths remain accepted, so Task 12 does not
encode the fixture's active-filament count or its later `[0,4]` selection.

`FilamentOptions` exposes one flat `gcode` child, and `ProjectSettings`
exposes the typed filament aggregate. Standalone child and parent output use
the same global lexical 53-key map without nesting, flattening, or DOM
buffering. The existing compatibility code has literal collisions for 51
keys; the exact complement is `adaptive_pressure_advance_model` and
`adaptive_pressure_advance_overhangs`, and no consumer is migrated here.
Legacy conversions remain Task 19A, active sizing and FDM normalization remain
Task 19B, nullable config-block omission remains Task 19C, consumer migrations
remain Tasks 20A and 20D, and final compatibility-parser removal remains Task
20E.

TDD began with the expected missing Task 12 types and
`ProjectSettings::filament`. Reviewed local gates pass 14 focused tests, 62
adjacent typed printer/process tests, 4,274 workspace tests with three
configured skips, the 22-test dynamic-value audit with one configured skip,
warning-denying `ares-core` all-target Clippy, rustfmt, native and WASM checks,
tracked/untracked whitespace checks, and the physical-LOC audit. Independent
fixed-source, TDD-plan, wrapper, inventory, frozen-byte quality, and final
specification reviews approve the slice under the user-approved temporary
OpenCode bypass.

### 2026-07-13 Complete typed filament project options

Task 13 ports the remaining exact 69 raw filament fields from fixed
OrcaSlicer v2.4.2: 48 FFF `PrintConfig` declarations at
`PrintConfig.hpp:1484-1650`, four nullable `PrintRegionConfig` ironing fields
at `PrintConfig.hpp:1153-1156` / `PrintConfig.cpp:3492-3538`, 16 generated
nullable retract overrides at `PrintConfig.cpp:63-84,7287-7318`, and direct
`pellet_flow_coefficient` at `PrintConfig.cpp:2639-2643`. All 69 wire values
are arrays. The aggregate histogram is 11 bool, three enum, 20 float, 30 int,
four percent, and one string vectors; the 48/4/16/1 partition histograms are
Print `8/1/6/30/2/1`, Region `0/0/3/0/1/0`, retract `3/2/10/0/1/0`, and one
pellet float vector.

Task 13 adds exactly 20 nullable fields: four singleton-nil region defaults
and 16 generated fields whose singleton defaults clone their extruder
sources. This completes 27 nullable filament fields with Task 12's seven, and
31 project-wide only after including Printer's four. The fixture preserves 42
two-entry and the exact 27 eight-entry vectors selected by
`PrintConfig.cpp:8375-8415`; all 69 differ from singleton defaults by
cardinality, with exactly 36 semantic overrides and 33 repeated defaults.
Raw arrays keep arbitrary valid lengths and nil elements without active
selection, resizing, inheritance, or cross-field normalization.

The three strict raw enum domains retain the six canonical overhang-threshold
tokens, four retract-lift-enforcement tokens, and four Z-hop tokens from
`PrintConfig.cpp:1227-1248,5282-5295,5320-5333` with defaults `95%`,
`All Surfaces`, and `Slope Lift`; the fixture carries `50%`, nil, and
`Spiral Lift`. `filament_notes` remains the sole raw string vector, and direct
`pellet_flow_coefficient` retains singleton default `0.4157`.
`FilamentOptions` exposes its four source children plus direct pellet storage
and streams one flat global lexical 122-key parent map through 41/41/40
helpers.

The compatibility layer has literal collisions for exactly 66 Task 13 names;
the exact complement is `chamber_minimal_temperature`,
`filament_long_retractions_when_cut`, and
`filament_retraction_distances_when_cut`. No consumer moves here. Task 16 now
selects the four region ironing vectors into concrete effective values; legacy
conversion remains Task 19A; active sizing,
normalization, and retract inheritance remain Task 19B; all-nil export remains
Task 19C; consumer migrations remain Tasks 20A and 20D; and final
compatibility-parser removal remains Task 20E.

TDD first failed only on the missing Task 13 interfaces. Reviewed local gates
pass 22 focused tests, all 14 adjacent Task 12 tests, all 4,296 workspace tests
with three configured skips, and the 22-test dynamic-value audit with one
configured skip. Warning-denying workspace Clippy, rustfmt, native and WASM
checks, whitespace checks, and the under-400-LOC gate are green. Independent
upstream, RED-test, final-specification, code-quality, and frozen-byte reviews
approve the slice under the user-approved temporary OpenCode bypass. The
five-job Tier 1 workflow gates the exact pushed commit before Task 14 begins.

### 2026-07-13 Type project/runtime residual raw options

Task 14 ports the exact 47-key residual at fixed OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`: 17 filtered `GCodeConfig`
declarations, 19 filtered FFF `PrintConfig` declarations, eight direct
project/preset registrations, and three separate metadata strings. The
boundary is `653 - 132 Printer - 352 Process - 122 Filament = 47`; literal
preset-list subtraction would incorrectly produce 48 because Task 12 already
owns the commented-out preset key `filament_colour`.

The 44 real options are concrete and non-nullable with histogram two bool
scalars, two bool vectors, two enum scalars, one enum vector, 19 float vectors,
one int scalar, four int vectors, one percent vector, two point vectors, two
string scalars, and eight string vectors. The corrected defaults retain
`extruder_ams_count=[]`, a 16-entry `flush_volumes_matrix` with zero diagonal
and `280` elsewhere, eight `140` flush-vector entries, and the remaining fixed
source values recorded in the Task 14 architecture ledger. Metadata is the
separate `from,name,version` sibling and defaults to empty strings.

The raw enum domains are exactly seven bed tokens including `Default Plate`
and canonical `Supertack Plate`, three filament-map tokens excluding UI-only
`Default`, and two nozzle-volume tokens `Standard` / `High Flow`. Legacy
spellings remain rejected here. Canonical fixture shape is 37 arrays and ten
scalar strings; its array histogram is `{1:6,2:14,4:15,8:2}`. Exactly seven
real fixture values equal defaults and 37 differ, while arbitrary valid raw
vector lengths remain accepted.

`ProjectRuntimeOptions` directly streams one globally lexical flat 44-key map,
and `PresetMetadata` streams lexical `from,name,version`. Tests prove the
pairwise-disjoint typed 653-key union, but production top-level
`ProjectSettings` load/save and cross-group dispatch remain Task 18. The
dynamic debt ledger records 31 literal collisions and the exact 13-key
no-collision complement without migrating consumers or changing the dynamic
baseline.

Task 17 retains all 17 effective G-code projections. Task 19A retains legacy
conversion; Task 19B retains active sizing, AMS/self-index interpretation,
normalization, and cross-field validation; Task 19C retains metadata exclusion
and special config-block export; and Tasks 20A-20E retain consumer migration
and final compatibility-parser removal.

TDD began with two frozen focused RED runs that failed only on missing planned
interfaces. Local verification passes 23 focused tests, 107 adjacent tests,
all 4,319 workspace tests with three configured skips, the 22-test dynamic
audit with one configured skip, rustfmt, warning-denying workspace Clippy,
native and WASM checks, release web binding generation, the real-3MF
Playwright test, whitespace and forbidden-dynamic scans, exact file ownership,
and the under-400-LOC gate. The largest changed Rust file is 290 lines.
Independent final specification and code-quality reviews approve the frozen
implementation under the user-authorized temporary OpenCode bypass. Commit
`dc47e069ede1caa307411d63ba29f78784630494` and exact-SHA Tier 1 run
`29253342315` are green across Windows, macOS, Ubuntu/Linux, format, and WASM.

### 2026-07-13 Resolve effective object options

Task 15 ports the fixed OrcaSlicer v2.4.2
`PrintConfig.hpp:917-1071` 126-field `PrintObjectConfig` boundary, ordered
model-object metadata from `Model.hpp`, `Config.cpp`, and
`Format/bbs_3mf.cpp`, and static object projection plus support-filament
clamps from `PrintObject.cpp:3555-3579`. `PrintApply.cpp` supplies the
normalized process-base and `num_extruders` recomputation contract, while
`PrintConfig.cpp:8520-8741` supplies the separately verified normalization
write sets.

One shared typed inventory drives the existing raw process group, an
all-absent sparse override struct, and a distinct effective object struct.
Ordered metadata uses last-write-wins for typed object fields, `name`, and
`module`, retains all non-object entries in source order, and reports a
malformed later duplicate by its key. Resolution copies the supplied process
base, applies only present fields, then clamps each support filament ID to `1`
only when it is strictly greater than the supplied extruder count. Fixed
monolithic and split normalization write sets have zero intersection with all
126 fields.

The real 3MF verifies object ID 2, `name=ksr_fdmtest_v4.drc`, typed region
override `extruder=1`, zero typed object overrides or residual object config,
two nozzle diameters, and complete effective equality with the typed process
base. Exactly 108 object fields
equal fixed defaults and 18 carry process overrides; the complete differing
key set is recorded in the architecture ledger. No fixture identity or value
is present in production code.

Six sequential slices are green and independently approved for inventory/base
identity, ordered metadata, sparse projection, exact clamps, normalization
zero-intersection, and real-document verification. The complete
pre-documentation 28-file Task 15 diff also has fresh literal
`SPEC VERDICT: APPROVE` and `QUALITY VERDICT: APPROVE`. Region projection is
now implemented by Task 16. G-code projection, top-level project storage,
legacy conversion, general
normalization/association, config export, consumer migration, geometry,
slicing, and final G-code byte parity remain owned by Tasks 17-20 and later
stages. Task 15 is released as pushed commit
`4fbb61282cdb73160414d2d9f67edacf61ba2e42`; exact-SHA Tier 1 run
`29273332261` is green across format, Ubuntu/Linux, WASM, macOS, and Windows,
satisfying the Task 16 entry gate.

### 2026-07-13 Resolve effective region options

Task 16 ports fixed OrcaSlicer v2.4.2
`PrintConfig.hpp:1074-1249::PrintRegionConfig`, region overlay construction at
`PrintObject.cpp:3582-3709`, model-part/modifier call sites at
`PrintApply.cpp:786-795,1021-1042`, ironing selection at
`Fill/Fill.cpp:1591-1604`, and the ordered 3MF metadata plus exact lexical
codec boundaries in `Format/bbs_3mf.cpp`, `Config.hpp`, and `Config.cpp`. The
Rust destination is concrete `RegionOptions` and its crate-private typed
handoff/resolver in `ares-core`, not a new independent pipeline.

The shared 149-field process inventory now also drives concrete effective
region state, with four selected ironing outputs for 153 fields total. Ordered
object/part metadata routes all region keys plus `extruder` into typed sparse
overrides, omits those consumed entries from residual storage, and retains
every unconsumed entry in source order, including structural metadata.
`RegionBase` represents model-part and modifier-parent inputs; resolution
implements their distinct ordered precedence and six-feature mask/fallback
behavior, followed by six ID clamps, sparse-density/fuzzy normalization, and
final top-surface-ID selection of all four nullable filament ironing vectors.

The real 3MF proves typed object `extruder=1`, all six effective feature IDs
equal to one, selected index zero, and concrete nil inheritance of `10%`,
`0.15`, `0.21`, and `30`, without fixture-specific production behavior. All
seven sequential slices and the frozen whole diff have independent
specification and quality approval. G-code projection remains Task 17;
cardinality, active sizing, and association remain Task 19B; consumer migration
remains Tasks 20A-20E; modifier graph construction, geometry, slicing, G-code
generation, and final byte parity remain later work. Task 16 is released as
pushed commit `2651c6376d0cc8229876471d0a4d5c6f98f84314`; exact-SHA Tier 1
run `29286285164` is green across format, Ubuntu/Linux, WASM, macOS, and
Windows.

### 2026-07-13 Project registered pre-normalization GCodeConfig options

Task 17 ports the registered projection boundary from fixed OrcaSlicer v2.4.2
`PrintConfig.hpp:759-776::StaticPrintConfig::StaticCache::finalize`,
`PrintConfig.hpp:1299-1476::GCodeConfig`, and static cache initialization at
`PrintConfig.cpp:10571-10585`. Of 151 active C++ members, unregistered legacy
input `thumbnail_size` and temporary non-Option placeholder
`bbl_bed_temperature_gcode` do not enter the 149-key registered set. The Rust
destination is the HPP-order compile-time ledger in `gcode_fields.rs` and the
public concrete `GCodeOptions` in `gcode_options.rs`, with only
`Clone`/`Debug`/`PartialEq` and a crate-private infallible `from_sources`.

The projection directly clones the pairwise-disjoint 62 printer, 17 process,
53 filament, and 17 project/residual typed source fields. It preserves the 69
scalar and 80 array raw shapes, including nine nullable arrays, without active
selection, resizing, inheritance, normalization, fallback, or runtime option
lookup. Independent inventory, type, all-field projection, 16-template,
four-opaque-string, shape, and bounded real-3MF tests are approved. The fixture
proof uses a test-only four-source split and retains one empty, 49 length-two,
19 length-four, ten length-eight, and one length-ten array; the 19 printer
variant, ten filament variant, and other 43 filament G-code arrays retain their
raw length-four, length-eight, and length-two shapes respectively. Production
remains WASM-safe and adds no I/O, dynamic/erased value, JSON round trip,
runtime registry, reference-G-code read or branch, or source-line pinning.

All four implementation slices and the frozen whole implementation have
independent specification and quality approval. Task 17 is released as pushed
commit `18e7065856bee306cd643ffe359023758a60befe`; exact-SHA Tier 1 run
`29294487109` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.
Task 18 began only after that gate completed. Task 19A retains legacy
conversion; Task 19B active sizing/selection, nullable retract overrides,
model recomputation, normalization, and final reprojection; Task 19C export;
Tasks 20A-20E consumers; Task 28 templates; and Task 29 document assembly.
Complete slicing and `ksr_fdmtest_v4` G-code byte parity are not yet
implemented.

### 2026-07-13 Strict typed project-settings load

Task 18 ports the fixed OrcaSlicer v2.4.2 load boundary at
`Config.cpp:573-685::set_deserialize_nothrow/set_deserialize/set_deserialize_raw`,
`Config.cpp:820-1100::ConfigBase::load_from_json`,
`Config.hpp:2763-2963::DynamicConfig`, and
`Format/bbs_3mf.cpp:210,1569-1573,1923-1926,2632-2653`. A streaming
`ProjectSettings` visitor dispatches all 653 fixture members directly into the
132 printer, 352 process, 122 filament, 44 project-runtime, and three preset
metadata fields. Input order is arbitrary, omitted members retain concrete
group defaults, and no production dynamic JSON value map or runtime registry
is introduced.

The public project domain now stores typed settings and exposes
`Project::settings()`; its raw settings bytes and production accessor are
removed. A bounded test-only archive oracle proves exact 653-member semantic
equality from the five standalone serializers. Unknown canonical keys and
duplicate canonical assignments are intentionally rejected with compact
key-specific diagnostics, while archive errors use
`invalid project settings JSON: ...`. Existing native JSON bool/number support
in the concrete Ares codecs remains compatible, although the real Orca fixture
uses only scalar strings and string arrays. No production
`Serialize<ProjectSettings>` or project-settings writer is added.

Slices 18.1-18.3 received their applicable independent per-slice reviews;
Slice 18.4 passed its verification-only isolation gate. The frozen whole
implementation then received independent whole-specification and whole-quality
approval. The typed loader remains isolated from `SliceOptions`, geometry,
slicing, and G-code generation, and the real-3MF native/browser path still
reaches `ProjectSlicingIncomplete` only after typed loading succeeds. Task 19A
retains legacy and complete-document composite conversion; Task 19B effective
sizing/selection/normalization; Task 19C only the effective `FullPrintConfig`
G-code `CONFIG_BLOCK`; and Tasks 20A-20E dynamic-consumer migration. Task 18 is
released as pushed commit `a2714d4a6a197c5e10aec1b686e80e9b66794fd6`;
exact-SHA Tier 1 run `29298974173` is green across format, Ubuntu/Linux, WASM,
macOS, and Windows. Complete fixture G-code byte parity is not yet implemented.

### 2026-07-13 Convert typed legacy project inputs

Task 19A ports the fixed OrcaSlicer v2.4.2 boundaries at
`PrintConfig.cpp:8033-8338`, `Config.cpp:573-685,885-1017` for implemented
typed lexical decode, JSON iteration, and derived slicing-state writes,
`PrintConfig.cpp:8099-8104,8121-8131` for the four deferred profile/UI input
rules, `Config.cpp:1018-1088` for deferred downstream profile-difference
bookkeeping, `Format/bbs_3mf.cpp:2119-2132,5088-5117` for ordered object and
part XML semantics, and `GCode/Thumbnails.cpp:530-577` into private concrete
typed conversion under `ares-core::options::typed_legacy`. Its compile-time
inventory records all 76 named source rules, 44 obsolete keys, 72 executable
inputs, and four deferred profile/UI bookkeeping inputs. It implements the
exact direct, conditional, global, pattern, wall-order, feature-filament, and
token transformations plus the twelve registered array targets'
empty-first-pass, typed-flattening, and complete-string second-pass semantics.
Obsolete inputs are consumed; deferred and unknown inputs fail with their exact
names.

Strict top-level project JSON shares canonical target presence across canonical
and legacy spellings and applies the two JSON-only derived slicing values after
iteration, independent of explicit-target order. Ordered object and part XML
instead perform one per-entry conversion, write concrete sparse owners or
retain non-owner canonical entries in place, preserve last-write-wins, and
receive neither the JSON-only effects nor the composite. Presence-aware
thumbnail conversion runs before builder resolution, honors explicit item and
top-level formats, defaults an absent format to PNG, and emits normalized
six-significant-digit `WIDTHxHEIGHT/FORMAT` items.

The public byte-oriented real-project path remains canonically idempotent, and
the generated browser WASM path still reaches the existing post-load
`ProjectSlicingIncomplete` boundary. `perimeter_feed_rate` and the unreachable
wiping-volume composite keys remain unavailable, while canonical
`flush_volumes_matrix` is unchanged. Obsolete checkout-dependent Orca source
pinning and generator mutation tests are removed; the 653-row semantic
inventory and deterministic fixed-commit generator remain byte-identical.

All five slices and the frozen whole implementation have independent
specification and quality approval. Fresh whole-task evidence passes 61
typed-legacy tests, 160 adjacent tests, all 4,484 workspace tests with two
configured skips, the dynamic-value audit, rustfmt, warning-denying Clippy,
native/WASM checks, browser proof, and fixed-source exclusion scans. Task 19A
is released as pushed commit `0e85302416904d0de604b969afd7f546fb8b3c1a`;
exact-SHA Tier 1 run `29313932330` is green across format, Ubuntu/Linux, WASM,
macOS, and Windows.

### 2026-07-14 Materialize typed project variants (Task 19B.1A)

Task 19B.1A ports fixed OrcaSlicer v2.4.2
`PrintConfig.cpp:8344-8473,8981-9054,9634-10023`,
`PrintConfig.cpp:588-606` for canonical typed variant spelling,
`PrintApply.cpp:1164-1173` for family order, `Print.cpp:3166-3175` for restoring
the saved pre-filament state before changed-map rematerialization, and
`Config.hpp:624-630` into the crate-private typed
`ares-core::options::project_variants` transform. It clones an unmaterialized
`ProjectSettings`, installs the supplied `filament_map`, and materializes
exactly two process, 24 printer variant-1, 15 stride-two printer variant-2, and
37 filament fields through their existing typed owners.

Printer variant 1 and process select the real project's raw indices `[0, 2]`.
Variant 2 then re-resolves the shortened printer selectors from the current
clone as `[0, 1]` and selects stride positions `[0, 1, 2, 3]`; filament uses
raw logical indices `[0, 4]`. Rematerialization must always restart from the
unmaterialized typed source, never a prior output. The external project
boundary returns keyed errors for out-of-range selected payloads instead of
the adjacent C++ first-value/default-output recovery. When an exact selector
match is missing, fixed C++ falls back to index zero or ID/zero recovery at
`PrintConfig.cpp:9677-9682,9840-9854`, while Ares returns
`SliceError::InvalidInput` naming the selector key.

The pure transform preserves every non-family field, remains isolated from
the dynamic compatibility path and adapters, and adds no file I/O, dynamic
option value, JSON round trip, or fixture/reference branch. The current Ares
scaffold remains a temporary compatibility shell until later source-cited
orchestration. Project slicing is not wired to the transform, so the real core
and browser WASM path still returns `ProjectSlicingIncomplete`.

Task 19B.1A's four TDD slices completed with 19/19 focused tests. The frozen
thirteen-path manifest
`96aa793696240f6d1a33d795e5e1ea308ee61a648fd2469d20263f98494d066b`
received independent specification-compliance, code-quality, and OpenCode
`VERDICT: APPROVE`; 235/235 adjacent typed tests, the 22/22 dynamic audit,
rustfmt, Clippy, fixture hashes, forbidden scans, and LOC checks passed. Task
19B.1A was released as commit
`da896a98719a621ad87a2317c23f1d27f0a3c6e5`; exact-SHA Tier 1 run
`29330209222` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.

Task 19B remains open. Task 19B.1B below owns the export/runtime split and
nullable retract overlay; Task 19B.2 owns model/layer configuration import and
association; and Task 19B.3 owns normalization plus effective
`FullPrintConfig` orchestration. Task 19C retains config export, Tasks 20A-20E
retain consumer migration/removal, and geometry, slicing, G-code, metadata,
post-processing, and complete `ksr_fdmtest_v4` byte parity remain later work.

### 2026-07-14 Resolve typed export/runtime retract views (Task 19B.1B)

Task 19B.1B ports fixed OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1` at
`PrintApply.cpp:222-263,1164-1191,1261-1283`,
`PrintConfig.cpp:7374-7392,10300-10332`, `Config.hpp:713-751`,
`Print.cpp:3166-3195`, `PrintConfig.hpp:1300-1478,1481-1610`, and
`GCode.cpp:2532-2534,5552-5557,5591-5594`. The Rust destination is the
crate-private typed `ares-core::options::project_config_views` plus its retract
overlay. It preserves the complete Task 19B.1A materialized input for export,
clones once for runtime, applies nullable retract values, and derives
`runtime_gcode` through the existing typed `GCodeOptions::from_sources`.

The twelve G-code-owned keys are `deretraction_speed`,
`long_retractions_when_cut`, `retract_before_wipe`, `retract_lift_above`,
`retract_lift_below`, `retract_lift_enforce`, `retract_restart_extra`,
`retraction_distances_when_cut`, `retraction_length`, `retraction_speed`,
`z_hop`, and `z_hop_types`; the four print-only keys are
`retract_when_changing_layer`, `retraction_minimum_travel`, `wipe`, and
`wipe_distance`. `travel_slope` is excluded. Empty vectors are no-ops;
nonempty overrides must match the map cardinality or return an error naming
the concrete `filament_*` key and `filament_map`; `Value` is direct; and `Nil`
uses the one-based machine map with invalid entries falling back to machine
element zero. The result preserves logical filament cardinality.

Gate `2` applies the normal bool/distance overlays. Gates `0` and `1` replace
the bool override with all-`Nil` entries before normal resolution but leave the
long-distance vector unchanged at physical cardinality, preserving the fixed
upstream typo. Map changes rerun the original Task 19B.1A materializer from raw
source before resolving fresh views. The old dynamic `filament_override`
scaffold is deleted with exactly its 31 baseline fingerprints.

Focused tests pass 13/13, adjacent project/G-code tests pass 79/79, and the
dynamic audit passes 22/22, including real-fixture and raw rematerialization
proof. Frozen implementation manifest
`eb06ab4a08293acf2b89b4e026fc52ac02887118eb1845dae50048456cc5eedd`
has independent whole `SPEC_COMPLIANCE`, `CODE_QUALITY`, and OpenCode
`VERDICT: APPROVE`. The byte/in-memory core remains portable across browser
WASM, Windows, macOS, and Linux. Public project slicing is deliberately not
wired and still returns `ProjectSlicingIncomplete`; no full G-code parity is
claimed.

Task 19B.1B was released as commit
`8e09be79881c6365100fac06ed064f487c75fb85`; exact-SHA Tier 1 run
`29345005311` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.
Task 19B.2 model/layer association, Task 19B.3 orchestration, Task 19C export,
Task 20E final dynamic removal, and complete normalized KSR G-code parity
remain open.

### 2026-07-14 Associate typed model and layer configuration (Task 19B.2)

Task 19B.2 ports fixed OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Canonical option lookup and wire
decode come from `PrintConfig.cpp:63-84,663-7328,7395-8031`,
`Config.cpp:258-318,573-685`, and the concrete `ConfigOption*` deserializers in
`Config.hpp`. Model identity, metadata, volume selection, naming, and color
fallback come from
`Format/bbs_3mf.cpp:744-764,2043-2168,3440-3513,3575-3735` and
`Format/bbs_3mf.cpp:3893-3908,4136-4400,4894-4954,5081-5126`,
`Model.hpp:354-370,865-918`, `Model.cpp:2717-2747`, and
`PrintConfig.hpp:2034-2128`. Optional layer ranges come from
`Format/bbs_3mf.cpp:209-216,1896-1904,2087-2095,2886-2940,7517-7545` and
`Slicing.hpp:150-151`; `PrintApply.cpp:342-383` normalization remains Task
19B.3.

The private `ares-core::options::model_config_deserialize` boundary now
classifies object, part, and layer metadata into existing typed sparse owners.
The canonical registry is exactly 751 sorted unique rows after adding 18 fixed
definitions and removing three legacy-only filament rows; their Task 19A
lowering rules remain. All 21 concrete wire kinds and the exact 650 typed-owner
/ 101 registry-only partition are validated without erased values, dynamic
maps, JSON round trips, fixture branches, or a wider public registry API.

Object metadata assigns object owners before region owners; part and layer
metadata assign only region owners. Other canonical typed-project values are
decoded through their existing concrete builder fields and discarded; the
registry-only complement is concretely validated and discarded; its five
scalar enum domains use `PrintConfig.cpp:402-419,481-485`. Model-path legacy
handling completes the three cumulative aliases plus
`different_settings_to_system`, and canonical/legacy duplicates remain XML
source-order last-write-wins.

`ProjectObject` now owns typed object/region overrides and layer ranges, while
`ProjectVolume` owns its typed five-way volume kind and region overrides.
Association preserves build-first object order, breadth-first leaf identity,
same-index then first-source part selection, default volumes for missing or
unmatched part settings, fixed fallback naming, exact structural scopes, and
ambiguous bare-ID rejection. The no-settings path derives the one-based object
extruder from typed `pid` and ordered production color groups with per-group
last color, submodel-first/root-replacement merge behavior, numeric group
ordering, and exact color deduplication.

Optional `Metadata/layer_config_ranges.xml` remains bounded and in-memory.
One ASCII case variant is accepted, multiple variants are rejected, one-based
ordinals target final object order, later duplicate options and exact ranges
win, and output is lexicographically sorted without normalizing finite
negative, reversed, gapped, or overlapping bounds. Boundary errors are strict,
keyed, and bounded.

The frozen 73-entry implementation manifest
`2b80a68423b3476a7f83676393d72bc6129c6f1ce9f15654cea50a2dd7496eb7`
received independent whole `SPEC COMPLIANCE`, whole `CODE QUALITY`, and
OpenCode default-model `VERDICT: APPROVE` decisions. Verification passed the
125/125 focused matrix, 4545/4545 workspace tests with two configured skips,
the 22/22 dynamic audit with one configured skip, rustfmt, warning-denying
Clippy, native/WASM checks, release WASM plus wasm-bindgen, the real-project
browser proof, fixture hashes, forbidden scans, diff validation, and the
sub-400-LOC audit.

Public project loading now exposes the richer typed domain, while slicing still
returns `ProjectSlicingIncomplete` and the complete CLI golden remains
configured skipped. Task 19B.3 retains normalization and effective
object/volume/material/layer orchestration; Task 19C retains config export;
Tasks 20A-20E retain consumer migration/removal; geometry, toolpaths, G-code,
and complete normalized KSR byte parity remain open. Task 19B.2 was released as
commit `d5a50bd64b7ebe048c80919edc6028b57f83fefa`; exact-SHA Tier 1 run
`29391775108` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.

### 2026-07-14 Resolve effective project configuration (Task 19B.3)

Task 19B.3 ports the effective project configuration boundary from fixed
OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. Typed normalization comes from
`PrintConfig.hpp:628-631` and `PrintConfig.cpp:8520-8740`; the cold two-apply
lifecycle and cardinality ownership come from
`PrintApply.cpp:1113-1194,1256-1283,1525-1768` and
`src/slic3r/GUI/PartPlate.cpp:3503-3510`. The remaining included boundaries are
`PrintApply.cpp:104-168,342-395,548-553,595-660,886-945,1662-1747`,
`PrintObject.cpp:3555-3709`, `PrintRegion.cpp:71-110`,
`Model.cpp:2512-2564`, `Print.cpp:451-546,588-591,3290-3301,3385-3388`, and
`Print.hpp:362-365,429-431`.

The typed resolver now validates project settings, runs one
`normalize_fdm_1` and the exact four source-ordered `normalize_fdm_2` calls,
propagates `_2` changed keys to their fixed owners, rematerializes the second
apply from a fresh normalized source, discards preliminary candidates, and
publishes only final candidates and views. Physical nozzle cardinality and
logical materialized-filament cardinality remain separate. Indexed vectors and
maps use their owning count; object/region/volume/layer selector validation and
support clamps use logical count; and the wipe selector satisfies both its
strict physical bound and logical output bound.

Raw layer ranges now receive the fixed sorted interval, gap/overlap, lookup,
source-index, `EPSILON`, and unconfigured-tail behavior. Printable instances
use the exact sorted transform grouping. The only geometry admitted here is
f32 Z-slab occupancy under composed print-object/source-volume transforms.
Each source object owns one candidate vector generated from the first sorted
group representative and shared by every group for that object. Candidates
preserve source-volume identity and apply process/object/volume/layer
precedence with project material explicitly `None`.

Bounded used-filament discovery composes supported effective-region roles, raw
model/volume/layer selectors, brim, support, raft, and explicit wipe sources at
their fixed timing and deduplication points. Negative and zero raft counts mean
no raft for brim participation; only `raft_layers > 0` activates raft support.
Strict logical-count boundaries govern role and support selectors, while the
wipe selector also retains its distinct physical assertion boundary.

Public project slicing now loads the 3MF and calls the resolver before
deliberately returning `ProjectSlicingIncomplete`. The complete CLI golden
remains configured skipped because geometry and G-code are not part of this
slice. Reference G-code is not used as a direct Task 19B.3 expectation; the
unchanged golden is only the final regression contract.

The frozen 51-entry implementation manifest
`23CCB91EC4BE509E43EDECEFD864B83B9D7CB2B5C4DA2F0FF08020F52A8D5DEB`
received independent whole `SPEC COMPLIANCE`, whole `CODE QUALITY`, and fresh
OpenCode `VERDICT: APPROVE` decisions with no findings. Current frozen-byte
verification passed 180/180 focused tests, 4625/4625 workspace tests with two
configured skips, the 22/22 dynamic audit with one configured skip, the 5/5
CLI contract with one configured golden skip, the 5/5 WASM contract, and the
real-project browser test. Rustfmt, warning-denying Clippy, native/WASM checks,
release WASM, wasm-bindgen, fixture hashes, forbidden scans, diff validation,
and the sub-400-LOC audit passed; the independent spec reviewer also ran a
broader 195/195 focused selection.

Task 19B.3 was released as commit
`99fb0beba0a48603cb7875591cf77d02c26fb525`; exact-SHA Tier 1 run
`29444150217` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.
Task 19C retains effective config-block serialization. Project material
documents, modifier-parent/painted-region geometry, painted/custom usage
sources, wipe sequencing, complete `FullPrintConfig` conversion outside this
bounded resolver, Tasks 20A-20E consumer migration/removal, geometry slicing,
toolpaths, G-code, metadata, post-processing, and final normalized KSR parity
remain deferred. The persistent full G-code parity goal remains open.

### 2026-07-16 Migrate typed profile inheritance and composition (Task 20A.1)

Task 20A.1 ports the in-memory profile subset from fixed OrcaSlicer v2.4.2
commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`. The named upstream boundaries
are `src/libslic3r/Preset.hpp:22-24,43-65`,
`src/libslic3r/Preset.cpp:491-504,1476-1494,1622-1703,3112-3140`, the
`full_fff_config(false, std::nullopt)` subset of
`src/libslic3r/PresetBundle.cpp:3884-4165`, and the concrete typed FFF owners
in `src/libslic3r/PrintConfig.hpp:695-914,916-1666`; the dynamic profile shell
at `src/libslic3r/PrintConfig.hpp:610-682` is replaced at the Rust API
boundary.

The approved local implementation performs an order-independent two-pass
streaming decode into sparse typed process, filament, or machine builders and
rejects wrong-kind, unknown, duplicate, malformed, and trailing input. It
inherits whole fields parent-first while preserving absence, resolves defaults
once, carries a child filament's root identity, and normalizes thumbnail fields
only after the final machine overlay. Deeper per-element nil inheritance and
variant-indexed diff mapping remain deferred.

Public merge output is the tagged by-value `MergedProfile`; typed composition
returns `ComposedProfile` with `ProjectSettings`, selected names, and positional
`ProfileGroupMetadata`. Compile-time sparse overlay is a zero-cost typed
operation with no runtime key/value registry. Exactly four filament
declaration groups opt into concrete append:
53 G-code, 48 print, four region, and 16 retract-override fields, plus direct
`pellet_flow_coefficient`, for 122 fields total. Selection order and empty
interior metadata slots are preserved.

The profile pair removes exactly 29 dynamic fingerprints, leaving the other
683 baseline rows byte-identical with no allowlist addition. Exactly two
obsolete retained-STL map-contract tests and the Orca source-citation-layout
inventory test are removed; typed behavioral coverage remains. The result is
not wired to `slice_project`, whose valid-project boundary remains
`ProjectSlicingIncomplete` after Task 19C export.

Profile management and compatibility-expression evaluation, remaining Task
20A consumers, Tasks 20B-20E, geometry, toolpaths, G-code, generated-by
metadata, post-processing, adapters, and complete normalized KSR parity remain
deferred. Task 20A.1 was released as commit
`e0c50564283744b3dd3388eeaa10f624a492ff1f`; exact-SHA Tier 1 run
`29488449752` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.

### 2026-07-16 Inherit typed filament variants by slot (Task 20A.2)

Task 20A.2 remains fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. The named upstream boundaries
are `PrintConfig.cpp:63-84,8375-8415,10209-10297`,
`Preset.cpp:231-278,922-945,1679-1697`,
`Config.hpp:558-580,624-665,812-837,921-931,1008-1016,1203-1218,1872-1879`,
and `libslic3r.h:52,306-310`. The Rust destination remains the concrete
filament option owners and typed profile resolver.

The approved implementation covers exactly the stride-one, no-extruder-ID
family of one `filament_extruder_variant` identity plus 36 data vectors. The
root derives cardinality from its resolved identity, applies the concrete
typed-default all-nil, empty, or no-reset class, and clears, truncates, or grows
each vector by its first value. Sparse descendants normalize only present
family fields, map the first exact identity match against the retained root
identity, ignore child-only variants, and never assign the identity as data.

Exactly 19 nullable float/percent vectors use local approximate comparison;
the other 17 vectors remain exact. A mapped nullable child `Nil` preserves the
source slot and equals only `Nil`. The `N == 0` path retains an empty root
identity and lets an implicit one-slot child reach source-length whole-copy
fallback; equality is checked before fallback, and fallback occurs before any
mapped-slot read.

The old filament diff scaffold and its exact eight dynamic findings are
removed, leaving 675 baseline findings and no allowlist addition. The task
does not wire profiles into project slicing, so valid projects still reach
`ProjectSlicingIncomplete` after the released Task 19C config writer.

Printer and process variants, stride-two behavior, profile-to-project wiring,
the remaining Task 20A work and Tasks 20B-20E, geometry, toolpaths, G-code,
generated-by metadata, post-processing, metadata byte parity, and complete
normalized KSR parity remain deferred. Task 20A.2 was released as commit
`4281e913b8eeaaeb6111cbefdf06f896f5c611aa`; exact-SHA Tier 1 run
`29520118127` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.

### 2026-07-16 Plan typed fixed project layers (Task 22A)

Task 22A remains fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. The named upstream boundaries are
`Slicing.hpp:25-38,44-52,66-85,98-114`,
`Slicing.cpp:24-43,62-70,106-146,228-304,713-866`,
`Model.cpp:1460-1499`, `PrintRegion.cpp:71-109`,
`PrintObject.cpp:3683-3686,3732-3833`,
`PrintObjectSlice.cpp:24-73,817-830`,
`PrintApply.cpp:104-167,1015-1054,1525-1621`, `Config.hpp:624-628`,
`libslic3r.h:46,48-60,300-310`, and
`Format/bbs_3mf.cpp:209-216,1896-1903,2087-2095,2824-2881`. The Rust
destination is private `ares-core::project_slice` ownership of
`SlicingParameters`, fixed profiles, planned print objects, and ordered
`PlannedLayer` records. Its prepared state owns the loaded project, one resolved
configuration, optional config block, and materialized planned records.

The approved supported subset adds typed presence for case-insensitive painted
layer-height profiles and object-owned range `layer_height`, rejecting either
before fixed planning. Raft, support, precise-Z, and resolved region ZAA are
also gated; typed true parameter-modifier `zaa_enabled` is conservatively
rejected until modifier geometry and region assignment are implemented.

Resolved objects retain stable source identity. Bounds use each source object's
first instance composed with every model-part volume transform, require every
transformed vertex to be finite, and include unreferenced vertices in max-Z.
The complete object-extruder source partition includes six
gated region feature selectors, print-wide brim, and object/volume fallback for
model parts and parameter modifiers. Sorted zero-based IDs feed Orca's
deliberate subtract-one/first-value nozzle-option indexing without
`filament_map`; bare range selectors enter only through occupied resolved
feature fallback.

Fixed profiles preserve first-layer insertion, regular-height coverage, strict
approximate compression, midpoint termination, and ordered pair-to-record
conversion. The shared generic budget allows exactly 100,000 `PlannedLayer`
records across all object/transform groups and rejects record 100,001.

The real committed 3MF alone prepares one print object with 460 records. The
first is `(id=0,height=0.2,print_z=0.2,slice_z=0.1)` and the final print-Z bits
are `0x4057000000000036`. Its Task 19C config block remains exactly 49,004 bytes
with SHA-256
`b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`.
Public slicing still returns `ProjectSlicingIncomplete` after planning and emits
no successful or placeholder G-code.

Variable/adaptive layers, modifier geometry, Clipper behavior, paths/G-code,
generated metadata, and successful full KSR parity remain explicitly deferred.
Task 22A was released as commit
`91fc19f1dbfc85d21431791d2d5acb78af818671`; exact-SHA Tier 1 run
`29543841835` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.

### 2026-07-17 Retain scaled raw mesh intersections (Task 22B)

Task 22B remains fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. The named upstream boundary is
the coordinate domain in `libslic3r.h` and `Point.hpp`, Bambu f32 mesh import,
winding, fresh centering, and component transforms in `Format/bbs_3mf.cpp`,
`TriangleMesh.cpp`, `Model.cpp`, and `Model.hpp`, object/volume identity and
slicing transforms in `ObjectID.hpp`, `Print.hpp`, `PrintObject.cpp`,
`PrintApply.cpp`, and `PrintObjectSlice.cpp`, and shared-edge/facet/multi-plane
raw intersection behavior in `TriangleMesh.cpp` and
`TriangleMeshSlicer.cpp`. The Rust destination is private `ares-core`
`geometry`, `mesh_slicer`, `project::load`, and `project_slice` ownership.

The approved subset materializes Bambu coordinates through f32, normalizes
winding once, omits empty geometry, compensates fresh-mesh centering, and uses
bounded iterative cycle preflight plus ancestry-free BFS component expansion.
It selects request-local scale only from resolved 3MF `printable_area`, checks
the half-open i64 boundary, constructs the raw center and centered slice
transform, retains one-based per-source-object volume ordinals, builds
shared-edge topology before intersection, and dispatches each face over ordered
Task 22A `slice_z` planes. Facet intersection retains strict f32 plane
comparisons and top-edge/on-plane ownership, directed endpoint provenance,
vertex-coordinate truncation, and interior `floor(value + 0.5)` conversion.
Three independent request-wide limits each accept exactly 1,000,000:
expanded-model occurrence/vertex/triangle units are claimed before scheduling
or materialization, dense layer slots are checked before allocation, and raw
lines are claimed before append. Nonempty layer ranges, distinct print-object
centering groups, shared mesh reuse, nonidentity shrink, and normalized edge
groups with more than two uses are gated without fallback.

The committed 3MF alone yields one model-part volume with 6,109 vertices,
12,234 triangles, 18,351 normalized shared edges, 460 layer slots, and 116,472
directed raw lines. The source-semantic and deterministic Ares-order encodings
have SHA-256
`a82b2d193c23c8ba499c7abd56e21cb9956f5444e9b51b1b261a7e9b67d26d21`
and `1a6e83f2d5f53b73fa7ba9cb6444909816276496361f7fb9f9305412d2045e79`.
The exact Task 19C config block remains 49,004 bytes with SHA-256
`b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`.

Distinct transform-group center rotation/decomposition, nonidentity XY/Z
shrink, full typed layer-range membership and slab filtering, importer-global
shared-mesh cache/reuse and compensation, absolute process-global `ObjectID`
values, and undefined pairing for normalized edge groups with more than two
uses remain deferred. Remaining `Line`/`Polyline`/`Polygon`/`ExPolygon` bounds,
area, containment, orientation, and non-clipping path-domain operations;
edge/vertex chaining, seed flags, open-chain joining and repair, loops, and path
ordering; and Clipper booleans, PolyTree/fill rules, union, offset,
simplification, closing, contour/hole construction, and polygon ordering are
also later slices.

Geometry consumption of `slicing_mode`, `slice_closing_radius`, `resolution`,
and XY compensation is deferred, as are negative/modifier booleans,
range/region assignment, painted segmentation, fuzzy skin, interlocking,
conical overhang, slicing-error repair, final cleanup, and reproduction of an
Orca TBB raw-append schedule. Surfaces, elephant-foot compensation, perimeters,
fill, brim, supports, toolpaths, motion, G-code assembly, generated metadata,
time estimation, and post-processing remain later slices. Embedded/external
presets, CLI overrides, UI behavior, any Ares-owned alternative pipeline, and
successful normalized KSR parity are likewise explicitly deferred. Supported
project slicing still returns `ProjectSlicingIncomplete`, but only after the
real private raw state is built. Whole specification, code-quality, and
default-model OpenCode implementation reviews are approved. Task 22B was
released as commit `455a0d12a9c6ac48f6e2796669b4300a6a6190a2`; exact-SHA
Tier 1 run `29610017653` is green across format, Ubuntu/Linux, WASM, macOS, and
Windows.

### 2026-07-17 Chain slices by triangle connectivity (Task 22C)

Task 22C remains fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. It ports integer ordered-point
storage and only the `TriangleMeshSlicer.cpp:1058-1161`
`chain_lines_by_triangle_connectivity` function at the first
`make_loops` call in `TriangleMeshSlicer.cpp:1383-1415`. The Rust destination
is private `ares-core` `geometry::polygon`, `mesh_slicer::chaining`, and
`project_slice::chained_intersections` ownership; the legacy f64 STL
segment/contour pipeline is not a fallback.

The implementation consumes each Task 22B raw layer once. Separate flat tagged
Edge/Vertex start indexes, raw face-order seeds, and original-index FIFO within
equal identities make the result deterministic. It connects only directed
last-B to candidate-A identities. Closed polygons omit the duplicated terminal
point without rotating or normalizing the path; open polylines preserve tagged
ends, ordered points, scaled f64 length, and initial unconsumed state. Project
object plans, volume order/ordinal/type, and every layer slot are preserved.

The committed 3MF alone yields 460 chained layer slots, 3,288 closed polygons,
zero open polylines, and 116,472 closed points. Exact face/seed-order and
normalized numeric encodings have SHA-256
`6654d9a95ef1bb024f986552b0e8c866ad55dcbe5de3af0cf9c34ff52372adbe`
and `7df1e0f90f90e4ff5ca6249c1ceb61e5e1aca74dbdb7b9153fffeff4cd165cdd`;
the latter is 2,190,993 bytes. The Task 19C config block remains 49,004 bytes
with SHA-256
`b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`.
Production traverses the chained state and still returns
`ProjectSlicingIncomplete`; no new Option or placeholder G-code is introduced.

Task 22D subsequently took the adjacent source-cited boundary in
`TriangleMeshSlicer.cpp:1163-1381,1428-1462` open-polyline length ordering,
exact identity joining and allowed reversal passes, nearest-endpoint search,
2 mm gap repair, and remaining loop-closing behavior; its implemented outcome
is recorded below. `slicing_mode`, hole ownership, Clipper,
negative/modifier booleans, regions, surfaces, perimeters, fill, supports,
toolpaths, G-code assembly, metadata, post-processing, and complete normalized
`ksr_fdmtest_v4` byte parity remained beyond Task 22C. The overall user-visible
G-code parity goal was still incomplete.

### 2026-07-17 Repair open slice polylines (Task 22D)

Task 22D remains fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. It ports `MultiPoint.hpp:172-187`
open length and signed area plus
`TriangleMeshSlicer.cpp:1163-1381,1428-1480` exact identity joining, nearest
endpoint gap repair, four-pass order, and final polygon-only return. The Rust
destination is private `ares-core` `mesh_slicer::chaining::{exact,gaps}` and
`project_slice::looped_intersections`; no legacy STL contour fallback is used.

The implemented order is exact(false), exact(true), gap(false), gap(true),
then residual-open discard. It preserves the signed Vertex/Edge key mapping
and zero collision, cached-versus-recomputed length rules, stale-end quirk,
strict 2 mm radius, conditional 30% closure heuristic, widened arithmetic,
deterministic original-index/endpoint-side ties, junction retention, and
source orientation gates. The 2 mm constant is scaled by request-local
`CoordinateScale` selected from resolved 3MF `printable_area`; it is not
`slice_closing_radius` and is not a new Option.

Project object plans, volume order/ordinal/type, layer slots, and polygon order
are consumed once into looped ownership. A high-level mutation oracle locks all
four passes, and a real project mesh produces and repairs a three-point open.
The KSR fixture already has zero opens, so it remains exactly 460 layers, 3,288
polygons, 116,472 points, and 2,190,993 encoded bytes. Face-order and normalized
hashes remain
`6654d9a95ef1bb024f986552b0e8c866ad55dcbe5de3af0cf9c34ff52372adbe`
and `7df1e0f90f90e4ff5ca6249c1ceb61e5e1aca74dbdb7b9153fffeff4cd165cdd`;
the 49,004-byte config block remains
`b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`.
Production still returns `ProjectSlicingIncomplete`, and full G-code parity is
not claimed.

Task 22E is implemented from `TriangleMeshSlicer.hpp:11-33`,
`PrintConfig.hpp:162-170,947`, `PrintConfig.cpp:307-312,6030-6042`,
`PrintObjectSlice.cpp:138-225`, and
`TriangleMeshSlicer.cpp:1483-1532,2003-2049`. The private
`mesh_slicer::slicing_mode` policy implements direct `Regular`, `EvenOdd`,
`Positive`, and `PositiveLargestContour` polygon behavior. The private
`project_slice::slicing_mode_intersections` adapter resolves external
`Regular`, `EvenOdd`, and `CloseHoles` from the 3MF object Option overlay,
preserves the original largest-contour intent for the later combination stage,
and applies the raw-stage mode in deterministic object, layer, and actual
source-volume order.

Spiral mode applies `PositiveLargestContour` only to model-part volumes above
the Option-derived bottom region; negative and modifier volumes keep the base
mode. Bottom-region membership uses the upstream layer-count-first rule and
strict widened `slice_z < bottom_shell_thickness - 1e-4` comparison. Validation
is limited to spiral-consumed Options. The KSR baseline remains exactly 460
layers, 3,288 polygons, 116,472 points, and the Task 22D face-order and
normalized hashes, while process, object-override, and spiral archive mutations
prove 3MF-only projection. Focused and full workspace tests, native and WASM
checks, the browser real-3MF test, code-quality review, default-model review,
and the independent six-dimensional implementation review pass.

### 2026-07-17 Port the safe Clipper 6 pre-closing union (Task 22F)

Task 22F remains fixed to OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. It ports the complete closed
Boolean/PolyTree dependency closure in
`deps_src/clipper/clipper.hpp:75-81,88-100,121-123,137,141-223,225-535` and
`deps_src/clipper/clipper.cpp:67-72,78-161,167-426,429-1614,1630-3340`, exact
full-range slopes from `Int128.hpp:234-277`, and the direct union, two-pass
overlap workaround, and tree ownership boundaries in
`ClipperUtils.cpp:169-204,303-350,634-668,737-740,812-814`. The Rust destination
is private `ares-core::geometry::{clipper,expolygon}` plus
`project_slice::pre_closing_unions`; ARD-0024 is accepted.

The safe typed-index engine preserves all closed operations, fill rules,
winding, intersection, horizontal, join, Paths, and PolyTree order. One
platform-neutral Rust sort-control rewrite reproduces the separately audited
MSVC STL 14.44 equal-key target. `union_ex` executes Paths first and a fresh
PolyTree second. Project ownership sorts by released `VolumeOrdinal`, projects
each retained slicing mode to its exact fill rule, preserves every layer slot,
and maps only external coordinate overflow to `InvalidInput`; there is no
fallback, native dependency, fixture branch, or output canonicalization.

The KSR pre-closing result is exactly 1,645,481 bytes with SHA-256
`209c6149c93994cc3ae6fa8e2f8f43dc9875b1b07b2320da9e67d8a2c43ab6e2`:
2,891 contours, 397 holes, and 99,260 points, with exact representative layer
matches and repeatability. Task 22F passes 50 focused tests, all full native and
WASM/browser gates, three whole-candidate implementation approvals, and the
independent six-dimensional review. Both committed KSR fixtures remain
unchanged. Production still returns `ProjectSlicingIncomplete`, so no
placeholder G-code or full normalized parity is claimed.

### 2026-07-17 Port closed ClipperOffset and project closing (Task 22G)

Task 22G is implemented from OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. It ports only closed
`ClipperOffset` from `clipper.hpp:138-139,144-167,538-575` and
`clipper.cpp:63-65,73-106,128-134,150-161,1000-1036,3345-3777`, the required
closed defaults and `offset_ex`/`offset2_ex` wrappers from
`ClipperUtils.hpp:17-34,326-355,389-393` and
`ClipperUtils.cpp:264-293,303-315,333-351,360-410,437-558,560-585`, and the exact
consumer in `TriangleMeshSlicer.hpp:20-46`,
`TriangleMeshSlicer.cpp:1738-1824,2003-2034`, and
`PrintObjectSlice.cpp:145-221`.

The safe Rust modules reuse Task 22F for closed Offset cleanup, preserve the
fixed joins, fill rules, path and ExPolygon ownership order, and execute
`offset2_ex` with one final PolyTree cleanup. The project stage resolves
`slice_closing_radius` only from the matching effective 3MF object, preserves
the upstream `f64` to `f32` to scaled `f64` to `f32` chain, and applies Miter
3.0. KSR resolves `0.049` at normal scale, producing `+49000/-49000` without a
fixture branch. Generic `closing*` overloads are not claimed as implemented and
remain outside this direct consumer slice; their `ClipperUtils.hpp:400-410` and
`ClipperUtils.cpp:592-610` ranges are context-only.

The KSR post-closing result is exactly 1,644,681 bytes with SHA-256
`29ffb501c54190dd4336cc1371fc5e480c5b87ac6a8184366bd072bf5cb90919`:
one object, one volume, 460 layers, 2,890 contours, 395 holes, and 99,212
points. Native and browser oracles are repeatable and byte-identical; both
fixtures remain unchanged. Task 22G passes all focused and full native,
WASM/browser, code-quality, default-model, and independent six-dimensional
gates. Production still returns `ProjectSlicingIncomplete`, so no placeholder
G-code or complete normalized parity is claimed.

### 2026-07-18 Port post-closing largest-contour selection (Task 22H)

Task 22H is implemented from OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. It ports
`TriangleMeshSlicer.cpp:2025-2037`, `ExPolygon.cpp:532-549`,
`ExPolygon.hpp:493-497`, and `Polygon.cpp:52-69`. The Rust stage preserves
serial signed-`f64` polygon area, strict first-positive maximum selection,
contour-only ranking, complete ExPolygon and ordered-hole ownership, and
post-closing per-layer `PositiveLargestContour` gating. It consumes only the
mode and spiral boundary already resolved from the complete 3MF; there is no
fixture, digest, count, or fixed-layer production branch.

The committed all-Regular project remains an exact geometry no-op at
1,644,681 H-checkpoint bytes, SHA-256
`e15967c36c0aa47a9a1a3fc31053587777359bedef796053022eaeb36ad49163`.
The primary complete three-Option 3MF mutation exercises 337 multi-ExPolygon
PLC layers and produces 427,465 bytes, SHA-256
`a0df3397e498306bfcade84b03721fe345d2f4b501e578a5b54df39faff44353`.
The independent threshold-21 mutation exercises 336 PLC layers from slot 21,
preserves multi-ExPolygon Regular slot 20, and produces 674,201 bytes,
SHA-256 `4b64a4e70bfceabf414572f6dbe13903245612908cbaf2d12985b6c1ed440214`.
Native and browser runs are repeatable, both fixtures remain unchanged, and
all focused/full, WASM, structural, whole-candidate, and independent
six-dimensional gates pass.

Production still returns `ProjectSlicingIncomplete`, so Task 22H emits no
placeholder or reference-derived G-code and does not complete normalized KSR
parity.

### 2026-07-18 Port resolution-driven simplification (Task 22I)

Task 22I is implemented from OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. It ports the 3MF-backed global
`resolution` mapping and mesh-slice consumer in `PrintConfig.hpp:1554-1562`,
`PrintConfig.cpp:5172-5179`, `PrintObjectSlice.cpp:166-177`,
`TriangleMeshSlicer.hpp:37-48`, and `TriangleMeshSlicer.cpp:2025-2044`, plus
the closed-loop Douglas-Peucker, three-union repair, and StrictlySimple Clipper
closure cited in the approved Task 22I specification.

`resolution <= 0.001` is an exact pre-traversal no-op. Any larger value selects
fixed `0.0025 mm`; the source's `f64` division then `f32` narrowing produces
exact scaled tolerances `2500.0` and `250.0` for Normal and LargeBed. The stage
runs after Task 22H for all four retained modes and simplifies each ExPolygon
independently through contour-first/source-order holes, a strict NonZero Paths
union, a non-strict NonZero Paths union, and a conditional non-strict PolyTree
union. The strict option defaults false for every predecessor caller, and no
fixture, digest, count, Option override, or fixed-layer production branch was
added.

The committed project produces 999,721 I-checkpoint bytes, SHA-256
`0dea485aea9f003db4dbadfd524e82cc2ad33327d3b447a7d985d57d82da72ef`,
with 2,890 contours, 395 holes, and 58,902 points. A complete `.001` mutation
is marker-only identity at 1,644,681 bytes, while `.0011` is byte-identical to
the committed enabled output. The three-Option archive produces 275,433 bytes,
SHA-256 `022cc958a38d5654e0a5fc4e2ca44d5e5ef068b7e57b271cb14151b11005343e`,
with 470 contours, 13 holes, and 16,245 points. The threshold-21 regression is
416,217 bytes, SHA-256
`185118681aad5de780a93d6f71f22f497dc7dc7dd82e038ec1feaf32b0f91294`,
with 569 contours, 127 holes, and 24,888 points. Native checks cover every
checkpoint above. For the committed, `.001`, `.0011`, and three-Option
archives, native and real Chromium checks reach exact EOF and agree on digest,
counts, ownership, and repeatability. Default WASM exports no Task 22 hooks and
the non-default feature exposes exactly the two Task 22I checkpoint hooks.

Production remains `ProjectSlicingIncomplete`, so normalized
`ksr_fdmtest_v4.gcode` equality and the persistent user goal are still
incomplete. Task 22J is next and must start with its own approved source-cited
slice of the adjacent `PrintObjectSlice.cpp` volume-to-region composition
boundary. Cross-volume negative/modifier composition, regions, surfaces,
perimeters, fill, supports, toolpaths, G-code assembly, metadata,
post-processing, and other `resolution` consumers remain deferred until their
own upstream-bounded slices.

### 2026-07-19 Port single-range volume region composition (Task 22J)

Task 22J is implemented from OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. It ports the volume-region data
and graph boundary in `Print.hpp:44-48,102-120,216-305,423-427` and
`Print.hpp:516-519,553-555,585-590`,
`PrintApply.cpp:342-405,542-553,582-592,699-724` and
`PrintApply.cpp:887-910,958-1057,1727-1739`, and
`PrintObject.cpp:3555-3710`, plus the exact
composition caller in `PrintObjectSlice.cpp:21,231-241,269-480,1149-1192` and
its Boolean, surface, and layer ownership dependencies cited in the approved
specification.

The Rust slice consumes only the loaded 3MF, resolved typed Options, selected
coordinate scale, and Task 22I geometry. It specializes the current one
implicit `[0, DBL_MAX)` range, separates stable physical occurrence identity
from source-order graph traversal, preserves a complete occurrence-keyed
sidecar, creates dense all-layer/all-region Internal surfaces, and implements
the source modifier, negative-volume, later-model, stable-order, and
same-region closing rules. Difference and Intersection rebuild ExPolygon
ownership through a fresh NonZero PolyTree. No fixture identity, target digest,
reference G-code, new Option default, native-only dependency, or alternate
geometry engine enters production.

The committed project exits at a repeatable 2,008,706-byte J checkpoint with
SHA-256 `2b474697f4afae95c9a55d709d8740d382a80b2969fc5118dc89e13c1906162d`:
one object, 460 layers, one occurrence and region, and 2,890 ExPolygons, 395
holes, and 58,902 points in each complete sidecar and retained stream. A real
3MF modifier/no-override pair starts from the same exact 478-byte I checkpoint
and produces distinct exact 1,054-byte and 698-byte J streams. Native and fresh
Chromium runs are repeatable, reach exact EOF, and agree on bytes and parsed
ownership; default WASM remains free of Task 22 exports. Both committed
fixtures remain unchanged.

Production remains `ProjectSlicingIncomplete`, so normalized
`ksr_fdmtest_v4.gcode` equality and the persistent user goal are still
incomplete. The next source-cited audit boundary is top-empty-layer removal in
`PrintObjectSlice.cpp:1194-1203`, `Layer.cpp:21-29`, and `Layer.hpp:169`.
`apply_conical_overhang` at `PrintObjectSlice.cpp:1206` and all later surface,
perimeter, fill, support, toolpath, G-code, metadata, and post-processing work
remain separate until their own approved slices.

### 2026-07-19 Port post-region top-empty-layer removal (Task 22K)

Task 22K ports OrcaSlicer v2.4.2
`PrintObjectSlice.cpp:1194-1203`, with emptiness semantics from
`Layer.cpp:21-29` and `SurfaceCollection.hpp:49-51`. For each post-region
object, Ares removes only the maximal suffix whose every region surface vector
is empty, truncates planned and region layers in lockstep, preserves surviving
IDs and interior empty layers, and leaves complete occurrence sidecars
unchanged. The slice introduces no Option or external input.

Native verification fixes the ten-object K checkpoint at 5,848 bytes /
`037b5e1b5aa9eb2f5c9c38f00a8d7a23768217fd7cc7ec13bb71f21d9edb3b07`
and the committed KSR K checkpoint at 2,008,706 bytes /
`c101e0f9ff863c7abe72cd1cb792fcd8e0074d8d6d2e77d3bb56c32eedba13be`.
KSR retains all 460 layers and its bytes after magic remain identical to Task
22J. Real top- and bottom-negative-slab 3MF projects prove opposite
`[nonempty, empty] -> 1` and `[empty, nonempty] -> 2` outcomes while retaining
both complete two-layer sidecars.

Fresh browser verification fixes independent J/K known-answer vectors at
433/385 bytes and reproduces the real-slab behavior using semantic-entry
digests
`36f49fc5ad0788dc63ce9e25111d5d758c67711137d368dc63eb76c5aee1e538`
and
`2001de693fbcc3781d733beebc8ace871cc42a2abe47865c51159192b9a94817`.
Two Chromium runs reach exact EOF and agree on KSR bytes, parsed ownership,
sidecars, and repeatability; default WASM exposes no Task 22 hook and the
feature bundle exposes exactly the two Task 22K checkpoint hooks.

Production still returns `ProjectSlicingIncomplete`, so normalized
`ksr_fdmtest_v4.gcode` equality and the persistent user goal remain
incomplete. Cancellation at `PrintObjectSlice.cpp:1204`,
`apply_conical_overhang` at `PrintObjectSlice.cpp:1206,1394-1509`, and all
later segmentation, compensation, surface, perimeter, fill, support, toolpath,
G-code, metadata, and post-processing behavior remain deferred. The next
source audit starts at the `1204-1206` caller sequence and must establish the
cancellation mapping and the complete conical-overhang/Option boundary before
implementation.

### 2026-07-19 Port conical-overhang region projection (Task 22L)

Task 22L ports the uncancelled success path at OrcaSlicer v2.4.2
`PrintObjectSlice.cpp:1204-1206,1394-1509`, with merged-layer semantics from
`Layer.cpp:117-136`. It consumes only effective object and ordered-region
Options resolved from the supplied 3MF: `make_overhang_printable_angle`,
`make_overhang_printable_hole_size`, nominal `layer_height`, and each region's
`make_overhang_printable`, `bottom_shell_layers`, `top_shell_layers`,
`sparse_infill_density`, and `wall_loops`. Validation runs for every object
before mutation, angle before hole size.

The implementation preserves the fixed f32 scale conversion points, reverse
layer-pair order, four-field merged footprints, protected small holes, Miter-3
offset, ordered region ownership, and fixed 10-coordinate safety-offset
difference. Affected surfaces are rebuilt as Internal with source-default
metadata; plan, sidecars, skipped layers, and unaffected metadata remain
unchanged. It adds no fixture identity, reference output, hardcoded shape,
fallback, or independent pipeline behavior.

Native verification covers 53 focused tests. The stepped disabled/enabled L
checkpoints are 490 bytes /
`0834c61cc48aece1afd52d060c5c2a58f7243124664ad0a7dd3f500d6735b790`
and 554 bytes /
`33038c51ffe6f41b0bdb8b921d6976f43b0c47f6f3be8ec3bee6cc5b9c7c2505`;
the ten-object transition is 5,848 bytes /
`fe46d60251dcf95590c71a3e55cafdf81e0fc6af5b3cb95d58d6c39ea693b264`.
The committed KSR disabled checkpoint is 2,008,706 bytes /
`7a71db2912970141adc436679621c25888c412e2010c44eccf1b49d7e8048b07`.
The native stepped disabled/enabled ZIP archives are 181,446 bytes /
`ee928a255109b491b0640da279b86d9282c573ec49a400e3cc4529eac915030e`
and 181,447 bytes /
`be286d7abb2bef8ab5e8b650657b114ea35c4dcff3a1463eba1a0dd278a89faa`;
the fflate ZIP archives are 190,380 bytes /
`c4c0ea05709a6fadd8b2d0d6d34dab1cad5420865c5993b58b9d8e91a8f73313`
and 190,381 bytes /
`130260c5c63846759aa66d25e68ff9bb07cf5aeec86ef7da9476c12761f3836d`.
Those physical ZIP bytes intentionally differ by encoder. Their shared
disabled/enabled semantic streams are 1,020,460 bytes /
`ade484830a6492b50c3233e51debf5eab1db7d3e3bbf81fa8cd72f10226ea9ef`
and 1,020,460 bytes /
`f61089d040d1edf002f1dedca66b433e4982e18b9ce69a6385aa42dbf4c780b9`;
the K/L checkpoints agree across encoders, reach exact EOF, and are repeatable.
Two fresh Chromium runs pass. Default WASM has no Task 22 hook and the feature
build exports exactly the two Task 22L checkpoint hooks.

Public slicing executes Task 22L and still returns
`ProjectSlicingIncomplete`, so normalized `ksr_fdmtest_v4.gcode` equality and
the persistent user goal remain incomplete. Caller and per-layer-pair
cancellation stay deferred until Ares has a public cancellation contract. The
next source audit starts at `PrintObjectSlice.cpp:1208-1225` for filament-count,
painted-facet, warning, and `apply_mm_segmentation` ownership. Fuzzy
segmentation, interlocking, `make_slices`, compensation, surface typing,
perimeters, fill, supports, toolpaths, G-code, metadata, and post-processing
remain later source-cited slices.

### 2026-07-20 Port single-region make_slices and elephant-foot compensation (Task 22M)

Task 22M ports the uncancelled single-region path from fixed OrcaSlicer v2.4.2
commit `8500fcdccaa10b5099ac20d252af3a7c560046f1` at
`PrintObjectSlice.cpp:1246-1276,1287-1292,1364-1387`, with island ordering from
`Layer.cpp:38-66` / `Layer.hpp:123-178`, the kernel from
`ElephantFootCompensation.cpp:20-28,233-447,465-532,544-644`, and the production
spatial index from `EdgeGrid.cpp:28-334` / `EdgeGrid.hpp:15-356`.

The project stage resolves elephant-foot, layer-count, raft, zero-only XY,
line-width, external-perimeter selector, nozzle, and planned-height values only
from the supplied 3MF. It preserves the source f32 ramp and scale conversion
sites; direct nozzle selection is not remapped through `filament_map`.
Validation and Flow resolution complete for all objects before mutation.
Nonzero XY and valid nonempty multi-region inputs fail with their exact feature
keys instead of silently taking an identity path.

Every retained layer runs `make_slices`. Enabled single-region layers preserve
ordered uncompensated `lslices`, run the full EdgeGrid-based variable-offset
kernel, replace compensated surfaces with default Internal metadata, and use
the fixed two-pass NonZero union. The independent fixed oracle uses a full
segment scan rather than the production grid; its one-pass-union mutant changes
`[left, nested, right]` to `[right, left, nested]` and is rejected. Plans,
sidecars, region ids, disabled surfaces, and unaffected layers remain exact.

The synthetic M aggregate is 10,351 bytes /
`c112246ff48b280eb803082749d74315e771d073b0407e45afde536e37fcf46d`.
The committed KSR L/M checkpoints are 2,008,706 /
`7a71db2912970141adc436679621c25888c412e2010c44eccf1b49d7e8048b07`
and 3,008,346 /
`91f6943a67fb7b42acbf6d4fbf9c98bc4bb91815df888ff5a99184bf53728d19`.
Rust 1.91 passes 81 Task 22M, 53 Task 22L, and all 509 Task 22 tests. Default
WASM exposes no Task 22 hook; the feature build exposes exactly the two M
checkpoint hooks. Two fresh Chromium runs each pass all five contracts,
including Option-only `fflate` archives and the complete KSR frame.

Public slicing still returns `ProjectSlicingIncomplete`; `ARES22M` is not
G-code, normalized KSR equality is not claimed, and the persistent goal remains
incomplete. Painted MMU/fuzzy segmentation, interlocking, nonzero XY,
multi-region compensation, surface classification, perimeters, fill, supports,
toolpaths, G-code, metadata, and post-processing remain separate source-cited
slices. For the active KSR path, the next fixed audit begins at
`PrintObject.cpp:452-560` (`PrintObject::make_perimeters` and its
`Layer::make_perimeters` call) after proving from the 3MF that the skipped
`PrintObjectSlice.cpp:1208-1243` gates are inactive.

### 2026-07-21 Port single-region perimeter inputs and Flow dispatch (Task 22N)

Task 22N ports the KSR-reached preparation seam from fixed OrcaSlicer v2.4.2
commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`: `PrintObject.cpp:453-558`,
`Layer.cpp:185-225`, `LayerRegion.cpp:21-58,82-142`, `PrintRegion.cpp:7-54`,
`PrintObject.cpp:3562-3565,3602-3661,3694-3700`,
`Flow.cpp:20-35,129-143,146-229`, `Flow.hpp:16-25,52-139`, and
`PerimeterGenerator.hpp:73-141`. The new crate-private
`project_slice::perimeters` boundary consumes post-M state and stops before
`PerimeterGenerator::process_classic()` or `process_arachne()`.

The stage validates every object before consuming state and creates one
optional record per planned layer. Records preserve object/occurrence/layer and
single-region identity, complete current/lower/upper/upper-same-region geometry
through owned indices, exact height and slice Z, four Flow values, spiral
state, occurrence-specific model rotation, and exhaustive Classic/Arachne
dispatch. Empty layers retain M state without a record, and zero-layer objects
retain an empty slot vector.

Flow resolution uses only effective 3MF Options. It preserves initial/role/
object/automatic width fallback, direct one-based selector normalization,
selected f32 nozzle percent math, absence of `filament_map`, fixed spacing and
volume narrowing, thick circular bridges, and every reachable nonthick
`with_cross_section` branch, including the canonical increase-else that rebuilds
width/spacing from old f32 area. The shared Task 22M constructor retains its
spacing-only contract for a valid `1e-30` width/height whose cached volume is
zero; Task 22N separately rejects nonpositive final role volumes before
consuming state. Invalid raw width/nozzle/height/bridge values fail in global
preflight, and a tiny positive `bridge_flow` whose thick or nonthick result
underflows is attributed to that Option. The fixed-release decrease-rounding
case follows Orca with assertions disabled: nozzle `100`, width `500%`, height
`2e-7`, and `bridge_flow=f64::MIN_POSITIVE` reaches the zero Flow and returns
`invalid Orca option bridge_flow` at the existing N boundary instead of a Rust
panic or WASM trap. Pure Flow, real in-memory 3MF/public Rust, and generated
real-archive Chromium regressions cover that error without changing the
25-object success aggregate. Spiral mode uses both bottom-shell gates. Aligned
rotation reads the matching occurrence's stored `(m00, m10)` column,
preserving signed zero, and spiral Arachne dispatches Classic only through the
fixed rule.

The independent tracked 25-object aggregate is 23,747 bytes /
`82ccfa1db8bcfea1c4689147561be8c7058c6fdefe0df9b7b8ad127e99487fd1`.
The committed KSR M/N checkpoints are 3,008,346 /
`91f6943a67fb7b42acbf6d4fbf9c98bc4bb91815df888ff5a99184bf53728d19`
and 7,083,888 /
`42e0053bffb3093a44597abd0a2b4e8b8c8c11d6f07003cb894399ad7dce3c6e`,
with all 460 planned-layer records populated. Real in-memory archives freeze
19 Flow Option pairs and six context pairs, including scoped selector
fallback, anti-`filament_map`, bridge, spiral, transform, and generator cases.
One additional non-family archive reducer changes only `bridge_flow` from `1`
to `1.0000001`, keeps M identical, produces two populated N slots, and freezes
the canonical increase-else bits. Native/browser regressions also preserve the
Task 22M volume-underflow predecessor and reject both tiny-positive bridge
modes at the N boundary.
Default WASM has no Task 22 export; the feature build has exactly the two N
exports. Strict N/M pre-fetch parser KATs and fresh optimized Chromium runs
cover exact EOF, the complete KSR frame, all Option families, repeatability,
and public incomplete behavior.
Final local gates pass 45 Task 22N, 82 Task 22M, all 555 Task 22, 5,191 full
`ares-core` tests with one configured skip, and 5,227 workspace tests with two
configured skips. Both fresh Chromium runs pass all nine contracts.

Public slicing still returns `ProjectSlicingIncomplete`; N is not G-code and
the persistent normalized KSR output goal remains incomplete. Classic/Arachne
process bodies, loop/extrusion output, precise spacing, dynamic top-one-wall,
overhang splitting, smaller external loops, perimeter gaps, multi-region
merging, fill, supports, toolpaths, G-code assembly, metadata, and
post-processing remain later source-cited slices. The earlier assumption that
`gap_fill_target=nowhere` suppresses perimeter gaps is retired: fixed
`PerimeterGenerator.cpp:1192,1325-1332,1573-1624` gates perimeter gaps on
`gap_infill_speed > 0`; KSR sets 250 and the reference has 470 Gap infill
feature blocks.

Task 22O must next port the complete KSR-reached Classic generator beginning at
`PerimeterGenerator.cpp:1144`, `PerimeterGenerator::process_classic()`, and
ending before `process_arachne()` at line 2093, plus only its reached helper
boundaries. Its exit gate must inventory and test the KSR precise-spacing,
top-one-wall, overhang, small-loop, and gap branches before claiming any
perimeter output.

### 2026-07-15 Serialize the exact effective config block (Task 19C)

Task 19C ports the Bambu effective-config export boundary from fixed
OrcaSlicer v2.4.2 commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`. The named upstream sources are
`Print.cpp:2618-2638`, `GCode.cpp:2030-2095,2461-2534,2637-2658,5591-5644`,
`Config.cpp:48-120,543-548,1715-1721`, the concrete value and nullable
serializers in `Config.hpp`, `PrintConfig.hpp:489-509`,
`PrintBase.hpp:517-518,558`, and `src/OrcaSlicer.cpp:6045-6060`. The Rust
destination is the crate-private `ares-core::options::config_export` module and
the existing project-slicing caller.

The approved implementation collects the canonical body only from
`ProjectConfigViews::full`: 132 printer, 352 process, 122 filament, and 44
project-runtime entries, for 650 sorted unique keys before nil omission. It
excludes preset metadata. Explicit serde semantic tags distinguish string
vectors, point groups, nullable vectors, and nil elements while remaining
transparent to existing JSON wire bytes; the new path does not use a JSON
round-trip, runtime registry, dynamic option map, or second 650-field access
table.

Empty and all-nil nullable vectors are omitted, mixed nullable vectors preserve
their `nil` positions, and empty non-nullable vectors remain present. The
writer applies the fixed nine banned keys, scaled flush-matrix multipliers,
typed filament-colour substitution, both selected and ordinary wipe-tower
coordinate forms, and the runtime-only first-layer nozzle/bed temperature tail.
The generic thumbnail canonicalizer now emits multi-value separators without
an added space; the config writer has no thumbnail-specific branch.

Only the exact case-sensitive `printer_model.starts_with("Bambu Lab")`
predicate enters the writer. The caller supplies plate index `0` and keeps the
atomically produced block in a private scratch buffer. Archive and final
materialization errors retain precedence; Bambu config-export errors are
observable before the incomplete boundary; non-Bambu projects skip the writer;
and every otherwise valid project still returns
`ProjectSlicingIncomplete`. No public partial-output or success API is added.

For `ksr_fdmtest_v4.project.3mf`, the block is exactly 49,004 bytes with
SHA-256
`b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`,
639 assignment lines, 637 unique keys, 15 omitted all-nil options, and five
retained empty non-nullable options. The two wipe-tower coordinate keys account
for the duplicate assignments. Production code contains no fixture name,
reference hash, reference G-code, or expected-size coupling.

The remaining executable source-path/line/symbol assertions were removed from
the project inventory test; its behavioral ownership, type/default,
projection, wire-shape, legacy-conversion, and fixture checks remain. The
39-path change set received independent whole spec-compliance, code-quality,
and default-model OpenCode `VERDICT: APPROVE` decisions. Pre-documentation
verification passed 29/29 config-export tests, 389/389 project tests, 4654/4654
workspace tests with two configured skips, 15/15 CLI tests with the complete
KSR golden as the sole CLI skip, warning-denying Clippy, native/WASM builds,
`wasm-bindgen 0.2.121`, a zero-vulnerability npm audit, and the real-project
browser test.

Geometry slicing, toolpaths, complete G-code assembly, generated-by metadata,
time estimation, post-processing, Tasks 20A-20E consumer migration and dynamic
shell removal, unsupported project material/painted/modifier sources,
selected-plate public plumbing, adapters, and final normalized KSR byte parity
remain deferred. Task 19C was released as commit
`656b32f987827b29d08010802ba03ef6ba822980`; exact-SHA Tier 1 run
`29457461048` is green across format, Ubuntu/Linux, WASM, macOS, and Windows.

### 2026-07-01 Consume prime tower brim width header slice

`prime_tower_brim_width` is now consumed through the source-cited Orca `PrintConfig.hpp:1581-1584`, `PrintConfig.cpp:6725-6734,7878-7879`, `GCode.cpp:5523-5574`, `Print.cpp:318-323,3150,3177-3179`, `GCode/WipeTower.cpp:1461-1468,3705-3707`, and `GCode/WipeTower2.cpp:1248-1256,2115-2119,2134-2136` boundary. The Rust destination is the existing `FilamentConfigExports` config-header snapshot plus `gcode_config_header.rs` serialization. The slice validates the already-registered scalar brim-width option with Orca's `-1` lower bound; exports explicit values and the `-1` auto sentinel as one `prime_tower_brim_width` config header line; carries legacy `wipe_tower_brim_width` input through the existing normalization path; rejects invalid values before G-code bytes are returned; and preserves omitted-value header output. Automatic brim-width calculation, wipe-tower placement, fake wipe-tower state, collision checks, cone/corner geometry, wall generation, mesh construction, purge-depth planning, rib-wall width recomputation, legacy `WipeTower` and `WipeTower2` runtime brim-width behavior, adjacent prime-tower interface options, flush-volume behavior, UI, CLI, WASM bindings, and Orca binary E2E wipe-tower parity remain deferred.

### 2026-07-01 Consume prime tower width header slice

`prime_tower_width` is now consumed through the source-cited Orca `PrintConfig.hpp:1577-1581`, `PrintConfig.cpp:6710-6716,7874-7875,8069-8074`, `GCode.cpp:5523-5574`, `Print.cpp:318-325,1002-1009,2838,3154,3168-3170`, `GCode/WipeTower.cpp:1461-1468,3707-3716`, and `GCode/WipeTower2.cpp:1248-1254,2024,2103,2227,2299` boundary. The Rust destination is the existing `FilamentConfigExports` config-header snapshot plus `gcode_config_header.rs` serialization. The slice validates the already-registered scalar width option with Orca's `2.0` lower bound; exports explicit values as one `prime_tower_width` config header line; carries legacy `wipe_tower_width` input through the existing normalization path; rejects invalid values before G-code bytes are returned; preserves omitted-value header output; and keeps obsolete `wipe_tower_per_color_wipe` ignored. Wipe-tower placement, fake wipe-tower state, collision checks, cone/corner geometry, purge-depth planning, rib-wall width recomputation, legacy `WipeTower` and `WipeTower2` runtime width state, adjacent prime-tower options, flush-volume behavior, UI, CLI, WASM bindings, and Orca binary E2E wipe-tower parity remain deferred.

### 2026-07-01 Consume wipe-tower coordinate header slice

`wipe_tower_x` and `wipe_tower_y` are now consumed through the source-cited Orca `PrintConfig.hpp:1576-1578`, `PrintConfig.cpp:6694-6708`, `GCode.cpp:5558-5574`, `Config.hpp:845-862,910-919`, `Print.cpp:267-269,1001-1004,2388,2545,2846`, `GCode/WipeTower.cpp:1463`, and `GCode/WipeTower2.cpp:1252` boundary. The Rust destination is the existing `FilamentConfigExports` config-header snapshot plus `gcode_config_header.rs` serialization. The slice validates the already-registered coordinate float vectors; exports the current single/default plate selected coordinate as one fixed three-decimal config header line per key; rejects invalid coordinate vectors before G-code bytes are returned; and preserves omitted-value header output. Orca's apparent generic duplicate `cfg.opt_serialize(key)` coordinate line, plate-index selection beyond the first/default value, part-plate coordinate logic, wipe-tower placement, fake wipe-tower state, collision checks, mesh/corner offsets, legacy `WipeTower` and `WipeTower2` runtime position state, `prime_tower_width`, `wipe_tower_per_color_wipe`, UI, CLI, WASM bindings, and Orca binary E2E wipe-tower parity remain deferred.

### 2026-07-01 Consume wipe-tower speed-spacing header slice

`wipe_tower_rotation_angle`, `wipe_tower_bridging`, `wipe_tower_extra_flow`, `wipe_tower_cone_angle`, `wipe_tower_extra_spacing`, and `wipe_tower_max_purge_speed` are now consumed through the source-cited Orca `PrintConfig.hpp:1581,1588-1589,1594-1596`, `PrintConfig.cpp:6718-6757,6872-6896`, `Print.cpp:267-269,337-339,353-355,1001-1004,2836-2844,3483`, `GCode/WipeTower.cpp:1467-1472,1489,2366-2368`, and `GCode/WipeTower2.cpp:1254-1262,1269,1927-1940,2018-2019,2084-2089,2545-2552` boundary. The Rust destination is the existing `FilamentConfigExports` config-header snapshot plus `gcode_config_header.rs` serialization. The slice validates the already-registered scalar float and percent options; exports explicit values as Orca-compatible config header lines; rejects invalid values before G-code bytes are returned; and preserves omitted-value header output. Wipe-tower rotation-aware placement, collision checks, cone base/corner construction, cone wall geometry, `WipeTower2` runtime state, purge-line width/depth changes, sparse wipe-tower grid bridging, max purge speed feedrate selection, sparse-layer speed fallback, legacy `WipeTower` bridging behavior, UI, CLI, WASM bindings, and Orca binary E2E wipe-tower parity remain deferred.

### 2026-07-01 Consume wipe-tower rib-wall header slice

`wipe_tower_wall_type`, `wipe_tower_extra_rib_length`, `wipe_tower_rib_width`, `wipe_tower_fillet_wall`, and `wipe_tower_filament` are now consumed through the source-cited Orca `PrintConfig.hpp:405-408,1597-1601`, `PrintConfig.cpp:558-563,6759-6808`, `Print.cpp:353-360,3363-3364,3474-3478`, `GCode/WipeTower.cpp:1364-1416,1478-1488`, and `GCode/WipeTower2.cpp:1262-1274` boundary. The Rust destination is the existing `FilamentConfigExports` config-header snapshot plus `gcode_config_header.rs` serialization. The slice validates the already-registered enum, float, bool, and int options; exports explicit values as Orca-compatible config header lines; rejects invalid values before G-code bytes are returned; and preserves omitted-value header output. Wipe-tower wall shape selection in path planning, rib-wall and cone geometry generation, fillet geometry, wipe-tower mesh construction, `WipeTower2` runtime state, purge generation, wipe-tower perimeter-filament selection, UI, CLI, WASM bindings, and Orca binary E2E wipe-tower parity remain deferred.

### 2026-07-01 Consume wipe-tower no-sparse-layers header slice

`wipe_tower_no_sparse_layers` is now consumed through the source-cited Orca `PrintConfig.hpp:1391`, `PrintConfig.cpp:5855-5861`, `GCode.cpp:1513-1519,1533-1538,1567-1569`, `GCode/WipeTower.cpp:1473`, and `Print.cpp:339` boundary. The Rust destination is the existing `FilamentConfigExports` config-header snapshot plus `gcode_config_header.rs` serialization. The slice validates the already-registered `ConfigOptionBool`, exports explicit values as `; wipe_tower_no_sparse_layers = 1` or `0`, rejects invalid values before G-code bytes are returned, and preserves omitted-value header output. Sparse wipe-tower layer suppression, wipe-tower Z adjustment, `m_no_sparse_layers` runtime state, wipe-tower reprocessing behavior, tool-change state, wipe-tower geometry, UI, CLI, WASM bindings, and Orca binary E2E wipe-tower parity remain deferred.

### 2026-07-01 Consume single-extruder priming runtime state slice

`single_extruder_multi_material_priming` is now consumed as typed runtime state through the source-cited Orca `PrintConfig.hpp:1390`, `PrintConfig.cpp:5863-5867`, and `GCode.cpp:2741-2745,2858-2861,3180-3185,3337-3339` boundary. The Rust destination is the existing `options/filament_change.rs` runtime snapshot plus machine-start placeholder rendering in `gcode_machine_start_placeholders.rs`. The slice validates the already-registered `ConfigOptionBool` default `false`, rejects invalid values before G-code bytes are returned, and preserves `[has_single_extruder_multi_material_priming] = 0` for omitted, false, and true raw values because Ares does not yet model Orca's Type2 wipe-tower and `has_wipe_tower` predicate. Wipe-tower presence, wipe-tower type runtime behavior, priming extrusion, initial-extruder changes, tool-change count/state, Tx emission/suppression, UI, CLI, WASM bindings, and Orca binary E2E wipe-tower priming parity remain deferred.

### 2026-07-01 Consume single-extruder filament-change runtime state slice

`single_extruder_multi_material` and `manual_filament_change` are now consumed as typed runtime state through the source-cited Orca `PrintConfig.hpp:1388-1389`, `PrintConfig.cpp:5807-5819`, `GCode.hpp:96,151`, and `GCode.cpp:1161-1162,1402,1463,7889-7890,7915,7929` boundary. The Rust destination is `options/filament_change.rs` plus validation-only consumption through `gcode_runtime_options::consume()` from `gcode::format_gcode()`. The slice validates the already-registered `ConfigOptionBool` defaults, rejects invalid values before G-code bytes are returned, and preserves current command output for valid values because Ares does not yet model tool-change state. Tool-change count/state, Tx emission/suppression, first-tool-change omission of `change_filament_gcode`, M600/PAUSE behavior, ramming, wipe tower, `single_extruder_multi_material_priming`, UI, CLI, WASM bindings, and Orca binary E2E filament-change parity remain deferred.

### 2026-07-01 Consume change filament G-code runtime state slice

`change_filament_gcode` is now consumed as typed runtime state through the source-cited Orca `PrintConfig.hpp:1392`, `PrintConfig.cpp:6516-6523,7880-7881,11118-11130`, and `GCode.cpp:7882-7894` boundary. The Rust destination is the existing `options/custom_gcode.rs` string accessor set plus validation-only consumption through `gcode_runtime_options::consume()` from `gcode::format_gcode()`. The slice validates the already-registered `ConfigOptionString`, preserves legacy `tool_change_gcode` renaming, rejects invalid values before G-code bytes are returned, and preserves current command output for valid values because Ares does not yet model tool-change insertion. Filament-change G-code insertion, tool-change state/count, next/previous extruder state, travel-point placeholders, `manual_filament_change` omission behavior, `single_extruder_multi_material`, full Orca placeholder parsing, UI, CLI, WASM bindings, and Orca binary E2E filament-change parity remain deferred.

### 2026-07-01 Consume timelapse type runtime state slice

`timelapse_type` is now consumed as typed runtime state through the source-cited Orca `PrintConfig.hpp:281-284,1615`, `PrintConfig.cpp:431-435,5728-5743`, `GCode.cpp:2129-2140,4514-4540,4982-5478`, and `GCodeProcessor.hpp:244-245` / `GCodeProcessor.cpp:6030-6058` boundary. The Rust destination is `options/timelapse_type.rs` plus validation-only consumption through `gcode_runtime_options::consume()` from `gcode::format_gcode()`. The slice parses Orca default/traditional/smooth enum strings plus current legacy-carried `"2"` as `Traditional`, rejects invalid values before G-code bytes are returned, and preserves current command output for valid values. Timelapse capture/video, `timelapse_warning_code`, warning result fields, warning emission, smooth prime-tower behavior beyond existing normalization, traditional support flags, wipe-tower timelapse behavior, `time_lapse_gcode` insertion changes, UI, CLI, WASM bindings, and Orca binary E2E timelapse parity remain deferred.

### 2026-06-30 Consume preheat runtime state slice

`preheat_time` and `preheat_steps` are now consumed as typed runtime state through the source-cited Orca `PrintConfig.hpp:1566-1567`, `PrintConfig.cpp:5757-5774`, `GCodeProcessor.hpp:831-832`, and `GCodeProcessor.cpp:1327-1328,1969-1974,2497-2498` boundary. The Rust destination is `options/preheat.rs` plus validation-only consumption from `gcode::format_gcode()`. The slice parses Orca defaults and ranges, rejects invalid preheat values before G-code bytes are returned, and preserves current command output for valid values. Orca backtrace buffering, `m_result.backtrace_enabled`, M104/M104.1 insertion, XL/single-extruder-multimaterial context detection, elapsed-time toolchange post-processing, UI, CLI, WASM bindings, and Orca binary E2E preheat parity remain deferred.

### 2026-06-30 Auto tree-support brim lower-bound proxy slice

`tree_support_auto_brim=true` now reaches Ares' current rectangular first-layer tree-support brim proxy through the source-cited Orca `PrintConfig.hpp:1015-1016`, `PrintConfig.cpp:6332-6343`, `Support/TreeSupport.hpp:435-439`, and `TreeSupport.cpp:1995-2013,2034,2146-2150` boundary. The Rust destination is `print_paths/support_tree_brim.rs`, the finalizer pass-through in `print_paths/generate.rs`, and focused `tree_support_brim` finalizer/G-code tests: zero-raft tree support with auto brim expands layer `0` closed rectangular `SupportMaterial` proxy paths by Orca's `MIN_BRANCH_RADIUS_FIRST_LAYER = 2.0` lower bound, even when `tree_support_brim_width = 0`, while `tree_support_auto_brim=false` keeps the manual `tree_support_brim_width` path. Non-tree support, raft-active support, non-first layers, open/non-rectangular paths, and non-support roles remain unchanged, with support-material metadata and G-code coordinate evidence preserved. Exact Orca dynamic auto width from tree node radius and distance-to-top state, branch-radius scaling, organic-radius derivation, arbitrary support polygons, full tree support generation, UI/CLI/WASM binding changes, and Orca binary E2E parity remain deferred.

### 2026-06-30 Tree support wall-count sheath proxy slice

`tree_support_wall_count > 0` now reaches Ares' current rectangular `SupportMaterial` support-base proxy through the source-cited Orca `PrintConfig.hpp:1014`, `PrintConfig.cpp:6390-6397`, `Support/SupportParameters.hpp:122-128`, `Support/SupportCommon.cpp:705-742,1774-1807`, `TreeSupport.cpp:1356,2674-2679`, and `TreeSupportCommon.hpp:84` boundary. The Rust destination is `print_paths/support_base_pattern_spacing.rs` plus the finalizer pass-through in `print_paths/generate.rs`: positive wall count emits a closed support-material sheath loop before inset support-base infill lines for closed rectangular support bodies, while zero or omitted wall count preserves the previous support-base output. The inset uses the current `support_material_width` proxy and preserves existing metadata, rectilinear-grid composition, zero-top-interface conversion, and G-code support-material coordinate emission. Multiple wall-count shells, exact Orca `Flow::scaled_spacing()` sheath offset, arbitrary support polygons, tree/organic support generation, support-layer storage, UI/CLI/WASM binding changes, and Orca binary E2E parity remain deferred.

### 2026-06-30 Zero-gap auto support-interface pattern proxy slice

`support_top_z_distance = 0` now reaches Ares' current closed rectangular `SupportMaterialInterface` print-path artifacts when `support_interface_pattern` is omitted or `auto`, through the source-cited Orca `PrintConfig.hpp:190-192,956`, `PrintConfig.cpp:333-340,5981-6000,6158-6176`, `Slicing.cpp:80-120`, and `Support/SupportParameters.hpp:129-138` boundary. The Rust destination is the existing `options/support_z_distance.rs` zero-gap helper, `print_paths/generate.rs`, `print_paths/support_interface_spacing.rs`, and focused support-Z/interface-pattern G-code tests: positive top interface layers plus zero top Z gap resolve the auto support-interface pattern to the same concentric rectangular proxy loops as explicit `concentric`, while positive/default top Z gap keeps `auto` and omitted pattern on the current single-family output and explicit `rectilinear` stays rectilinear. Bottom-gap behavior, support contact-layer topology, soluble-interface filament resolution, raft/base/bottom/tree/organic support interfaces, arbitrary polygon `FillConcentric` parity, UI/CLI/WASM binding changes, and Orca binary E2E parity remain deferred.

### 2026-06-30 Support interface concentric pattern proxy slice

`support_interface_pattern = concentric` now reaches Ares' current closed rectangular `SupportMaterialInterface` print-path artifacts through the source-cited Orca `PrintConfig.hpp:190-192`, `PrintConfig.cpp:333-340,6158-6176`, `Support/SupportParameters.hpp:103-138`, `Support/SupportCommon.cpp:1563-1592,1694-1733`, and `Fill/FillConcentric.hpp/cpp` boundary. The Rust destination remains `print_paths/support_interface_spacing.rs` plus focused support-interface pattern tests: eligible closed rectangular support-interface paths emit outermost-first nested closed rectangular loops using the existing `support_interface_spacing + support interface width` pitch, preserve source metadata, and intentionally ignore the extra `support_interface_loop_pattern` shell because the concentric output already includes the outer loop. Omitted, `auto`, `rectilinear`, `grid`, `rectilinear_interlaced`, `support_interface_top_layers = 0`, and `support_ironing = true` behavior stays within the previous scoped semantics. Full Orca `FillConcentric` polygon clipping, holes, path chaining, exact soluble-interface `auto` resolution, true `rectilinear_interlaced` alternation, raft/contact/base-interface classification, tree/organic support, arbitrary polygons, UI/CLI/WASM binding changes, and Orca binary E2E support parity remain deferred.

### 2026-06-30 Support interface rectilinear-interlaced pattern proxy slice

`support_interface_pattern = rectilinear_interlaced` now reaches Ares' current closed rectangular `SupportMaterialInterface` print-path artifacts through the source-cited Orca `PrintConfig.hpp:190-192`, `PrintConfig.cpp:333-340,6158-6176`, `Support/SupportParameters.hpp:103-161,277-278`, `Support/SupportCommon.cpp:1589-1592,1713-1718,1748-1754`, and `TreeSupport.cpp:1476,1554-1585,2426-2452` boundary. The Rust destination remains `print_paths/support_interface_spacing.rs` plus focused support-interface pattern tests, including `support_interface_pattern_interlaced.rs`: eligible rectangular support-interface paths now emit open rectilinear lines at a no-raft classic-support interlaced proxy angle, using `45deg` for even proxy layer ids and `-45deg` for odd proxy layer ids, reusing the existing support-interface pitch, metadata preservation, and loop-pattern outer shell behavior. Omitted, `auto`, `rectilinear`, `grid`, `concentric`, `support_interface_top_layers = 0`, and `support_ironing = true` behavior stays within the previous scoped semantics. Exact Orca `SupportLayer::interface_id()` sequencing, roof/floor grouping, tree/organic 0/90 interlacing, raft-derived interlaced angles, arbitrary polygon `FillRectilinear` behavior, UI/CLI/WASM binding changes, and Orca binary E2E parity remain deferred.

### 2026-06-30 Support style snug support-body proxy slice

`support_style = snug` now reaches Ares' current rectangular support-body proxy through the source-cited Orca `PrintConfig.hpp:179-180,975`, `PrintConfig.cpp:6204-6230`, `Support/SupportParameters.hpp:183-197`, `Support/SupportMaterial.cpp:620-626,637-732,845-858`, and `ClipperUtils.hpp:400-403` / `ClipperUtils.cpp:592-598` boundary. The Rust destination is `print_paths/support_style_snug.rs` plus the finalizer ordering in `print_paths/generate.rs`: resolved normal-support snug style merges same-layer closed rectangular `SupportMaterial` proxy bodies whose bounds overlap after Orca's `2.0` mm closing-radius inflation, preserving first-source metadata and running before support-object clipping and support-base spacing. Exact `smooth_outward`, arbitrary `ExPolygon` closing, holes, support-layer storage, generated interface contact behavior, full normal/tree support generation, UI/CLI/WASM binding changes, and Orca binary E2E parity remain deferred.

### 2026-06-29 Ordinary ironing solid-infill rotation consumption slice

Ordinary rectilinear ironing now consumes solid-infill rotation state through the source-cited Orca `PrintConfig.hpp:1096-1097,1145-1146`, `PrintConfig.cpp:2868-2880,3887-3899,4231-4246`, and `Fill.cpp:52-80,1598-1599` boundary. The Rust destination is `options/ironing_type.rs`, `print_paths/ironing.rs`, and ordinary-ironing pipeline tests. The slice parses `solid_infill_direction` with Orca's `45` degree default and simple `solid_infill_rotate_template` values for ordinary ironing, applies the selected solid-infill base before `ironing_angle`, preserves fixed-angle absolute behavior, and keeps support ironing unchanged. Advanced rotate-template metalanguage, full Orca `Fill` internals, binary E2E parity, UI, CLI, WASM bindings, support-ironing rotation behavior, and new options remain deferred.

### 2026-06-29 Support object skip-flush runtime consumption slice

`support_object_skip_flush` is now consumed as typed runtime state through the source-cited Orca `PrintConfig.hpp:1339`, `PrintConfig.cpp:2500-2501`, downstream `GCode.cpp:3265,5116`, and `Preset.cpp:1345` boundary. The Rust destination is `options/support_object_skip_flush.rs` plus `gcode_object_labels::ObjectLabelConfig::from_options()`. The slice parses the boolean option with Orca's default `false`, rejects invalid values during G-code configuration, and preserves current generated G-code for valid true/false values. Support-object skip-flush output behavior, object-specific filament instance labels, wipe/purge behavior, sequential and by-layer toolchange behavior, multi-object semantics, header emission, UI, CLI, WASM bindings, registry definitions, and generated output changes remain deferred.

### 2026-06-29 Support type runtime option slice

`support_type` is now consumed as typed runtime state through the source-cited Orca `PrintConfig.hpp:195-209,950`, `PrintConfig.cpp:342-348,5910-5925,7925-7929`, and representative downstream `PrintObject.cpp`, `Support/TreeSupport3D.cpp`, `Support/TreeSupport.cpp`, `Support/SupportMaterial.cpp`, and `Support/SupportParameters.hpp` boundary. The slice parses and validates the four canonical enum values plus existing legacy migrations, rejects invalid values before model loading, and keeps current Ares support geometry unchanged for every valid value. Normal/tree support generation, auto/manual enforcer routing, tree support branching, support invalidation graph, `is_tree_slim()`/support style composition, and Orca binary E2E support parity remain deferred.

### 2026-06-29 Support style runtime option slice

`support_style` is now consumed as typed runtime state through the source-cited Orca `PrintConfig.hpp:179-181,202-204,975`, `PrintConfig.cpp:322-331,6204-6230`, `Support/SupportParameters.hpp:183-195`, and representative downstream `Support/TreeSupport.cpp`, `Support/SupportMaterial.cpp`, `Support/SupportCommon.cpp`, and `Print.cpp` boundary. The slice parses Orca's default, normal, and tree style enum strings, resolves default and support-type-mismatched styles through the upstream fallback rules, exposes the `is_tree_slim()` relationship, rejects invalid values before model loading, and keeps current Ares support geometry unchanged for every valid value. Normal support generation, tree/organic support generation, support blockers/enforcers, support material fill behavior, support-style-specific geometry, and Orca binary E2E support parity remain deferred.

### 2026-06-29 Support placement runtime option slice

`support_object_xy_distance`, `support_object_first_layer_gap`, `support_on_build_plate_only`, `support_critical_regions_only`, and `support_remove_small_overhang` are now consumed as typed runtime state through the source-cited Orca `PrintConfig.hpp:953-955,995-996`, `PrintConfig.cpp:5927-5949,5959-5979,7906-7911,8074`, and representative downstream `PrintObject.cpp`, `Support/SupportParameters.hpp`, `Support/TreeSupportCommon.hpp`, `Support/TreeSupport.cpp`, and `Support/SupportMaterial.cpp` boundary. The slice parses Orca defaults and inclusive millimeter ranges, preserves existing legacy percentage-string removal for `support_object_xy_distance` and obsolete `support_remove_small_overhangs` ignoring, rejects invalid values before model loading, and keeps current Ares support geometry unchanged. Build-plate-only filtering, critical-region support generation, small-overhang removal, support/object XY collision offsets, first-layer support gaps, support invalidation graph parity, and Orca binary E2E support parity remain deferred.

### 2026-06-29 Support threshold runtime option slice

`independent_support_layer_height`, `support_threshold_angle`, and `support_threshold_overlap` are now consumed as typed runtime state through the source-cited Orca `PrintConfig.hpp:984,993-994,1618`, `PrintConfig.cpp:6232-6262`, `Config.hpp:1303-1310`, `Config.cpp:321-337`, and `Support/SupportMaterial.cpp:1390-1469,2136-2140,2317-2338` boundary. The slice parses the boolean independent support layer-height flag, validates the integer `0..=90` threshold angle, preserves `FloatOrPercent` absolute-vs-percent form for overlap while enforcing the raw stored-value `0..=100` range, rejects invalid values before model loading, and now uses `print_paths/support_threshold_contacts.rs` to synthesize normal-auto rectangular `SupportMaterialInterface` contact proxy paths below rectangular overhang contours when `enable_support` is true. The proxy uses Orca's positive-angle lower-layer-height formula, angle-zero external-perimeter-width overlap fallback, and expand-back contact restoration before existing support placement filters. Independent support layer-height synchronization, full support generation, non-rectangular threshold overhang detection, support layer projection/storage, tree/manual support threshold behavior, wipe-tower validation, UI, CLI, WASM bindings, and Orca binary E2E parity remain deferred.

### 2026-06-29 Support interface not-for-body runtime option slice

`support_interface_not_for_body` is now consumed as typed runtime state through the source-cited Orca `PrintConfig.hpp:961`, `PrintConfig.cpp:6036-6041`, and representative downstream `PrintObject.cpp`, `GCode/ToolOrdering.cpp`, `GCode.cpp`, and `Preset.cpp` boundary. The slice parses the boolean option with Orca's default `true`, rejects invalid values before model loading and during extrusion option construction, and now routes current Ares support-body extrusion output away from the fixed first interface selector when `support_filament` is Auto/0, `support_interface_filament` is Orca selector `1` (Ares selector `0`), and another selector exists. `SupportMaterialInterface` remains owned by `support_interface_filament`, while current support geometry-width resolution and print paths remain unchanged; generated G-code changes only through the resulting support-body E delta in that scoped selector case. Support invalidation, full ToolOrdering/T commands, flush-matrix and soluble/min-flush ranking, flush-into-support override behavior, layer/object-aware support/interface/object gating, UI, CLI, WASM bindings, registry definitions, and Orca binary E2E parity remain deferred.

### 2026-06-29 Raft layers support material activation slice

`raft_layers > 0` now participates in Ares' current support proxy activation through the source-cited Orca `PrintConfig.hpp:943`, `PrintConfig.cpp:5028-5037`, `Print.hpp:429-431`, and `Slicing.cpp:116-124,194-218` boundary. The Rust destination is the new focused `raft_layers` runtime parser plus the existing brim EFC outline raft gate and final `print_paths` support proxy filter: positive raft layers preserve current `SupportMaterial`, `SupportMaterialInterface`, and support-interface Ironing proxy artifacts even when `enable_support` is absent or false and `enforce_support_layers` is absent or zero, while omitted or zero raft layers keep disabled-support filtering. Real raft layer generation, raft contact/base/interface planning, other raft options, per-object `has_raft()` modeling, support material generation, support layer synchronization, UI, CLI, WASM bindings, and Orca binary E2E support parity remain deferred.

### 2026-06-29 Raft first-layer expansion proxy slice

`raft_first_layer_expansion` now reaches Ares' current raft-active first-layer rectangular support proxy through the source-cited Orca `PrintConfig.hpp:942`, `PrintConfig.cpp:5018-5026`, `Support/SupportCommon.cpp:286-349`, `Support/TreeSupport.cpp:1394-1400`, and `Support/TreeSupport.cpp:2352-2364` boundary. The Rust destination is the new focused parser in `options/raft.rs` plus the existing `print_paths/support_interface.rs` rectangular support proxy transform: positive `raft_layers` applies Orca's default `2.0` mm or the configured non-negative value to layer `0` closed rectangular `SupportMaterial` and `SupportMaterialInterface` proxy paths after `support_expansion` and before support base/interface spacing, support ironing, and G-code emission. Full raft layer generation, classic-support `inflate_factor_fine` compensation, no-raft normal-support first-layer expansion, raft density/contact/expansion behavior, arbitrary support polygons, support-layer storage, UI, CLI, WASM bindings, and Orca binary E2E support parity remain deferred.

### 2026-06-30 Raft first-layer density proxy slice

`raft_first_layer_density` now reaches Ares' current first-layer rectangular support proxy through the source-cited Orca `PrintConfig.hpp:941`, `PrintConfig.cpp:5008-5016`, `Support/SupportCommon.cpp:1496-1500,1778-1786`, and `Support/TreeSupport.cpp:1407-1410,1453-1457,1612-1616` boundary. The Rust destination is the new focused parser in `options/raft.rs` plus `print_paths/support_base_pattern_spacing.rs`: layer `0` closed rectangular `SupportMaterial` proxy paths use `support_material_width / (raft_first_layer_density / 100)` while non-first layers keep `support_base_pattern_spacing + support_material_width`, including rectilinear-grid composition and existing support/raft expansion ordering. Full raft/support generation, tree-support area ownership, sheath/perimeter generation, exact Orca fill engine behavior, arbitrary polygons, raft contact/base/interface planning, UI, CLI, WASM bindings, and Orca binary E2E parity remain deferred.

### 2026-06-30 Raft expansion proxy slice

`raft_expansion` now reaches Ares' current raft-active rectangular support proxy through the source-cited Orca `PrintConfig.hpp:940`, `PrintConfig.cpp:4999-5006`, `Support/SupportMaterial.cpp:1401,1575-1580`, and `Support/TreeSupport3D.cpp:1029-1030,1049-1056` boundary. The Rust destination is the new focused parser in `options/raft.rs` plus `print_paths/support_interface.rs`: positive `raft_layers` applies Orca's default `1.5` mm or the configured non-negative value to existing closed rectangular `SupportMaterial` and `SupportMaterialInterface` proxy paths whose `layer_id < raft_layers`, after `support_expansion` and before `raft_first_layer_expansion`, support spacing, support ironing, and G-code emission. Full Orca raft layer generation, raft contact/base/interface planning, arbitrary support polygon offsetting, tree-support element pruning beyond the rectangular proxy stream, support-layer storage/synchronization, UI, CLI, WASM bindings, and Orca binary E2E support parity remain deferred.

### 2026-06-30 Support object XY distance proxy slice

`support_object_xy_distance` now reaches Ares' current contour-aware rectangular support proxy through the source-cited Orca `PrintConfig.hpp:995`, `PrintConfig.cpp:5927-5936`, `Support/SupportParameters.hpp:84`, `Support/TreeSupportCommon.hpp:70-74`, `Support/SupportMaterial.cpp:445,2730,3103,3111-3190`, and `PrintObject.cpp:1034-1038` boundary. The Rust destination is `print_paths/generate.rs`, new `print_paths/support_object_xy_distance.rs`, and the contour-aware pipeline/test-support finalizer path: closed rectangular `SupportMaterial` and `SupportMaterialInterface` proxy paths on non-raft layers are clipped against same-layer rectangular object contours inflated by the configured XY distance after support/raft expansion and before support base/interface spacing, support ironing, and G-code emission, while no-context `finalize_print_paths(paths, options)` preserves prior behavior. Full Orca support-layer Z-overlap scanning, `no_overlap_xy_gap`, `sharp_tail_xy_gap`, ExPolygon offsetting/clipping, support contact generation, `support_object_first_layer_gap`, tree/organic support collision geometry, UI, CLI, WASM bindings, and Orca binary E2E support parity remain deferred.

### 2026-06-30 Support object first layer gap proxy slice

`support_object_first_layer_gap` now reaches Ares' current contour-aware rectangular support proxy through the source-cited Orca `PrintConfig.hpp:996`, `PrintConfig.cpp:5938-5947`, `Support/SupportParameters.hpp:84-85,240-241`, `Support/SupportCommon.cpp:286-288,376-388`, `Support/TreeSupportCommon.hpp:70-74`, and `Support/TreeSupport.cpp:2082-2084,2356-2361` boundary. The Rust destination remains `print_paths/generate.rs` and `print_paths/support_object_xy_distance.rs`: closed rectangular `SupportMaterial` and `SupportMaterialInterface` proxy paths on non-raft layer `0` are clipped against same-layer rectangular object contours inflated by the configured first-layer gap, while upper non-raft layers continue using `support_object_xy_distance`. True first support layer detection independent of `layer_id == 0`, Orca raft contact/base/interface generation and trimming parity, support-layer storage, tree/organic support collision geometry, ExPolygon clipping, UI, CLI, WASM bindings, and Orca binary E2E support parity remain deferred.

### 2026-06-30 Support on build plate only proxy slice

`support_on_build_plate_only` now reaches Ares' current rectangular support proxy through the source-cited Orca `PrintConfig.hpp:953`, `PrintConfig.cpp:5959-5964`, `Support/SupportMaterial.hpp:28-45`, and `Support/SupportMaterial.cpp:1299-1323,1388-1464,2511-2521,2593-2608,2685-2695` boundary. The Rust destination is `print_paths/generate.rs` plus `print_paths/support_on_build_plate_only.rs`: when contour context is available, build-plate-only mode keeps layer `0` and raft-layer rectangular support proxy anchors, then drops upper `SupportMaterial` and `SupportMaterialInterface` rectangles that do not overlap retained support rectangles on the immediately lower layer before support spacing, support ironing, and G-code emission. Full Orca `buildplate_covered` object-surface accumulation, support contact generation, projection-grid propagation, tree/organic routing, non-rectangular ExPolygon clipping, UI, CLI, WASM bindings, and Orca binary E2E parity remain deferred.

### 2026-06-30 Support remove small overhang proxy slice

`support_remove_small_overhang` now reaches Ares' current rectangular support proxy through the source-cited Orca `PrintConfig.hpp:955`, `PrintConfig.cpp:5974-5979`, `Support/SupportMaterial.cpp:2032-2050,2244-2305`, and `Support/TreeSupport.cpp:688-715,1003-1040` boundary. The Rust destination is `print_paths/generate.rs`, `print_paths/support_remove_small_overhang.rs`, and the generic `ExtrusionOptions` line-width accessor: with contour context available, enabled small-overhang removal drops closed rectangular `SupportMaterial` and `SupportMaterialInterface` proxy paths whose width or height is less than four times the resolved generic `line_width`, after support/object clipping and build-plate-only filtering but before support spacing, support ironing, and G-code emission. Full Orca overhang cluster formation, sharp-tail and cantilever exemptions, tree/organic routing, non-rectangular ExPolygon erosion, support contact/projection layers, UI, CLI, WASM bindings, and Orca binary E2E parity remain deferred.

### 2026-06-30 Support critical regions only proxy slice

`support_critical_regions_only` now reaches Ares' current tree(auto)-only rectangular support proxy through the source-cited Orca `PrintConfig.hpp:954`, `PrintConfig.cpp:5967-5973`, `PrintObject.cpp:1166-1172,1186-1191,1519-1528`, and `Support/TreeSupport.cpp:688,1086-1089` boundary. The Rust destination is `print_paths/generate.rs` plus `print_paths/support_critical_regions_only.rs`: with contour context available, enabled critical-regions-only mode removes ordinary closed rectangular `SupportMaterial` and `SupportMaterialInterface` proxy islands for `support_type = tree(auto)` after support/object clipping, build-plate-only filtering, and small-overhang pruning but before support spacing, support ironing, and G-code emission. Because current Ares rectangular proxy paths do not carry Orca cantilever or sharp-tail metadata, full Orca parity remains deferred to source-cited support contact generation, cantilever/sharp-tail detection, bottom-bridge invalidation, tree/organic routing, UI, CLI, WASM bindings, and Orca binary E2E support parity.

### 2026-06-29 Enforce support layers proxy activation slice

`enforce_support_layers > 0` now participates in Ares' current support proxy activation through the source-cited Orca `PrintConfig.hpp:948-958`, `PrintConfig.cpp:6013-6025`, `Print.hpp:429-431`, `Slicing.cpp:124-132`, `Support/SupportMaterial.hpp:28`, and `PrintConfig.cpp:10228-10233` boundary. The Rust destination remains the existing `support_z_distance` option parser plus the final `print_paths` support proxy filter: positive enforced support layers preserve current `SupportMaterial`, `SupportMaterialInterface`, and support-interface Ironing proxy artifacts even when `enable_support` is absent or false, while omitted or zero enforced layers keep disabled-support filtering. Real enforced support layer generation, support blockers/enforcers, raft-driven support material activation, per-object `has_support()` modeling, support layer synchronization, tree/organic support, UI, CLI, WASM bindings, and Orca binary E2E support parity remain deferred.

### 2026-06-29 Support enable runtime option slice

`enable_support` is now consumed as typed runtime state through the source-cited Orca `PrintConfig.hpp:948`, `PrintConfig.cpp:5903-5908`, `Slicing.cpp:124-130`, `Print.hpp:429-431`, `Support/SupportMaterial.hpp:28`, and representative downstream `Print.cpp` support-material stage boundary. The slice parses the boolean option with Orca's default `false`, rejects invalid values before model loading, preserves existing spiral-vase CLI validation reporting for `enable_support`, and removes current Ares support proxy print paths, support-interface ironing proxy paths, downstream moves/extrusions/speeds, diagnostics counts, and emitted G-code when `enable_support` is false or omitted. Existing support proxy behavior is preserved when `enable_support` is true. Real Orca support generation, support blockers/enforcers, `enforce_support_layers` support generation, raft support material, per-object support state, support-used propagation, tree/organic support generation, UI, CLI, WASM bindings, and Orca binary E2E support parity remain deferred.

### 2026-06-29 Classic tree support runtime option slice

`tree_support_branch_distance`, `tree_support_tip_diameter`, `tree_support_branch_diameter`, `tree_support_branch_angle`, `tree_support_branch_diameter_angle`, `tree_support_angle_slow`, `tree_support_wall_count`, `tree_support_auto_brim`, and `tree_support_brim_width` are now consumed as typed runtime options through the source-cited Orca `PrintConfig.hpp:1008-1016` and `PrintConfig.cpp:6264-6273,6286-6296,6298-6306,6332-6336,6338-6343,6345-6354,6356-6364,6366-6378,6390-6397` boundary. The slice parses Orca defaults and inclusive ranges, validates the raw millimeter, degree, boolean, and wall-count values before model loading in `run_slicing_pipeline()`. The `tree_support_auto_brim=false` plus `tree_support_brim_width` manual branch reaches Ares' first-layer rectangular tree-support proxy by expanding closed `SupportMaterial` rectangles before support base spacing, source-cited to `Support/TreeSupport.cpp:2034,2146-2150`; a later 2026-06-30 follow-up consumes `tree_support_auto_brim=true` through Orca's `2.0` mm first-layer lower-bound proxy while dynamic node-radius width remains deferred. Full classic tree support geometry, dynamic auto tree brim width, branch merging, wall-loop emission, support-material invalidation, scaled-coordinate conversion, radian conversion, and Orca binary E2E geometry parity remain deferred.

### 2026-06-29 Organic tree support runtime option slice

`tree_support_branch_distance_organic`, `tree_support_top_rate`, `tree_support_branch_diameter_organic`, and `tree_support_branch_angle_organic` are now consumed as typed runtime options through the source-cited Orca `PrintConfig.hpp:1034-1037`, `PrintConfig.cpp:6275-6284,6308-6316,6318-6330,6380-6388`, `Support/TreeSupportCommon.hpp:86-91`, and `PrintObject.cpp:1224-1232` boundary. The slice parses Orca defaults and inclusive ranges, validates the raw millimeter, percent, and degree values before model loading in `run_slicing_pipeline()`, and keeps current Ares support geometry unchanged. Organic/tree support generation, support-material invalidation, scaled-coordinate conversion, radian conversion, branch merging, support-tip clamping, wall-count behavior, preferred-angle behavior, brim handling, and Orca binary E2E geometry parity remain deferred.

### 2026-06-29 Wall maximum resolution/deviation runtime option slice

`wall_maximum_resolution` and `wall_maximum_deviation` are now consumed as typed perimeter options through the source-cited Orca `PrintConfig.hpp:1030-1031`, `PrintConfig.cpp:7076-7097`, `Arachne/WallToolPaths.hpp:19-20,35-36`, `Arachne/WallToolPaths.cpp:58-62,487-488,702-710`, and `PrintObject.cpp:1353-1364` boundary. The slice parses Orca defaults and inclusive millimeter ranges, exposes raw millimeter values on `PerimeterOptions`, and applies them to eligible Arachne closed-loop perimeter simplification in emitted perimeter/print-path G-code geometry. Arachne scaled-coordinate conversion, `Arachne::WallToolPathsParams` parity, exact variable-width wall path simplification, smallest-segment filtering parity, extrusion-area deviation behavior, variable-width lines, and Orca binary E2E geometry parity remain deferred.

### 2026-06-29 Minimum feature and bead-width runtime option slice

`min_feature_size`, `initial_layer_min_bead_width`, and `min_bead_width` are now consumed as typed perimeter options and by the current rectangular thin-wall runtime through the source-cited Orca `PrintConfig.hpp:1025-1027`, `PrintConfig.cpp:7051-7060,7099-7119`, `Arachne/WallToolPaths.hpp:17`, `Arachne/WallToolPaths.cpp:26-44,77-78,521-535`, `Arachne/BeadingStrategy/BeadingStrategyFactory.cpp:39-45`, `Arachne/BeadingStrategy/WideningBeadingStrategy.cpp:27-41,57-64`, and Classic `PerimeterGenerator.cpp:1247-1253` boundary. The slice parses Orca defaults and min-only percent semantics, accepts values above `100`, converts percentages from the minimum configured nozzle diameter, suppresses Arachne rectangular open thin-wall centerlines whose collapsed-axis thickness is below `min_feature_size`, preserves Classic centerlines under the same values, and carries the first-layer or later-layer minimum bead width into print paths, toolpath moves, extrusion moves, and E computation for surviving rectangular thin walls. Upstream Arachne `fill_outline_gaps` remains always on, but Ares still gates the temporary rectangular shell through `detect_thin_wall`; full Arachne, arbitrary thin-wall discovery, split/add thresholds, variable-width closed walls, skeletal trapezoidation, path simplification, adjacent wall-maximum-resolution/deviation behavior, and Orca binary E2E geometry parity remain deferred.

### 2026-06-29 Wall transition parameter runtime option slice

`wall_transition_length`, `wall_transition_filter_deviation`, `wall_transition_angle`, and `wall_distribution_count` are now consumed as typed perimeter options through the source-cited Orca `PrintConfig.hpp:1021-1024`, `PrintConfig.cpp:7003-7049`, `Config.hpp:954`, `Arachne/WallToolPaths.cpp:47-54,519-553`, `Arachne/BeadingStrategy/BeadingStrategy.cpp:29-33`, `Arachne/BeadingStrategy/BeadingStrategyFactory.cpp:33-45`, and `Arachne/SkeletalTrapezoidation.cpp:893-919,952-958` boundary. The slice parses Orca defaults and ranges, preserves min-only percentage semantics, converts transition length/filter deviation from the minimum configured nozzle diameter, exposes both raw and millimeter values on `PerimeterOptions`, and applies `wall_transition_filter_deviation` plus `wall_distribution_count` to Ares' current Arachne rectangular open thin-wall proxy by suppressing centerlines whose distributed over-width deviation exceeds the configured filter margin. Current default narrow thin-wall behavior, Classic thin-wall behavior, and closed perimeter loops remain preserved; positive bead-count transition length behavior, `wall_transition_angle` geometry, full `Arachne::WallToolPathsParams` parity, beading strategy parity, skeletal trapezoidation, wall split/add thresholds, variable-width lines, and Orca binary E2E geometry parity remain deferred.

### 2026-06-29 Wall generator runtime option slice

`wall_generator` is now consumed as a typed perimeter option through the source-cited Orca `PrintConfig.hpp:294-300,1020`, `PrintConfig.cpp:520-524,6989-7001`, `LayerRegion.cpp:121-124`, and `Arachne/WallToolPaths.cpp:30-65,482-553,684-708` boundary. The slice parses Orca's `classic` and `arachne` enum values, defaults to Orca's `arachne`, rejects invalid values at `SliceOptions::perimeter_options()`, and exposes the selected value on `PerimeterOptions`. Current Ares perimeter geometry remains the existing classic-style compatibility shell for both enum values; `process_arachne()`, variable-width walls, beading strategy, skeletal trapezoidation, transition filtering, min-feature/min-bead behavior, outline/path simplification, spiral-mode generator fallback, geometry differences between `classic` and `arachne`, and Orca binary E2E geometry parity remain deferred.

### 2026-06-29 Support bottom interface spacing bottom-only runtime slice

`support_bottom_interface_spacing` now reaches Ares' current bottom-only rectangular `SupportMaterialInterface` proxy through the source-cited Orca `PrintConfig.hpp:1019`, `PrintConfig.cpp:6115-6122`, `Support/SupportParameters.hpp:103-110,116-120,245-251`, and `Support/SupportCommon.cpp:1696-1741` boundary. The slice parses finite non-negative numeric values and numeric strings, defaults to Orca's `0.5` mm value, rejects invalid values before support-ironing preservation, and uses `support_bottom_interface_spacing + support interface width` only when `support_interface_top_layers = 0` and the resolved bottom interface layer count is positive. Default, top-enabled, and mixed generic interface rectangles remain owned by `support_interface_spacing` until Ares has source-cited contact-layer classification. Full support contact generation, bottom-contact/interface path metadata, mixed top/bottom density and spacing routing, exact upstream `bottom_interfaces = top_interfaces && support_interface_bottom_layers != 0` behavior, exact flow-spacing parity, bottom smoothing, bridge-flow bottom contacts, raft/base interface layers, tree/organic support, and Orca binary E2E geometry parity remain deferred.

### 2026-06-29 Make overhang printable hole-size runtime slice

`make_overhang_printable_hole_size` now reaches Ares' current rectangular `make_overhang_printable` contour transform through the source-cited Orca `PrintConfig.hpp:1032-1033`, `PrintConfig.cpp:4850-4877`, and `PrintObjectSlice.cpp:1397-1496` boundary. The slice keeps the default `0` value as the existing fill-all branch, recognizes lower-layer nested axis-aligned rectangles as Ares' current hole proxy, preserves nested rectangles whose area is strictly smaller than the configured hole-size when an upper rectangle fully covers them, and leaves equal-area, larger-area, non-covered, non-nested, and non-rectangular cases on the existing projection path. Full Orca ExPolygon hole topology, boolean difference and union, partially clipped conical material around holes, arbitrary polygons, multi-region ownership, and Orca binary E2E geometry parity remain deferred.

### 2026-06-29 Staggered inner seams perimeter runtime slice

`staggered_inner_seams` now reaches Ares' current rectangular internal perimeter runtime through the source-cited Orca `PrintConfig.hpp:944-945`, `PrintConfig.cpp:5375-5380`, and `GCode/SeamPlacer.cpp:1601-1628` boundary. The slice parses Orca's boolean option with default `false`, keeps disabled behavior unchanged, applies the option only after existing concrete `seam_position = back` placement, shifts internal rectangular seam starts by the per-loop shrink depth clamped to the internal line width, inserts an edge-interpolated split point on Ares' four-corner rectangular compatibility shell, and leaves external/overhang paths plus non-Back seam positions unchanged. Full Orca `SeamPlacer` candidate selection, nearest/aligned/random semantics, dense-polyline vertex-walk parity, concave-corner projection, scarf seams, Arachne/T-junction loop handling, object-level seam history, non-rectangular internal loops, support/bridge/wipe-tower/UI behavior, and Orca binary E2E seam parity remain deferred.

### 2026-06-29 Support interface loop pattern print-path runtime slice

`support_interface_loop_pattern` now reaches current closed rectangular `SupportMaterialInterface` print-path artifacts through the source-cited Orca `PrintConfig.hpp:962`, `PrintConfig.cpp:6055-6060`, `Support/SupportCommon.cpp:831-856,1426-1428,1625-1646`, and `LoopInterfaceProcessor` top-contact loop boundary. The slice parses Orca's boolean option with default `false`, validates it before support-ironing preservation, emits one closed support-interface contour before generated interface fill lines for eligible closed rectangles, preserves path metadata and extrusion role, keeps grid ordering as loop then interface-angle lines then base-angle lines, leaves support-ironing rectangles solid without an added loop, and remains a no-op after `support_interface_top_layers = 0` converts interface paths to `SupportMaterial`. Full `LoopInterfaceProcessor::generate` parity, top-contact-only classification, bottom/contact/base-interface separation, overhang trimming, flow-width centerline offsets, arbitrary expolygons/holes, tree/organic support, raft contacts, soluble-interface interactions, sheath interactions, and Orca binary E2E support parity remain deferred.

### 2026-06-29 Support interface pattern print-path runtime slice

`support_interface_pattern` now reaches current closed rectangular `SupportMaterialInterface` print-path artifacts through the source-cited Orca `PrintConfig.hpp:190-192`, `PrintConfig.cpp:333-340,6158-6176`, `Support/SupportParameters.hpp:103-138`, `Support/SupportCommon.cpp:1563-1592,1694-1733`, `TreeSupport.cpp:1497-1509,1554-1585`, and `Fill/FillRectilinear.hpp/cpp` grid-fill boundary. The slice parses Orca's enum strings with the `auto` default, validates the option before support-ironing preservation, keeps `auto`, `rectilinear`, `concentric`, and `rectilinear_interlaced` on the current single interface-angle family, maps `grid` to interface-angle lines followed by base-angle lines using `support_interface_spacing + support interface width`, preserves path metadata, and leaves non-target paths unchanged. Full support-region generation, exact `ipGrid` clipping/chaining, soluble-interface `auto` resolution, true concentric and rectilinear-interlaced generators, tree/raft/contact variants, arbitrary polygon clipping, and Orca binary E2E support parity remain deferred.

### 2026-06-28 Support base pattern print-path runtime slice

`support_base_pattern` now reaches current closed rectangular `SupportMaterial` print-path artifacts through the source-cited Orca `PrintConfig.hpp:172-177,969`, `PrintConfig.cpp:312-320,6133-6156,7931-7932`, `Support/SupportParameters.hpp:122-128`, `Support/SupportCommon.cpp:1430-1432`, and `Fill/FillRectilinear.hpp:153-163` boundary. The slice parses Orca's enum strings, keeps `default` and `rectilinear` on the existing single base-angle family, maps `rectilinear-grid` plus Ares' legacy-preserved `grid` value to base-angle and perpendicular line families, composes with `support_angle` and `support_base_pattern_spacing`, preserves path metadata, and lets zero top-interface layers consume the base pattern after role conversion. Honeycomb, lightning, hollow/tree/organic generators, density/sheath fill selection, arbitrary polygon clipping, raft/first-layer variants, path chaining, and Orca binary E2E support parity remain deferred.

### 2026-06-28 Support angle print-path runtime slice

`support_angle` now reaches current closed rectangular `SupportMaterial` and `SupportMaterialInterface` print-path artifacts through the source-cited Orca `PrintConfig.hpp:952`, `PrintConfig.cpp:5949-5957`, `Support/SupportParameters.hpp:103-104`, and `Support/SupportCommon.cpp` support-fill angle boundary. The slice parses Orca's finite `0..=359` degree option from numeric values and numeric strings, defaults to `0`, rotates base support lines by `support_angle`, rotates interface support lines by `support_angle + 90` for the current rectilinear/grid-compatible scaffold, clips rotated chords to rectangular bounds, preserves path metadata, keeps zero-interface-layer converted paths on the base angle, and leaves support-ironing interface rectangles solid before ironing. Full Orca support-area generation, arbitrary polygon fill/clipping, support pattern/style-specific angle selection, layer alternation, tree/organic support, first-layer/raft variants, path chaining, and Orca binary E2E geometry parity remain deferred.

### 2026-06-28 Support base pattern spacing print-path runtime slice

`support_base_pattern_spacing` now reaches current closed rectangular `SupportMaterial` print-path artifacts through the source-cited Orca `PrintConfig.hpp`, `PrintConfig.cpp`, `Support/SupportParameters.hpp`, and `Support/SupportCommon.cpp` boundary. The slice parses finite non-negative numeric values and numeric strings, defaults to Orca's `2.5` mm value, converts eligible solid base support rectangles into open support-material lines using `support_base_pattern_spacing + support material extrusion width`, preserves path metadata, runs after top-layer role rewriting and support expansion, and leaves remaining support-interface paths to the existing interface spacing/ironing passes. The support-angle runtime slice now owns base line direction. Full Orca support-area generation, exact support material flow spacing and density parity, support base pattern/style behavior, arbitrary polygon clipping, path chaining, and Orca binary E2E geometry parity remain deferred.

### 2026-06-28 Support interface spacing print-path runtime slice

`support_interface_spacing` now reaches current closed rectangular `SupportMaterialInterface` print-path artifacts through the source-cited Orca `PrintConfig.hpp:967`, `PrintConfig.cpp:6104-6112`, and `Support/SupportParameters.hpp:106-107` boundary. The slice parses finite non-negative numeric values and numeric strings, defaults to Orca's `0.5` mm value, converts eligible solid interface rectangles into open interface lines using `support_interface_spacing + support interface extrusion width`, preserves path metadata, runs after top-layer role rewriting and support expansion, and keeps support-ironing interfaces solid. The support-angle runtime slice now owns interface line direction as `support_angle + 90`, making the default interface direction vertical. Full Orca support-area generation, exact flow spacing and density parity, support interface pattern selection, arbitrary polygon clipping, and Orca binary E2E geometry parity remain deferred.

### 2026-06-28 Support expansion print-path runtime slice

`support_expansion` now reaches concrete Ares support print-path artifacts through the source-cited Orca `PrintConfig.hpp:972-973`, `PrintConfig.cpp:6187-6193`, and `Support/SupportMaterial.cpp:1396,1517` boundary. The slice parses finite numeric values and numeric strings in millimeters, expands or shrinks current closed rectangular `SupportMaterial` and `SupportMaterialInterface` paths after support-interface top-layer role rewriting and before support ironing, drops collapsed shrunk rectangles, and preserves retained path metadata so support ironing duplicates inherit the expanded support-interface geometry. Full Orca support-area generation from overhang polygons, arbitrary polygon offsetting, tree support, support pattern spacing/interface pattern behavior, and Orca binary E2E geometry parity remain deferred.

### 2026-06-28 Support ironing G-code label runtime slice

`support_ironing` support-derived Ironing paths now emit Ares diagnostic and move-comment labels as `support_ironing` through the source-cited Orca `PrintConfig.hpp:997-1000`, `PrintConfig.cpp:6406-6446`, `Support/SupportParameters.hpp:58-61`, `Support/SupportCommon.cpp:1877-1907`, and `GCode.cpp:6110-6140` boundary. The slice keeps the internal role as `PrintPathRole::Ironing`, preserves support-interface extrusion metadata for `support_ironing_flow`, keeps ordinary Ironing labeled as `ironing`, and maps Orca's `support ironing` support-fill label into Ares' snake_case diagnostic token style. Full support contact-layer polygon generation, support transition label parity, raw space-containing label text, role-change custom G-code semantics, multi-extruder support ownership, and Orca binary E2E geometry parity remain deferred.

### 2026-06-27 LockedZag sparse infill runtime slice

`sparse_infill_pattern = lockedzag` now reaches concrete Ares sparse infill geometry, print paths, extrusion moves, speed moves, and G-code through the source-cited Orca `PrintConfig.hpp:87-96,1126-1131`, `PrintConfig.cpp:2928-2938,3898-3962`, `Fill/FillBase.cpp:40-76`, `Fill.cpp:877-881,987-1002,1298-1312`, `Fill/FillRectilinear.cpp:2761-2765,3390-3396,3866-3943`, and `Fill/FillRectilinear.hpp:210-224` boundary. The slice accepts the Orca sparse `lockedzag` pattern, routes it through Ares' current sparse scanline compatibility shell, preserves consistent layer alignment, layer-id `infill_shift_step`, symmetric Y-axis mirroring, alternating sparse segment direction, and Orca's single-line branch for `fill_multiline`. Full `FillLockedZag::fill_surface_locked_zag` skin/skeleton polygon splitting, skin/skeleton densities, lock/skin depths, multi-width skin/skeleton flows, exact offset/intersection parity, path chaining, link filtering, multi-region ownership, and Orca binary E2E geometry parity remain deferred.

### 2026-06-27 Fill multiline runtime slice

`fill_multiline` now reaches concrete Ares sparse infill geometry, print-path, extrusion, speed, and G-code coordinates through the source-cited Orca `PrintConfig.hpp:1135`, `PrintConfig.cpp:2906-2913`, `Fill/Fill.cpp:925-926`, `Fill/FillRectilinear.cpp:2996-3021,3390-3396`, and `FillBase.cpp:2712-2762` boundary. The slice parses Orca's `1..=10` integer option with default `1`, applies Orca-style source-spacing multiplication plus sparse line-width neighbor expansion to sparse `Rectilinear`/`AlignedRectilinear`/`Line`/`Grid`, and keeps solid, bottom/top surface, internal-bridge, ZigZag, and CrossZag behavior single-line for this slice. Full `ClipperOffset` round-end geometry, contracted-surface intersection/re-clipping, path chaining, CrossHatch-specific multiline, multi-region/object ownership, and Orca binary E2E geometry parity remain deferred.

### 2026-06-27 Ironing angle runtime slice

`ironing_angle`, legacy `ironing_direction`, and `ironing_angle_fixed` now reach concrete Ares ordinary Ironing print-path and G-code coordinates through the source-cited Orca `PrintConfig.hpp:1137-1146`, `PrintConfig.cpp:4231-4250`, `PrintConfig.cpp:8005-8007`, `Fill/Fill.cpp:1598-1599`, and `FillBase.cpp:306-311` boundary. The slice parses Orca's finite `0..=359` degree angle plus boolean fixed-angle flag after existing legacy normalization, clips rectangular rectilinear ordinary Ironing scanlines to the current inset rectangle at the selected angle, preserves default horizontal geometry at `0` degrees, alternates non-fixed odd layers by `+90` degrees, suppresses that alternation when fixed, keeps concentric ordinary Ironing independent from angle, and leaves support Ironing paths unchanged. Full Orca `calculate_infill_rotation_angle(...)` parity using solid infill direction/template options, arbitrary polygon/hole clipping, `FillRectilinear` chaining/path ordering/link generation, non-rectangular island handling, support-interface ironing angle behavior, multi-extruder region ownership, and Orca binary E2E geometry parity remain deferred.

### 2026-06-27 Support ironing pattern runtime slice

`support_ironing_pattern` now reaches concrete Ares support-interface Ironing print-path and G-code coordinates through the source-cited Orca `PrintConfig.hpp:997-1000`, `PrintConfig.cpp:6406-6446`, `Support/SupportParameters.hpp:58-61`, and `Support/SupportCommon.cpp:1877-1907` boundary. The slice parses Orca's `rectilinear` default and accepted `concentric` value after existing legacy `zig-zag` normalization, preserves current rectilinear open-line support Ironing behavior, and generates closed concentric rectangular support Ironing loops for Ares' current closed rectangular support-interface compatibility shell while preserving support-interface extrusion metadata and `support_ironing_flow` scaling. Full support contact-layer polygon discovery, `polys_to_iron` clipping, exact `FillConcentric` / `FillRectilinear` parity, support ironing angle selection, non-rectangular clipping, holes, island chaining, path ordering, `link_max_length`, multi-extruder support ownership, and Orca binary E2E geometry parity remain deferred.

### 2026-06-27 Ironing pattern runtime slice

`ironing_pattern` now reaches concrete Ares ordinary Ironing print-path and G-code coordinates through the source-cited Orca `PrintConfig.hpp:1137-1151`, `PrintConfig.cpp:4178-4188`, and `Fill/Fill.cpp:1507-1718` boundary. The slice parses Orca's `rectilinear` default and accepted `concentric` value, preserves existing rectilinear open-line behavior, and generates closed concentric rectangular Ironing loops inside Ares' current ordinary Ironing inset compatibility shell for closed rectangular top/solid paths with positive spacing. Full Orca `Layer::make_ironing` polygon fill parity, non-rectangular concentric clipping, holes, path chaining, angle/fixed-angle/direction behavior, ironing expansion, support-specific `support_ironing_pattern`, multi-extruder grouping, and Orca binary E2E geometry parity remain deferred.

### 2026-06-27 Support ironing spacing runtime slice

`support_ironing_spacing` now reaches concrete Ares support-interface ironing print-path and G-code coordinates through the source-cited Orca `PrintConfig.hpp:997-1000`, `PrintConfig.cpp:6406-6446`, `Support/SupportParameters.hpp:58-61`, `Support/SupportCommon.cpp:1877-1907`, and `GCode.cpp:6110-6140` boundary. The slice parses Orca's `0.1` mm default and `0.0..=1.0` range, keeps zero spacing as the existing single support-interface duplicate compatibility behavior, and generates open rectilinear support ironing lines for closed rectangular support-interface paths while preserving support-interface extrusion metadata and `support_ironing_flow` scaling. Full support contact-layer polygon discovery, `support_ironing_pattern`, support ironing angle selection, non-rectangular clipping, support fill reordering/chaining parity, multi-extruder support ownership, and Orca binary E2E geometry parity remain deferred.

### 2026-06-27 Ironing spacing runtime slice

`ironing_spacing` and first-value `filament_ironing_spacing` now reach concrete Ares ordinary Ironing print-path and G-code coordinates through the source-cited Orca `PrintConfig.hpp:1137-1151`, `PrintConfig.cpp:3385-3395`, `PrintConfig.cpp:4202-4210`, and `Fill/Fill.cpp:1500-1725,1511-1512,1584-1588,1693-1700` boundary. The slice parses Orca's ordinary `0.1` mm default plus nullable filament override fallback, treats first-value `null` or `nil` as fallback to ordinary spacing, and generates open rectilinear Ironing lines inside Ares' current inset closed-rectangle ordinary Ironing compatibility shell. Full Orca `Layer::make_ironing` polygon fill generation, spacing-driven extrusion-height/flow parity, `ironing_pattern`, angle/fixed-angle/direction behavior, support-specific `support_ironing_spacing`, multi-extruder current-filament selection, non-rectangular polygon clipping, and Orca binary E2E geometry parity remain deferred.

### 2026-06-29 Fuzzy skin coherent noise runtime slice

`fuzzy_skin_noise_type = perlin|billow|ridgedmulti|voronoi`, `fuzzy_skin_scale`, `fuzzy_skin_octaves`, and `fuzzy_skin_persistence` now reach concrete Ares closed-polyline fuzzy skin runtime through the source-cited Orca `PrintConfig.hpp:65-72,1112,1114-1116`, `PrintConfig.cpp:202-210,3491-3543`, and `Feature/FuzzySkin/FuzzySkin.cpp:41-64,296-333,434-441` boundary. The slice accepts all registered fuzzy noise enum values, threads `LayerContours::print_z()` into coherent-noise sampling as Orca's `slice_z` coordinate, and implements deterministic no-dependency Perlin/Billow/RidgedMulti/Voronoi compatibility shells over Ares' current external and `allwalls` internal closed-polyline fuzzy-skin ownership. Exact libnoise algorithm and seed parity, Arachne extrusion/combined width modes, painted fuzzy regions, fuzzy hole topology, arbitrary polygon clipping/splitting, multi-region fuzzy-effect merging, and full Orca binary E2E geometry parity remain deferred.

### 2026-06-27 Fuzzy skin ripple runtime slice

`fuzzy_skin_noise_type = ripple`, `fuzzy_skin_ripples_per_layer`, `fuzzy_skin_ripple_offset`, and `fuzzy_skin_layers_between_ripple_offset` now reach concrete Ares external-perimeter, print-path, move, extrusion, speed, and G-code coordinate behavior through the source-cited Orca `PrintConfig.hpp:65-72,1112,1117-1119`, `PrintConfig.cpp:3491-3515,3545-3576`, and `Feature/FuzzySkin/FuzzySkin.cpp:70-220,296-300,434-441,507-513` boundary. The slice parses Orca's ripple options and applies the closed-polyline arc-length sine-wave displacement in the existing external fuzzy-skin compatibility shell. `allwalls` internal rectangular wall-loop ownership is now consumed for Ares' generated internal perimeter loops; Arachne extrusion/combined width modes, hole ownership and broader fuzzy ownership, painted fuzzy regions, and full Orca binary E2E geometry parity remain deferred.

### 2026-06-27 Filament ironing inset runtime slice

`filament_ironing_inset` now reaches concrete Ares ordinary Ironing print-path and G-code coordinates through the source-cited Orca `PrintConfig.hpp:1147-1151`, `PrintConfig.cpp:3397-3407`, `PrintConfig.cpp:4212-4220`, and `Fill/Fill.cpp:1584-1591,1687-1689` boundary. The slice parses Orca's nullable millimeter override, uses Ares' current first single-active-filament value, falls back to ordinary `ironing_inset` when missing or `nil`, preserves the selected `0` value as half the first nozzle diameter, and routes the effective value through the existing ordinary Ironing line and rectangular-loop inset behavior. Multi-extruder current-filament selection, `filament_ironing_spacing`, full Orca `Layer::make_ironing` polygon fill generation, ironing pattern/spacing/angle/expansion behavior, support ironing inset behavior, non-rectangular polygon offsetting, and Orca binary E2E geometry parity remain deferred.

### 2026-06-27 Ironing inset runtime slice

`ironing_inset` now reaches concrete Ares ordinary Ironing print-path and G-code coordinates through the source-cited Orca `PrintConfig.hpp:1142`, `PrintConfig.cpp:4212-4220`, and `Fill/Fill.cpp:1501-1720,1687-1689` boundary. The slice parses Orca's millimeter `0.0..=100.0` option, resolves the `0` default to half the first nozzle diameter, shortens two-point ordinary Ironing duplicates by the effective inset, insets closed four-corner rectangular ordinary Ironing loops, duplicates unordered/crossed and degenerate non-eligible shapes unchanged, and drops collapsed ordinary Ironing duplicates. Full Orca `Layer::make_ironing` polygon fill generation, `ironing_pattern`, `ironing_spacing`, ironing angle/fixed-angle/expansion behavior, filament-specific ironing overrides, non-rectangular polygon offsetting, region grouping, support ironing inset behavior, and Orca binary E2E geometry parity remain deferred.

### 2026-06-27 Minimum wall length runtime slice

`min_length_factor` now reaches concrete Ares perimeter, print-path, extrusion, and G-code behavior through the source-cited Orca `PrintConfig.hpp:1039`, `PrintConfig.cpp:7062-7074`, and `Arachne/WallToolPaths.cpp:34-37,684-699` boundary. The slice parses Orca's `0.5` default and `0.0..=25.0` range, removes Ares' existing rectangular open thin-wall centerlines shorter than `external_line_width * min_length_factor` on non-top/bottom layers, and preserves first/topmost layers with the Orca `external_line_width / 2` protection threshold. Full Arachne variable-width wall lines, `is_odd`, junction min-width scanning, non-rectangular open wall generation, complete top/bottom surface classification, `min_feature_size`, `min_bead_width`, `initial_layer_min_bead_width`, wall transition filtering, wall maximum resolution/deviation, complete Orca Arachne parity, and Orca binary E2E geometry parity remain deferred.

### 2026-06-27 Ironing type path runtime slice

`ironing_type` now reaches concrete Ares ordinary Ironing print paths and G-code through the source-cited Orca `PrintConfig.hpp:100-105`, `PrintConfig.hpp:1137-1151`, `PrintConfig.cpp:257-263`, `PrintConfig.cpp:4161-4176`, `Fill/Fill.cpp:1499-1720`, and `PrintObject.cpp:699-715` boundary. The slice parses Orca's `no ironing`, `top`, `topmost`, and `solid` values, defaults to no ironing, duplicates existing Ares top or solid-area infill paths as `PrintPathRole::Ironing`, appends ordinary ironing after the source paths on each layer, and keeps support-interface ironing gated by `support_ironing`. Full Orca `Fill::make_ironing` polygon generation, pattern, spacing, inset, angle, filament spacing/inset overrides, extruder-region grouping, whole-face versus just-infill selection, and Orca binary E2E geometry parity remain deferred.

### 2026-06-27 Spiral finishing flow runtime slice

`spiral_finishing_flow_ratio` now reaches concrete Ares relative-E vase-mode G-code behavior through the source-cited Orca `PrintConfig.hpp:1563`, `PrintConfig.cpp:5717-5726`, and `GCode/SpiralVase.cpp:122-160,207-215` boundary. The slice parses Orca's finite `0..=1` float with default `0`, keeps the relative-E plus `spiral_mode` gate, preserves normal final-layer output, then appends duplicate final-layer print moves scaled from full flow down toward the configured finishing ratio by end-of-move printed-XY progress. Full continuous-Z spiral post-processing, XY smoothing, short-segment filtering, absolute-E tapering, non-print `transition_gcode` duplication, and full Orca `SpiralVase.cpp` parity remain deferred.

### 2026-06-27 Spiral starting flow runtime slice

`spiral_starting_flow_ratio` now reaches concrete Ares relative-E vase-mode G-code through the source-cited Orca `PrintConfig.hpp:1564`, `PrintConfig.cpp:5706-5715`, and `GCode/SpiralVase.cpp:118-151` boundary. The slice parses Orca's finite `0..=1` ratio, applies transition-in extrusion tapering only on the first vase body layer after normalized bottom shell layers, uses end-of-move printed-XY progress for scaling, and carries the scaled E offset forward so following relative-E moves do not compensate with catch-up extrusion. Full continuous-Z spiral post-processing, XY smoothing, absolute-E tapering, and full Orca SpiralVase parity remain deferred.

### 2026-06-27 Skirt height aggregate runtime slice

`skirt_height` now reaches concrete Ares combined-skirt geometry, print-path, diagnostics, and G-code behavior through the source-cited Orca `PrintConfig.hpp:1552-1555`, `PrintConfig.cpp:5559-5566,10173-10175`, `Print.cpp:2593-2738`, and `GCode.cpp:4257-4365` boundary. Combined skirts now use aggregate bounds from every layer that receives a skirt under the configured height, preserving a stable upper-layer skirt footprint for shrinking contours while keeping the existing first-layer brim envelope composition. Full Orca convex hulls, support/wipe-tower/raft point collection, per-object aggregate skirts, object instances, sequential-print state, multi-extruder skirt-loop ownership, variable layer-height skirt flow recomputation, and Orca binary E2E parity remain deferred.

### 2026-06-27 Wrapping exclude area gate runtime slice

`wrapping_exclude_area` now reaches concrete Ares wrapping-detection G-code behavior through the source-cited Orca `PrintConfig.hpp:1348-1360`, `PrintConfig.cpp:3987-4005,4312-4317`, `GCode.hpp:98`, and `GCode.cpp:5052-5062` boundary. Ares only inserts `wrapping_detection_gcode` when wrapping detection is enabled, the custom G-code template is non-empty, the layer is inside `wrapping_detection_layers`, and the configured exclude area contains more than two finite points. Invalid configured exclude-area values are rejected when the wrapping-detection layer-custom-G-code path is evaluated, even if wrapping detection is disabled or the template is empty. Full clumping-detection geometry, object intersection checks, wipe tower behavior, GUI plate editing, viewer display, multi-filament gating beyond Ares' current single-active path, and exact `ConfigOptionPoints` serialization parity remain deferred.

### 2026-06-27 Support ironing flow runtime slice

`support_ironing_flow` now reaches concrete Ares support-ironing G-code extrusion through the source-cited Orca `PrintConfig.hpp:997-1000`, `PrintConfig.cpp:6406-6444`, `Support/SupportParameters.hpp:58-61`, and `Support/SupportCommon.cpp:1898-1912` boundary. Existing duplicated support-interface ironing paths keep the internal `Ironing` role for speed and fan behavior, but use support-interface extrusion width, hardware, and flow with an effective layer height scaled by `support_ironing_flow`; `0%` keeps the ironing path visible with zero additional E. Full support ironing fill generation, `support_ironing_pattern`, `support_ironing_spacing`, multi-extruder ownership beyond Ares' current single-active support-interface path, and full Orca support-generation parity remain deferred.

### 2026-06-27 Support ironing paths runtime slice

`support_ironing` now reaches concrete Ares print-path, extrusion, speed, fan, hardware, and G-code behavior for existing support-interface paths through the source-cited Orca `PrintConfig.hpp:997-1000`, `PrintConfig.cpp:6406-6449`, `Support/SupportParameters.hpp:58-61`, `Support/SupportCommon.cpp:1635,1879-1907`, and `GCode.cpp:6115-6140` boundary. The slice parses the boolean option with Orca's default `false`, duplicates each existing `SupportMaterialInterface` path as an `Ironing` path after gap-fill filtering and before toolpath generation, preserves source path metadata, and lets the duplicate consume Ares' existing ironing speed, flow, fan, and hardware channels. Full support ironing fill generation from top contact-layer polygons, `support_ironing_pattern`, `support_ironing_spacing`, distinct `support_ironing_flow`, support-generation invalidation graph parity, and full Orca E2E geometry parity remain deferred.

### 2026-06-26 Filament shrink XY runtime slice

`filament_shrink` now reaches concrete Ares model, contour, and G-code behavior through the source-cited Orca `PrintConfig.hpp:1621`, `PrintConfig.cpp:2571-2582`, `Print.cpp:3628-3662`, `PrintApply.cpp:137-152`, `PrintApply.cpp:1526`, and `Geometry.hpp:471` boundary. The slice parses the first configured Orca percent value, scales loaded model XY vertices by `100 / filament_shrink` before Ares layer slicing, preserves existing Z shrinkage planning through `filament_shrinkage_compensation_z`, and lets downstream contour, perimeter, move, extrusion, speed, and G-code stages consume the compensated geometry. Multi-extruder shrinkage mismatch disabling, full instance transformation matrices, 3MF object transforms, object-distance revalidation after compensation, UI warnings, and full Orca E2E parity remain deferred.

### 2026-06-26 Ironing flow runtime slice

`ironing_flow` and first-value `filament_ironing_flow` now reach concrete Ares Ironing-role G-code extrusion deltas through the source-cited Orca `PrintConfig.hpp:1137-1151`, `PrintConfig.cpp:3372-3383`, `PrintConfig.cpp:4190-4200`, and `Fill/Fill.cpp:1584-1597` boundary. The slice maps Orca's default 10% ironing flow and nullable filament override fallback onto the existing `PrintPathRole::Ironing` extrusion multiplier, keeps Ironing independent from `top_solid_infill_flow_ratio`, and preserves existing print, filament, and first-layer flow composition. Full Orca `Fill::make_ironing` path generation, spacing/inset/pattern/angle behavior, support ironing flow, current-extruder indexed multi-filament override selection, and Orca binary E2E geometry parity remain deferred.

### 2026-06-26 Filament ironing speed runtime slice

`filament_ironing_speed` now reaches concrete Ares Ironing role G-code feedrates through the source-cited Orca `PrintConfig.hpp:1137-1151`, `PrintConfig.cpp:3409-3418`, `Fill/Fill.cpp:1584-1597`, and `GCode.cpp:6468-6469` boundary. The slice parses the first configured nullable filament speed override, treats `nil` as fallback to `ironing_speed`, validates Orca's minimum 1 mm/s range, and preserves first-layer `initial_layer_infill_speed` precedence. Full `Fill::make_ironing` path generation, filament-specific flow/spacing/inset, current-extruder indexed multi-filament selection, support-interface ironing, and Orca binary E2E geometry parity remain deferred.

### 2026-06-26 Ironing speed runtime slice

`ironing_speed` now reaches concrete Ares speed moves and G-code feedrates through the source-cited Orca `PrintConfig.hpp:1137-1144`, `PrintConfig.cpp:4222-4230`, and `GCode.cpp:6468-6469` boundary. Existing `PrintPathRole::Ironing` paths use Orca's default 20 mm/s or the configured absolute speed for non-first-layer moves, while first-layer ironing preserves Ares' infill-like initial-layer speed precedence. True ironing path generation, `ironing_type`/pattern/flow/spacing/inset/angle behavior, support-interface ironing generation, multi-extruder ironing ownership, and full Orca `Fill::make_ironing` parity remain deferred.

### 2026-06-26 Skirt distance around brim runtime slice

`skirt_distance` now composes with actual generated first-layer brim output through the source-cited Orca `Print.cpp` skirt/brim ordering boundary. Non-draft-shield combined skirts are generated after brims and use the first-layer brim envelope as the offset base, so the configured distance is measured from the outermost brim path while final print paths and diagnostics keep their existing skirt-before-brim surface ordering. No-brim, draft-shield, and per-object skirt behavior remain preserved; full Orca polygon convex hull, support/wipe-tower/raft brim ownership, draft-shield brim trimming, and `get_real_skirt_dist` public helper parity remain deferred.

### 2026-06-26 Per-object skirt runtime slice

`skirt_type = "perobject"` now reaches concrete Ares skirt, print-path, extrusion, speed, and G-code behavior through the source-cited Orca `SkirtType::stPerObject` boundary in `PrintConfig.hpp`, `PrintConfig.cpp`, `Print::_make_skirt`, and `GCode.cpp`. The slice maps Orca's per-object branch onto Ares' current contour scaffold by generating one rectangular skirt per outer contour while preserving combined global skirt behavior and reusing min-skirt-length, draft-shield, single-loop draft-shield, and start-angle handling. Full `PrintObject` ownership, instance offsets, support-layer hulls, convex hull offsets, object-specific start angles, by-object sequence gating, and Orca binary E2E parity remain deferred.

### 2026-06-26 Spiral vase base infill runtime slice

`spiral_mode` now reaches concrete Ares base-infill behavior through the source-cited Orca `PrintConfig.hpp:1560`, `PrintConfig.cpp:5678-5684,8355-8369`, `PrintObject.cpp:1492-1514,1690-1695`, and `LayerRegion.cpp:81-97,899-919` boundary. After normalization forces `wall_loops = 1`, `top_shell_layers = 0`, and `sparse_infill_density = 0`, Ares still emits solid bottom-base infill for the configured `bottom_shell_layers`, marks the final multi-layer base layer as top solid infill, leaves layers above the base empty, and preserves non-spiral zero-density empty infill. Continuous-Z vase G-code, spiral smoothing/flow options, exact Orca `SurfaceCollection` propagation, bottom-thickness start boundaries beyond existing shell-thickness classification, multi-region/support/raft/ironing interactions, and full Orca E2E parity remain deferred.

### 2026-06-26 Wipe-on-loops G-code runtime slice

`wipe_on_loops` now reaches concrete Ares G-code through the source-cited Orca `PrintConfig.hpp:1185`, `PrintConfig.cpp:5510-5515`, and `GCode.cpp:5926-5961` boundary. The slice parses the bool option with Orca's default `false`, emits a zero-extrusion `move inwards before travel` after supported closed external perimeter loops when `wall_loops > 1`, preserves `wipe_before_external_loop` as a separate option, and keeps full Orca hole, scarf, split multipath, Arachne, winding-side, and loop-object parity deferred.

### 2026-06-25 Detect thin wall rectangular runtime slice

`detect_thin_wall` now reaches concrete Ares perimeter, print-path, print-domain, move, extrusion, speed, and G-code behavior through the source-cited Orca `PrintConfig.hpp:1164-1165`, `PrintConfig.cpp:6508-6514`, `PerimeterGenerator.hpp:117-124`, and `PerimeterGenerator.cpp:230-244,1243-1267` boundary. The slice parses the bool option with Orca's default `false`, converts Ares' current narrow rectangular wall-gap condition into an open external perimeter centerline when enabled, suppresses the corresponding wall gap-fill artifact, keeps default-disabled wall gap-fill behavior unchanged, and preserves solid-surface `gap_fill_target` behavior. Full Orca polygonal medial-axis thin-wall detection, variable-width `ThickPolylines`, Arachne thin-wall paths, smaller external-width fallback when disabled, thin-wall holes/order reversal, multi-region interactions, and overlap clipping beyond the current rectangular scaffold remain deferred.

### 2026-06-25 Nozzle temperature range compatibility runtime slice

`nozzle_temperature_range_low` and `nozzle_temperature_range_high` now reach concrete Ares slicing validation through the source-cited Orca `PrintConfig.hpp:1571-1572`, `PrintConfig.cpp:6487-6501`, and `Print.cpp:1052-1100,1177-1234` boundary. The slice parses Orca-compatible integer vectors with defaults `190`/`240`, validates `low < high`, computes the effective filament count from temperature, range, filament, and hardware vectors, applies Orca first-value fallback, and rejects mutually incompatible multi-filament temperature pairs before model loading or G-code output. Orca `MaterialType::get_temperature_range` zero-range fallback, `enable_high_low_temp_mixed_printing` warning preference, by-object extruder-set validation, UI warnings/localization, full multi-toolchange ownership, and wipe-tower/support extruder interactions remain deferred.

### 2026-06-25 Seam position back runtime slice

`seam_position = "back"` now reaches concrete Ares perimeter start-point behavior through the source-cited Orca `PrintConfig.hpp:211-213`, `PrintConfig.hpp:944`, `PrintConfig.cpp:350-357`, `PrintConfig.cpp:5357-5373`, and `GCode/SeamPlacer.cpp:742-797` boundary. The slice parses Orca's seam-position enum strings, defaults omitted values to `aligned`, rejects malformed values at the perimeter option boundary, and rotates generated perimeter loops for `back` to the first max-Y vertex so downstream print-path and G-code markers consume the option. Full Orca `SeamPlacer` candidate generation, blockers/enforcers, visibility/occlusion, alignment, nearest/random behavior, seam projection, scarf seams, staggered inner seams, seam gaps, multi-object coordination, and UI/preset behavior remain deferred.

### 2026-06-25 Head-wrap detect-zone placeholder runtime slice

`head_wrap_detect_zone` now reaches concrete Ares `machine_start_gcode` placeholder output through the source-cited Orca `PrintConfig.hpp:1485`, `PrintConfig.cpp:6503-6506`, `PrintConfig.cpp:10900`, and `GCode.cpp:2890-2931` boundary. The slice renders `[in_head_wrap_detect_zone]` as `1` when the configured zone bounds intersect Ares' current first-layer print bounds and `0` for missing, empty, `0x0`, or non-intersecting zones, while preserving machine-start-only placeholder scope. Full Orca object projection union, exact polygon intersection, plate offset handling, multi-object/wipe-tower/support hull ownership, calibration-mode geometry, GUI zone editing, clumping-detection placement behavior, and multi-extruder interactions remain deferred.

### 2026-06-24 Support line width runtime slice

`support_line_width` now reaches concrete Ares support-material and support-interface extrusion/G-code behavior through the source-cited Orca `PrintConfig.hpp:960`, `PrintConfig.cpp:6043-6053`, and `Flow.cpp:54-55,214-250` boundary. The slice parses numeric and percent values over nozzle diameter with Orca's default zero fallback, routes constructed `support_material` and `support_material_interface` paths through the support width for E deltas, preserves first-layer `initial_layer_line_width` precedence, and composes with existing support speed, flow-ratio, fan, and role mapping behavior. Full support generation, tree-support geometry, transition/roof/bottom support widths, multi-extruder nozzle selection beyond Ares' first-value path, full `Flow` class parity, and UI/preset behavior remain deferred.

### 2026-06-24 Layer-change slope Z-hop type runtime slice

`z_hop_types`, `filament_z_hop_types`, and `travel_slope` now reach concrete Ares layer-change retraction Z-hop G-code through the source-cited Orca `PrintConfig.hpp:246-250`, `PrintConfig.hpp:1377-1378`, `PrintConfig.cpp:5149-5171`, `PrintConfig.cpp:7170-7224`, `PrintConfig.cpp:8188-8201`, `Extruder.cpp:215-218`, `GCode.cpp:5617-5629`, `GCode.cpp:7443-7455`, and `GCodeWriter.cpp:623-648,719-747` boundary. The slice keeps explicit `Normal Lift` as vertical layer-change lift, makes default/explicit `Slope Lift` consume the next same-layer travel as raised XYZ slope-top or raised-target travel, falls back to vertical lift before unretract when no same-layer travel consumes the pending slope lift, and preserves firmware/E-axis retraction, restart extra, filament Z-hop, lift gates, and lift-enforce behavior. Orca's layer-change Auto-to-Spiral conversion, spiral lift arc output, eager lift, multi-extruder/current-filament selection beyond Ares' first-value path, and broader `GCode::retract` orchestration remain deferred.

### 2026-06-27 Auto Lift Z-hop runtime slice

`z_hop_types = Auto Lift` now reaches concrete Ares ordinary-travel and layer-change Z-hop G-code through the source-cited Orca `PrintConfig.hpp:382-388`, `PrintConfig.hpp:1375-1378`, `PrintConfig.cpp:526-530`, `PrintConfig.cpp:5149-5162`, and `GCode.cpp:5625-5628,7443-7455,7539-7544,7573-7578` boundary. Ares now keeps Auto as a distinct mode, emits slope lift for current ordinary travel paths without overhang-crossing data, and emits spiral lift for layer-change retraction per Orca's layer-change override. Full ordinary-travel overhang-crossing Auto selection, toolchange/nozzle-change/cut/wipe-tower Auto lift selection, and multi-extruder current-filament runtime selection beyond Ares' current first-value path remain deferred.

### 2026-06-24 Travel slope Z-hop type runtime slice

`z_hop_types`, `filament_z_hop_types`, and `travel_slope` now reach concrete Ares ordinary-travel Z-hop G-code through the source-cited Orca `PrintConfig.hpp:246-250`, `PrintConfig.hpp:1377-1378`, `PrintConfig.cpp:527-530`, `PrintConfig.cpp:5149-5169`, `PrintConfig.cpp:63-84`, `PrintConfig.cpp:7122-7152`, `PrintConfig.cpp:7164-7224`, `PrintConfig.cpp:8188-8201`, `Extruder.cpp:215-218`, `GCode.cpp:7443-7455`, `GCode.cpp:7458-7578`, `GCodeWriter.cpp:623-648`, and `GCodeWriter.cpp:719-747` boundary. The slice validates unprefixed and filament-prefixed Z-hop type values plus `travel_slope`, uses the first configured single-active-filament type, treats first `nil`/`null` as fallback, applies Orca's default `Slope Lift`, emits ordinary-travel slope lift as an early raised XYZ travel when the travel is long enough, emits one raised XYZ travel for too-short slope moves, and keeps accepted `Auto Lift` / `Spiral Lift` values on explicit normal-lift fallback until their source boundaries are implemented. Spiral arcs, Auto overhang selection, layer-change slope/spiral lift, eager lift, non-straight travel paths, avoid-crossing-perimeters, toolchange/cut/wipe-tower retractions, multi-extruder current-filament selection beyond Ares' first-value path, and full `GCode::retract` orchestration remain deferred.

### 2026-06-24 Filament wipe override runtime slice

`filament_wipe` now reaches concrete Ares ordinary travel wipe/retraction G-code through the source-cited Orca `PrintConfig.cpp:63-84`, `PrintConfig.cpp:6628-6633`, `PrintConfig.cpp:7122-7152`, `PrintConfig.cpp:7164-7224`, `PrintConfig.cpp:8188-8208`, and `GCode.cpp:7589-7599` boundary. The slice validates the prefixed nullable bool override, uses the first configured single-active-filament explicit value to override unprefixed `wipe`, treats first `nil`/`null` as fallback to the unprefixed value, and composes with the existing `wipe_distance`, `retract_before_wipe`, role-based wipe speed, wipe speed, retraction, Z-hop, reduce-infill, ordinary travel, and pending layer-change paths. Full Orca dynamic config merge, multi-extruder/current-filament selection, `filament_wipe_distance`, `filament_retract_before_wipe`, toolchange wipe, layer-change-specific wipe output, wipe tower/MMU behavior, loop/seam/scarf wipe orchestration, avoid-crossing-perimeters, and full `GCode::retract` parity remain deferred.

### 2026-06-24 Filament wipe distance override runtime slice

`filament_wipe_distance` now reaches concrete Ares ordinary travel wipe/retraction G-code through the source-cited Orca `PrintConfig.cpp:63-84`, `PrintConfig.cpp:6635-6644`, `PrintConfig.cpp:7122-7152`, `PrintConfig.cpp:7164-7224`, `PrintConfig.cpp:8188-8208`, `GCode.cpp:343-350`, and `GCode.cpp:7589-7599` boundary. The slice validates the prefixed nullable float override, uses the first configured single-active-filament explicit non-negative value to override unprefixed `wipe_distance`, treats first `nil`/`null` as fallback to the unprefixed value, and composes with the existing `wipe`, `filament_wipe`, `retract_before_wipe`, role-based wipe speed, wipe speed, retraction, Z-hop, reduce-infill, ordinary travel, and pending layer-change paths. Full Orca dynamic config merge, multi-extruder/current-filament selection, `filament_retract_before_wipe`, toolchange wipe, layer-change-specific wipe output, wipe tower/MMU behavior, loop/seam/scarf wipe orchestration, avoid-crossing-perimeters, and full `GCode::retract` parity remain deferred.

### 2026-06-24 Filament retract-before-wipe override runtime slice

`filament_retract_before_wipe` now reaches concrete Ares ordinary travel wipe/retraction G-code through the source-cited Orca `PrintConfig.cpp:63-84`, `PrintConfig.cpp:5055-5062`, `PrintConfig.cpp:7122-7152`, `PrintConfig.cpp:7164-7224`, `PrintConfig.cpp:8188-8208`, `GCode.cpp:329-350`, and `GCode.cpp:7589-7599` boundary. The slice validates the prefixed nullable percent override, uses the first configured single-active-filament explicit `0..=100` value to override unprefixed `retract_before_wipe`, treats first `nil`/`null` as fallback to the unprefixed value, and composes with the existing `wipe`, `filament_wipe`, `wipe_distance`, `filament_wipe_distance`, role-based wipe speed, wipe speed, retraction, Z-hop, reduce-infill, ordinary travel, and pending layer-change paths. Full Orca dynamic config merge, multi-extruder/current-filament selection, toolchange wipe, layer-change-specific wipe output, wipe tower/MMU behavior, loop/seam/scarf wipe orchestration, avoid-crossing-perimeters, and full `GCode::retract` parity remain deferred.

### 2026-06-24 Filament retract lift gates override runtime slice

`filament_retract_lift_above`, `filament_retract_lift_below`, and `filament_retract_lift_enforce` now reach concrete Ares ordinary travel and layer-change Z-hop lift/restore G-code through the source-cited Orca `PrintConfig.cpp:63-84`, `PrintConfig.cpp:5173-5200`, `PrintConfig.cpp:7122-7152`, `PrintConfig.cpp:7164-7224`, `PrintConfig.cpp:8188-8201`, `GCodeWriter.cpp:623-648`, `GCodeWriter.cpp:652-674`, and `GCode.cpp:7606-7637` boundary. The slice validates the prefixed nullable lower/upper lift gates and nullable lift-enforce enum, uses the first configured single-active-filament explicit value to override the unprefixed gates, treats first `nil`/`null` as fallback to the unprefixed value, and composes with the existing `z_hop`, `filament_z_hop`, retraction, wipe, reduce-infill, minimum-travel, ordinary travel, layer-change, and previous-role paths. Full Orca dynamic config merge, multi-extruder/current-filament selection, `filament_z_hop_types`, slope/spiral/auto lift, toolchange/cut/wipe-tower retractions, seam/scarf behavior, avoid-crossing-perimeters, support/internal exceptions, ironing-specific top eligibility, and full `GCode::retract` orchestration remain deferred.

### 2026-06-24 Filament retract-when-changing-layer override runtime slice

`filament_retract_when_changing_layer` now reaches concrete Ares layer-change retraction G-code through the source-cited Orca `PrintConfig.cpp:63-84`, `PrintConfig.cpp:5062-5067`, `PrintConfig.cpp:7122-7152`, `PrintConfig.cpp:7164-7224`, `PrintConfig.cpp:8187-8208`, and `GCode.cpp:5625-5628` boundary. The slice validates the prefixed nullable bool override, uses the first configured single-active-filament explicit value to override unprefixed `retract_when_changing_layer`, treats first `nil`/`null` as fallback to the unprefixed value, and composes with the existing layer-change length, restart-extra, speed, firmware, wipe, Z-hop, and lift-gate paths. Full Orca dynamic config merge, multi-extruder/current-filament selection, `filament_z_hop_types`, filament-prefixed lift and wipe settings, toolchange/cut/wipe-tower retractions, seam/scarf behavior, avoid-crossing-perimeters, and full `GCode::retract` orchestration remain deferred.

### 2026-06-24 Filament retract restart extra override runtime slice

`filament_retract_restart_extra` now reaches concrete Ares ordinary travel and layer-change unretract G-code through the source-cited Orca `PrintConfig.cpp:63-84`, `PrintConfig.cpp:5306-5313`, `PrintConfig.cpp:7136-7152`, `PrintConfig.cpp:7164-7224`, `PrintConfig.cpp:8188-8200`, `Extruder.cpp:200-203`, and `GCodeWriter.cpp:1004-1012` boundary. The slice validates the prefixed filament override, uses its first configured single-active-filament value as the effective restart-extra amount, preserves the existing unprefixed `retract_restart_extra` fallback when the prefixed option is absent, and composes with the existing retraction length, speed, wipe, Z-hop, minimum-travel, reduce-infill, ordinary travel, and layer-change paths. Full Orca dynamic config merge, nullable inheritance, multi-extruder/current-filament selection, toolchange restart extra, other filament-prefixed retract options, avoid-crossing-perimeters, seam/scarf behavior, support/internal exceptions, wipe tower, and full `GCode::retract` orchestration remain deferred.

### 2026-06-24 Filament retraction minimum travel override runtime slice

`filament_retraction_minimum_travel` now reaches concrete Ares ordinary XY travel retraction G-code through the source-cited Orca `PrintConfig.cpp:63-84`, `PrintConfig.cpp:5048-5054`, `PrintConfig.cpp:7136-7152`, `PrintConfig.cpp:7164-7224`, `PrintConfig.cpp:8188-8208`, `GCode.cpp:7280-7330`, and `GCode.cpp:7458-7602` boundary. The slice validates the prefixed filament override, uses its first configured single-active-filament value as the effective minimum-travel threshold, preserves the existing unprefixed `retraction_minimum_travel` fallback when the prefixed option is absent, and composes with the existing retract/unretract, wipe, Z-hop, reduce-infill, and pending layer-change paths. Full Orca dynamic config merge, nullable inheritance, multi-extruder/current-filament selection, other filament-prefixed retract options, short-travel acceleration/jerk, avoid-crossing-perimeters, support/internal exceptions, toolchange, wipe tower, and full `GCode::retract` orchestration remain deferred.

### 2026-06-24 Filament Z-hop override runtime slice

`filament_z_hop` now reaches concrete Ares ordinary travel and layer-change Z-hop G-code through the source-cited Orca `PrintConfig.cpp:63-84`, `PrintConfig.cpp:5122-5131`, `PrintConfig.cpp:7137-7149`, `PrintConfig.cpp:7164-7188`, `PrintConfig.cpp:8188-8201`, `GCodeWriter.cpp:623-648`, `GCodeWriter.cpp:1084-1092`, and `Extruder.cpp:179-182` boundary. The slice parses the first configured single-active-filament override value, lets it replace the unprefixed `z_hop` height in the existing lift/restore path, preserves `filament_z_hop = 0` no-hop behavior while leaving retract/unretract intact, validates invalid prefixed vectors before G-code output, and keeps existing lower/upper/enforce gates on the effective Z-hop. Full Orca dynamic config merge, nullable `nil` fallback, multi-extruder/current-filament selection, `filament_z_hop_types`, filament-prefixed lift gates, non-vertical lift modes, toolchange/cut/wipe-tower retractions, seam/scarf behavior, and full Orca `GCode::retract` orchestration remain deferred.

### 2026-06-24 Elephant-foot solid-infill density runtime slice

`elefant_foot_layers_density` now reaches concrete Ares internal solid infill paths and G-code through the source-cited Orca `PrintConfig.cpp:727-747`, `PrintConfig.hpp:929-931`, `PrintObject.cpp:1159-1161`, and `Fill/Fill.cpp:1338-1344` boundary. The slice parses Orca's 50..=100 percent range plus positive `elefant_foot_compensation_layers`, applies Orca's second-layer-to-compensation-layer linear density ramp only to internal solid infill spacing, preserves bottom/top surface, sparse, bridge, and internal-bridge density behavior, and keeps full elephant-foot polygon shrinking, brim outline changes, SLA `elefant_foot_min_width`, and UI behavior deferred.

### 2026-06-24 Wipe travel retraction runtime slice

`wipe`, `wipe_distance`, `retract_before_wipe`, `role_based_wipe_speed`, and `wipe_speed` now reach concrete Ares ordinary travel retraction G-code through the source-cited Orca `PrintConfig.hpp:1183-1184,1367,1569,1573`, `PrintConfig.cpp:5055-5060,5502-5539,6628-6641`, and `GCode.cpp:312-360,426-505,7589-7599` boundary. The slice parses Orca's defaults, validates supplied wipe runtime values, stores the previous printed straight segment and its feedrate, chooses role-based wipe feedrate or `wipe_speed` resolved over `travel_speed`, clamps wipe speed to at least 10 mm/s, moves speed-limited excess retraction before the wipe, emits zero-E wipe moves for `retract_before_wipe = 100%`, and clamps wipe distance to the available segment length. Full Orca multi-point wipe path storage, loop clipping, toolchange wipe, wipe tower/MMU integration, adaptive pressure-advance wipe handling, avoid-crossing-perimeters interactions, and multi-extruder retraction state remain deferred.

### 2026-06-24 Gap fill target solid-surface runtime slice

`gap_fill_target` now reaches concrete Ares solid-surface gap-fill paths and G-code through the source-cited Orca `PrintConfig.hpp:241-244`, `PrintConfig.cpp:393-398`, `PrintConfig.cpp:1141-1168`, and `Fill/FillBase.cpp:195-244` boundary. The slice parses Orca's `everywhere` / `topbottom` / `nowhere` enum strings with default `nowhere`, appends rectangular solid-surface `gap_fill` paths for top/bottom solid roles under `topbottom` and internal solid roles under `everywhere`, uses Ares' existing solid infill line width for the rectangular eligibility/inset geometry, preserves classic wall/perimeter gap fill independence from this option, and composes with existing gap-fill speed, flow, extrusion, print-domain extras, G-code role comments, bridge-layer suppression, and `filter_out_gap_fill`. Full Orca `no_overlap_expolygons`, polygon union/diff/intersection, medial-axis variable-width polylines, partial solid-surface remnants, holes, multi-region remnants, and bridge-surface splitting beyond Ares' current whole-layer bridge classification remain deferred.

### 2026-06-24 Extra external bridge layer runtime slice

`enable_extra_bridge_layer` now reaches concrete Ares external bridge G-code output through the source-cited Orca `PrintConfig.hpp:236-239`, `PrintConfig.cpp:384-390,1871-1900`, and `PrintObject.cpp:1704-1797` boundary. The slice parses the four Orca enum strings, emits the layer above an unsupported external bridge as existing `bridge` output for `external_bridge_only` and `apply_to_all`, composes with current `bridge_density` / `bridge_angle` / bridge speed-flow/thick/fan bridge-role behavior, and keeps exact polygon splitting plus the internal bridge half deferred.

### 2026-06-24 Internal bridge filter runtime slice

`dont_filter_internal_bridges` now reaches concrete Ares internal-bridge infill and G-code output through the source-cited Orca `PrintConfig.hpp:231-235`, `PrintConfig.hpp:988`, `PrintConfig.cpp:377-382`, `PrintConfig.cpp:1902-1928`, and `PrintObject.cpp:2430-2459` boundary. The slice parses Orca's `disabled` / `limited` / `nofilter` enum, applies an Ares-local whole-layer span filter to internal-bridge density generation, keeps small default-filtered internal solid layers as solid infill, and lets `limited` / `nofilter` emit existing `internal_bridge` paths and G-code for small eligible layers. Full Orca polygon unsupported-area filtering, per-contour splitting, extra bridge layers, support-aware ownership, lightning infill interactions, and multi-region bridge classification remain deferred.

### 2026-06-24 Adaptive bridge pressure advance runtime slice

`adaptive_pressure_advance_bridges` now reaches concrete Ares pressure advance G-code through the source-cited Orca `PrintConfig.hpp:1302-1308`, `PrintConfig.cpp:2252-2319`, `GCode.cpp:6657-6770`, and `GCode/AdaptivePAProcessor.cpp:221-272` boundary. The slice parses `adaptive_pressure_advance` and the bridge PA value, keeps base startup PA unchanged, switches to bridge PA before Ares bridge-like print roles, restores base PA on the next non-bridge print move, and keeps duplicate PA commands suppressed while consecutive bridge-like moves stay in the same state. The full adaptive PA model, `adaptive_pressure_advance_model`, `adaptive_pressure_advance_overhangs` flow/speed recalculation, `;PA_Change` post-processing, calibration modes, multi-extruder toolchange state, and debug APA comments remain deferred.

### 2026-06-24 Reduce infill retraction runtime slice

`reduce_infill_retraction` now reaches concrete Ares ordinary travel retraction G-code through the source-cited Orca `PrintConfig.hpp:1544`, `PrintConfig.cpp:4829-4835`, and `GCode.cpp:7280-7289,7458-7578` boundary pinned to local OrcaSlicer commit `f3cb1992d6e6f3bca3dec6dd52ecd10dee640d24`. The slice parses the bool option with Orca's default `false`, reuses effective sparse-infill density including spiral-mode normalization, suppresses ordinary retract/unretract and z-hop for same-layer internal-infill-to-internal-infill travel when sparse infill is enabled, and preserves retraction for disabled/default, zero-density, perimeter-source, and perimeter-target cases. Full `travel_inside_internal_regions` geometry, support/tree-support island suppression, avoid-crossing-perimeters rerouting, wipe behavior, wipe tower travel, and multi-extruder retraction state remain deferred.

### 2026-06-24 Small-area flow model header runtime slice

`small_area_infill_flow_compensation_model` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.hpp:1463-1464`, `PrintConfig.cpp:4359-4371`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing small-area model parser plus Orca-compatible `ConfigOptionStrings` serialization, emits configured array or scalar-string model entries as `; small_area_infill_flow_compensation_model = ...`, rejects malformed/empty/non-string/fewer-than-two-point values before BTT thumbnail header suppression, preserves upstream-adjacent ordering after `support_multi_bed_types`, and keeps `has_scarf_joint_seam`, full `append_full_config` parity, UI/preset behavior, object/material overrides, and additional small-area flow execution changes deferred.

### 2026-06-24 MMU scalar config header runtime slice

`cooling_tube_retraction`, `cooling_tube_length`, `high_current_on_filament_swap`, `parking_pos_retraction`, `extra_loading_move`, `machine_load_filament_time`, `machine_tool_change_time`, and `machine_unload_filament_time` now reach concrete Ares G-code header output through the source-cited Orca `PrintConfig.hpp:1427-1435`, `PrintConfig.cpp:2472-2497`, `PrintConfig.cpp:4779-4819`, and `GCode.cpp:5523-5575` boundary. The slice serializes finite scalar floats with Orca-compatible decimal formatting, allows negative `extra_loading_move`, serializes `high_current_on_filament_swap` as `1`/`0`, rejects invalid values before BTT thumbnail header suppression, preserves upstream-adjacent ordering before `filament_loading_speed`, and keeps MMU loading/unloading motion, high-current firmware commands, parking/cooling-tube movement, timing/statistics integration, wipe tower generation, WipeTower2 behavior, and exhaustive `append_full_config` parity deferred.

### 2026-06-23 Wipe tower config header runtime slice

`wipe_tower_type`, `purge_in_prime_tower`, `enable_filament_ramming`, `tool_change_on_wipe_tower`, and `support_multi_bed_types` now reach concrete Ares G-code header output through the source-cited Orca `PrintConfig.hpp:74-77`, `PrintConfig.cpp:212-216`, `PrintConfig.hpp:1457-1461`, `PrintConfig.cpp:3825-3830`, `PrintConfig.cpp:5821-5849`, and `GCode.cpp:5523-5575` boundary. The slice serializes `wipe_tower_type` as the upstream `type1`/`type2` key, serializes scalar bools as `1`/`0`, rejects invalid values before BTT thumbnail header suppression, preserves upstream-adjacent ordering after `filament_stamping_distance`, and keeps wipe tower geometry, toolchange travel, single-extruder multimaterial priming, `has_wipe_tower` truth, toolchange count computation, support multi-bed UI behavior, and WipeTower2 purge/ramming execution deferred.

### 2026-06-23 Filament stamping header runtime slice

`filament_stamping_loading_speed` and `filament_stamping_distance` now reach concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2655-2668`, `PrintConfig.hpp:1455-1456`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible finite non-negative float-vector header serialization path, emits configured values as `; filament_stamping_* = ...`, rejects invalid values before BTT thumbnail header suppression, preserves upstream-adjacent ordering after `filament_multitool_ramming_flow`, and keeps `WipeTower2.cpp:1366-1367` parameter transfer plus `WipeTower2.cpp:1784-1805` stamping movement, extrusion/retraction, turning-point, cooling-tube, and there-and-back behavior deferred.

### 2026-06-23 Filament multitool ramming header runtime slice

`filament_multitool_ramming`, `filament_multitool_ramming_volume`, and `filament_multitool_ramming_flow` now reach concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2752-2774`, `PrintConfig.hpp:1452-1454`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible bool-vector and finite non-negative float-vector header serialization paths, emits configured values as `; filament_multitool_ramming* = ...`, rejects invalid values before BTT thumbnail header suppression, preserves upstream-adjacent ordering after `filament_ramming_parameters`, and keeps `WipeTower2.cpp:1391-1405` multitool ramming enablement, ramming-speed vector, ramming-time calculation, and wipe-tower motion deferred.

### 2026-06-23 Filament ramming parameters header runtime slice

`filament_ramming_parameters` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2745-2750`, `PrintConfig.hpp:1451`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible `ConfigOptionStrings` header serialization path, emits configured string-vector values as `; filament_ramming_parameters = ...`, rejects invalid values before BTT thumbnail header suppression, preserves upstream-adjacent ordering after `filament_cooling_final_speed`, and keeps WipeTower/WipeTower2 ramming parameter parsing plus ramming execution deferred.

### 2026-06-23 Filament tower interface print temperature header runtime slice

`filament_tower_interface_print_temp` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2729-2735`, `PrintConfig.hpp:1449`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible `ConfigOptionInts` header serialization path, emits configured integer-vector values as `; filament_tower_interface_print_temp = ...`, accepts the upstream `-1` sentinel plus non-negative temperatures, rejects invalid values before BTT thumbnail header suppression, preserves upstream-adjacent ordering between `filament_tower_interface_purge_volume` and `filament_cooling_final_speed`, and keeps max recommended nozzle temperature fallback plus WipeTower/WipeTower2 interface temperature execution deferred.

### 2026-06-23 Filament tower interface purge volume header runtime slice

`filament_tower_interface_purge_volume` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2721-2727`, `PrintConfig.hpp:1448`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible `ConfigOptionFloats` header serialization path, emits configured non-negative finite float-vector values as `; filament_tower_interface_purge_volume = ...`, rejects invalid values before BTT thumbnail header suppression, preserves upstream-adjacent ordering between `filament_tower_ironing_area` and `filament_cooling_final_speed`, and keeps WipeTower/WipeTower2 purge execution plus neighboring print temperature, stamping, and ramming behavior deferred.

### 2026-06-23 Filament tower ironing area header runtime slice

`filament_tower_ironing_area` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2713-2719`, `PrintConfig.hpp:1447`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible `ConfigOptionFloats` header serialization path, emits configured non-negative finite float-vector values as `; filament_tower_ironing_area = ...`, rejects invalid values before BTT thumbnail header suppression, preserves upstream-adjacent ordering between `filament_tower_interface_pre_extrusion_length` and `filament_cooling_final_speed`, and keeps WipeTower/WipeTower2 ironing movement plus neighboring purge volume, print temperature, stamping, and ramming behavior deferred.

### 2026-06-23 Filament tower interface pre-extrusion length header runtime slice

`filament_tower_interface_pre_extrusion_length` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2705-2711`, `PrintConfig.hpp:1446`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible `ConfigOptionFloats` header serialization path, emits configured non-negative finite float-vector values as `; filament_tower_interface_pre_extrusion_length = ...`, rejects invalid values before BTT thumbnail header suppression, preserves upstream-adjacent ordering between `filament_tower_interface_pre_extrusion_dist` and `filament_cooling_final_speed`, and keeps WipeTower/WipeTower2 interface pre-extrusion movement plus neighboring ironing area, purge volume, print temperature, stamping, and ramming behavior deferred.

### 2026-06-23 Filament tower interface pre-extrusion distance header runtime slice

`filament_tower_interface_pre_extrusion_dist` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2697-2703`, `PrintConfig.hpp:1445`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible `ConfigOptionFloats` header serialization path, emits configured non-negative finite float-vector values as `; filament_tower_interface_pre_extrusion_dist = ...`, rejects invalid values before BTT thumbnail header suppression, preserves upstream-adjacent ordering between `filament_cooling_before_tower` and `filament_cooling_final_speed`, and keeps wipe-tower interface pre-extrusion movement plus neighboring interface length, ironing area, purge volume, print temperature, stamping, and ramming behavior deferred.

### 2026-06-23 Filament cooling before tower header runtime slice

`filament_cooling_before_tower` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2689-2695`, `PrintConfig.hpp:1444`, `Config.hpp:879-915`, and `GCode.cpp:5523-5575` boundary. The slice consumes the upstream `ConfigOptionFloatsNullable` nil serialization behavior for scalar `null`, `"nil"`, mixed numeric/nil arrays, and mixed separated strings, emits mixed vectors as `; filament_cooling_before_tower = nil,10,...`, omits non-empty all-nil configured vectors, preserves empty JSON arrays as an empty header value, rejects invalid values before BTT thumbnail header suppression, preserves upstream-adjacent ordering after `filament_minimal_purge_on_wipe_tower`, and keeps wipe-tower cooling/interface execution deferred.

### 2026-06-23 Filament minimal purge header runtime slice

`filament_minimal_purge_on_wipe_tower` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2678-2687`, `PrintConfig.hpp:1443`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible `ConfigOptionFloats` header serialization path, emits configured non-negative finite float-vector values as `; filament_minimal_purge_on_wipe_tower = ...`, rejects invalid values before BTT thumbnail header suppression, preserves upstream-adjacent ordering between `filament_cooling_initial_speed` and `filament_cooling_final_speed`, and keeps `WipeTower2.cpp:1343`, `2185-2198`, and `2302` purge-volume execution plus wipe-tower movement/toolchange behavior deferred.

### 2026-06-23 Filament cooling final speed header runtime slice

`filament_cooling_final_speed` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2737-2743`, `PrintConfig.hpp:1450`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible `ConfigOptionFloats` header serialization path, emits configured non-negative finite float-vector values as `; filament_cooling_final_speed = ...`, rejects invalid values before BTT thumbnail header suppression, and keeps `WipeTower2.cpp:1365` single-extruder MM cooling final-speed execution, tower interface behavior, stamping, ramming, full `append_full_config` exhaustive export, and movement/extrusion behavior deferred.

### 2026-06-23 Filament cooling initial speed header runtime slice

`filament_cooling_initial_speed` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2670-2676`, `PrintConfig.hpp:1442`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible `ConfigOptionFloats` header serialization path, emits configured non-negative finite float-vector values as `; filament_cooling_initial_speed = ...`, rejects invalid values before BTT thumbnail header suppression, and keeps `WipeTower2.cpp:1364` single-extruder MM cooling speed execution, `filament_cooling_final_speed`, stamping, ramming, tower interface behavior, full `append_full_config` exhaustive export, and movement/extrusion behavior deferred.

### 2026-06-23 Filament cooling moves header runtime slice

`filament_cooling_moves` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2646-2653`, `PrintConfig.hpp:1441`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible `ConfigOptionInts` header serialization path, emits configured integer-vector values in Orca's `0..=20` range as `; filament_cooling_moves = ...`, rejects invalid values before BTT thumbnail header suppression, and keeps `WipeTower2.cpp:1363` single-extruder MM cooling-move execution, toolchange path generation, cooling speeds, full `append_full_config` exhaustive export, and movement/extrusion behavior deferred.

### 2026-06-23 Filament toolchange delay header runtime slice

`filament_toolchange_delay` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2636-2644`, `PrintConfig.hpp:1440`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible `ConfigOptionFloats` header serialization path, emits configured non-negative finite float-vector values as `; filament_toolchange_delay = ...`, rejects invalid values before BTT thumbnail header suppression, and keeps `WipeTower2.cpp:1362` runtime delay behavior, wipe-tower loading/unloading path generation, toolchange G-code execution, full `append_full_config` exhaustive export, and movement/extrusion behavior deferred.

### 2026-06-23 Filament load/unload speed header runtime slice

`filament_loading_speed`, `filament_loading_speed_start`, `filament_unloading_speed`, and `filament_unloading_speed_start` now reach concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2603-2634`, `PrintConfig.hpp:1436-1439`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible `ConfigOptionFloats` header serialization path, emits configured non-negative finite float-vector values as `; filament_*_speed = ...`, rejects invalid values before BTT thumbnail header suppression, and keeps wipe-tower loading/unloading path generation, ramming behavior, toolchange G-code, `filament_toolchange_delay`, `filament_cooling_*`, full `append_full_config` exhaustive export, and movement/extrusion behavior deferred.

### 2026-06-23 Filament adhesiveness category header runtime slice

`filament_adhesiveness_category` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2596-2601`, `PrintConfig.hpp:1320`, and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible `ConfigOptionInts` header serialization path, emits configured non-negative integer-vector values as `; filament_adhesiveness_category = ...`, rejects invalid values before BTT thumbnail header suppression, and keeps material adhesion policy, build-plate/UI behavior, wipe-tower loading/unloading behavior, toolchange runtime behavior, full `append_full_config` exhaustive export, and movement/extrusion behavior deferred.

### 2026-06-23 Ordinary travel retract lift-enforce runtime slice

`retract_lift_enforce` now reaches concrete Ares ordinary travel Z-hop G-code through the source-cited Orca `PrintConfig.hpp:390-395`, `PrintConfig.cpp:5187-5200`, `GCode.cpp:7185-7187`, `GCode.cpp:7280-7370`, `GCode.cpp:7582-7637`, `GCode.hpp:580-582`, and `GCodeWriter.cpp:623-648` boundary. The slice keeps ordinary travel retract/travel/unretract behavior intact while gating the vertical lift by `All Surfaces`, `Top Only`, `Bottom Only`, and `Top and Bottom`, preserves the previous non-gap-fill top-solid role across `GapFill`, and composes with the existing `z_hop`, `retract_lift_above`, and `retract_lift_below` gates. Ironing-triggered top eligibility, toolchange/wipe/wipe-tower lift enforcement, support/internal-infill suppression, avoid-crossing behavior, non-vertical Orca lift types, and multi-extruder per-tool lift enforcement remain deferred.

### 2026-06-23 Ordinary travel Z-hop runtime slice

`z_hop`, `retract_lift_above`, and `retract_lift_below` now reach concrete Ares ordinary travel retraction G-code through the source-cited Orca `PrintConfig.cpp:5122-5147`, `GCode.cpp:7280-7435`, `GCode.cpp:7458-7580`, `GCode.cpp:7582-7637`, and `GCodeWriter.cpp:623-648,1084-1092` boundary. Long ordinary travel retractions now emit normal vertical lift after retract and restore before unretract, honor the lower/upper Z-hop gates, work for both E-axis and firmware retraction, and suppress stale previous-layer restore when a lifted ordinary travel retraction crosses a layer boundary. `z_hop_types`, `travel_slope`, slope/spiral/auto lift, avoid-crossing-perimeters, support/internal-infill exceptions, wipe, toolchange retraction, wipe tower travel, and multi-extruder Z-hop state remain deferred.

### 2026-06-23 Retraction minimum travel runtime slice

`retraction_minimum_travel` now reaches concrete Ares ordinary XY travel retraction G-code through the source-cited Orca `PrintConfig.cpp:5048-5054`, `GCode.cpp:7280-7435`, `GCode.cpp:7458-7595`, and `GCodeWriter.cpp:1004-1078` boundary. The slice parses the first single-extruder minimum-travel value, defaults to Orca's 2 mm threshold, emits retract before eligible long ordinary travel and unretract before the next print move, composes with existing retraction length/speed/restart-extra/firmware/E-axis behavior, and keeps layer-change retraction independent. Wipe, avoid-crossing-perimeters, support/internal-infill exceptions, ordinary-travel Z-hop/lift type selection, toolchange retraction, wipe tower travel, and multi-extruder retraction state remain deferred.

### 2026-06-23 Draft shield zero-loop skirt runtime slice

`draft_shield` now consumes Orca's zero-loop skirt override through the source-cited `PrintConfig.cpp::get_real_skirt_dist`, `Print::has_infinite_skirt`, and `GCode::generate_skirt` boundary. When draft shield is enabled and `skirt_loops = 0`, Ares' existing combined-skirt generator uses one effective loop, so the behavior reaches skirt artifacts, print paths, extrusion/speed moves, diagnostics, and final G-code. Disabled draft shield with zero loops remains silent, positive loop counts remain unchanged, and existing `single_loop_draft_shield` / `min_skirt_length` behavior is preserved. Full Orca convex-hull/object-height/support-layer/wipe-tower/per-object/multi-extruder/sequential-print draft-shield parity remains deferred.

### 2026-06-23 Extruder printable height validation runtime slice

`extruder_printable_height` now reaches concrete Ares layer-height validation through the source-cited Orca `PrintConfig.hpp` FDM `PrintConfig` tuple, `PrintConfig.cpp` option definition, `Print.cpp::get_extruder_printable_height`, `PrintObject::detect_extruder_geometric_unprintables`, and `GCodeProcessor` per-extruder max-Z validation boundary. The slice enforces the first configured extruder height as a stricter limit than global `printable_height` in Ares' current single-active-extruder path, treats missing/null/zero as no per-extruder override, rejects invalid runtime values, and preserves global `printable_height` fallback. Full Orca multi-extruder geometric unprintable-filament detection, per-filament maps, `extruder_printable_area`, object-specific diagnostics, timelapse liftable-extruder behavior, and SLA printable-height behavior remain deferred.

### 2026-06-23 Printable height validation runtime slice

`printable_height` now reaches concrete Ares slicing validation through the source-cited Orca `PrintConfig.cpp` option definition, `PrintConfig.hpp` FDM `PrintConfig` ownership boundary, `Print.cpp` generated-layer height check, and `GCodeProcessor.cpp` path `max_print_z` validation boundary. The slice rejects planned layer `print_z` above the configured machine height before G-code output, accepts numeric and numeric-string values including equality at the limit, preserves existing `[max_print_height]` placeholder rounding, and keeps `extruder_printable_height`, bed/extruder area validation, localized object error reporting, shrinkage-compensation-specific messages, and SLA printable-height behavior deferred.

### 2026-06-23 Machine start stat reserved placeholders runtime slice

`print_time_sec` and `used_filament_length` now reach concrete Ares `file_start_gcode` and `machine_start_gcode` output through the source-cited Orca `GCode.cpp:2524-2525`, `GCode.cpp:3079-3082`, and `GCode/GCodeProcessor.cpp:58-79,1108-1140` boundary. Ares still maps user placeholders to Orca-style `@PRINT_TIME_SEC@` and `@USED_FILAMENT_LENGTH@` reserved tags first, then post-processes final G-code before line numbering so those tags become two-decimal print seconds and used filament meters from the finalized Ares speed/extrusion moves. Full Orca `GCodeProcessor` time-estimation parity, multi-extruder/per-filament statistics, full placeholder parser parity, public option storage/export, UI/preset behavior, and movement/temperature-command changes remain deferred.

### 2026-06-23 Bed temperature initial layer vector placeholder runtime slice

`bed_temperature_initial_layer_vector` now reaches concrete Ares `machine_start_gcode` placeholder output through the source-cited Orca `GCode.cpp:2996-3000` and `GCode.cpp:3082-3101` boundary. The slice renders `[bed_temperature_initial_layer_vector]` as an empty string to match Orca's current `ConfigOptionString()` parser value, composes with existing bed-temperature placeholders, and keeps any future non-empty vector semantics, full placeholder parser parity, public option storage/export, UI/preset behavior, and movement/temperature-command changes deferred.

### 2026-06-23 BBL bed temperature placeholder runtime slice

`bbl_bed_temperature_gcode` now reaches concrete Ares `machine_start_gcode` placeholder output through the source-cited Orca `PrintConfig.hpp:1353-1355` and `GCode.cpp:2996,3082-3101` boundary. The slice renders `[bbl_bed_temperature_gcode]` as `0` to match Orca's current hard-coded placeholder value, keeps user-supplied option input from changing that placeholder, composes with existing bed-temperature placeholders, and keeps full option storage/export semantics, any future true branch, bed-temperature formula expansion, expression parsing, UI/preset behavior, and movement/temperature-command changes deferred.

### 2026-06-23 Project filament colour header runtime slice

`filament_multi_colour` and `filament_colour_new` now reach concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2385-2390`, `PrintConfig.hpp:1608-1612`, and `GCode.cpp:5523-5575` boundary. The slice reuses existing Orca-compatible `ConfigOptionStrings` and `ConfigOptionFloats` header serialization, emits configured values as `; filament_multi_colour = ...` and `; filament_colour_new = ...`, rejects invalid values before BTT thumbnail header suppression, and keeps full `append_full_config` exhaustive export, UI project-filament color semantics, calculated-before-slicing color use, flush matrix correction, wipe tower/tool-change behavior, and movement/extrusion behavior deferred.

### 2026-06-22 First layer height placeholder runtime slice

`initial_layer_print_height` now reaches concrete Ares `machine_start_gcode` output through the source-cited Orca `GCode.cpp` `first_layer_height` placeholder setup, `PrintConfig.cpp` `initial_layer_print_height` option definition, `PrintConfig.hpp` `PrintConfig` ownership boundary, and `PrintConfig.cpp` custom-placeholder definition. The slice renders `[first_layer_height]` with Orca's default `0.2`, accepts numeric-string values, composes with existing machine-start placeholders, preserves literal `[first_layer_height]` outside the machine-start scope, rejects invalid placeholder values before G-code output, and keeps layer-planning migration to `initial_layer_print_height`, percentage forms, adjacent `max_print_height`/model/plate/temperature placeholders, brace expression parsing, and full Orca placeholder parser parity deferred.

### 2026-06-22 Z offset placeholder runtime slice

`z_offset` now reaches concrete Ares `machine_start_gcode` output through the source-cited Orca `GCode.cpp` placeholder setup, `PrintConfig.cpp` `z_offset` option definition, and `PrintConfig.hpp` `PrintConfig` ownership boundary. The slice renders `[z_offset]` from the existing `SliceOptions::z_offset()` parser with Orca's default `0`, accepts negative and numeric-string values, composes with existing machine-start placeholders, preserves literal `[z_offset]` outside the machine-start scope, and keeps adjacent `max_print_height`, model/plate/first-layer placeholders, brace expression parsing, exact Orca serialization punctuation, and generated Z-move behavior changes deferred.

### 2026-06-22 Retract length placeholder runtime slice

`retract_length` now reaches concrete Ares `machine_start_gcode` output through the source-cited Orca `GCode.cpp` placeholder setup, `PrintConfig.cpp` `retraction_length` option definition, and `PlaceholderParser.cpp` vector expansion boundary. The slice renders `[retract_length]` from the effective first/current `retraction_length` value with Orca's default `0.8`, composes with existing machine-start placeholders, preserves literal `[retract_length]` outside the machine-start scope, and keeps explicit vector indexing, expression parser parity, current-tool selection beyond Ares' initial extruder scope, tool-change retract placeholders, filament override routing, wipe tower/tool changes, and generated retraction behavior changes deferred.

### 2026-06-22 Num extruders placeholder runtime slice

`num_extruders` now reaches concrete Ares `machine_start_gcode` output through the source-cited Orca `GCode.cpp` placeholder setup and `PrintConfig.cpp` custom-placeholder boundary. The slice renders `[num_extruders]` from the effective `nozzle_diameter` vector length already parsed into `HardwareOptions`, composes with existing machine-start placeholders, and keeps initial/current tool placeholders, support/non-support filament placeholders, hotend mapping, `is_extruder_used`, `retract_length`, tool changes, wipe tower behavior, layer-change scope expansion, vector indexing, expression parsing, and full Orca placeholder parser parity deferred.

### 2026-06-22 Total layer count placeholder runtime slice

`total_layer_count` now reaches concrete Ares `machine_start_gcode` output through the source-cited Orca `GCode.cpp:2855`, `GCode.cpp:3079-3082`, and `PrintConfig.cpp:10927-10929` boundary. The slice renders `[total_layer_count]` from the actual planned `pipeline.layers().len()` before startup temperature suppression and keeps layer-change, time-lapse, role-change, end-G-code, file-start, filament-start, filament-end, expression parsing, vector indexing, sequential object placeholders, wipe tower/tool-change placeholders, and full Orca placeholder parser parity deferred.

### 2026-06-22 Fan speed-up time runtime slice

`fan_speedup_time` and `fan_speedup_overhangs` now reach concrete Ares part-cooling fan G-code through the source-cited Orca `PrintConfig.hpp:1311-1312`, `PrintConfig.cpp:3710-3727`, `GCode.cpp:3676-3684`, `GCode.cpp:3774-3782`, and `GCode/FanMover.cpp` boundary. The slice parses Orca's default disabled speed-up time and overhang-only gate, rejects invalid runtime values, and adds a bounded same-layer one-move lookahead so eligible bridge/overhang fan upshifts can be emitted before the previous same-layer generated move. Full Orca `FanMover` buffering, exact seconds-based move splitting, cross-layer movement, custom G-code fan commands, arcs, multi-extruder fan routing, toolchange/wipe-tower fan handling, and negative `fan_speedup_time` `D`-option behavior remain deferred.

### 2026-06-24 Filament retraction length override runtime slice

`filament_retraction_length` now reaches concrete Ares travel and layer-change retraction G-code through the source-cited Orca `PrintConfig.cpp:63-82`, `PrintConfig.cpp:5068-5075`, `Extruder.cpp:174-177`, and `GCodeWriter.cpp:1004-1048` boundary. The slice parses the first single-extruder runtime override value, lets the filament-prefixed length override the existing unprefixed `retraction_length`, preserves zero-length disable semantics, and keeps existing retraction speed, restart-extra, z-hop, wipe, firmware, and minimum-travel paths. Full Orca dynamic config merge, nullable `nil` fallback, multi-extruder selection, toolchange/cut/wipe-tower retractions, adjacent filament-prefixed retract options, seam/scarf behavior, and full Orca `GCode::retract` orchestration remain deferred.

### 2026-06-24 Filament retraction speed override runtime slice

`filament_retraction_speed` and `filament_deretraction_speed` now reach concrete Ares travel and layer-change retraction G-code through the source-cited Orca `PrintConfig.cpp:63-84`, `PrintConfig.cpp:5322-5337`, `PrintConfig.cpp:7167-7224`, `Extruder.cpp:184-198`, and `GCodeWriter.cpp:1004-1078` boundary. The slice parses first single-extruder runtime override values, lets filament-prefixed speeds override the existing unprefixed retraction speeds, preserves zero deretraction fallback to the effective retraction speed, and keeps the existing G-code writer feedrate path. Full Orca dynamic config merge, nullable `nil` fallback, multi-extruder selection, toolchange/cut/wipe-tower retractions, lift-type variants, seam/scarf behavior, and full Orca `GCode::retract` orchestration remain deferred.

### 2026-06-23 Retract lift enforce layer-change runtime slice

`retract_lift_enforce` now reaches concrete Ares layer-change Z-hop G-code through the source-cited Orca `PrintConfig.hpp:390-395`, `PrintConfig.cpp:534-540`, `PrintConfig.cpp:5187-5200`, `GCode.cpp:5622-5628`, `GCode.cpp:7606-7634`, `GCode.hpp:580-582`, and `GCodeWriter.cpp:623-644` boundary. The slice parses Orca's `All Surfaces`, `Top Only`, `Bottom Only`, and `Top and Bottom` enum strings, defaults to `All Surfaces`, validates malformed string lists before output, gates layer-change Z-hop by the previous non-gap-fill print role and first-layer transition, and preserves the existing `z_hop`, `retract_lift_above`, and `retract_lift_below` gates. Ordinary travel/toolchange/wipe retraction, multi-extruder per-tool enforcement, non-vertical Orca lift types, ironing-triggered lift enforcement, seam/scarf behavior, and full Orca `GCode::retract` orchestration remain deferred.

### 2026-06-22 Z-hop layer-change retraction runtime slice

`z_hop`, `retract_lift_above`, and `retract_lift_below` now reach concrete Ares layer-change retraction G-code through the source-cited Orca `PrintConfig.cpp:5122-5159`, `GCode.cpp:5622-5628`, `GCode.cpp:7606-7634`, `GCodeWriter.cpp:623-644`, and `GCodeWriter.cpp:1084-1090` boundary. The slice parses Orca's default `z_hop = 0.4`, emits normal vertical Z-hop lift / restore moves around pending layer-change unretract when the pre-change Z passes the lift gates, preserves `z_hop = 0` no-hop behavior, and applies to both E-axis and firmware layer-change retraction. `z_hop_types`, `travel_slope`, slope/spiral/auto lift, ordinary travel retraction, wipe, toolchange retraction, multi-extruder Z-hop selection, `retract_lift_enforce`, top/bottom-surface enforcement, seam/scarf behavior, spiral vase behavior, and full Orca `GCode::retract` orchestration remain deferred.

### 2026-06-22 Retract restart extra layer-change runtime slice

`retract_restart_extra` now reaches concrete Ares layer-change unretract G-code through the source-cited Orca `PrintConfig.cpp:5306-5313`, `GCodeWriter.cpp:1004-1078`, and `GCode/GCodeProcessor.cpp:4806-4817` boundary. The slice parses the first single-extruder restart-extra value, keeps layer-change retract distance at `retraction_length`, emits pending unretract distance as `retraction_length + retract_restart_extra`, preserves relative and absolute E state through `GCodeWriter`, and keeps `retract_restart_extra_toolchange`, wipe, z-hop, firmware retraction, long/nozzle-cut retraction, multi-extruder state, travel-minimum triggers, seam/scarf behavior, and full Orca `GCode::retract` orchestration deferred.

### 2026-06-22 Layer-change retraction runtime slice

`retract_when_changing_layer`, `retraction_length`, `retraction_speed`, and `deretraction_speed` now reach concrete Ares G-code output through the source-cited Orca `PrintConfig.cpp:5062-5074`, `PrintConfig.cpp:5322-5337`, `GCode.cpp:5625-5628`, and `GCodeWriter.cpp:1004-1078` boundary. The slice parses first single-extruder runtime values, emits configured retract/unretract E moves around non-first layer Z transitions, updates relative and absolute E state through `GCodeWriter`, and keeps z-hop, wipe, firmware/toolchange retraction, long/nozzle-cut retraction, multi-extruder state, travel-minimum triggers, seam/scarf behavior, and full Orca `GCode::retract` orchestration deferred.

### 2026-06-22 Filament colour type header runtime slice

`filament_colour_type` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.cpp:2388-2390` and `GCode.cpp:5523-5575` boundary. The slice reuses the existing Orca-compatible `ConfigOptionStrings` header serialization path, emits configured string-vector values as `; filament_colour_type = ...`, rejects invalid values before BTT thumbnail header suppression, and keeps `filament_multi_colour`, `filament_colour_new`, full `append_full_config` exhaustive export, UI gradient behavior, and movement/extrusion behavior deferred.

### 2026-06-22 Filament cooling before tower placeholder runtime slice

`filament_cooling_before_tower` now reaches concrete Ares `machine_start_gcode` placeholder output through the source-cited Orca `GCode.cpp:2841-2853`, `PrintConfig.hpp:1444`, `PrintConfig.cpp:2689-2695`, and `Config.hpp:879-915` boundary. The slice renders `[filament_cooling_before_tower]` with the upstream default `10`, accepts existing Ares numeric vector forms plus nullable `nil` entries, rejects invalid values with `SliceError::InvalidInput`, and keeps rendered custom start G-code in the startup temperature suppression path. Full wipe-tower/tool-change dynamic config, filament-count expansion, contact/first-layer zero fill, cooling moves, purge path generation, and full Orca placeholder expression parity remain deferred.

### 2026-06-22 Flush placeholders start G-code runtime slice

Ported the `filament_flush_volumetric_speed` and `filament_flush_temp` placeholder path from `OrcaSlicer/src/libslic3r/GCode.cpp:2841-2853`, with option-definition context from `PrintConfig.hpp:1343-1344`, `PrintConfig.cpp:2442-2460`, `PrintConfig.cpp:2462-2470`, and `PrintConfig.cpp:6495-6501`, into `ares-core` machine-start G-code rendering. Ares now resolves zero flush speeds through `filament_max_volumetric_speed`, resolves zero flush temperatures through `nozzle_temperature_range_high`, renders `[flush_volumetric_speeds]` and `[flush_temperatures]`, validates runtime vector inputs, and keeps full wipe-tower/tool-change flushing plus `filament_cooling_before_tower` deferred.

### 2026-06-22 Filament and printer notes header runtime slice

`filament_notes` and `printer_notes` now reach concrete Ares G-code header comments through the source-cited Orca `PrintConfig.hpp:1631-1634` / `PrintConfig.cpp:2375-2382` / `PrintConfig.cpp:4963-4970` / `GCode.cpp:5523-5575` boundary. The slice extends the existing `notes` header path, validates scalar printer notes and string-vector filament notes, splits multiline note text into repeated header comments, follows BTT thumbnail header suppression, and preserves movement/extrusion commands. Full Orca config-block serialization, exact note quoting parity, BBL/non-BBL config-block placement, UI note editing, multi-extruder note mapping beyond valid vector preservation, and Prusa XL detection remain deferred.

### 2026-06-22 Filament cost statistics runtime slice

`filament_cost` now reaches concrete Ares footer statistics through the source-cited Orca `PrintConfig.hpp:1330` / `PrintConfig.cpp:2837-2843` / `GCode.cpp:2279-2343` / `GCode.cpp:3471-3488` boundary. The slice formats single-extruder used filament length, volume, optional weight, and optional cost from generated extrusion totals, the first effective filament diameter, `filament_density`, and `filament_cost`, emitting the report before final `M2`. Multi-extruder zero filling, wipe-tower accounting, persistent `PrintStatistics`, UI statistics, non-BBL total footer lines, estimated-time placeholders, and full config-block output remain deferred.

### 2026-06-22 Temperature vitrification start G-code runtime slice

`temperature_vitrification` now reaches concrete Ares `machine_start_gcode` placeholder output through the source-cited Orca `PrintConfig.hpp:1332` / `PrintConfig.cpp:2828-2835` / `GCode.cpp:2982-3004` boundary. The slice renders `[min_vitrification_temperature]` with the upstream default `100`, parses Orca-style non-negative integer vector forms, uses the minimum provided softening temperature, rejects invalid values with `SliceError::InvalidInput`, and keeps the rendered custom start string in the existing startup suppression path. Full Orca placeholder parser parity, brace-form start placeholders, writer-extruder `ConfigOptionInts::get_at` fallback semantics, nearby bed/chamber/temperature placeholders, UI door/glass guidance, and filament-cost statistics remain deferred.

### 2026-06-22 Filament density header runtime slice

`filament_density` now reaches concrete Ares G-code header output through the source-cited Orca `PrintConfig.hpp` / `PrintConfig.cpp` / `GCode.cpp:2570-2572` boundary. The slice reuses Ares' existing Orca-style numeric vector parsing, emits `; filament_density = ...` beside `filament_diameter`, preserves the upstream default `0`, rejects non-finite and negative density values at G-code formatting time, and keeps movement/extrusion commands unchanged. Orca statistics weight/cost calculations, wipe-tower material accounting, UI statistics, exact colon-form `ConfigOptionFloats::serialize()` header punctuation, and full config-block generation remain deferred.

### 2026-06-22 Profile notes header runtime slice

`notes` now reaches concrete Ares G-code header comments through the source-cited Orca `PrintConfig.hpp` / `PrintConfig.cpp` / `GCode.cpp::append_full_config` boundary. Missing or empty notes stay silent, non-empty notes emit line-wise `; notes = ...` comments, invalid non-string values are rejected at G-code formatting time, and BTT thumbnail header suppression continues to skip all header notes. Full Orca config-block serialization, `filament_notes`, `printer_notes`, BBL-specific header/config blocks, UI/profile-editor behavior, and exact `ConfigOptionString` escaping parity remain deferred.

### 2026-06-21 Travel acceleration G-code runtime slice

`travel_acceleration` and `initial_layer_travel_acceleration` now reach Orca-style separate travel acceleration G-code through the source-cited `PrintConfig.hpp` / `GCode.cpp::travel_to` / `GCodeWriter.cpp::set_acceleration_internal` boundary. Supported separate-travel flavors emit `M204 T` for Marlin2/RepRapFirmware travel, `M202` for Repetier travel, and keep print acceleration on `M204 P` / `M201`; Marlin legacy and Klipper behavior remain unchanged. Full Orca travel planning, short-travel role-specific acceleration, machine travel acceleration clamps, avoid-crossing-perimeter detours, retraction/wipe interactions, and multi-extruder/wipe-tower integration remain deferred.

### 2026-06-21 Adaptive bed mesh placeholder runtime slice

`bed_mesh_min`, `bed_mesh_max`, `bed_mesh_probe_distance`, and `adaptive_bed_mesh_margin` now reach concrete `machine_start_gcode` placeholder output through the source-cited Orca `PrintConfig.hpp` / `PrintConfig.cpp` / `GCode.cpp:2871-2963` boundary. The slice computes adaptive mesh min/max from Ares' current first-layer print paths, applies Orca-style margin and min/max clamps, renders probe counts plus `bed_mesh_algo`, and uses one rendered machine-start string for both startup suppression checks and final emission. Full Orca placeholder parser parity, first-layer convex hull placeholders, head-wrap detection, max-print-Z placeholders, wipe tower/support/multi-object hull handling, calibration-mode bbox behavior, and automatic bed probing G-code remain deferred.

### 2026-06-21 Auxiliary fan layer ramp runtime slice

`close_additional_fan_first_x_layers` and `additional_fan_full_speed_layer` now reach concrete Ares auxiliary fan `M106 P2` G-code through the source-cited Orca `PrintConfig.hpp` / `PrintConfig.cpp` / `GCode/CoolingBuffer.cpp` / `GCodeWriter.cpp` boundary. The slice uses the auxiliary-specific first-layer suppression key, applies Orca's documented linear ramp to `additional_cooling_fan_speed`, preserves default `M106 P2` output and shutdown behavior, and keeps Klipper auxiliary fan suppression unchanged. `first_x_layer_fan_speed`, full Orca `CoolingBuffer` multi-extruder state, force-resume marker placement, support/ironing marker interactions, and custom G-code post-processor placement remain deferred.

### 2026-06-21 Fan kickstart runtime slice

`fan_kickstart` now maps to concrete Ares part-cooling fan G-code through the source-cited Orca `PrintConfig.hpp` / `PrintConfig.cpp` / `GCode.cpp` / `GCode/FanMover.cpp` / `GCodeWriter.cpp` boundary. The slice parses the non-negative seconds value, emits a 100% `M106` kickstart pulse for generated part-cooling fan upshifts above 10 percentage points, restores the target after Ares emitted-move time reaches Orca's scaled duration formula, and preserves existing fan flavor plus `part_cooling_fan_min_pwm` formatting. Full Orca `FanMover` command reordering, G1/G0 splitting, fan-speedup-time scheduling, overhang-only speedup filtering, custom G-code M106 post-processing, multi-extruder fan routing, wipe-tower fan handling, and Bambu-specific fan addressing remain deferred.

### 2026-06-21 Small-area infill flow compensation runtime slice

`small_area_infill_flow_compensation` and `small_area_infill_flow_compensation_model` now reach concrete Ares extrusion E deltas through the source-cited Orca `PrintConfig.hpp` / `PrintConfig.cpp` / `GCode/SmallAreaInfillFlowCompensator.cpp` / `GCode.cpp::_needSAFC` / `GCode/PchipInterpolatorHelper.cpp` boundary. The slice parses Orca's default model, validates explicit string-list or serialized model forms, applies the PCHIP multiplier only to supported short `solid_infill`, `top_solid_infill`, and `bottom_surface` segments behind the rectilinear/monotonic pattern gate, and leaves geometry, ordering, speed, fan, acceleration, and jerk unchanged. Full Orca G-code writer path collection, path splitting/merging, multi-region/object/extruder context, support/ironing/scarf/seam/wipe-tower interactions, and Orca binary E2E parity remain deferred.

### 2026-06-21 Curled overhang slowdown runtime slice

`slowdown_for_curled_perimeters` now reaches concrete Ares overhang-perimeter speed selection through the source-cited Orca `PrintConfig.hpp` / `PrintConfig.cpp` / `GCode.cpp` dynamic overhang speed boundary. Within Ares' current whole-path unsupported-span approximation, the final severe-overhang bucket uses `overhang_4_4_speed` by default and switches to resolved `bridge_speed` when `slowdown_for_curled_perimeters = false`, before volumetric caps and layer-time slowdown. Full Orca `ExtrusionQualityEstimator`, exact 13% segment-level overlap classification, per-point previous-layer distance, path subdivision, bridge dynamic speed bands, scarf/sloped interactions, raft/object gates, and multi-object estimator state remain deferred.

### 2026-06-20 Overhang speed bands runtime slice

`overhang_1_4_speed`, `overhang_2_4_speed`, `overhang_3_4_speed`, and `overhang_4_4_speed` now reach concrete Ares overhang-perimeter speed selection through the source-cited Orca `PrintConfig.hpp` / `PrintConfig.cpp` / `GCode.cpp` / `GCode/ExtrusionProcessor.hpp` boundary. Ares uses its current whole-path unsupported-span estimate to select speed bands before volumetric caps and layer-time slowdown. Full Orca `ExtrusionQualityEstimator`, per-point overlap distance, path subdivision, curled-line slowdown, bridge dynamic speed bands, scarf/sloped interactions, raft checks, and multi-object estimator state remain deferred.

### 2026-06-20 Volumetric rate slope runtime slice

`max_volumetric_extrusion_rate_slope`, `max_volumetric_extrusion_rate_slope_segment_length`, and `extrusion_rate_smoothing_external_perimeter_only` now reach concrete Ares speed generation through the source-cited Orca `PrintConfig.cpp` / `PrintConfig.hpp` / `GCode/PressureEqualizer.cpp` boundary. This slice parses the Orca defaults and bounds, carries the values through `SpeedOptions`, and applies a forward positive print-flow jump limiter before layer-time slowdown. Full Orca `PressureEqualizer` G-code parsing/rewriting, negative-rate smoothing, move subsegment splitting, future/backward passes, bridge/ironing exclusions beyond current Ares roles, multi-extruder smoothing state, and binary E2E parity remain deferred.

### 2026-06-20 Fan cooling layer time runtime slice

`fan_cooling_layer_time` now maps to concrete part-cooling fan G-code through the source-cited Orca `PrintConfig.cpp` / `PrintConfig.hpp` / `GCode/CoolingBuffer.cpp` boundary. The slice parses the first configured single-extruder value, computes layer time from finalized Ares speed moves, interpolates the baseline `M106` fan command between `fan_max_speed` and `fan_min_speed` for short layers, and preserves first-layer fan suppression plus full-fan-layer ramp behavior. Full Orca `CoolingBuffer`, `reduce_fan_stop_start_freq`, multi-extruder fan ranges, support-interface/ironing/custom cooldown markers, arcs, wipes, and wipe-tower cooling remain deferred.

### 2026-06-20 Wall infill order infill-first runtime slice

Legacy `wall_infill_order` values beginning with `infill/` now consume the upstream Orca `Config.cpp` infill-first migration by deriving `is_infill_first = true` while preserving the existing `wall_sequence` migration. This makes legacy profiles reach Ares' existing first-layer wall-first and later-layer infill-first print-path/G-code ordering. Full Orca multi-region ordering, wipe-tower/tool-change interactions, object scheduling, and additional `wall_infill_order` UI/preset semantics remain deferred.

### 2026-06-20 Pressure advance G-code runtime slice

`enable_pressure_advance` and `pressure_advance` now map to concrete first-filament startup G-code through the source-cited Orca `PrintConfig.cpp` / `PrintConfig.hpp` / `GCodeWriter.cpp` / `GCode.cpp` boundary. The slice parses the first enable/value pair, emits the active-flavor PA command after machine/filament start G-code, and preserves disabled/default behavior. Adaptive PA, BBL-specific printer detection, PA reset processor state, tool-change PA, calibration modes, and wipe-tower/nozzle-change integration remain deferred.

### 2026-06-20 Bed exclude area runtime slice

`bed_exclude_area` now maps to concrete current-model XY exclusion validation through the source-cited Orca `PrintConfig.cpp` / `PrintConfig.hpp` / `Print.cpp` boundary. The slice parses Orca point strings and JSON point arrays, treats the default `0x0` as inactive, computes the loaded STL model XY bounds, and rejects slices whose bounds intersect the configured exclusion polygon before layer planning. Full Orca `get_bed_shape_with_excluded_area` polygon boolean behavior, sequential/by-object clearance, filament-change cutter routing, `printable_area`/`extruder_printable_area`, plate-origin shifts, UI/preset behavior, and multi-object placement remain deferred.

### 2026-06-20 Infill wall overlap runtime slice

`infill_wall_overlap` and `top_bottom_infill_wall_overlap` now map to concrete rectangle-only Ares infill clipping behavior through the source-cited Orca `ConfigOptionPercent` / `PerimeterGenerator.cpp` boundary. The slice stores raw percent values, computes overlap against the current rectangular wall/infill runtime reference, uses top/bottom overlap on first/topmost/top-bottom solid surfaces, and changes downstream print paths plus G-code comments. Full Orca `fill_surfaces`, `fill_no_overlap`, polygon offsets, holes, multiple islands, Arachne, and alternate-path overlap-base parity remain deferred.

### 2026-06-19 Internal bridge density runtime slice

Non-default `internal_bridge_density` now maps to concrete Ares dense middle-layer internal bridge scanline spacing and `InternalBridge` G-code path counts through the source-cited Orca `PrintConfig.cpp` / `Surface.hpp` / `Fill.cpp` / `FillRectilinear.cpp` boundary. Defaults preserve existing middle-layer `solid_infill` line counts and role, while lower densities route the affected dense middle output through the existing `InternalBridge` downstream speed, flow, fan, extrusion, and G-code role. Full Orca `SurfaceCollection` ownership, `stSecondInternalBridge`, `enable_extra_bridge_layer`, `dont_filter_internal_bridges`, automatic bridge detection, support-aware ownership, and per-island internal bridge classification remain deferred.

### 2026-06-19 Internal bridge angle runtime slice

`internal_bridge_angle` now maps to concrete Ares internal bridge scanline direction through the source-cited Orca `PrintConfig.cpp` / `PrintObject.cpp` / `Surface.hpp` / `FillBase.cpp` boundary. The slice reuses Ares' current non-default `internal_bridge_density < 100` dense middle-layer internal bridge boundary: positive values remain fixed angle overrides, and `internal_bridge_angle = 0` now uses bounded automatic direction from the combined current bridge-contour bounds while preserving square or degenerate bounds. True Orca anchor-line `determine_bridging_angle(...)` scoring, `SurfaceCollection` ownership, `stSecondInternalBridge`, `enable_extra_bridge_layer`, and support-aware bridge classification remain deferred.

### 2026-06-19 CrossZag sparse infill runtime slice

`sparse_infill_pattern = "crosszag"` now maps to concrete Ares sparse infill generation through the source-cited Orca `FillCrossZag` / `FillRectilinear::fill_surface_by_lines` boundary. The slice also consumes `infill_shift_step` for CrossZag scanline placement and extends `symmetric_infill_y_axis` to CrossZag. `lockedzag` remains deferred until the upstream `FillLockedZag::fill_surface_locked_zag` skin/skeleton lock-region boundary is ported.

### 2026-06-19 External bridge angle runtime slice

`bridge_angle` now maps to concrete Ares external bottom bridge infill direction through the source-cited Orca `PrintConfig.cpp` / `LayerRegion.cpp` / `BridgeDetector.hpp` / `FillBase.cpp` boundary. The slice consumes positive `bridge_angle` values and the bounded `bridge_angle = 0` automatic direction for the existing `bridge_no_support` fully unsupported bottom-surface path, using the combined current bridge-contour bounds to choose the long-span scanline direction while preserving square or degenerate bounds. Full Orca bridge detector coverage scoring, anchor-region polygon difference, `internal_bridge_angle`, support generation, and mixed per-surface bridge classification remain deferred.

### 2026-06-19 External bridge density runtime slice

`bridge_density` now maps to concrete Ares external bottom bridge infill spacing through the source-cited Orca `PrintConfig.cpp` / `Fill.cpp` / `FillRectilinear.cpp` boundary. The slice consumes non-default bridge densities only for the existing `bridge_no_support` fully unsupported bottom-surface path, composes with `bridge_angle`, and keeps final `PrintPathRole::Bridge` classification on the shared unsupported-layer predicate. `internal_bridge_density`, full per-surface bridge ownership, bridge detector parity, support generation, and bridge-flow spacing parity remain deferred.

### 2026-06-19 Surface density runtime slice

`top_surface_density` and `bottom_surface_density` now drive concrete Ares top/bottom surface scanline spacing and G-code line counts through the source-cited Orca `PrintConfig.cpp` / `FillRectilinear.cpp` boundary. Defaults preserve current solid-surface output; `top_surface_density=0` suppresses top surface infill paths in the existing wall/infill split; fully unsupported bottom bridges keep `bridge_density` precedence. Full Orca `SurfaceCollection` ownership, mixed per-island density surfaces, internal bridge density, ironing, and support-density behavior remain deferred.

## M1: Basic slicer framework skeleton
Create the two-crate Rust workspace under `crates/`, define the first platform-neutral core API, keep file I/O in the CLI adapter, preserve all Orca options dynamically, and document crate boundaries.

Exit criteria are tracked in `docs/milestones/m1-basic-slicer-framework.md`.

## M2: STL model import
Parse STL bytes inside `ares-core` into model triangles, keep filesystem access in adapters, and surface imported model metadata through placeholder G-code. 3MF archive/XML geometry extraction is deferred to a later model-import milestone.

Exit criteria are tracked in `docs/milestones/m2-stl-model-import.md`.

## M3: Layer planning and first typed options
Map the first Orca-compatible options (`layer_height`, `initial_layer_height`) into typed accessors, compute STL model Z bounds, and generate deterministic layer plans plus layer-aware G-code metadata.

Exit criteria are tracked in `docs/milestones/m3-layer-planning.md`.

## M4: XY slice segments
Intersect imported STL triangles with planned layer Z planes and expose deterministic per-layer XY line segments through the core API and CLI output.

Exit criteria are tracked in `docs/milestones/m4-xy-slice-segments.md`.

## M5: Contour stitching
Stitch deterministic per-layer XY slice segments into simple closed contours, rejecting open or branching graphs until polygon repair milestones handle them explicitly.

Exit criteria are tracked in `docs/milestones/m5-contour-stitching.md`.

## M6: Hardware option typing
Map the first OrcaSlicer machine/filament vector options (`nozzle_diameter`, `filament_diameter`, `min_layer_height`, `max_layer_height`) into typed Rust accessors while preserving unknown keys.

Exit criteria are tracked in `docs/milestones/m6-hardware-option-typing.md`.

## M7: Profile fragment inheritance
Resolve OrcaSlicer-style in-memory process, filament, and machine profile fragments into `SliceOptions` through deterministic same-kind inheritance chains.

Exit criteria are tracked in `docs/milestones/m7-profile-fragment-inheritance.md`.

## M8: Full profile composition and option groups
Compose process, filament, and machine profile groups into full print configs while preserving unknown keys and preparing substitution/compatibility behavior.

Exit criteria are tracked in `docs/milestones/m8-full-profile-composition-and-option-groups.md`.

Task 20A.1 supersedes only the old M7/M8 dynamic contracts: profile merging
and composition now reject unknown keys and return concrete typed owners rather
than preserving an unknown-value map or returning `SliceOptions`. The milestone
entries remain as historical sequencing records; profile management and later
compatibility behavior are still deferred to their source-cited tasks.

## M9: Slicing pipeline diagnostics
Expose the current model import, layer planning, XY segment, and contour stitching stages as a reusable in-memory pipeline result with deterministic diagnostics.

Exit criteria are tracked in `docs/milestones/m9-slicing-pipeline-diagnostics.md`.

## M10: Perimeter path generation
Generate first external perimeter path artifacts from closed contours, starting with simple single-island contours before full polygon repair and offset parity.

Exit criteria are tracked in `docs/milestones/m10-perimeter-path-generation.md`.

## M11: Infill path generation
Generate first deterministic sparse rectilinear infill path artifacts inside simple contour boundaries before fill-surface, solid-infill, and extrusion milestones.

Exit criteria are tracked in `docs/milestones/m11-infill-path-generation.md`.

## M12: Ordered print path artifacts
Combine perimeter and sparse infill artifacts into deterministic layer-level print path artifacts using Orca-compatible wall/infill ordering before extrusion planning and support/skirt/brim milestones.

Exit criteria are tracked in `docs/milestones/m12-ordered-print-path-artifacts.md`.

## M13: Toolpath move emission
Convert ordered print path artifacts into deterministic travel/print moves and emit the first path-following `G0`/`G1` XY commands before extrusion values and speed planning.

Exit criteria are tracked in `docs/milestones/m13-toolpath-move-emission.md`.

## M14: Extrusion value emission
Attach deterministic absolute filament `E` values to current print moves using role line widths, layer heights, and filament diameter before speed and retraction planning.

Exit criteria are tracked in `docs/milestones/m14-extrusion-value-emission.md`.

## M15: Speed / feedrate emission
Attach deterministic movement speeds to current travel/print moves and emit first `F` feedrate values before support, retraction, acceleration, and full G-code parity.

Exit criteria are tracked in `docs/milestones/m15-speed-feedrate-emission.md`.

## M16: Skirt path emission
Generate deterministic first skirt path artifacts around current contours and route them through existing print path, move, extrusion, speed, and G-code output stages.

Exit criteria are tracked in `docs/milestones/m16-skirt-path-emission.md`.

## M17: Brim path emission
Generate deterministic first-layer brim path artifacts around current contours and route them through existing print path, move, extrusion, speed, and G-code output stages.

Exit criteria are tracked in `docs/milestones/m17-brim-path-emission.md`.

## M18: Bridge option and role scaffold
Add typed Orca bridge options plus bridge print-role flow and speed behavior before bridge geometry detection is implemented.

Exit criteria are tracked in `docs/milestones/m18-bridge-option-role-scaffold.md`.

## M19: libslic3r/libvgcode architecture alignment
Realign Ares around a Rust rewrite of OrcaSlicer's `libslic3r` and `libvgcode` boundaries before more slicing feature work. Inventory current modules, record the non-negotiable port boundary, and reject independent pipeline design.

Exit criteria are tracked in `docs/milestones/m19-libslic3r-libvgcode-architecture-alignment.md`.

## M20: libslic3r crate boundary foundation
Study `OrcaSlicer/src/libslic3r` and `OrcaSlicer/src/libvgcode`, record the Rust workspace crate boundary decision, keep active crates limited to `ares-core` and `ares-cli`, and preserve the simple async core slicing API plus CLI STL slicing command.

Exit criteria are tracked in `docs/milestones/m20-libslic3r-geometry-model-config-boundary-port.md`.

## M21: libslic3r print domain foundation
After the crate-boundary foundation, port the first behavior-preserving `libslic3r` print domain concepts from `Surface.hpp`, `ExtrusionEntity.hpp`, `ExtrusionEntityCollection.hpp`, `Layer.hpp`, and `Print.hpp` into `ares-core` while preserving the simple byte slicing API and current G-code output.

Exit criteria are tracked in `docs/milestones/m21-libslic3r-print-layer-region-extrusion-entity-boundary-port.md`.

## M22: PrintConfig option registry foundation
Port the first `libslic3r::PrintConfig` option-definition boundary into `ares-core` by adding source-cited metadata for currently parsed options, preparing later milestones to cover all OrcaSlicer options incrementally.

Exit criteria are tracked in `docs/milestones/m22-libslic3r-gcode-writer-planner-parity.md`.

## M23: libvgcode rendering-neutral G-code data model port
Create `crates/ares-vgcode` and port the first rendering-neutral data concepts from `OrcaSlicer/src/libvgcode`, including shared type vocabulary, `PathVertex`, `GCodeInputData`, `ColorPrint`, `Range`, `ViewRange`, and `Layers`, while deferring OpenGL/viewer runtime, parser runtime, `ExtrusionRoles.*` display metadata, and the full `ColorRange.*` object.

Exit criteria are tracked in `docs/milestones/m23-libvgcode-rendering-neutral-gcode-data-model-port.md`.

## M24: libslic3r GCodeWriter boundary
Port the first platform-neutral movement writer boundary from `OrcaSlicer/src/libslic3r/GCodeWriter.*` into `ares-core` and route existing executable G-code movement commands through that upstream-aligned writer without adding new Ares pipeline stages.

Exit criteria are tracked in `docs/milestones/m24-libslic3r-gcode-writer-boundary.md`.

## M25: WASM/browser API over rewritten core boundaries
Expose the already rewritten `libslic3r`-aligned core byte slicing API and rendering-neutral `libvgcode` data boundary through a browser WASM adapter crate without native filesystem, terminal, UI, OpenGL, or independent pipeline assumptions.

Exit criteria are tracked in `docs/milestones/m25-wasm-browser-api-over-rewritten-core-boundaries.md`.

## M26: PrintConfig common params option registry
Port the first common `libslic3r::PrintConfigDef::init_common_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:672-782` into `ares-core` registry metadata while deferring typed behavior.

Exit criteria are tracked in `docs/milestones/m26-print-config-common-params-registry.md`.

## M27: PrintConfig physical printer option registry
Port the physical-printer common `libslic3r::PrintConfigDef::init_common_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:786-894` into `ares-core` registry metadata while deferring typed behavior and network/UI integrations.

Exit criteria are tracked in `docs/milestones/m27-print-config-physical-printer-registry.md`.

## M28: PrintConfig FFF travel avoidance option registry
Port the first FFF-specific `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:897-921` into `ares-core` registry metadata while deferring typed behavior and travel-planning implementation.

Exit criteria are tracked in `docs/milestones/m28-print-config-fff-travel-avoidance-registry.md`.

## M29: PrintConfig bed temperature other-layers option registry
Port the FFF other-layer bed temperature `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:923-982` into `ares-core` registry metadata while deferring typed behavior and temperature G-code implementation.

Exit criteria are tracked in `docs/milestones/m29-print-config-bed-temp-other-layers-registry.md`.

## M30: PrintConfig bed temperature initial-layer option registry
Port the FFF first-layer bed temperature `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:984-1041` into `ares-core` registry metadata while deferring typed behavior and temperature G-code implementation.

Exit criteria are tracked in `docs/milestones/m30-print-config-bed-temp-initial-layer-registry.md`.

## M31: PrintConfig bed type and filament sequence option registry
Port the FFF bed type and filament sequence `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1043-1108` into `ares-core` registry metadata while deferring typed behavior, bed-selection behavior, print-order behavior, and G-code implementation.

Exit criteria are tracked in `docs/milestones/m31-print-config-bed-type-sequence-registry.md`.

## M32: PrintConfig shell and gap-fill option registry
Port the FFF before-layer-change G-code, bottom shell, and gap-fill target `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1110-1168` into `ares-core` registry metadata while deferring typed behavior, G-code hook execution, bottom-shell behavior, and gap-fill behavior.

Exit criteria are tracked in `docs/milestones/m32-print-config-shell-gap-registry.md`.

## M33: PrintConfig overhang fan option registry
Port the FFF overhang/bridge fan `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1170-1211` into `ares-core` registry metadata while deferring typed behavior, cooling behavior, bridge-detection behavior, and fan G-code behavior.

Exit criteria are tracked in `docs/milestones/m33-print-config-overhang-fan-registry.md`.

## M34: PrintConfig bridge angle and density option registry
Port the FFF bridge angle and bridge density `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1213-1264` into `ares-core` registry metadata while deferring typed behavior, bridge planning, bridge density spacing, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m34-print-config-bridge-angle-density-registry.md`.

## M35: PrintConfig solid infill flow ratio option registry
Port the FFF top and bottom solid infill flow ratio `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1286-1305` into `ares-core` registry metadata while deferring typed behavior, flow planning, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m35-print-config-solid-infill-flow-registry.md`.

## M36: PrintConfig other-flow gate option registry
Port the FFF `set_other_flow_ratios` and `first_layer_flow_ratio` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1307-1323` into `ares-core` registry metadata while deferring typed behavior, runtime flow scaling, preset behavior, object override behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m36-print-config-other-flow-gate-registry.md`.

## M37: PrintConfig wall flow ratio option registry
Port the FFF `outer_wall_flow_ratio` and `inner_wall_flow_ratio` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1324-1343` into `ares-core` registry metadata while splitting the registry definition table and deferring typed behavior, runtime flow scaling, preset behavior, object override behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m37-print-config-wall-flow-ratio-registry.md`.

## M38: PrintConfig overhang and sparse infill flow ratio option registry
Port the FFF `overhang_flow_ratio` and `sparse_infill_flow_ratio` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1344-1363` into `ares-core` registry metadata. This historical registry milestone deferred typed behavior and runtime flow scaling when it was completed. A later source-cited overhang slice now consumes `overhang_flow_ratio` for fully unsupported rectangular overhang perimeter extrusion when `set_other_flow_ratios` is enabled; sparse infill flow scaling, preset behavior, object override behavior, and runtime behavior outside that overhang perimeter path remain deferred.

Exit criteria are tracked in `docs/milestones/m38-print-config-overhang-sparse-flow-ratio-registry.md`.

## M39: PrintConfig internal solid and gap-fill flow ratio option registry
Port the FFF `internal_solid_infill_flow_ratio` and `gap_fill_flow_ratio` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1364-1383` into `ares-core` registry metadata. This historical registry milestone deferred typed/runtime behavior when it was completed. Later source-cited slices now consume `internal_solid_infill_flow_ratio` for solid-infill extrusion when `set_other_flow_ratios` is enabled, consume constructed `gap_fill` print paths through `ExtrusionRole::GapFill`, `gap_fill_flow_ratio`, and `gap_infill_speed` in G-code, and consume `filter_out_gap_fill` by dropping constructed `gap_fill` paths shorter than the configured millimeter threshold before downstream moves/extrusion/G-code. Full gap geometry generation, `gap_fill_target`, classic/Arachne gap detection, solid-surface gap generation, preset behavior, object override behavior beyond the existing `set_other_flow_ratios` gate, and runtime behavior outside constructed gap-fill paths remain deferred.

Exit criteria are tracked in `docs/milestones/m39-print-config-internal-gap-flow-ratio-registry.md`.

## M40: PrintConfig support flow ratio option registry
Port the FFF `support_flow_ratio` and `support_interface_flow_ratio` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1384-1403` into `ares-core` registry metadata. This historical registry milestone deferred typed/runtime behavior when it was completed. Later source-cited slices now consume `support_flow_ratio` for constructed `PrintPathRole::SupportMaterial` extrusion and `support_interface_flow_ratio` for constructed `PrintPathRole::SupportMaterialInterface` extrusion when `set_other_flow_ratios` is enabled. Support generation, support transition roles, preset behavior, object override behavior beyond the existing `set_other_flow_ratios` gate, and runtime behavior outside constructed support-material/support-interface paths remain deferred.

Exit criteria are tracked in `docs/milestones/m40-print-config-support-flow-ratio-registry.md`.

## M41: PrintConfig one-wall quality option registry
Port the FFF `precise_outer_wall`, `only_one_wall_top`, `min_width_top_surface`, `only_one_wall_first_layer`, and `extra_perimeters_on_overhangs` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1404-1444` into `ares-core` registry metadata. This historical registry milestone deferred typed behavior and all one-wall runtime behavior when it was completed. Later source-cited slices now consume `only_one_wall_first_layer` and `only_one_wall_top` perimeter/G-code behavior, and a rectangle-only `extra_perimeters_on_overhangs` slice adds one inset overhang perimeter for fully unsupported rectangular contours. A later rectangle-only `precise_outer_wall` classic perimeter slice consumes Orca's width-vs-spacing gate for `InnerOuter`, uses rounded-rectangle line spacing for non-precise and non-InnerOuter rectangular loop offsets, and shares the next-loop spacing with the rectangle-only `extra_perimeters_on_overhangs` path. A later rectangle-only `min_width_top_surface` slice parses Orca's default `300%` over effective internal perimeter width and suppresses top-surface infill for rectangular top contours narrower than the threshold while preserving zero-threshold and non-rectangular current behavior. Arachne wall spacing and thresholds, full `Flow` parity, variable-width paths, full Orca top/non-top polygon splitting, Arachne top-surface threshold behavior, full overhang polygon clipping, fill-surface subtraction, Orca binary E2E parity, and runtime behavior not covered by those later slices remain deferred.

Exit criteria are tracked in `docs/milestones/m41-print-config-one-wall-quality-registry.md`.

## M42: PrintConfig overhang reversal option registry
Port the FFF `overhang_reverse`, `overhang_reverse_internal_only`, `counterbore_hole_bridging`, and `overhang_reverse_threshold` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1446-1498` into `ares-core` registry metadata. This historical registry milestone deferred typed/runtime behavior when it was completed. Later source-cited overhang-reversal slices now consume `overhang_reverse` for already-classified rectangular `PerimeterRole::Overhang` paths on zero-based odd layer ids, matching Orca's `PerimeterGenerator.cpp` even-GUI-layer gate and changing downstream print path, toolpath move, and G-code order. `overhang_reverse_internal_only` is consumed for rectangular multi-wall overhang contours by preserving external overhang path direction while reversing generated internal perimeter paths. `overhang_reverse_threshold` is consumed for Ares' current rectangular overhang-reversal boundary: it parses Orca's default `50%` over external perimeter width, accepts mm or percent values up to 20 mm, gates reversal for fully unsupported rectangular contours by comparing against `max(width,height)`, and is ignored when `detect_overhang_wall` is disabled so zero-based odd layers still reverse external-role paths. `counterbore_hole_bridging = sacrificiallayer` is consumed in Ares' bridge/infill/gap-fill classification: the unsupported bottom-surface bridge override is suppressed so the layer remains bottom-surface density/pattern, keeps solid-surface gap-fill eligibility, and emits final G-code role/speed as bottom surface; `partiallybridge` remains parsed and default-equivalent pending Orca counterbore geometry parity. Partial overhang clipping, full loop-role `reorient_perimeters` parity, Arachne, fuzzy skin, holes, supports, raft gates, and full `detect_steep_overhang` parity remain deferred.

Exit criteria are tracked in `docs/milestones/m42-print-config-overhang-reversal-registry.md`.

## M43: PrintConfig overhang speed option registry
Port the FFF `enable_overhang_speed`, `slowdown_for_curled_perimeters`, `overhang_1_4_speed`, `overhang_2_4_speed`, `overhang_3_4_speed`, and `overhang_4_4_speed` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1500-1570` into `ares-core` registry metadata. This historical registry milestone deferred typed/runtime behavior when it was completed. Later source-cited overhang slices now consume `enable_overhang_speed`, all four overhang speed bands, and the `slowdown_for_curled_perimeters` final-bucket branch for fully unsupported rectangular overhang perimeter loops detected through the direct previous layer, using Ares' current whole-path unsupported-span estimate for band selection. Full Orca `ExtrusionQualityEstimator`, partial clipping, exact segment-level overhang percentage estimation, path subdivision, bridge dynamic speed bands, raft/object gates, and multi-object estimator state remain deferred.

Exit criteria are tracked in `docs/milestones/m43-print-config-overhang-speed-registry.md`.

## M44: PrintConfig brim flow and combine option registry
Port the FFF `brim_flow_ratio`, `brim_use_efc_outline`, and `combine_brims` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1637-1663` into `ares-core` registry metadata. This historical registry milestone deferred typed/runtime behavior when it was completed. Later source-cited slices now consume `brim_flow_ratio` for brim extrusion, `combine_brims` for first-layer outer brim envelopes, and `brim_use_efc_outline` for Orca-gated rectangle-scaffold outer brim bounds based on the active elephant-foot compensation offset. Full Orca EFC surface generation, polygon/ex-polygon outline selection, support brim EFC behavior, painted/ear EFC snapping, preset behavior, object override behavior, and runtime behavior outside those consumed paths remain deferred.

Exit criteria are tracked in `docs/milestones/m44-print-config-brim-flow-combine-registry.md`.

## M45: PrintConfig brim ear option registry
Port the FFF `brim_ears`, `brim_ears_max_angle`, and `brim_ears_detection_length` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1665-1693` into `ares-core` registry metadata while deferring typed behavior, brim-ear sharp-edge detection, detection-radius decimation, max-angle behavior, brim generation, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m45-print-config-brim-ear-registry.md`.

## M46: PrintConfig compatible profile option registry
Port the FFF `compatible_printers`, `upward_compatible_machine`, `compatible_printers_condition`, `compatible_prints`, `compatible_prints_condition`, `compatible_machine_expression_group`, `compatible_process_expression_group`, `different_settings_to_system`, and `print_compatible_printers` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1695-1748` into `ares-core` registry metadata while deferring typed behavior, compatibility-expression evaluation, preset filtering, profile composition behavior changes, project-file persistence semantics, CLI no-CLI enforcement, UI behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m46-print-config-compatible-profile-registry.md`.

## M47: PrintConfig print sequence and order option registry
Port the FFF `print_sequence` and `print_order` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1750-1770` into `ares-core` registry metadata while deferring typed behavior, print-sequence scheduling, object-by-object constraints, intra-layer object ordering, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m47-print-config-print-sequence-order-registry.md`.

## M48: PrintConfig cooling and default profile option registry
Port the FFF `slow_down_for_layer_cooling`, `default_acceleration`, `default_filament_profile`, `default_print_profile`, `activate_air_filtration`, `activate_air_filtration_during_print`, `activate_air_filtration_on_completion`, `during_print_exhaust_fan_speed`, `complete_print_exhaust_fan_speed`, and `close_fan_the_first_x_layers` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1772-1845` into `ares-core` registry metadata while deferring typed behavior, cooling slowdown, acceleration planning, default profile selection, air-filtration/exhaust fan behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m48-print-config-cooling-default-profile-registry.md`.

## M49: PrintConfig internal bridge option registry
Port the FFF bridge/internal-bridge `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1847-1938` into `ares-core` registry metadata while deferring typed behavior, bridge support decisions, internal bridge filtering, extra bridge layer generation, bridge geometry, slicing behavior, support generation, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m49-print-config-internal-bridge-registry.md`.

## M50: PrintConfig G-code and shell pattern option registry
Port the FFF `machine_end_gcode`, `printing_by_object_gcode`, `filament_end_gcode`, `ensure_vertical_shell_thickness`, `top_surface_pattern`, `bottom_surface_pattern`, and `internal_solid_infill_pattern` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1940-2025` into `ares-core` registry metadata while deferring typed behavior, custom G-code execution, object-by-object scheduling, vertical shell behavior, solid infill pattern generation, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m50-print-config-gcode-shell-pattern-registry.md`.

## M51: PrintConfig wall ordering and small perimeter option registry
Port the adjacent FFF wall/small-perimeter `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2027-2110` into `ares-core` registry metadata by preserving already registered outer-wall/infill-first keys and adding missing `small_perimeter_speed`, `small_perimeter_threshold`, `wall_sequence`, and `wall_direction` metadata while deferring typed behavior, wall ordering, wall direction path generation, small-perimeter speed application, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m51-print-config-wall-small-perimeter-registry.md`.

## M52: PrintConfig extruder clearance option registry
Port the adjacent FFF extruder-clearance and nozzle-height `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2127-2160` into `ares-core` registry metadata while splitting the registry definition table so Rust files remain under 400 LOC. Defer typed behavior, collision avoidance, by-object scheduling, nozzle-height behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m52-print-config-extruder-clearance-registry.md`.

## M53: PrintConfig bed mesh option registry
Port the adjacent FFF adaptive bed mesh `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2162-2200` into `ares-core` registry metadata, including the `coPoint` value kind required by `bed_mesh_min`, `bed_mesh_max`, and `bed_mesh_probe_distance`. Defer adaptive mesh behavior, probing constraints, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m53-print-config-bed-mesh-registry.md`.

## M54: PrintConfig extruder visual and offset option registry
Port the adjacent FFF extruder visual/offset `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2202-2225` into `ares-core` registry metadata. Defer UI color behavior, firmware/tool offset behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m54-print-config-extruder-visual-offset-registry.md`.

## M55: PrintConfig filament and print flow ratio option registry
Port the adjacent FFF filament/print flow ratio `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2227-2250` into `ares-core` registry metadata, including an Ares `FloatsNullable` metadata kind for `filament_flow_ratio`, whose upstream definition is `coFloats` with `nullable = true` and a `ConfigOptionFloatsNullable` default. Defer flow scaling behavior, object/material override behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m55-print-config-flow-ratio-registry.md`.

## M56: PrintConfig pressure advance option registry
Port the adjacent FFF pressure advance `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2252-2262` into `ares-core` registry metadata. Defer runtime pressure advance, firmware-specific linear advance behavior, adaptive pressure advance behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m56-print-config-pressure-advance-registry.md`.

## M57: PrintConfig adaptive pressure advance option registry
Port the adjacent FFF adaptive pressure advance `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2264-2320` into `ares-core` registry metadata. Defer runtime adaptive pressure advance, calibration-model parsing/fitting, firmware-specific behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m57-print-config-adaptive-pressure-advance-registry.md`.

## M58: PrintConfig cooling slowdown option registry
Port the adjacent FFF cooling-slowdown `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2334-2347` into `ares-core` registry metadata while splitting the registry definition table so Rust files remain under 400 LOC. Preserve existing metadata while moving sorted shards; defer fan runtime behavior, layer-time slowdown behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m58-print-config-cooling-slowdown-registry.md`.

## M59: PrintConfig fan cooling and filament color note option registry
Port the adjacent FFF fan cooling layer time, default filament color, filament color, and filament notes `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2349-2382` into `ares-core` registry metadata. Defer fan runtime behavior, color UI behavior, note UI behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m59-print-config-fan-filament-color-note-registry.md`.

## M60: PrintConfig filament mapping and hardware flag option registry
Port the adjacent FFF filament multi-color, filament mapping, nozzle hardness, and filament-switcher hardware flag `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2385-2440` into `ares-core` registry metadata. Defer filament mapping runtime behavior, dynamic map behavior, filament switcher behavior, nozzle-HRC validation, UI behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m60-print-config-filament-mapping-hardware-registry.md`.

## M61: PrintConfig filament flush and toolchange timing option registry
Port the adjacent FFF filament flush temperature, flush volumetric speed, max volumetric speed, and filament/tool-change timing `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2442-2497` into `ares-core` registry metadata. Add an `IntsNullable` metadata kind for `filament_flush_temp`; defer flushing runtime behavior, volumetric speed limiting, tool-change timing behavior, UI behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m61-print-config-filament-flush-timing-registry.md`.

## M62: PrintConfig bed temperature and flush dataset option registry
Port the adjacent FFF support skip-flush, bed temperature formula, nozzle flush dataset, and filament diameter source-refresh `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2500-2523` into `ares-core` registry metadata only. Preserve existing `filament_diameter` typed/runtime behavior while refreshing its source citation; defer bed-temperature selection behavior, nozzle flush dataset behavior, support-object skip-flush behavior, UI/runtime behavior, extrusion behavior, slicing behavior, G-code behavior, and the following `pellet_flow_coefficient`/adaptive-volumetric/shrink options.

Exit criteria are tracked in `docs/milestones/m62-print-config-bed-flush-diameter-registry.md`.

## M63: PrintConfig pellet flow and shrinkage option registry
Port the adjacent FFF pellet flow coefficient, adaptive volumetric speed metadata, volumetric speed coefficients, and filament shrinkage `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2551-2594` into `ares-core` registry metadata. Pellet-to-diameter conversion is now consumed by runtime `filament_diameters()` parsing; defer adaptive volumetric speed limiting, shrinkage scaling, UI/runtime behavior beyond this conversion, extrusion behavior beyond the existing diameter consumers, slicing behavior beyond the existing diameter consumers, G-code behavior beyond the existing diameter consumers, and following options.

Exit criteria are tracked in `docs/milestones/m63-print-config-pellet-shrinkage-registry.md`.

## M64: PrintConfig filament load/unload speed option registry
Port the adjacent FFF filament adhesiveness category and loading/unloading speed `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2596-2634` into `ares-core` registry metadata. Defer wipe-tower loading/unloading behavior, ramming/toolchange runtime, UI/runtime behavior, extrusion behavior, slicing behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m64-print-config-filament-load-unload-speed-registry.md`.

## M65: PrintConfig filament cooling and stamping option registry
Port the adjacent FFF filament toolchange delay, cooling moves, stamping, and initial cooling speed `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2636-2676` into `ares-core` registry metadata. Defer wipe-tower cooling/stamping behavior, ramming/toolchange runtime, UI/runtime behavior, extrusion behavior, slicing behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m65-print-config-filament-cooling-stamping-registry.md`.

## M66: PrintConfig filament tower purge and interface option registry
Port the adjacent FFF filament minimal purge, wipe-tower cooling, and tower interface pre-extrusion/ironing `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2678-2719` into `ares-core` registry metadata. Defer wipe-tower purge/cooling/interface runtime behavior, UI/runtime behavior, extrusion behavior, slicing behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m66-print-config-filament-tower-purge-interface-registry.md`.

## M67: PrintConfig filament tower temperature and final cooling option registry
Port the adjacent FFF filament tower interface purge volume, tower interface print temperature, and final cooling speed `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2721-2743` into `ares-core` registry metadata while splitting the current `pre_middle` registry shard so Rust files remain under 400 LOC. Defer wipe-tower purge/temperature/final-cooling runtime behavior, UI/runtime behavior, extrusion behavior, slicing behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m67-print-config-filament-tower-temperature-final-cooling-registry.md`.

## M68: PrintConfig filament ramming option registry
Port the adjacent FFF filament ramming parameters and multitool ramming `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2745-2774` into `ares-core` registry metadata. Defer ramming parameter parsing/editing/runtime, wipe-tower behavior, UI/runtime behavior, extrusion behavior, slicing behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m68-print-config-filament-ramming-registry.md`.

## M70: PrintConfig filament material and support option registry
Port the adjacent FFF filament material/statistics/support `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2776-2826` into `ares-core` registry metadata. Defer material database enum population, density statistics behavior, soluble/support behavior, filament change-length behavior, extruder printability behavior, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m70-print-config-filament-material-support-registry.md`.

## M71: PrintConfig filament identity and statistics option registry
Port the adjacent FFF filament softening-temperature, price/statistics, identity, and vendor `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2828-2859` into `ares-core` registry metadata. Defer softening-temperature behavior, filament price/statistics behavior, settings-id/ids identity behavior, vendor behavior, CLI/no-CLI behavior, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m71-print-config-filament-identity-statistics-registry.md`.

## M72: PrintConfig infill direction and extra solid option registry
Port the adjacent FFF infill direction, sparse density, model-aligned direction, extra solid infill, and multiline infill `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2861-2913` into `ares-core` registry metadata. `extra_solid_infills` is now consumed by a source-cited runtime slice that ports the `check_layer_id_pattern()` schedule grammar and converts matching otherwise-sparse layers to internal solid infill. Defer remaining infill-angle behavior, model-aligned direction behavior, multiline infill behavior, UI/runtime behavior, adjacent slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m72-print-config-infill-direction-extra-solid-registry.md`.

## M73: PrintConfig gyroid optimization and sparse infill pattern option registry
Port the adjacent FFF gyroid optimization and sparse infill pattern `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2915-2985` into `ares-core` registry metadata. Defer gyroid optimization behavior, sparse infill pattern runtime behavior, enum value storage, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m73-print-config-gyroid-sparse-pattern-registry.md`.

## M74: PrintConfig lateral lattice and infill anchor option registry
Port the adjacent FFF lateral lattice angle, infill overhang angle, and infill anchor `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:2987-3066` into `ares-core` registry metadata. Defer lateral lattice behavior, infill overhang behavior, infill anchor behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:3068+` acceleration/following options.

Exit criteria are tracked in `docs/milestones/m74-print-config-lateral-lattice-infill-anchor-registry.md`.

## M75: PrintConfig acceleration option registry
Port the adjacent FFF acceleration and accel-to-decel `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3068-3167` into `ares-core` registry metadata. Defer acceleration resolution, accel-to-decel behavior, ratio metadata/runtime behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:3169+` jerk/following options.

Exit criteria are tracked in `docs/milestones/m75-print-config-acceleration-registry.md`.

## M76: PrintConfig wall, infill, and travel jerk option registry
Port the adjacent FFF wall, infill, top surface, first layer, travel, and first-layer travel jerk `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3188-3249` into `ares-core` registry metadata. Defer `default_jerk`, `default_junction_deviation`, jerk runtime behavior, ratio metadata/runtime behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:3251+` initial-layer line-width/height/following options.

Exit criteria are tracked in `docs/milestones/m76-print-config-jerk-registry.md`.

## M77: PrintConfig default jerk registry with pre-middle shard split
Port the adjacent FFF `default_jerk` and `default_junction_deviation` `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3169-3186` into `ares-core` registry metadata while splitting the oversized pre-middle registry shard. Defer default jerk behavior, junction-deviation behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:3251+` initial-layer line-width/height/following options.

Exit criteria are tracked in `docs/milestones/m77-print-config-default-jerk-registry-shard-split.md`.

## M78: PrintConfig initial-layer line, speed, and slow-down registry
Port the adjacent FFF initial-layer line width, print height, speed, travel speed, and slow-down `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3251-3314` into `ares-core` registry metadata. Defer initial-layer line-width resolution, print-height behavior, speed behavior, travel-speed ratio behavior, slow-down behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:3316+` nozzle/fan/following options.

Exit criteria are tracked in `docs/milestones/m78-print-config-initial-layer-line-speed-registry.md`.

## M79: PrintConfig first-layer temperature and fan-speed registry
Port the adjacent FFF first-layer nozzle temperature and fan-speed `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3316-3370` into `ares-core` registry metadata. Defer nozzle-temperature behavior, fan-speed behavior, disable semantics, override behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:3372+` filament-ironing/following options.

Exit criteria are tracked in `docs/milestones/m79-print-config-first-layer-fan-temperature-registry.md`.

## M80: PrintConfig filament ironing override option registry
Port the adjacent FFF filament-specific ironing override `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3372-3418` into `ares-core` registry metadata. Add metadata-only `PercentsNullable` for nullable percent-vector registry metadata; defer ironing runtime behavior, filament override resolution, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:3420+` fuzzy-skin/following options.

Exit criteria are tracked in `docs/milestones/m80-print-config-filament-ironing-registry.md`.

## M81: PrintConfig fuzzy-skin option registry
Port the adjacent FFF fuzzy-skin `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3420-3576` into `ares-core` registry metadata. This historical registry milestone deferred runtime behavior when it was completed. A later source-cited slice now consumes `fuzzy_skin`, `fuzzy_skin_thickness`, `fuzzy_skin_point_distance`, and `fuzzy_skin_first_layer` for Ares' current external perimeter path boundary, using `PrintConfig.hpp:50-58,1108-1119`, `PrintConfig.cpp:192-200,3420-3566`, `PrintObject.cpp:3458-3459`, `PerimeterGenerator.cpp:150-163`, and `Feature/FuzzySkin/FuzzySkin.cpp:294-344,476-490,561-575` as the upstream boundary. The runtime slice validates finite thickness `0..=2` and point distance `0..=5`, preserves Orca's effective-disable gate for point distance below `0.01` or thickness below `0.001`, honors the first-layer gate, and applies a deterministic classic compatibility shell to external paths for `external`, `all`, and `allwalls` so final G-code path coordinates change. A subsequent slice consumes `allwalls` for Ares' generated rectangular internal wall loops by reusing the existing classic/ripple closed-polyline displacement before overhang reversal and seam positioning. Painted fuzzy-skin facets, hole-only ownership, exact Orca random/noise parity, broader ripple/noise module parity, Arachne width/extrusion modes, merge-region behavior, 3MF/UI behavior, multi-region config merging, and `PrintConfig.cpp:3578+` filter-out-gap-fill/following options remain deferred.

Exit criteria are tracked in `docs/milestones/m81-print-config-fuzzy-skin-registry.md`.

## M82: PrintConfig process and G-code utility option registry
Port the adjacent FFF process/G-code utility `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3578-3643` into `ares-core` registry metadata. Defer gap filtering, gap speed behavior, precise-Z behavior, arc fitting, line-number output, first-layer scan integration, power-loss recovery G-code behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code output behavior, and `PrintConfig.cpp:3652+` nozzle-type/following options.

Exit criteria are tracked in `docs/milestones/m82-print-config-process-gcode-utility-registry.md`.

## M83: PrintConfig nozzle material and hardness option registry
Port the adjacent FFF nozzle material/hardness `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3652-3679` into `ares-core` registry metadata. Add metadata-only `EnumsNullable` for nullable generic enum-vector registry metadata; defer nozzle material compatibility behavior, nozzle hardness validation, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:3681+` printer-structure/following options.

Exit criteria are tracked in `docs/milestones/m83-print-config-nozzle-material-hardness-registry.md`.

## M84: PrintConfig printer structure and fan speed-up option registry
Port the adjacent FFF printer-structure, best-object-position, auxiliary-fan, and fan speed-up/kick-start `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3681-3738` into `ares-core` registry metadata. Defer printer-structure behavior, best-object-position/auto-arrange behavior, fan speed-up scheduling, fan kick-start G-code/PWM behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:3740+` part-cooling PWM/following options.

Exit criteria are tracked in `docs/milestones/m84-print-config-printer-structure-fan-speedup-registry.md`.

## M85: PrintConfig fan PWM, cost, and printer support option registry
Port the adjacent FFF part-cooling PWM clamp, printer time-cost, chamber-temperature support, and air-filtration support `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3740-3783` into `ares-core` registry metadata. Defer fan PWM clamp behavior, time-cost calculation, chamber-temperature control, air-filtration G-code behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:3785+` gcode-flavor/following options.

Exit criteria are tracked in `docs/milestones/m85-print-config-fan-pwm-cost-printer-support-registry.md`.

## M86: PrintConfig G-code flavor and object-label option registry
Port the adjacent FFF G-code flavor, pellet-printer, multi-bed, object-label, exclude-object, and verbose-G-code `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3785-3851` into `ares-core` registry metadata. G-code flavor runtime selection, object-label/exclude-object G-code emission, and pellet-printer effective-diameter behavior are now implemented; defer multi-bed behavior, verbose comment emission, typed accessors, UI/runtime behavior beyond this conversion, slicing behavior beyond the existing diameter consumers, extrusion behavior beyond the existing diameter consumers, remaining G-code behavior, and `PrintConfig.cpp:3853+` infill-combination/following options.

Exit criteria are tracked in `docs/milestones/m86-print-config-gcode-flavor-object-label-registry.md`.

## M87: PrintConfig infill combination and rotation-template option registry
Port the adjacent FFF infill-combination, infill-shift, sparse-infill rotation-template, and solid-infill rotation-template `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3853-3896` into `ares-core` registry metadata. Defer infill-combination behavior, infill-shift behavior, rotation-template parsing/application, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:3898+` skeleton/skin infill/following options.

Exit criteria are tracked in `docs/milestones/m87-print-config-infill-combination-rotation-registry.md`.

## M88: PrintConfig skin, skeleton, and combined-infill option registry
Port the adjacent FFF skin/skeleton infill density, depth, line-width, symmetric-infill, and combined-infill max-layer-height `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3898-3984` into `ares-core` registry metadata. Defer skin/skeleton infill behavior, infill-lock behavior, skin/skeleton line-width resolution, symmetric-infill behavior, combined-infill max-layer-height behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:3986+` BBS clumping/wrapping detection/following options.

Exit criteria are tracked in `docs/milestones/m88-print-config-skin-skeleton-infill-registry.md`.

## M89: PrintConfig wrapping detection and sparse-infill utility option registry
Port the adjacent FFF clumping/wrapping detection, sparse-infill filament, sparse-infill line width, infill/wall overlap, top/bottom infill/wall overlap, and sparse-infill speed `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:3987-4061` into `ares-core` registry metadata while splitting the near-limit pre-middle process registry shard. Defer wrapping/clumping detection behavior, wrapping exclude geometry, sparse-infill filament routing, line-width resolution, speed behavior, typed accessors, UI/runtime behavior, slicing behavior outside the later rectangle-only overlap slice, extrusion behavior, G-code behavior outside that later slice, and `PrintConfig.cpp:4063+` inherits/inherits_group/following options. A later source-cited runtime slice now consumes `infill_wall_overlap` and `top_bottom_infill_wall_overlap` for rectangle-only infill clipping and G-code-visible print paths; full polygon fill-surface parity remains deferred.

Exit criteria are tracked in `docs/milestones/m89-print-config-wrapping-sparse-infill-registry.md`.

## M90: PrintConfig inheritance, MMU interlocking, and calibration flag option registry
Port the adjacent FFF profile inheritance, interface-shell, MMU segmented-region, interlocking, and flowrate-calibration flag `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4063-4159` into `ares-core` registry metadata while splitting the near-limit middle registry shard. Defer inheritance resolution, interface-shell behavior, segmented-region/interlocking geometry behavior, typed accessors, UI/runtime behavior beyond consumed slices, slicing behavior beyond consumed slices, extrusion behavior beyond consumed slices, G-code behavior beyond consumed slices, and `PrintConfig.cpp:4161+` ironing/following options. A later source-cited runtime slice now consumes `calib_flowrate_topinfill_special_order` from `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1070`, `PrintConfig.cpp:4156-4159`, and `Fill/FillBase.cpp:166-183` by reversing Ares top-surface scanline segment direction after deterministic candidate sorting and existing pattern direction selection. This changes `LayerInfills`, `LayerPrintPaths`, and emitted `;INFILL:solid:` / `;PRINT_PATH:top_solid_infill:` G-code coordinates for top solid infill only. Full Orca `ExtrusionEntityCollection::no_sort`, Archimedean-chords center-spiral ordering from `FillPlanePath.cpp:133-155`, expolygon chaining, Arachne, and binary E2E geometry parity remain deferred.

Exit criteria are tracked in `docs/milestones/m90-print-config-inheritance-interlocking-registry.md`.

## M91: PrintConfig ironing and Z contouring option registry
Port the adjacent FFF ironing and Z-layer anti-aliasing / Z contouring `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4161-4293` into `ares-core` registry metadata. Defer ironing behavior, Z contouring behavior, slicing-plane changes, fill-direction alternation behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:4295+` layer-change G-code/following options.

Exit criteria are tracked in `docs/milestones/m91-print-config-ironing-zaa-registry.md`.

## M92: PrintConfig custom G-code, machine limit flag, and small-area flow option registry
Port the adjacent FFF custom G-code, machine-limit emission flag, small-area infill flow compensation, and scarf-seam marker `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4295-4375` into `ares-core` registry metadata while splitting the near-limit tail registry shard. Defer custom G-code insertion, machine-limit emission, small-area flow compensation, scarf-seam behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:4377+` machine-axis-limit-loop/following options.

Exit criteria are tracked in `docs/milestones/m92-print-config-custom-gcode-small-area-registry.md`.

## M93: PrintConfig machine speed and acceleration limit option registry
Port the adjacent machine XYZE maximum speed and maximum acceleration `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4377-4428` into `ares-core` registry metadata while splitting the near-limit late registry shard. Defer machine-limit emission, M201/M203 G-code output, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:4429+` machine-jerk/junction/min-rate/acceleration PRT/resonance/input-shaping/following options.

Exit criteria are tracked in `docs/milestones/m93-print-config-machine-speed-acceleration-registry.md`.

## M94: PrintConfig machine jerk, min-rate, and acceleration PRT option registry
Port the adjacent machine XYZE jerk, junction deviation, minimum feedrate, and M204 P/R/T acceleration `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4429-4514` into `ares-core` registry metadata while splitting the near-limit registry key expectation test list. Defer machine-limit emission, M204/M205 G-code output, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:4516+` resonance/input-shaping/following options.

Exit criteria are tracked in `docs/milestones/m94-print-config-machine-jerk-rate-acceleration-registry.md`.

## M95: PrintConfig resonance avoidance and input shaping option registry
Port the adjacent resonance avoidance and input shaping `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4516-4589` into `ares-core` registry metadata. Defer resonance speed behavior, input-shaping G-code emission, firmware override/disable behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:4591+` fan/layer-height/extrusion-rate/nozzle/following options.

Exit criteria are tracked in `docs/milestones/m95-print-config-resonance-input-shaping-registry.md`.

## M96: PrintConfig fan max and extrusion-rate smoothing option registry
Port the adjacent fan maximum, max-layer-height citation, and extrusion-rate smoothing `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4591-4648` into `ares-core` registry metadata. Defer cooling behavior, extrusion-rate smoothing behavior, arc-fitting interaction, speed planning, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:4651+` fan-min/additional-cooling/min-layer/nozzle/following options.

Exit criteria are tracked in `docs/milestones/m96-print-config-fan-max-extrusion-smoothing-registry.md`.

## M97: PrintConfig auxiliary fan, min layer, and nozzle option registry
Port the adjacent fan-minimum, auxiliary fan, min-layer/nozzle citation, and slow-down minimum speed `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4651-4721` into `ares-core` registry metadata while splitting the near-limit public count fixture. Defer cooling behavior, auxiliary fan G-code, adaptive layer-height behavior, speed planning, nozzle behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:4723+` notes/host/following options.

Exit criteria are tracked in `docs/milestones/m97-print-config-aux-fan-layer-nozzle-registry.md`.

## M98: PrintConfig notes, host, nozzle-volume, and MMU parking registry
Port the adjacent FFF notes, printer-host type, nozzle-volume, cooling-tube, high-current filament swap, and parking-position `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4723-4810` into `ares-core` registry metadata. Defer printer-host upload/integration behavior, MMU cooling-tube loading/unloading behavior, high-current filament-swap behavior, parking-position runtime behavior, nozzle-volume runtime behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:4812+` extra-loading/start-end/following options.

Exit criteria are tracked in `docs/milestones/m98-print-config-host-mmu-parking-registry.md`.

## M99: PrintConfig loading move, start/end points, ooze, and filename registry
Port the adjacent FFF extra-loading, start/end point, infill-retraction suppression flag, ooze-prevention flag, and filename-format `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4812-4848` into `ares-core` registry metadata while splitting the near-limit known-count fixture. Defer MMU loading/unloading behavior, cutter/start-end point runtime behavior, infill retraction suppression, ooze-prevention temperature behavior, filename template rendering, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:4850+` make-overhang/following options.

Exit criteria are tracked in `docs/milestones/m99-print-config-loading-ooze-filename-registry.md`.

## M100: PrintConfig make-overhang and wall option registry
Port the adjacent FFF make-overhang-printable, overhang-wall detection, wall-filament, and inner-wall width/speed `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4850-4916` into `ares-core` registry metadata. This historical registry milestone deferred runtime behavior when it was completed. A later source-cited overhang slice now consumes `detect_overhang_wall` for the first rectangular unsupported perimeter path; make-overhang geometry modification, wall-filament routing, inner-wall line-width resolution, inner-wall speed planning, full polygon overhang-wall detection, UI/runtime behavior outside that slice, and `PrintConfig.cpp:4918+` wall-loop/post-process/following options remain deferred.

Exit criteria are tracked in `docs/milestones/m100-print-config-overhang-wall-registry.md`.

## M101: PrintConfig wall loop, post-process, and printer identity registry
Port the adjacent FFF wall-loop, alternate-extra-wall, post-processing scripts, process role-change G-code, printer identity, and print/printer settings-id `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4918-4986` into `ares-core` registry metadata. Defer wall-loop generation, alternate-extra-wall planning, post-processing script execution, process role-change G-code insertion, printer identity/settings-id behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, filesystem script execution behavior, and `PrintConfig.cpp:4988+` raft/resolution/retraction/following options.

Exit criteria are tracked in `docs/milestones/m101-print-config-wall-post-process-printer-registry.md`.

## M102: PrintConfig raft, resolution, and retraction trigger registry
Port the adjacent FFF raft support, path resolution, and initial retraction trigger `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:4988-5066` into `ares-core` registry metadata. Defer raft generation/support geometry, contour simplification/path resolution behavior, retraction trigger planning, wipe planning, layer-change retraction behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:5068+` retraction length/long retraction/toolchange/Z-hop/following options.

Exit criteria are tracked in `docs/milestones/m102-print-config-raft-resolution-retraction-registry.md`.

## M103: PrintConfig retraction length, cut, and toolchange registry
Port the adjacent FFF base retraction length, long retraction when cut/extruder-change, retraction distance, and toolchange retraction `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5068-5120` into `ares-core` registry metadata. Defer retraction planning, filament-cut long retraction behavior, extruder-change long retraction behavior, toolchange retraction behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:5122+` Z-hop/extruder/nozzle-volume/following options.

Exit criteria are tracked in `docs/milestones/m103-print-config-retraction-length-cut-toolchange-registry.md`.

## M104: PrintConfig Z-hop, lift-boundary, and extruder/nozzle type registry
Port the adjacent FFF Z-hop, Z-hop boundary/type, travel-slope, lift-enforcement, extruder type, nozzle-volume type, and default nozzle-volume type `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5122-5237` plus enum maps in `PrintConfig.cpp:526-540` and `565-575` into `ares-core` registry metadata. Defer Z-hop movement behavior, slope/spiral lift behavior, lift-surface enforcement behavior, extruder/nozzle variant resolution, preset/project nozzle-volume synchronization, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:5239+` extruder variant/AMS/following options.

Exit criteria are tracked in `docs/milestones/m104-print-config-zhop-extruder-type-registry.md`.

## M105: PrintConfig extruder variant and ID registry
Port the adjacent FFF extruder variant list, AMS count, printer/print/filament extruder IDs, and printer/print/filament extruder variant `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5239-5304` into `ares-core` registry metadata. Defer extruder variant normalization, AMS-count parsing helpers, printer/print/filament extruder mapping, preset compatibility behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:5306+` restart/retraction speed/following options.

Exit criteria are tracked in `docs/milestones/m105-print-config-extruder-variant-id-registry.md`.

## M106: PrintConfig restart, retraction speed, M73, and seam registry
Port the adjacent FFF restart-extra, retraction/deretraction speed, firmware retraction, calibration mark, M73 disable, seam-position, staggered-inner-seam, and seam-gap `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5306-5390` plus `SeamPosition` enum map in `PrintConfig.hpp:211-213` and `PrintConfig.cpp:350-357` into `ares-core` registry metadata. Defer restart/retraction runtime behavior, firmware-retraction G10/G11 behavior, M73 suppression behavior, calibration mark generation, seam placement/staggering/gap geometry behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:5392+` scarf seam/following options.

Exit criteria are tracked in `docs/milestones/m106-print-config-restart-speed-seam-registry.md`.

## M107: PrintConfig scarf seam registry
Port the adjacent FFF scarf-seam `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5392-5500` plus `SeamScarfType` enum map in `PrintConfig.hpp:216-220` and `PrintConfig.cpp:360-365` into `ares-core` registry metadata. Defer scarf seam planning, conditional scarf selection, overhang estimation, scarf speed/flow behavior, seam slope geometry, inner-wall scarf behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:5502+` wipe-speed/following options.

Exit criteria are tracked in `docs/milestones/m107-print-config-scarf-seam-registry.md`.

## M108: PrintConfig wipe speed and loop registry
Port the adjacent FFF wipe-speed and loop-wipe `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5502-5538` into `ares-core` registry metadata. Defer role-based wipe speed selection, wipe speed calculation, loop-wipe movement, external-loop wipe placement, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:5540+` skirt/draft-shield/following options.

Exit criteria are tracked in `docs/milestones/m108-print-config-wipe-speed-loop-registry.md`.

## M109: PrintConfig skirt and draft-shield registry
Port the adjacent FFF skirt and draft-shield `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5540-5627` plus `SkirtType`/`DraftShield` enum maps in `PrintConfig.hpp:286-292` and `PrintConfig.cpp:437-447` into `ares-core` registry metadata. Defer skirt generation behavior, skirt start-angle placement, draft-shield geometry, single-loop-after-first-layer behavior, combined/per-object skirt behavior, minimum-skirt-length loop calculation, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:5629+` slowdown/minimum-sparse/following options.

Exit criteria are tracked in `docs/milestones/m109-print-config-skirt-draft-shield-registry.md`.

## M110: PrintConfig slowdown and solid-infill registry
Port the adjacent FFF layer-time slowdown, minimum sparse infill area, and solid-infill filament `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5629-5655` into `ares-core` registry metadata. Defer layer-time slowdown behavior, sparse-area solid-fill replacement behavior, solid-infill extruder selection behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:5657+` internal-solid-infill/spiral/following options.

Exit criteria are tracked in `docs/milestones/m110-print-config-slowdown-solid-infill-registry.md`.

## M111: PrintConfig internal solid infill and spiral registry
Port the adjacent FFF internal-solid-infill line width/speed and spiral-vase `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5657-5726` into `ares-core` registry metadata. Defer internal-solid-infill runtime behavior, spiral-vase path generation, Z/XY smoothing behavior, spiral transition flow behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:5728+` timelapse/standby/preheat/following options.

Exit criteria are tracked in `docs/milestones/m111-print-config-internal-solid-spiral-registry.md`.

## M112: PrintConfig timelapse and preheat registry
Port the adjacent FFF timelapse, standby temperature delta, and preheat `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5728-5774` plus `TimelapseType` enum map in `PrintConfig.hpp:281-284` and `PrintConfig.cpp:431-435` into `ares-core` registry metadata. Defer timelapse capture/validation behavior, ooze-prevention standby temperature behavior, preheat command insertion, remaining preheat post-processing behavior, timelapse warning/insertion behavior, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:5777+` file/machine/filament start-G-code/following options. A mechanical registry-table shard split is allowed only to keep Rust files below 400 LOC.

Exit criteria are tracked in `docs/milestones/m112-print-config-timelapse-preheat-registry.md`.

## M113: PrintConfig start G-code and filament-change registry
Port the adjacent FFF file/machine/filament start G-code, single-extruder multi-material, and manual filament-change `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5777-5819` into `ares-core` registry metadata. Defer start-G-code emission, placeholder expansion, single-extruder multi-material behavior, manual filament-change Tx omission/M600 behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:5821+` wipe-tower/ramming/tool-change/following options.

Exit criteria are tracked in `docs/milestones/m113-print-config-start-gcode-filament-change-registry.md`.

## M114: PrintConfig wipe-tower and ramming registry
Port the adjacent FFF wipe-tower type, purge-in-prime-tower, filament ramming, tool-change-on-wipe-tower, and sparse-layer wipe-tower `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5821-5861` plus `WipeTowerType` enum map in `PrintConfig.hpp:74-77` and `PrintConfig.cpp:212-216` into `ares-core` registry metadata. Defer wipe-tower implementation selection, prime-tower purge behavior, filament ramming behavior, tool-change travel behavior, sparse-layer wipe-tower suppression, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:5863+` single-extruder priming/slice-closing/slicing-mode/support/following options. A mechanical registry-table shard split is allowed only to keep Rust files below 400 LOC.

Exit criteria are tracked in `docs/milestones/m114-print-config-wipe-tower-ramming-registry.md`.

## M115: PrintConfig priming, slicing mode, Z offset, and support-enable registry
Port the adjacent FFF single-extruder priming, slice gap closing, slicing mode, Z offset, and support-enable `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5863-5908` plus `SlicingMode` enum map in `PrintConfig.hpp:162-170` and `PrintConfig.cpp:305-310` into `ares-core` registry metadata. Defer single-extruder priming behavior, mesh gap-closing behavior, slicing-mode polygon rules, Z-offset application, support generation, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:5910+` support-type/support-distance/following options.

Exit criteria are tracked in `docs/milestones/m115-print-config-priming-slicing-support-registry.md`.

## M116: PrintConfig support type and support placement registry
Port the adjacent FFF support type, support/object placement gap, support pattern angle, and support placement filter `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5910-5979` plus `SupportType` enum map in `PrintConfig.hpp:195-209` and `PrintConfig.cpp:342-348` into `ares-core` registry metadata. Defer support generation, tree/normal/manual support selection behavior, support enforcer/blocker handling, support geometry, support pattern placement, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:5981+` support top/bottom Z-distance/following options.

Exit criteria are tracked in `docs/milestones/m116-print-config-support-type-placement-registry.md`.

## M117: PrintConfig support Z-distance and enforced layers registry
Port the adjacent FFF support top/bottom Z-distance and enforced-support-layers `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5981-6025` into `ares-core` registry metadata. Defer support Z-gap application, independent support layer-height rounding, enforced support material generation, support geometry, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6027+` support-filament/following options.

Exit criteria are tracked in `docs/milestones/m117-print-config-support-z-distance-enforce-registry.md`.

## M118: PrintConfig support filament registry
Port the FFF support/raft base filament `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6027-6034` into `ares-core` registry metadata. Defer support/raft filament routing, support material selection, raft/support generation, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6036+` support-interface-not-for-body/support-line-width/following options.

Exit criteria are tracked in `docs/milestones/m118-print-config-support-filament-registry.md`.

## M119: PrintConfig support interface base avoidance and line width registry
Port the adjacent FFF support interface base-avoidance and support line-width `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6036-6053` into `ares-core` registry metadata, with a mechanical support registry shard split to keep Rust files below 400 LOC. Defer support interface filament routing, support/raft base material selection, support line-width resolution, nozzle-diameter ratio computation, support geometry, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6055+` support-interface-loop-pattern/following options.

Exit criteria are tracked in `docs/milestones/m119-print-config-support-interface-line-width-registry.md`.

## M120: PrintConfig support interface loop, filament, layers, and spacing registry
Port the adjacent FFF support interface loop-pattern, interface filament, top/bottom interface layer count, and top interface spacing `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6055-6112` into `ares-core` registry metadata. `support_interface_top_layers = 0` is now partially consumed for existing Ares `PrintPathRole::SupportMaterialInterface` paths through the source-cited `Support/SupportParameters.hpp` and `Support/SupportCommon.cpp` boundary: existing support-interface paths are routed to `SupportMaterial` before support ironing, so support speed/flow and the base layer fan baseline apply, support-interface role overrides no longer apply, and support-interface ironing duplication no longer applies. Defer full support interface loop generation, support interface filament routing, positive top-layer support contact generation, bottom interface layer-count behavior, top interface spacing geometry, solid interface forcing beyond this role rewrite, support geometry, typed accessors, UI/runtime behavior, and `PrintConfig.cpp:6114+` support-bottom-interface-spacing/support-interface-speed/following options.

Exit criteria are tracked in `docs/milestones/m120-print-config-support-interface-layers-spacing-registry.md`.

## M121: PrintConfig support bottom interface spacing, interface speed, and patterns registry
Port the adjacent FFF support bottom-interface spacing, support-interface speed, support base pattern, and support interface pattern `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6114-6176` plus `SupportMaterialPattern` and `SupportMaterialInterfacePattern` enum maps in `PrintConfig.hpp:172-177`, `PrintConfig.hpp:190-192`, `PrintConfig.cpp:312-320`, and `PrintConfig.cpp:333-340` into `ares-core` registry metadata. This historical registry milestone deferred typed/runtime behavior when it was completed. A later source-cited slice now consumes `support_interface_speed` for non-first-layer constructed `PrintPathRole::SupportMaterialInterface` print speed and G-code feedrate while preserving Ares's existing first-layer infill-speed policy. Support bottom-interface spacing behavior, support base/interface pattern selection behavior, support geometry, support transition speed behavior, preset/object override behavior, and `PrintConfig.cpp:6178+` support-base-pattern-spacing/support-expansion/support-speed/support-style/following options remain deferred.

Exit criteria are tracked in `docs/milestones/m121-print-config-support-interface-pattern-speed-registry.md`.

## M122: PrintConfig support pattern spacing, speed, expansion, and style registry
Port the adjacent FFF support base-pattern spacing, normal support expansion, support speed, and support style `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6178-6230` plus the `SupportMaterialStyle` enum map in `PrintConfig.hpp:179-181` and `PrintConfig.cpp:322-331` into `ares-core` registry metadata. This historical registry milestone deferred typed/runtime behavior when it was completed. A later source-cited slice now consumes `support_speed` for non-first-layer constructed `PrintPathRole::SupportMaterial` print speed and G-code feedrate while preserving Ares's existing first-layer infill-speed policy. Support spacing behavior, support expansion behavior, support style selection behavior, support geometry, support transition speed behavior, preset/object override behavior, and `PrintConfig.cpp:6232+` independent-support-layer-height/following options remain deferred.

Exit criteria are tracked in `docs/milestones/m122-print-config-support-pattern-spacing-style-registry.md`.

## M123: PrintConfig support independent layer height and threshold registry
Port the adjacent FFF independent support layer height, support threshold angle, and support threshold overlap `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6232-6262` into `ares-core` registry metadata. Defer independent support layer-height behavior, prime-tower invalidation behavior, support threshold angle/overlap behavior, support generation, support geometry, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6264+` tree-support/following options.

Exit criteria are tracked in `docs/milestones/m123-print-config-support-threshold-layer-height-registry.md`.

## M124: PrintConfig tree support branch and tip registry
Port the adjacent FFF tree-support branch angle, preferred branch angle, branch distance, branch density, auto brim, brim width, and tip diameter `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6264-6354` into `ares-core` registry metadata. Defer tree-support generation, organic support branch routing, branch-density behavior, auto-brim width calculation, tree-support brim geometry, support geometry, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6356+` branch-diameter/following options.

Exit criteria are tracked in `docs/milestones/m124-print-config-tree-support-branch-tip-registry.md`.

## M125: PrintConfig tree support diameter, wall, and infill registry
Port the adjacent FFF tree-support branch diameter, branch diameter angle, organic branch diameter, support wall count, and tree-support-with-infill `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6356-6404` into `ares-core` registry metadata. Defer tree-support generation, branch-diameter tapering, organic support branch routing, wall-loop generation, tree-support infill generation, support geometry, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6406+` support-ironing/following options.

Exit criteria are tracked in `docs/milestones/m125-print-config-tree-support-diameter-wall-registry.md`.

## M126: PrintConfig support ironing registry
Port the adjacent FFF support interface ironing enable, pattern, flow, and spacing `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6406-6446` plus the `InfillPattern` enum map in `PrintConfig.hpp:87-98` and `PrintConfig.cpp:225-255` into `ares-core` registry metadata. Defer support interface ironing behavior, pattern application, flow/spacing behavior, support geometry, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6448+` chamber-temperature/following options.

Exit criteria are tracked in `docs/milestones/m126-print-config-support-ironing-registry.md`.

## M127: PrintConfig chamber temperature registry
Port the adjacent FFF chamber-temperature control and chamber-temperature `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6448-6476` into `ares-core` registry metadata. Defer chamber temperature control behavior, M191/M141 command emission, chamber-temperature start-G-code variable handling, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6478+` nozzle-temperature/following options.

Exit criteria are tracked in `docs/milestones/m127-print-config-chamber-temperature-registry.md`.

## M128: PrintConfig nozzle temperature registry
Port the adjacent FFF nozzle temperature, nozzle temperature range high, and nozzle temperature range low `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6478-6501` into `ares-core` registry metadata. Defer nozzle temperature behavior, temperature-range validation, M104/M109 emission, start-G-code variable handling, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6503+` head-wrap/thin-wall/G-code/following options.

Exit criteria are tracked in `docs/milestones/m128-print-config-nozzle-temperature-registry.md`.

## M129: PrintConfig head-wrap detect zone and thin-wall registry
Port the adjacent FFF head-wrap detect zone and thin-wall detection `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6503-6514` into `ares-core` registry metadata. Defer head-wrap/clumping detection behavior, probe-zone behavior, thin-wall geometric detection, single-line thin-wall generation, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6516+` G-code/top-surface/following options.

Exit criteria are tracked in `docs/milestones/m129-print-config-head-wrap-thin-wall-registry.md`.

## M130: PrintConfig change G-code registry
Port the adjacent filament-change and extrusion-role-change G-code `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6516-6541` into `ares-core` registry metadata. Defer filament-change G-code insertion, extrusion-role-change G-code insertion, tool-change behavior, active-filament-specific behavior, placeholder expansion, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6543+` top-surface/following options.

Exit criteria are tracked in `docs/milestones/m130-print-config-change-gcode-registry.md`.

## M131: PrintConfig top-surface and top-shell registry
Port the adjacent FFF top-surface line-width/speed and top-shell layers/thickness `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6543-6584` into `ares-core` registry metadata. Defer top-surface line-width computation, top-surface speed planning, top-shell layer adjustment from thickness, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6586+` top/bottom surface-density/following options.

Exit criteria are tracked in `docs/milestones/m131-print-config-top-surface-shell-registry.md`.


## M132: PrintConfig surface-density registry
Port the adjacent FFF top and bottom surface-density `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6586-6607` into `ares-core` registry metadata. Defer top/bottom surface-density runtime interpretation, surface pattern behavior, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6610+` travel-speed/following options.

Exit criteria are tracked in `docs/milestones/m132-print-config-surface-density-registry.md`.


## M133: PrintConfig travel-speed registry
Port the adjacent FFF travel-speed and Z-travel-speed `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6610-6626` into `ares-core` registry metadata. Defer travel-speed/Z-travel runtime behavior, speed planning, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6628+` wipe/prime-tower/following options.

Exit criteria are tracked in `docs/milestones/m133-print-config-travel-speed-registry.md`.


## M134: PrintConfig wipe and prime-tower base registry
Port the adjacent FFF wipe and prime-tower base `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6628-6657` into `ares-core` registry metadata. Defer wipe movement behavior, wipe-distance movement planning, prime tower generation/internal-rib behavior, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6659+` flush-volume/prime-volume/wipe-tower/following options. A mechanical registry-table shard split is allowed only to keep Rust files below 400 LOC.

Exit criteria are tracked in `docs/milestones/m134-print-config-wipe-prime-tower-base-registry.md`.


## M135: PrintConfig flush and prime-volume registry
Port the adjacent FFF flush-volume and prime-volume `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6659-6692` into `ares-core` registry metadata. Defer flush-volume vector/matrix interpretation, flush multiplier application, prime-volume use, prime tower behavior, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6694+` wipe-tower placement/dimension/following options.

Exit criteria are tracked in `docs/milestones/m135-print-config-flush-prime-volume-registry.md`.


## M136: PrintConfig wipe-tower placement and width registry
Port the adjacent FFF wipe-tower X/Y placement and prime-tower width `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6694-6716` into `ares-core` registry metadata. Defer wipe-tower placement behavior, partplate placement logic, prime-tower width use, prime tower generation, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6718+` wipe-tower rotation/brim/cone/following options.

Exit criteria are tracked in `docs/milestones/m136-print-config-wipe-tower-placement-width-registry.md`.


## M137: PrintConfig wipe-tower angle and brim registry
Port the adjacent FFF wipe-tower rotation, prime-tower brim width, and wipe-tower cone angle `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6718-6744` into `ares-core` registry metadata. Defer wipe-tower rotation behavior, prime-tower brim width use/auto calculation, cone stabilization behavior, prime tower generation, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6746+` wipe-tower purge-speed/wall-type/following options.

Exit criteria are tracked in `docs/milestones/m137-print-config-wipe-tower-angle-brim-registry.md`.


## M138: PrintConfig wipe-tower max purge speed registry
Port the wipe-tower maximum purge speed `libslic3r::PrintConfigDef::init_fff_params` option definition from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6746-6757` into `ares-core` registry metadata. Defer wipe-tower purge-speed selection, sparse-layer speed fallback, filament max-volumetric-speed comparison, prime tower generation, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6759+` wipe-tower wall-type/extra-rib/following options.

Exit criteria are tracked in `docs/milestones/m138-print-config-wipe-tower-max-purge-speed-registry.md`.


## M139: PrintConfig wipe-tower wall type registry
Port the wipe-tower wall type `libslic3r::PrintConfigDef::init_fff_params` enum option definition from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6759-6773` plus `WipeTowerWallType` enum map from `PrintConfig.hpp:405-408` and `PrintConfig.cpp:558-563` into `ares-core` registry metadata. Defer runtime rectangle/cone/rib wall-shape selection, cone/fillet/rib geometry, prime tower generation, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6775+` extra-rib/rib-width/fillet/filament/following options.

Exit criteria are tracked in `docs/milestones/m139-print-config-wipe-tower-wall-type-registry.md`.


## M140: PrintConfig wipe-tower rib and filament registry
Port the adjacent FFF wipe-tower extra rib length, rib width, fillet wall, and perimeter filament `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6775-6808` into `ares-core` registry metadata. Include only the mechanical registry-table shard split needed to keep Rust files below 400 LOC. Defer rib sizing behavior, rib-width constraints, fillet wall geometry, wipe-tower filament selection, non-soluble preference, prime tower generation, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6810+` wiping-volumes/skip-points/following options.

Exit criteria are tracked in `docs/milestones/m140-print-config-wipe-tower-rib-filament-registry.md`.

## M141: PrintConfig prime-tower interface registry
Port the adjacent FFF wiping-volume and prime-tower interface `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6810-6845` into `ares-core` registry metadata. Defer wiping-volume interpretation, purge-volume computation, prime-tower skip-point behavior, flat-ironing behavior, tower-interface feature handling, interface cooldown, prime-tower infill-gap application, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6847+` flush-into/following options.

Exit criteria are tracked in `docs/milestones/m141-print-config-prime-tower-interface-registry.md`.

## M142: PrintConfig flush-into registry
Port the adjacent FFF flush-into `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6847-6870` into `ares-core` registry metadata. Defer purge routing into object infill/support/selected objects, prime-tower dependency behavior, color-mixing/object assignment behavior, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6872+` wipe-tower bridging/extra-spacing/extra-flow/idle-temperature/following options.

Exit criteria are tracked in `docs/milestones/m142-print-config-flush-into-registry.md`.

## M143: PrintConfig wipe-tower extra and idle-temperature registry
Port the adjacent FFF wipe-tower bridging, wipe-tower purge-line extra spacing/flow, and idle-temperature `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6872-6905` into `ares-core` registry metadata. Defer wipe-tower bridging distance behavior, purge-line spacing/flow application, idle-temperature/ooze-prevention behavior, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6907+` XY compensation/polyhole/following options.

Exit criteria are tracked in `docs/milestones/m143-print-config-wipe-tower-extra-idle-temperature-registry.md`.

## M144: PrintConfig XY compensation and polyhole registry
Port the adjacent FFF XY hole/contour compensation and polyhole `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6907-6954` into `ares-core` registry metadata. Defer XY compensation application, polyhole detection/conversion/twist behavior, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6956+` thumbnails/following options.

Exit criteria are tracked in `docs/milestones/m144-print-config-xy-polyhole-registry.md`.

## M145: PrintConfig thumbnails registry
Port the adjacent G-code thumbnail `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6956-6978` and `GCodeThumbnailsFormat` enum map from `PrintConfig.hpp:397-399` / `PrintConfig.cpp:542-549` into `ares-core` registry metadata. Defer thumbnail string validation/normalization, thumbnail image generation/encoding, G-code thumbnail embedding, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:6980+` use-relative-E/wall-generator/following options.

Exit criteria are tracked in `docs/milestones/m145-print-config-thumbnails-registry.md`.

## M146: PrintConfig relative E and wall-generator registry
Port the adjacent relative extrusion and wall-generator `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:6980-7001` and `PerimeterGeneratorType` enum map from `PrintConfig.hpp:294-300` / `PrintConfig.cpp:520-524` into `ares-core` registry metadata. Defer relative-E G-code behavior, wipe-tower relative-E validation, classic/Arachne perimeter generation, wall-transition behavior, typed accessors, UI/runtime behavior, slicing behavior, geometry behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7003+` wall-transition/following options.

Exit criteria are tracked in `docs/milestones/m146-print-config-relative-e-wall-generator-registry.md`.

## M147: PrintConfig wall-transition registry
Port the adjacent wall-transition `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7003-7049` and fields from `PrintConfig.hpp:1021-1024` into `ares-core` registry metadata. Defer Arachne/classic perimeter generation, wall-transition planning/geometry behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7051+` `min_feature_size`/following options.

Exit criteria are tracked in `docs/milestones/m147-print-config-wall-transition-registry.md`.

## M148: PrintConfig minimum feature and wall-length registry
Port the adjacent minimum feature and minimum wall-length `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7051-7074` and fields from `PrintConfig.hpp:1025` and `PrintConfig.hpp:1039` into `ares-core` registry metadata. Defer minimum feature filtering/widening, short wall pruning, Arachne/classic perimeter generation, wall-transition planning/geometry behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7076+` wall-resolution/bead-width/following options.

Exit criteria are tracked in `docs/milestones/m148-print-config-min-feature-length-registry.md`.

## M149: PrintConfig wall maximum resolution registry
Port the adjacent wall maximum resolution/deviation `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7076-7097` and fields from `PrintConfig.hpp:1030-1031` into `ares-core` registry metadata. Defer wall path simplification, maximum deviation enforcement, Arachne/classic perimeter generation, wall-transition planning/geometry behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7099+` bead-width/following options.

2026-06-29 update: `wall_maximum_resolution` and `wall_maximum_deviation` now affect eligible Arachne closed-loop perimeter simplification in Ares and are visible in emitted perimeter/print-path G-code geometry. Exact Orca variable-width `ExtrusionLine` simplification metrics, scaled-coordinate rounding, junction-width/accumulated-area handling, and prepared-outline repair remain deferred.

Exit criteria are tracked in `docs/milestones/m149-print-config-wall-maximum-registry.md`.

## M150: PrintConfig bead-width registry
Port the adjacent first-layer minimum bead-width and minimum bead-width `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7099-7119` and fields from `PrintConfig.hpp:1026-1027` into `ares-core` registry metadata. Defer first-layer bead-width selection, minimum wall-width replacement, thin-feature widening, Arachne/classic perimeter generation, wall-transition planning/geometry behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7121+` filament override/following behavior.

Exit criteria are tracked in `docs/milestones/m150-print-config-bead-width-registry.md`.

## M151: PrintConfig filament extruder override registry
Port the generated filament extruder override `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7121-7156`, the key list from `PrintConfig.cpp:63-83`, and declaration from `PrintConfig.hpp:512` into `ares-core` registry metadata. Defer runtime filament override resolution/following behavior, retraction/z-hop/wipe/toolpath behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7158-7165` `detect_narrow_internal_solid_infill`.

Exit criteria are tracked in `docs/milestones/m151-print-config-filament-extruder-override-registry.md`.

## M152: PrintConfig narrow internal solid infill registry
Port the adjacent narrow internal solid infill detection `libslic3r::PrintConfigDef::init_fff_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7154-7161` and field from `PrintConfig.hpp:1017` into `ares-core` registry metadata. Defer narrow internal solid infill detection, concentric/rectilinear pattern selection, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7164+` extruder/filament option-key initialization behavior.

Exit criteria are tracked in `docs/milestones/m152-print-config-narrow-internal-solid-infill-registry.md`.

## M153: PrintConfig extruder/filament option key lists
Port the adjacent `PrintConfigDef` extruder and filament option-key list accessors from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7164-7227` and `PrintConfig.hpp:569-593` into `ares-core` read-only registry API data. Defer runtime array-option expansion by extruder count, filament override/following behavior, retraction/z-hop/wipe/toolpath behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7229+` SLA parameter definitions.

Exit criteria are tracked in `docs/milestones/m153-print-config-extruder-filament-key-lists.md`.

## M154: PrintConfig SLA display and tilt registry
Port the first SLA printer display/tilt `libslic3r::PrintConfigDef::init_sla_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7235-7310`, enum metadata from `PrintConfig.cpp:400-404` and `PrintConfig.hpp:260-263`, and fields from `PrintConfig.hpp:1830-1836` and `PrintConfig.hpp:1845-1847` into `ares-core` registry metadata. Defer SLA display orientation/mirroring/pixel behavior, tilt timing behavior, area-fill behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7312+` later SLA settings.

Exit criteria are tracked in `docs/milestones/m154-print-config-sla-display-tilt-registry.md`.

## M155: PrintConfig SLA relative correction registry
Port the next SLA printer correction `libslic3r::PrintConfigDef::init_sla_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7312-7318` and field from `PrintConfig.hpp:1837` into `ares-core` registry metadata. Defer SLA relative correction/scaling behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7320+` later SLA settings.

Exit criteria are tracked in `docs/milestones/m155-print-config-sla-relative-correction-registry.md`.

## M156: PrintConfig SLA axis and absolute correction registry
Port the next SLA printer correction `libslic3r::PrintConfigDef::init_sla_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7320-7349` and fields from `PrintConfig.hpp:1838-1841` into `ares-core` registry metadata. Defer SLA correction/scaling behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7351+` later SLA settings.

Exit criteria are tracked in `docs/milestones/m156-print-config-sla-axis-absolute-correction-registry.md`.

## M157: PrintConfig SLA foot and gamma correction registry
Port the next SLA printer correction `libslic3r::PrintConfigDef::init_sla_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7351-7367` and fields from `PrintConfig.hpp:1843-1844` into `ares-core` registry metadata. Defer SLA foot/gamma correction behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7370+` SLA material settings.

Exit criteria are tracked in `docs/milestones/m157-print-config-sla-foot-gamma-registry.md`.

## M158: PrintConfig SLA material identity and cost registry
Port the first SLA material settings from `libslic3r::PrintConfigDef::init_sla_params` in `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7370-7423` and fields from `PrintConfig.hpp:1811-1814` into `ares-core` registry metadata. Defer SLA material behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7425+` later SLA settings.

Exit criteria are tracked in `docs/milestones/m158-print-config-sla-material-identity-cost-registry.md`.

## M159: PrintConfig SLA exposure time registry
Port the next SLA faded-layer and exposure-time `libslic3r::PrintConfigDef::init_sla_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7425-7477`, material fields from `PrintConfig.hpp:1815-1816`, and printer exposure-bound fields from `PrintConfig.hpp:1848-1851` into `ares-core` registry metadata. Defer SLA exposure timing behavior, faded-layer behavior, material correction behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7479+` later SLA settings.

Exit criteria are tracked in `docs/milestones/m159-print-config-sla-exposure-time-registry.md`.

## M160: PrintConfig SLA material correction registry
Port the next SLA material correction `libslic3r::PrintConfigDef::init_sla_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7479-7505` and fields from `PrintConfig.hpp:1817-1820` into `ares-core` registry metadata, with a mechanical registry shard split to keep Rust files below 400 LOC. Defer SLA material correction/scaling behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, `material_print_speed`, and `PrintConfig.cpp:7507+` later SLA settings.

Exit criteria are tracked in `docs/milestones/m160-print-config-sla-material-correction-registry.md`.

## M161: PrintConfig SLA profile identifiers registry
Port the next SLA material/print profile identifier `libslic3r::PrintConfigDef::init_sla_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7507-7535` into `ares-core` registry metadata. Defer SLA profile-selection behavior, settings-id resolution, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, `material_print_speed`, and `PrintConfig.cpp:7537+` later SLA support settings.

Exit criteria are tracked in `docs/milestones/m161-print-config-sla-profile-identifiers-registry.md`.

## M162: PrintConfig SLA support head and pillar registry
Port the next SLA support head/pillar `libslic3r::PrintConfigDef::init_sla_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7537-7611`, enum metadata from `PrintConfig.hpp:265-269` / `PrintConfig.cpp:406-411`, and fields from `PrintConfig.hpp:1674-1696` into `ares-core` registry metadata. Defer SLA support generation, support-head geometry, pillar geometry, bridge planning, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, `material_print_speed`, and `PrintConfig.cpp:7613+` later SLA support settings.

Exit criteria are tracked in `docs/milestones/m162-print-config-sla-support-head-pillar-registry.md`.

## M163: PrintConfig SLA support base and placement registry
Port the next SLA support base/placement `libslic3r::PrintConfigDef::init_sla_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7613-7694` and fields from `PrintConfig.hpp:1698-1727` into `ares-core` registry metadata. Defer SLA support generation, support base geometry, support placement, pillar/link planning, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, `material_print_speed`, and `PrintConfig.cpp:7696+` support points/pad/later SLA settings.

Exit criteria are tracked in `docs/milestones/m163-print-config-sla-support-base-placement-registry.md`.

## M164: PrintConfig SLA support points registry
Port the next automatic SLA support-points `libslic3r::PrintConfigDef::init_sla_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7696-7710` and fields from `PrintConfig.hpp:1729-1731` into `ares-core` registry metadata. Defer automatic support-point placement, SLA support generation, support geometry, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, `material_print_speed`, and `PrintConfig.cpp:7712+` pad/later SLA settings.

Exit criteria are tracked in `docs/milestones/m164-print-config-sla-support-points-registry.md`.

## M165: PrintConfig SLA pad base registry
Port the next SLA pad/base-pool `libslic3r::PrintConfigDef::init_sla_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7712-7766` and fields from `PrintConfig.hpp:1733-1755` into `ares-core` registry metadata, with a mechanical expected-key shard split to keep Rust files below 400 LOC. Defer SLA pad generation, pad/base geometry, base-pool merging, wall slope behavior, zero-elevation pad mode, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7768+` zero-elevation pad/hollowing/material settings.

Exit criteria are tracked in `docs/milestones/m165-print-config-sla-pad-base-registry.md`.

## M166: PrintConfig SLA zero-elevation pad registry
Port the next SLA zero-elevation object-pad `libslic3r::PrintConfigDef::init_sla_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7768-7817` and fields from `PrintConfig.hpp:1757-1780` into `ares-core` registry metadata, with a mechanical registry definition shard split to keep Rust files below 400 LOC. Defer zero-elevation pad behavior, object-pad connector geometry, SLA pad generation, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7819+` hollowing/material/later SLA settings.

Exit criteria are tracked in `docs/milestones/m166-print-config-sla-pad-zero-elevation-registry.md`.

## M167: PrintConfig SLA hollowing registry
Port the next SLA hollowing `libslic3r::PrintConfigDef::init_sla_params` option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7819-7853` and fields from `PrintConfig.hpp:1791-1802` into `ares-core` registry metadata. Defer SLA hollowing runtime behavior, cavity generation, wall-thickness enforcement, drain-hole behavior, OpenVDB/voxel behavior, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7855+` material-speed/later SLA settings.

Exit criteria are tracked in `docs/milestones/m167-print-config-sla-hollowing-registry.md`.

## M168: PrintConfig SLA material speed registry
Port the final `init_sla_params` SLA material-speed option-definition slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7855-7864`, enum metadata from `PrintConfig.cpp:413-417` / `PrintConfig.hpp:1805`, and field from `PrintConfig.hpp:1821` into `ares-core` registry metadata. Defer SLA material-speed runtime behavior, SL1 export profile selection, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, G-code behavior, and `PrintConfig.cpp:7867+` legacy handling/later non-`init_sla_params` behavior.

Exit criteria are tracked in `docs/milestones/m168-print-config-sla-material-speed-registry.md`.

## M169: PrintConfig legacy key aliases
Port the first simple legacy-option normalization slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7867-7899` into `ares-core` `SliceOptions` ingestion. Defer `PrintConfig.cpp:7900+` percentage-value erasure, cumulative-key renames, cooling/timelapse/support enum migrations, recursive `different_settings_to_system` normalization, later legacy handling, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m169-print-config-legacy-key-aliases.md`.

## M170: PrintConfig legacy simple value migrations
Port the next simple legacy-option normalization slice from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7900-7932` into `ares-core` `SliceOptions` ingestion. Defer `PrintConfig.cpp:7933+` recursive `different_settings_to_system`, overhang-fan threshold migration, wall sequence migration, nozzle/extruder variant value replacements, power-loss recovery enum migration, shell-thickness migration, later aliases/value migrations, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m170-print-config-legacy-simple-value-migrations.md`.

## M171: PrintConfig legacy different-settings key-list normalization
Port the `different_settings_to_system` recursive key-list normalization branch from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7933-7943` into `ares-core` `SliceOptions` ingestion. Defer `PrintConfig.cpp:7944+` overhang-fan threshold migration, wall sequence migration, nozzle/extruder variant value replacements, power-loss recovery enum migration, shell-thickness migration, later aliases/value migrations, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m171-print-config-legacy-different-settings-key-list.md`.

## M172: PrintConfig legacy wall sequence migrations
Port the `overhang_fan_threshold` and `wall_infill_order` legacy normalization branches from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7944-7958` into `ares-core` `SliceOptions` ingestion. Defer `PrintConfig.cpp:7959+` nozzle/extruder variant value replacements, extruder type migration, power-loss recovery enum migration, shell-thickness migration, later aliases/value migrations, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m172-print-config-legacy-wall-sequence-migrations.md`.

## M173: PrintConfig legacy extruder variant values
Port the nozzle/extruder variant string replacement and `extruder_type` legacy normalization branches from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7959-7970` into `ares-core` `SliceOptions` ingestion. Defer `PrintConfig.cpp:7971+` power-loss recovery enum migration, shell-thickness migration, rotate solid infill migration, later aliases/value migrations, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m173-print-config-legacy-extruder-variant-values.md`.

## M174: PrintConfig legacy recovery shell rotation migrations
Port the `enable_power_loss_recovery`, `ensure_vertical_shell_thickness`, and `rotate_solid_infill_direction` legacy normalization branches from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7971-7991` into `ares-core` `SliceOptions` ingestion. Defer `PrintConfig.cpp:7992+` infill-anchor aliases, chamber/thumbnail aliases, top-one-wall migration, ironing aliases/value migration, pattern migrations, filament migrations, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m174-print-config-legacy-recovery-shell-rotation.md`.

## M175: PrintConfig legacy alias and top-wall migrations
Port the next legacy alias and conditional top-wall branches from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:7992-8004` into `ares-core` `SliceOptions` ingestion. Defer `PrintConfig.cpp:8005+` `ironing_direction` alias, negative `ironing_angle` migration, counterbore spelling alias, draft-shield value migration, pattern migrations, filament migrations, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m175-print-config-legacy-alias-top-wall.md`.

## M176: PrintConfig legacy ironing and draft-shield migrations
Port the `ironing_direction`, negative `ironing_angle`, `counterbole_hole_bridging`, and `draft_shield` legacy branches from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8005-8012` into `ares-core` `SliceOptions` ingestion. Defer `PrintConfig.cpp:8013+` pattern migrations, filament migrations, prime-tower rib migrations, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m176-print-config-legacy-ironing-draft-shield.md`.

## M177: PrintConfig legacy pattern migrations
Port the six-key `zig-zag` to `rectilinear` legacy pattern branch from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8013-8019` into `ares-core` `SliceOptions` ingestion. Defer `PrintConfig.cpp:8020+` filament map/type migrations, prime-tower rib migrations, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m177-print-config-legacy-pattern-migrations.md`.

## M178: PrintConfig legacy filament migrations
Port the `filament_map_mode` and `filament_type` legacy branches from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8020-8045` into `ares-core` `SliceOptions` ingestion. Defer `PrintConfig.cpp:8046+` prime-tower rib migrations, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m178-print-config-legacy-filament-migrations.md`.

## M179: PrintConfig legacy prime-tower rib and hardware migrations
Port the prime-tower rib aliases, `extruder_clearance_max_radius`, `machine_switch_extruder_time`, and `wall_direction` legacy branch from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8046-8067` into `ares-core` `SliceOptions` ingestion. Defer `PrintConfig.cpp:8069+` obsolete-key ignore handling, final key validation, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m179-print-config-legacy-prime-tower-rib-hardware.md`.

## M180: PrintConfig legacy obsolete-key ignore list
Port the obsolete configuration key ignore set from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8069-8091` into `ares-core` `SliceOptions` ingestion. Defer `PrintConfig.cpp:8093+` final key validation, composite legacy handling, typed accessors, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m180-print-config-legacy-obsolete-key-ignore.md`.

## M181: PrintConfig legacy thumbnail composite normalization
Port the thumbnail composite normalization branch from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8099-8130` plus `OrcaSlicer/src/libslic3r/GCode/Thumbnails.cpp:530-578` parser behavior into `ares-core` `SliceOptions` ingestion. Defer `PrintConfig.cpp:8093-8096` final key validation until the registry can safely validate all Orca options, and defer `PrintConfig.cpp:8132+` wiping-volume matrix handling, thumbnail rendering, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m181-print-config-legacy-thumbnail-composite.md`.

## M182: PrintConfig legacy wiping-volumes matrix composite
Port the wiping-volumes matrix composite inference branch from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8132-8150` into `ares-core` `SliceOptions` ingestion. Defer `PrintConfig.cpp:8093-8096` final key validation until the registry can safely validate all Orca options, and defer typed purge-volume behavior, prime tower behavior, UI/runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m182-print-config-legacy-wiping-volumes-composite.md`.

## M183: GCodeThumbnails definition parser API
Port the rendering-neutral thumbnail definition parser boundary from `OrcaSlicer/src/libslic3r/GCode/Thumbnails.hpp:16-41`, `OrcaSlicer/src/libslic3r/GCode/Thumbnails.cpp:530-604`, `PrintConfig.hpp:397-399`, and `PrintConfig.cpp:542-549` into `ares-core` as a reusable API for option ingestion and future UI/adapter consumers. Defer compression, image generation, file/G-code export, UI runtime behavior, slicing behavior, extrusion behavior, and G-code writer behavior.

Exit criteria are tracked in `docs/milestones/m183-gcode-thumbnails-definition-parser-api.md`.

## M184: PrintConfig variant option key sets
Port the global variant-related `std::set<std::string>` option key sets from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8154-8287` into `ares-core` as read-only registry API data for future UI/API consumers and later source-cited variant-expansion milestones. Defer commented-out candidate keys, variant expansion/lookup behavior, silent-mode behavior, extruder-count expansion, filament override behavior, typed accessors, option parsing behavior, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m184-print-config-variant-option-sets.md`.

## M185: PrintConfig min object distance API
Port `min_object_distance(const ConfigBase&)` from `OrcaSlicer/src/libslic3r/PrintConfig.hpp:602-603` and `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8305-8329` into `ares-core` as `SliceOptions::min_object_distance()` for future UI/arrange consumers. Defer object arrangement/placement algorithms, `PrintConfig.cpp:8332+` normalization/runtime behavior, variant expansion, silent-mode behavior, option parsing changes outside this API, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m185-print-config-min-object-distance-api.md`.

## M186: PrintConfig normalize_fdm extruder role propagation
Port the first `DynamicPrintConfig::normalize_fdm` branch from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8332-8353` into `ares-core` as the initial explicit `SliceOptions::normalize_fdm(used_filaments)` API behavior. Defer commented-out support propagation, `PrintConfig.cpp:8355+` spiral-mode, resolution, prime-tower and later normalization branches, automatic deserialization normalization, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m186-print-config-normalize-fdm-extruder-roles.md`.

## M187: PrintConfig normalize_fdm spiral mode normalization
Port the `spiral_mode` branch from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8355-8369` into the existing explicit `SliceOptions::normalize_fdm(used_filaments)` API. Defer `PrintConfig.cpp:8372+` resolution, prime-tower and later normalization branches, CLI spiral validation, automatic deserialization normalization, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m187-print-config-normalize-fdm-spiral-mode.md`.

## M188: PrintConfig normalize_fdm resolution clamp
Port the optional `resolution` lower-bound clamp from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8372-8374` into the existing explicit `SliceOptions::normalize_fdm(used_filaments)` API. Defer `PrintConfig.cpp:8376+` prime-tower and later normalization branches, automatic deserialization normalization, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m188-print-config-normalize-fdm-resolution-clamp.md`.

## M189: PrintConfig normalize_fdm prime tower normalization
Port the prime-tower normalization branch from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8376-8401` into the existing explicit `SliceOptions::normalize_fdm(used_filaments)` API. Defer commented-out adaptive-layer-height handling, commented-out independent-support-height re-enable behavior, `PrintConfig.cpp:8403+` split/duplicate normalization branches, automatic deserialization normalization, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m189-print-config-normalize-fdm-prime-tower.md`.

## M190: PrintConfig normalize_fdm_2 prime tower changed keys
Port the changed-key-returning prime-tower normalization branch from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8452-8505` and declaration context from `PrintConfig.hpp:628-631` into `ares-core` as an explicit `SliceOptions::normalize_fdm_2(num_objects, used_filaments)` API for future UI/API consumers. Defer `normalize_fdm_1`, commented-out adaptive-layer-height handling, commented-out independent-support re-enable behavior, automatic `Print::Apply` integration, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m190-print-config-normalize-fdm-2-prime-tower.md`.

## M191: PrintConfig handle_legacy_sla correction expansion
Port the SLA legacy correction-vector expansion from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8507-8527`, declaration context from `PrintConfig.hpp:693`, and call-site context from `Preset.cpp:486` / `Model.cpp:456` into `ares-core` `SliceOptions` ingestion. Defer `PrintConfig.cpp:8529+` parameter sizing and extruder-variant behavior, preset/model loading machinery, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m191-print-config-handle-legacy-sla-corrections.md`.

## M192: PrintConfig get_parameter_size API
Port `DynamicPrintConfig::get_parameter_size` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8529-8556`, declaration context from `PrintConfig.hpp:633`, and already-ported M184 variant key-set context into `ares-core` as read-only `SliceOptions::parameter_size(param_name, extruder_nums)` for UI/config consumers. Defer `PrintConfig.cpp:8558+` extruder-variant extension, `set_num_extruders`, `set_num_filaments`, vector resizing, `FullPrintConfig::defaults`, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m192-print-config-parameter-size-api.md`.

## M193: PrintConfig extend_extruder_variant API
Port `extend_extruder_variant` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8558-8591`, call-site context from `PrintConfig.cpp:8593-8596`, option-definition anchors from `PrintConfig.cpp:5239-5264`, and declaration context from `PrintConfig.hpp:634` into `ares-core` as explicit `SliceOptions::extend_extruder_variant(num_extruders)`. Defer `PrintConfig.cpp:8597+` generic option-vector resizing, `set_num_filaments`, validation, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m193-print-config-extend-extruder-variant-api.md`.

## M194: PrintConfig set_num_extruders vector resizing API
Port `DynamicPrintConfig::set_num_extruders` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8593-8610`, using already-ported M193 `extend_extruder_variant`, already-ported M192 `get_parameter_size`, M184 `extruder_option_keys`, `Config.hpp:635-663` vector resize semantics, and `Config.cpp:295-315` default creation context into `ares-core` as explicit `SliceOptions::set_num_extruders(num_extruders)`. Defer `set_num_filaments`, validation, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m194-print-config-set-num-extruders-resize-api.md`.

## M195: PrintConfig set_num_filaments vector resizing API
Port `DynamicPrintConfig::set_num_filaments` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8612-8627`, using M184 `filament_option_keys`, `Config.hpp:635-663` vector resize semantics, and `Config.cpp:295-315` default creation context into `ares-core` as explicit `SliceOptions::set_num_filaments(num_filaments)`. Defer validation, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m195-print-config-set-num-filaments-resize-api.md`.

## M196: PrintConfig validate basic dimension and count checks
Port the initial validation checks from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10088-10128`, plus `libslic3r.h:60` scaling-factor context, into `ares-core` as explicit `SliceOptions::validate_basic_fdm_options()`. Defer full `DynamicPrintConfig::validate` dispatch, `FullPrintConfig` materialization, `PrintConfig.cpp:10131+` validation checks, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m196-print-config-validate-basic-dimensions.md`.

## M197: PrintConfig validate firmware retraction compatibility
Port the firmware-retraction validation checks from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10131-10145`, plus `PrintConfig.cpp:161-176` and `PrintConfig.hpp:33-46` G-code flavor mapping context, into `ares-core` as explicit `SliceOptions::validate_firmware_retraction_options()`. Defer `PrintConfig.cpp:10147-10150` `gcode_flavor` enum validation, `PrintConfig.cpp:10152+` validation checks, full `DynamicPrintConfig::validate` dispatch, `FullPrintConfig` materialization, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m197-print-config-validate-firmware-retraction.md`.

## M198: PrintConfig validate gcode flavor enum value
Port the `gcode_flavor` enum validation check from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10147-10150`, plus `PrintConfig.cpp:3785-3817` / `PrintConfig.hpp:1355` active option enum-value context and `PrintConfig.cpp:161-176` / `PrintConfig.hpp:33-46` serialization mapping context, into `ares-core` as explicit `SliceOptions::validate_gcode_flavor_option()`. Defer `PrintConfig.cpp:10152+` validation checks, full `DynamicPrintConfig::validate` dispatch, `FullPrintConfig` materialization, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m198-print-config-validate-gcode-flavor-enum.md`.

## M199: PrintConfig validate infill pattern enum values
Port the infill-pattern enum validation checks from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10152-10170`, plus `PrintConfig.cpp:1986-2025` top/bottom/internal solid pattern option enum-value context, `PrintConfig.cpp:2928-2985` sparse infill pattern option enum-value context, and `PrintConfig.cpp:225-255` / `PrintConfig.hpp:87-98` `InfillPattern` serialization context, into `ares-core` as explicit `SliceOptions::validate_infill_pattern_options()`. Defer `PrintConfig.cpp:10172+` validation checks, full `DynamicPrintConfig::validate` dispatch, `FullPrintConfig` materialization, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m199-print-config-validate-infill-pattern-enums.md`.

## M200: PrintConfig validate skirt height and bridge flow ratios
Port the skirt-height and bridge-flow validation checks from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10172-10185`, plus `PrintConfig.cpp:1266-1284` / `PrintConfig.hpp:1083-1084` bridge-flow option context and `PrintConfig.cpp:5559-5565` / `PrintConfig.hpp:1553` skirt-height option context, into `ares-core` as explicit `SliceOptions::validate_skirt_and_bridge_flow_options()`. Preserve the upstream source behavior where the `internal_bridge_flow` error is also guarded by `cfg.bridge_flow <= 0`. Defer `PrintConfig.cpp:10187+` validation checks, full `DynamicPrintConfig::validate` dispatch, `FullPrintConfig` materialization, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m200-print-config-validate-skirt-and-bridge-flow.md`.

## M201: PrintConfig validate extruder clearance dimensions
Port the extruder-clearance validation checks from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10187-10198`, plus `PrintConfig.cpp:2127-2160` and `PrintConfig.hpp:1513-1516` option-definition/default context, into `ares-core` as explicit `SliceOptions::validate_extruder_clearance_options()`. Defer `PrintConfig.cpp:10200+` validation checks, full `DynamicPrintConfig::validate` dispatch, `FullPrintConfig` materialization, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m201-print-config-validate-extruder-clearance.md`.

## M202: PrintConfig validate filament flow ratio
Port the filament-flow-ratio validation check from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10200-10205`, plus `PrintConfig.cpp:2227-2237` / `PrintConfig.hpp:1301` option-definition/default context, into `ares-core` as explicit `SliceOptions::validate_filament_flow_ratio_options()`. Preserve existing Ares numeric-vector serialization for `invalid value {serialized_vector}` messages and split the growing validation implementation before adding new logic so modified Rust files remain under 400 LOC. Defer `PrintConfig.cpp:10207+` validation checks, full `DynamicPrintConfig::validate` dispatch, `FullPrintConfig` materialization, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m202-print-config-validate-filament-flow-ratio.md`.

## M203: PrintConfig validate spiral vase CLI constraints
Port the CLI-only spiral-vase validation block from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10207-10235`, plus `PrintConfig.cpp:2881-2889`, `4918-4924`, `5678-5684`, `5903-5908`, `6013-6025`, `6564-6573` and `PrintConfig.hpp:948`, `958`, `1101`, `1158`, `1167`, `1560` option-definition/default and field-type context, into `ares-core` as explicit `SliceOptions::validate_spiral_vase_cli_options()`. Preserve the upstream `cfg.spiral_mode && under_cli` scope and exact constrained-key predicates for `wall_loops`, `sparse_infill_density`, `top_shell_layers`, `enable_support`, and `enforce_support_layers`. Defer non-CLI popup correction behavior, `PrintConfig.cpp:10237+` validation checks, full `DynamicPrintConfig::validate` dispatch, `FullPrintConfig` materialization, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m203-print-config-validate-spiral-vase-cli.md`.

## M204: PrintConfig validate extrusion width limit
Port the extrusion-width upper-limit validation block from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10237-10261`, plus `libslic3r.h:68`, `Config.hpp:1259-1285`, `Config.cpp:690-743`, `Config.cpp:745-753`, `PrintConfig.cpp:2027-2037`, `3251-3261`, `3944-3962`, `4016-4026`, `4896-4906`, `5657-5667`, `6043-6053`, `6543-6553` and `PrintConfig.hpp:960`, `1093`, `1122`, `1130`, `1131`, `1155`, `1162`, `1166`, `1527` multiplier, FloatOrPercent, no-argument message-value lookup, option-definition/default, and field-type context, into `ares-core` as explicit `SliceOptions::validate_extrusion_width_options()`. Preserve the source `max_nozzle_diameter` computation, nine-key width list, strict `> 5 * max_nozzle_diameter` predicate, no-argument `cfg.get_abs_value(key)` message-value path where well-defined, and the documented explicit-base message deviation for percent line-width values affected by the upstream `XXX_extrusion_width` no-argument lookup FIXME. Defer `PrintConfig.cpp:10263+` generic out-of-range validation, full `DynamicPrintConfig::validate` dispatch, `FullPrintConfig` materialization, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m204-print-config-validate-extrusion-width-limit.md`.

## M205: PrintConfig validate line-width numeric ranges
Port the first bounded line-width slice of the generic numeric out-of-range validation loop from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10263-10294`, plus `Config.cpp:321-338`, `Config.hpp:2476-2481`, `Config.hpp:1259-1299`, `PrintConfig.cpp:2027-2037`, `2322-2332`, `3251-3261`, `4016-4026`, `4896-4906`, `5657-5667`, `6043-6053`, `6543-6553` and `PrintConfig.hpp:960`, `1093`, `1122`, `1155`, `1162`, `1166`, `1527` predicate, FloatOrPercent, option min/max/default, and field-type context, into `ares-core` as explicit `SliceOptions::validate_line_width_range_options()`. Preserve the upstream raw-value `coFloatOrPercent` range predicate, including the source 1e-4 approximate min/max boundary acceptance and source-style range message for `line_width` and the finite-max M204 line-width keys. Defer min-only `skin_infill_line_width` and `skeleton_infill_line_width`. Defer all other numeric keys/types in the generic range loop, full `DynamicPrintConfig::validate` dispatch, `FullPrintConfig` materialization, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m205-print-config-validate-line-width-ranges.md`.

## M206: PrintConfig validate min-only line-width numeric ranges
Port the two min-only line-width options deferred from M205 in the generic numeric out-of-range validation loop from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10263-10294`, plus `Config.cpp:321-338`, `Config.hpp:2476-2481`, `Config.hpp:1259-1299`, `PrintConfig.cpp:3944-3962`, and `PrintConfig.hpp:1130-1131` predicate, default `FLT_MAX` max, FloatOrPercent, option min/default, and field-type context, into the existing `ares-core` `SliceOptions::validate_line_width_range_options()` API. Preserve the upstream raw-value `coFloatOrPercent` range predicate, including source 1e-4 approximate min/max boundary acceptance and source-style range messages for `skin_infill_line_width` and `skeleton_infill_line_width` with range `[0,FLT_MAX]`. Defer all other numeric keys/types in the generic range loop, full `DynamicPrintConfig::validate` dispatch, `FullPrintConfig` materialization, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m206-print-config-validate-min-only-line-width-ranges.md`.

## M207: PrintConfig validate FFF aggregate API
Port the source-order aggregation behavior of OrcaSlicer's FFF validation function from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10088-10308` into `ares-core` as `SliceOptions::validate_fff_options(under_cli)`, composing the already ported M196-M206 validation slices in source order. Preserve C++ `std::map::emplace` first-write-wins duplicate-key behavior, call the existing spiral-vase CLI slice only when aggregate `under_cli` is true, and preserve extrusion-width-before-generic-range same-key suppression. Defer `DynamicPrintConfig::validate` printer-technology dispatch, `FullPrintConfig` materialization, SLA/non-FFF behavior, unported generic numeric keys/types, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m207-print-config-validate-fff-aggregate-api.md`.

## M208: DynamicPrintConfig validate printer-technology dispatch
Port the printer-technology dispatch shell from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8629-8647`, plus `PrintConfig.hpp:641`, `PrintConfig.cpp:131-135`, and `PrintConfig.cpp:676-682` declaration, enum mapping, and default/option-definition context, into `ares-core` as `SliceOptions::validate_print_config(under_cli)`. Preserve absent `printer_technology` defaulting to FFF, FFF dispatch to the existing M207 `validate_fff_options(under_cli)`, and non-FFF/SLA empty-map behavior. Defer `FullPrintConfig fpc; fpc.apply(*this, true)`, typed C++ enum storage/deserialization, future SLA validation, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m208-print-config-dynamic-validate-dispatch.md`.

## Support Z-distance runtime status
Ares now parses and consumes `support_top_z_distance`, `support_bottom_z_distance`, and `enforce_support_layers` as crate-internal runtime state before support print-path finalization rewrites. This slice is source-cited to `OrcaSlicer/src/libslic3r/PrintConfig.hpp:956-958`, `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5981-6025`, and the `zero_topZ_contact` derivation in `OrcaSlicer/src/libslic3r/Slicing.cpp:81-120`; `SupportZDistanceOptions::zero_top_contact()` is the Rust state mapping for that upstream predicate. Ares enforces finite/range checks at the input boundary according to the Orca option-definition limits, including `enforce_support_layers` in `0..=5000`.

This milestone does not change support geometry. Contact-layer topology, support invalidation, raft contact handling, and enforced-support region generation remain deferred to future source-cited support-generator slices.

## M209: DynamicPrintConfig filament type display API
Port the UI-facing support-filament display mapping from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8649-8714`, plus `PrintConfig.hpp:657`, `Config.hpp:624-630`, `Config.hpp:1886-1892`, `PrintConfig.cpp:2784-2797`, `PrintConfig.cpp:2812-2816`, and `PrintConfig.hpp:1322`, `1327` declaration, vector `get_at`, and filament option context, into `ares-core` as `SliceOptions::filament_type_display(id)`. Preserve source returned/displayed value distinctions, support IDs `GFS00`/`GFS01`, PLA/PA support fallbacks, and first-value vector fallback. Defer `PrintConfig.cpp:8716+`, plural `filament_ids` profile behavior, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m209-print-config-filament-type-display-api.md`.

## M210: DynamicPrintConfig different extruders API
Port the different-extruder detection helper from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8716-8742`, plus `PrintConfig.hpp:660`, `Config.hpp:624-630`, `PrintConfig.hpp:412-421`, `PrintConfig.cpp:565-575`, `PrintConfig.cpp:5202-5225`, and `PrintConfig.hpp:1408-1409` declaration, vector `get_at`, enum serialization, and option context, into `ares-core` as `SliceOptions::is_using_different_extruders()`. Preserve source behavior where missing or single `nozzle_diameter` returns false, missing enum vectors return false, later `extruder_type` or `nozzle_volume_type` differences return true, and enum vectors use first-value fallback. Defer `PrintConfig.cpp:8744+`, `support_different_extruders`, `get_index_for_extruder`, variant lookup, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m210-print-config-different-extruders-api.md`.

## M211: DynamicPrintConfig support different extruders API
Port the support-different-extruders helper from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8744-8766`, plus `PrintConfig.hpp:661`, `Config.hpp:624-630`, `PrintConfig.cpp:5239-5244`, and existing `nozzle_diameter` option context, into `ares-core` as `SliceOptions::support_different_extruders()`. Preserve source behavior where the resolved nozzle vector length becomes the returned extruder count, missing `extruder_variant_list` returns false, variant strings are split with source comma/token-compress semantics, unique variant tokens across nozzle-indexed `extruder_variant_list.get_at(index)` values determine support, and variant vectors use first-value fallback. Defer `PrintConfig.cpp:8768+`, `get_index_for_extruder`, generated variant IDs, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m211-print-config-support-different-extruders-api.md`.

## M212: DynamicPrintConfig get_index_for_extruder no-id lookup
Port the no-id-map branch of `DynamicPrintConfig::get_index_for_extruder` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8768-8818`, plus `PrintConfig.hpp:662`, `PrintConfig.cpp:586-604`, `PrintConfig.hpp:412-421`, `PrintConfig.cpp:565-575`, `Config.hpp:624-630`, `PrintConfig.cpp:5252-5264`, `PrintConfig.cpp:5272-5284`, and `PrintConfig.cpp:5292-5304` declaration, variant-string, enum-map, vector `get_at`, and printer/print/filament ID option context, into `ares-core` as `SliceOptions::get_index_for_extruder_no_id(...)`. Preserve source missing-option `-1`, source-order first-match lookup, `index * stride` return including zero stride, and no-match `-1`. Defer the `id_opt` branch, generated extruder IDs, `extruder_variant_list` generated ID lookup, `extruder_or_filament_id` matching, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m212-print-config-get-index-for-extruder-no-id.md`.

## M213: DynamicPrintConfig get_index_for_extruder complete-id lookup
Port the complete-ID-map branch of `DynamicPrintConfig::get_index_for_extruder` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8768-8818`, plus `PrintConfig.hpp:662`, `PrintConfig.cpp:586-604`, `PrintConfig.hpp:412-421`, `PrintConfig.cpp:565-575`, `Config.hpp:624-630`, `PrintConfig.cpp:5252-5264`, `PrintConfig.cpp:5272-5284`, and `PrintConfig.cpp:5292-5304` declaration, variant-string, enum-map, vector `get_at`, and printer/print/filament ID option context, into `ares-core` as `SliceOptions::get_index_for_extruder_complete_id_map(ExtruderIndexIdMapLookup { ... })`. Preserve source missing-variant-option `-1`, source-order first variant+ID match, `index * stride` return including zero stride, and no-match `-1`. Defer incomplete-ID generated extruder IDs, `extruder_variant_list` generated ID lookup, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m213-print-config-get-index-for-extruder-complete-id-map.md`.

## M214: DynamicPrintConfig get_index_for_extruder generated-ID lookup
Port the incomplete-ID generated-extruder-ID branch of `DynamicPrintConfig::get_index_for_extruder` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8768-8818`, plus `PrintConfig.hpp:662`, `PrintConfig.cpp:586-604`, `PrintConfig.hpp:412-421`, `PrintConfig.cpp:565-575`, `Config.hpp:624-630`, `PrintConfig.cpp:5239-5244`, `PrintConfig.cpp:5252-5264`, `PrintConfig.cpp:5272-5284`, and `PrintConfig.cpp:5292-5304` declaration, variant-string, enum-map, vector `get_at`, `extruder_variant_list`, and representative option context, into `ares-core` as `SliceOptions::get_index_for_extruder_generated_id_map(ExtruderIndexIdMapLookup { ... })`. Preserve source missing-variant-option `-1`, source generated ID `0` when `extruder_variant_list` is absent or target index is beyond non-empty trimmed tokens, generated ID `extruder_index + 1` from comma-split/trimmed/non-empty variant-token order, source-order first variant+generated-ID match, `index * stride` return including zero stride, and no-match `-1`. Defer preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m214-print-config-get-index-for-extruder-generated-id.md`.

## M215: DynamicPrintConfig update_values_from_single_to_multi string/int copy
Port the `coStrings` and `coInts` copy branches of `DynamicPrintConfig::update_values_from_single_to_multi` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8824-8865`, plus `PrintConfig.hpp:670`, `PrintConfig.cpp:8826-8831`, `PrintConfig.cpp:8833-8843`, `PrintConfig.cpp:5252-5264`, `PrintConfig.cpp:5272-5284`, and `PrintConfig.cpp:5292-5304` declaration, missing-variant guard, option-definition lookup, and representative option context, into `ares-core` as `SliceOptions::update_values_from_single_to_multi_string_int_keys(...)`. Preserve source missing-variant `-1`, source-equivalent sorted/unique key processing, unknown-key skip, missing-source skip, and full string/int vector copy for supported source keys. Defer float, FloatOrPercent, bool resize branches, `update_values_from_multi_to_multi`, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m215-print-config-update-single-to-multi-string-int.md`.

## M216: DynamicPrintConfig update_values_from_single_to_multi float limit
Port the `coFloats` resize-and-limit branch of `DynamicPrintConfig::update_values_from_single_to_multi` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8824-8880`, plus `PrintConfig.hpp:670`, `PrintConfig.cpp:8826-8831`, `PrintConfig.cpp:8833-8843`, `PrintConfig.cpp:8866-8880`, `Config.hpp:635-662`, `Config.hpp:812-870`, and `PrintConfig.cpp:766-773`, `2349-2357`, `4591-4599` declaration, guard, option-definition lookup, float-vector branch, vector resize, and representative option context, into `ares-core` by extending the single-to-multi update helper with `OptionValueKind::Floats` behavior. Preserve source missing-variant `-1`, source-equivalent sorted/unique key processing, unknown-key skip, missing-source skip, target resize by truncating or duplicating the first/default value, and per-index target-value limiting only when the target is greater than the source. Defer FloatOrPercent, bool resize branches, `update_values_from_multi_to_multi`, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m216-print-config-update-single-to-multi-float-limit.md`.

## M217: DynamicPrintConfig update_values_from_single_to_multi FloatOrPercent limit
Port the `coFloatsOrPercents` resize-and-limit branch of `DynamicPrintConfig::update_values_from_single_to_multi` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8824-8896`, plus `PrintConfig.hpp:670`, `PrintConfig.cpp:8826-8831`, `PrintConfig.cpp:8833-8843`, `PrintConfig.cpp:8881-8896`, `Config.hpp:31-39`, `Config.hpp:635-662`, `Config.hpp:1318-1448`, and `PrintConfig.cpp:2027-2037`, `2322-2332`, `3104-3112` declaration, guard, option-definition lookup, FloatOrPercent branch, value/percent data shape, vector resize, parse/serialize, and representative option context, into `ares-core` by extending the single-to-multi update helper with `OptionValueKind::FloatOrPercent` behavior. Preserve source missing-variant `-1`, source-equivalent sorted/unique key processing, unknown-key skip, missing-source skip, target resize by truncating or duplicating the first/default value, and per-index target-value limiting only when the target numeric value is greater than the source numeric value while copying/preserving the complete percent flag. Defer bool resize branch, `update_values_from_multi_to_multi`, preset/model loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m217-print-config-update-single-to-multi-float-or-percent-limit.md`.

## M218: DynamicPrintConfig update_values_from_single_to_multi bool resize
Port the `coBools` resize-only branch of `DynamicPrintConfig::update_values_from_single_to_multi` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8824-8908`, plus `PrintConfig.hpp:670`, `PrintConfig.cpp:8826-8831`, `PrintConfig.cpp:8833-8843`, `PrintConfig.cpp:8897-8908`, `Config.hpp:635-662`, and representative bool option context from `PrintConfig.cpp:1800-1804`, `2252-2255`, `2812-2816`, and `6628-6633`, into `ares-core` by extending the single-to-multi update helper with `OptionValueKind::Bools` behavior. Preserve source missing-variant `-1`, source-equivalent sorted/unique key processing, unknown-key skip, missing-source skip, source bool-vector length assertion as an Ares `InvalidInput`, target resize by truncating or duplicating the first/default value, and the source branch's resize-only behavior where source bool values are not copied after resize. Defer `update_values_from_multi_to_multi`, preset/profile loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m218-print-config-update-single-to-multi-bool-resize.md`.

## M219: DynamicPrintConfig update_values_from_multi_to_multi string/int copy
Port the guard, variant-index preparation, and `coStrings`/`coInts` copy branches of `DynamicPrintConfig::update_values_from_multi_to_multi` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8982-9064`, plus `PrintConfig.hpp:671`, `PrintConfig.cpp:8984-9017`, `PrintConfig.cpp:8988-8993`, `PrintConfig.cpp:9019-9031`, and representative string/int option context from `PrintConfig.cpp:5252-5264`, `5272-5284`, and `5292-5304`, into `ares-core` as `SliceOptions::update_values_from_multi_to_multi_string_int_keys(...)`. Preserve source missing current/new variant or new id `-1`, source-equivalent same-variant and new-variant-index preparation, sorted/unique key processing, unknown-key skip, missing-source skip, and full string/int vector copy for supported new-config source keys. Defer float, FloatOrPercent, and bool old-value merge branches, `update_values_from_multi_to_multi_2`, preset/profile loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m219-print-config-update-multi-to-multi-string-int.md`.

## M220: DynamicPrintConfig update_values_from_multi_to_multi float merge
Port the `coFloats` old-value merge branch of `DynamicPrintConfig::update_values_from_multi_to_multi` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8982-9093`, plus `PrintConfig.hpp:671`, `PrintConfig.cpp:8984-9017`, `PrintConfig.cpp:8988-8993`, `PrintConfig.cpp:9019-9031`, existing string/int branch context from `PrintConfig.cpp:9032-9064`, `PrintConfig.cpp:9065-9093`, `Config.hpp:635-662`, and representative float option context from `PrintConfig.cpp:766-773`, `2349-2357`, and `4591-4599`, into `ares-core` by extending the multi-to-multi update helper with `OptionValueKind::Floats` behavior. Preserve source missing current/new variant or new id `-1`, source-equivalent same-variant and new-variant-index preparation, sorted/unique key processing, unknown-key skip, missing-source skip, full source float vector copy, and lower old same-variant value preservation at matching new variant indices. Defer FloatOrPercent and bool old-value merge branches, `update_values_from_multi_to_multi_2`, preset/profile loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m220-print-config-update-multi-to-multi-float-merge.md`.

## M221: DynamicPrintConfig update_values_from_multi_to_multi FloatOrPercent merge
Port the `coFloatsOrPercents` old-value merge branch of `DynamicPrintConfig::update_values_from_multi_to_multi` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8982-9123`, plus `PrintConfig.hpp:671`, `PrintConfig.cpp:8984-9017`, `PrintConfig.cpp:8988-8993`, `PrintConfig.cpp:9019-9031`, existing string/int/float branch context from `PrintConfig.cpp:9032-9093`, `PrintConfig.cpp:9095-9123`, `Config.hpp:31-42`, `Config.hpp:1318-1448`, and representative FloatOrPercent option context from `PrintConfig.cpp:2027-2037`, `2322-2332`, and `3104-3112`, into `ares-core` by extending the multi-to-multi update helper with `OptionValueKind::FloatOrPercent` behavior. Preserve source missing current/new variant or new id `-1`, source-equivalent same-variant and new-variant-index preparation, sorted/unique key processing, unknown-key skip, missing-source skip, full source FloatOrPercent vector copy, and lower old same-variant numeric value preservation at matching new variant indices while preserving the selected value's percent flag. Defer bool old-value merge branch, `update_values_from_multi_to_multi_2`, preset/profile loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m221-print-config-update-multi-to-multi-float-or-percent-merge.md`.

## M222: DynamicPrintConfig update_values_from_multi_to_multi bool merge
Port the `coBools` old-value merge branch of `DynamicPrintConfig::update_values_from_multi_to_multi` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8982-9155`, plus `PrintConfig.hpp:671`, `PrintConfig.cpp:8984-9017`, `PrintConfig.cpp:8988-8993`, `PrintConfig.cpp:9019-9031`, existing string/int/float/FloatOrPercent branch context from `PrintConfig.cpp:9032-9123`, `PrintConfig.cpp:9125-9155`, `Config.hpp:635-662`, and representative bool option context from `PrintConfig.cpp:1800-1804`, `2252-2255`, `2812-2816`, and `6628-6633`, into `ares-core` by extending the multi-to-multi update helper with `OptionValueKind::Bools` behavior. Preserve source missing current/new variant or new id `-1`, source-equivalent same-variant and new-variant-index preparation, sorted/unique key processing, unknown-key skip, missing-source skip, full source bool vector copy, and old same-variant true-value preservation at matching new variant indices. Defer `update_values_from_multi_to_multi_2`, preset/profile loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m222-print-config-update-multi-to-multi-bool-merge.md`.

## M223: DynamicPrintConfig update_values_from_multi_to_multi_2 float nullable merge
Port the first `coFloats` nullable branch of `DynamicPrintConfig::update_values_from_multi_to_multi_2` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9165-9221`, plus `PrintConfig.hpp:676`, `PrintConfig.cpp:9172-9190`, `PrintConfig.cpp:9191-9197`, `PrintConfig.cpp:9199-9221`, and nullable float nil semantics from `Config.hpp:837-838` and `Config.hpp:952`, into `ares-core` as a source/destination variant remap helper for float and nullable-float option vectors. Preserve source iteration over keys present in `self`, key-set filtering, same-variant index lookup without all-source fallback, destination-config baseline copy, missing destination key rejection, nil source skip, and minimum non-nil source value overwrite. Defer FloatOrPercent and bool branches, `update_values_from_multi_to_single_2`, preset/profile loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m223-print-config-update-multi-to-multi-2-float-nullable-merge.md`.

## M224: DynamicPrintConfig update_values_from_multi_to_multi_2 FloatOrPercent nullable merge
Port the `coFloatsOrPercents` nullable branch of `DynamicPrintConfig::update_values_from_multi_to_multi_2` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9165-9246`, plus `PrintConfig.hpp:676`, `PrintConfig.cpp:9172-9197`, `PrintConfig.cpp:9223-9246`, `Config.hpp:31-42`, and nullable FloatOrPercent nil semantics from `Config.hpp:1344-1345` and `Config.hpp:1450`, into the existing `ares-core` source/destination variant remap helper. Preserve source iteration over keys present in `self`, key-set filtering, same-variant index lookup without all-source fallback, destination-config baseline copy, nil source skip, strict-`<` candidate overwrite from an initial `9999%` sentinel, equal-or-greater-than-sentinel behavior, and selected source percent-flag preservation only when a source value replaces the sentinel. Defer the bool branch, `update_values_from_multi_to_single_2`, preset/profile loading, UI runtime behavior, slicing behavior, extrusion behavior, and G-code behavior.

Exit criteria are tracked in `docs/milestones/m224-print-config-update-multi-to-multi-2-float-or-percent-nullable-merge.md`.

## M225: DynamicPrintConfig update_values_from_multi_to_multi_2 bool nullable merge
Port the `coBools` nullable branch of `DynamicPrintConfig::update_values_from_multi_to_multi_2` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9165-9272`, plus `PrintConfig.hpp:676`, `PrintConfig.cpp:9172-9197`, `PrintConfig.cpp:9247-9272`, and nullable bool storage/nil semantics from `Config.hpp:1857-1967`, into the existing `ares-core` source/destination variant remap helper. Preserve source iteration over keys present in `self`, key-set filtering, same-variant index lookup without all-source fallback, destination-config baseline copy, nil source skip, first non-nil same-variant bool overwrite, and destination preservation when no non-nil match exists. Defer `update_values_from_multi_to_single_2`, preset/profile loading, UI runtime behavior, slicing behavior, extrusion behavior, G-code behavior, crate changes, and dependency changes.

Exit criteria are tracked in `docs/milestones/m225-print-config-update-multi-to-multi-2-bool-nullable-merge.md`.

## M226: DynamicPrintConfig update_values_from_multi_to_single_2 float nullable collapse
Port the first `coFloats` nullable branch of the commented `DynamicPrintConfig::update_values_from_multi_to_single_2` helper from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9286-9321`, plus `PrintConfig.hpp:673-674`, `PrintConfig.cpp:9290-9304`, `PrintConfig.cpp:9307-9325`, and nullable float nil semantics from `Config.hpp:837-838` and `Config.hpp:952`, into `ares-core` as a source-present key-set filtered helper that collapses float/nullable-float option vectors to one value. Preserve nil skip, the upstream `9999.0` strict-less sentinel minimum selection, collapse by erasing entries after index `0`, and original-first-entry preservation when no source value replaces the sentinel. Defer FloatOrPercent and bool branches, preset/profile loading, UI runtime behavior, slicing behavior, extrusion behavior, G-code behavior, crate changes, and dependency changes.

Exit criteria are tracked in `docs/milestones/m226-print-config-update-multi-to-single-2-float-nullable-collapse.md`.

## M227: DynamicPrintConfig update_values_from_multi_to_single_2 FloatOrPercent nullable collapse
Port the `coFloatsOrPercents` nullable branch of the commented `DynamicPrintConfig::update_values_from_multi_to_single_2` helper from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9286-9344`, plus `PrintConfig.hpp:673-674`, `PrintConfig.cpp:9290-9304`, `PrintConfig.cpp:9326-9344`, `Config.hpp:31-42`, and nullable FloatOrPercent nil semantics from `Config.hpp:1344-1345` and `Config.hpp:1450`, into the existing `ares-core` multi-to-single-2 helper. Preserve nil skip, the upstream `9999%` strict raw-value sentinel selection, selected source percent-flag preservation, collapse by erasing entries after index `0`, and original-first-entry preservation when no source value replaces the sentinel. Defer the bool branch, preset/profile loading, UI runtime behavior, slicing behavior, extrusion behavior, G-code behavior, crate changes, and dependency changes.

Exit criteria are tracked in `docs/milestones/m227-print-config-update-multi-to-single-2-float-or-percent-nullable-collapse.md`.

## M228: DynamicPrintConfig update_values_from_multi_to_single_2 bool nullable collapse
Port the `coBools` nullable branch of the commented `DynamicPrintConfig::update_values_from_multi_to_single_2` helper from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9286-9371`, plus `PrintConfig.hpp:673-674`, `PrintConfig.cpp:9290-9304`, `PrintConfig.cpp:9345-9363`, and nullable bool storage/nil semantics from `Config.hpp:1857-1967`, into the existing `ares-core` multi-to-single-2 helper. Preserve nil skip, first non-nil bool selection, `false` as a real selected value, collapse by erasing entries after index `0`, and original-first-entry preservation when no source bool is selected. Defer preset/profile loading, UI runtime behavior, slicing behavior, extrusion behavior, G-code behavior, crate changes, and dependency changes.

Exit criteria are tracked in `docs/milestones/m228-print-config-update-multi-to-single-2-bool-nullable-collapse.md`.

## M229: DynamicPrintConfig filament identity query API
Port OrcaSlicer's zero-argument filament identity query helpers from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9373-9396`, plus declaration context from `PrintConfig.hpp:678-681`, `filament_type` option context from `PrintConfig.cpp:2784-2797` and `PrintConfig.hpp:1322`, and `filament_vendor` option context from `PrintConfig.cpp:2854-2859` and `PrintConfig.hpp:1326`, into `ares-core` as read-only `SliceOptions::filament_vendor()` and `SliceOptions::filament_type()` APIs. Preserve first-entry string-vector return behavior and empty-string fallback for absent or empty vectors. Defer `update_values_to_printer_extruders`, multiple-filament identity behavior, preset/profile loading, UI runtime behavior, slicing behavior, extrusion behavior, G-code behavior, crate changes, and dependency changes.

Exit criteria are tracked in `docs/milestones/m229-print-config-filament-identity-query-api.md`.

## M230: DynamicPrintConfig update_values_to_printer_extruders string/int copy
Port the guard, variant-index preparation, and `coStrings`/`coInts` copy branches of `DynamicPrintConfig::update_values_to_printer_extruders(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9398-9489`, plus declaration context from `PrintConfig.hpp:663`, prerequisite helper context from `PrintConfig.cpp:8744-8818`, vector `get_at` fallback semantics from `Config.hpp:624-630`, and representative string/int option context from `PrintConfig.cpp:5252-5264`, `5272-5284`, and `5292-5304`, into `ares-core` as `SliceOptions::update_values_to_printer_extruders_string_int_keys(...)`. Preserve the support-different-extruders guard, missing enum-vector skip, selected/all-extruder variant-index preparation, all-extruder negative-index fallback to zero, sorted/unique key processing, and string/int `get_at` copy semantics. Defer float, percent, FloatOrPercent, bool, enum, multiple-filament, preset/profile, UI runtime, slicing, extrusion, G-code, crate, and dependency changes.

Exit criteria are tracked in `docs/milestones/m230-print-config-update-to-printer-extruders-string-int.md`.

## M231: DynamicPrintConfig update_values_to_printer_extruders float/percent copy
Port the `coFloats` and `coPercents` copy branches of `DynamicPrintConfig::update_values_to_printer_extruders(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9491-9517`, plus setup context from `PrintConfig.cpp:9398-9462`, declaration context from `PrintConfig.hpp:663`, vector `get_at` fallback semantics from `Config.hpp:624-630`, float-vector context from `Config.hpp:812-870`, and representative float/percent option context from `PrintConfig.cpp:2227-2237`, `4591-4599`, `4651-4658`, `737-747`, and `6839-6845`, into `ares-core` by extending the existing `SliceOptions::update_values_to_printer_extruders_string_int_keys(...)` helper with float/percent copy behavior. Preserve the support-different-extruders guard, missing enum-vector skip, selected/all-extruder variant-index preparation, all-extruder negative-index fallback to zero, sorted/unique key processing, source `get_at` fallback, nullable `"nil"` preservation, finite numeric JSON copy, and no-partial-mutation behavior. Defer FloatOrPercent, bool, enum, multiple-filament, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m231-print-config-update-to-printer-extruders-float-percent.md`.

## M232: DynamicPrintConfig update_values_to_printer_extruders FloatOrPercent copy
Port the `coFloatsOrPercents` copy branch of `DynamicPrintConfig::update_values_to_printer_extruders(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9519-9532`, plus setup context from `PrintConfig.cpp:9398-9462`, declaration context from `PrintConfig.hpp:663`, vector `get_at` fallback semantics from `Config.hpp:624-630`, FloatOrPercent storage/serialization context from `Config.hpp:31-42` and `Config.hpp:1318-1450`, and representative FloatOrPercent option context from `PrintConfig.cpp:3017-3043`, `3045-3066`, `3104-3112`, `4016-4026`, and `6936-6947`, into `ares-core` by extending the existing `SliceOptions::update_values_to_printer_extruders_string_int_keys(...)` helper with FloatOrPercent copy behavior. Preserve the support-different-extruders guard, missing enum-vector skip, selected/all-extruder variant-index preparation, all-extruder negative-index fallback to zero, sorted/unique key processing, source `get_at` fallback, absolute/percent representation, and no-partial-mutation behavior. Defer bool, enum, multiple-filament, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m232-print-config-update-to-printer-extruders-float-or-percent.md`.

## M233: DynamicPrintConfig update_values_to_printer_extruders bool copy
Port the `coBools` copy branch of `DynamicPrintConfig::update_values_to_printer_extruders(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9534-9547`, plus setup context from `PrintConfig.cpp:9398-9462`, declaration context from `PrintConfig.hpp:663`, vector `get_at` fallback semantics from `Config.hpp:624-630`, bool vector and nullable bool context from `Config.hpp:1857-1967`, and representative bool-vector option context from `PrintConfig.cpp:1800-1804`, `2252-2255`, `2334-2338`, `5062-5066`, and `5081-5086`, into `ares-core` by extending the existing `SliceOptions::update_values_to_printer_extruders_string_int_keys(...)` helper with bool copy behavior. Preserve the support-different-extruders guard, missing enum-vector skip, selected/all-extruder variant-index preparation, all-extruder negative-index fallback to zero, sorted/unique key processing, source `get_at` fallback, nullable `"nil"` preservation for nullable bool vectors, and no-partial-mutation behavior. Defer enum, multiple-filament, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m233-print-config-update-to-printer-extruders-bool.md`.

## M234: DynamicPrintConfig update_values_to_printer_extruders enum copy
Port the `coEnums` copy branch of `DynamicPrintConfig::update_values_to_printer_extruders(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9549-9560`, plus setup context from `PrintConfig.cpp:9398-9462`, declaration context from `PrintConfig.hpp:663`, vector `get_at` fallback semantics from `Config.hpp:624-630`, generic enum vector and nullable enum context from `Config.hpp:2101-2201`, and representative enum-vector option context from `PrintConfig.cpp:5149-5162`, `5187-5200`, `5215-5225`, `CommonDefs.hpp:12-20`, and `PrintConfig.cpp:3652-3669`, into `ares-core` by extending the existing `SliceOptions::update_values_to_printer_extruders_string_int_keys(...)` helper with enum copy behavior. Preserve the support-different-extruders guard, missing enum-vector skip, selected/all-extruder variant-index preparation, all-extruder negative-index fallback to zero, sorted/unique key processing, source `get_at` fallback, nullable `"nil"` preservation for nullable enum vectors, and no-partial-mutation behavior. Defer multiple-filament, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m234-print-config-update-to-printer-extruders-enum.md`.

## M235: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments string/int copy
Port the guard, filament-map setup, per-filament variant-index preparation, and `coStrings`/`coInts` copy branches of `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9569-9675`, plus declaration context from `PrintConfig.hpp:664`, `filament_map` option context from `PrintConfig.cpp:2401-2405`, `filament_extruder_variant` / `filament_self_index` context from `PrintConfig.cpp:5292-5304`, vector `get_at` fallback semantics from `Config.hpp:624-630`, and lookup context from `PrintConfig.cpp:8744-8818`, into `ares-core` as a multiple-filament update helper. Preserve the support-different-extruders guard, missing-prerequisite skip behavior, `filament_map` mapped extruder lookup, per-filament `get_index_for_extruder` resolution, negative lookup fallback to matching id or zero, sorted/unique key processing, unknown/missing/unsupported key skip, default output slots for out-of-range variant indices, and no-partial-mutation behavior. Defer float, percent, FloatOrPercent, bool, enum, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m235-print-config-update-to-printer-extruders-multiple-filament-string-int.md`.

## M236: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments float/percent copy
Port the `coFloats` and `coPercents` copy branches of `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9676-9717`, plus setup context from `PrintConfig.cpp:9569-9633`, declaration context from `PrintConfig.hpp:664`, vector `get_at` fallback semantics from `Config.hpp:624-630`, float/nullable float and percent vector context from `Config.hpp:812-1091 and Config.hpp:1204-1257`, and representative filament numeric option context from `PrintConfig.cpp:2462-2470`, `5055-5060`, and `5068-5075`, into `ares-core` by extending the existing multiple-filament update helper. Preserve M235 guard, missing-prerequisite skip behavior, `filament_map` mapped extruder lookup, per-filament variant-index preparation, negative lookup fallback, sorted/unique key processing, unknown/missing/unsupported key skip, numeric zero default output slots for out-of-range indices and empty source vectors, nullable `"nil"` preservation for nullable numeric kinds, and no-partial-mutation behavior. Defer FloatOrPercent, bool, enum, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m236-print-config-update-to-printer-extruders-multiple-filament-float-percent.md`.

## M237: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments FloatOrPercent copy
Port the `coFloatsOrPercents` copy branch of `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9718-9738`, plus setup context from `PrintConfig.cpp:9569-9633`, declaration context from `PrintConfig.hpp:664`, vector `get_at` fallback semantics from `Config.hpp:624-630`, FloatOrPercent storage/serialization context from `Config.hpp:31-42` and `Config.hpp:1318-1450`, and representative FloatOrPercent option context from `PrintConfig.cpp:3017-3043`, `3045-3066`, `3104-3112`, `4016-4026`, and `6936-6947`, into `ares-core` by extending the existing multiple-filament update helper. Preserve M235/M236 guard, missing-prerequisite skip behavior, `filament_map` mapped extruder lookup, per-filament variant-index preparation, negative lookup fallback, sorted/unique key processing, unknown/missing/unsupported key skip, numeric zero default output slots for out-of-range indices and empty source vectors, absolute/percent representation, and no-partial-mutation behavior. Defer bool, enum, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m237-print-config-update-to-printer-extruders-multiple-filament-float-or-percent.md`.

## M238: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments bool copy
Port the `coBools` copy branch of `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9739-9758`, plus setup context from `PrintConfig.cpp:9569-9633`, declaration context from `PrintConfig.hpp:664`, bool vector and nullable bool storage context from `Config.hpp:1857-1967`, and representative bool option context from `PrintConfig.cpp:2252-2255`, `2557-2565`, `5062-5066`, `5081-5086`, and `6628-6633`, into `ares-core` by extending the existing multiple-filament update helper. Preserve M235-M237 guard, missing-prerequisite skip behavior, `filament_map` mapped extruder lookup, per-filament variant-index preparation, negative lookup fallback, sorted/unique key processing, unknown/missing/unsupported key skip, `false` default output slots for out-of-range indices and empty source vectors, nullable `"nil"` preservation for nullable bool kinds, and no-partial-mutation behavior. Defer enum, default unsupported logging, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m238-print-config-update-to-printer-extruders-multiple-filament-bool.md`.

## M239: DynamicPrintConfig update_values_to_printer_extruders_for_multiple_filaments enum copy
Port the `coEnums` copy branch of `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9760-9780`, plus setup context from `PrintConfig.cpp:9569-9633`, declaration context from `PrintConfig.hpp:664`, enum vector and nullable enum storage context from `Config.hpp:2101-2201`, and representative enum option context from `PrintConfig.cpp:5149-5162`, `5187-5200`, `5202-5213`, `5215-5225`, and `3652-3669`, into `ares-core` by extending the existing multiple-filament update helper. Preserve M235-M238 guard, missing-prerequisite skip behavior, `filament_map` mapped extruder lookup, per-filament variant-index preparation, negative lookup fallback, sorted/unique key processing, unknown/missing/unsupported key skip, empty-string default output slots for out-of-range indices and empty source vectors, nullable `"nil"` preservation for nullable enum kinds, and no-partial-mutation behavior. Defer default unsupported logging, preset/profile, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m239-print-config-update-to-printer-extruders-multiple-filament-enum.md`.

## M240: DynamicPrintConfig normalize stride-2 float vectors
Port the anonymous-namespace `normalize_stride2_floats(...)` helper from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9789-9830`, plus downstream use context from `PrintConfig.cpp:9922-9942`, declaration context from `PrintConfig.hpp:666-668`, float vector storage context from `Config.hpp:812-870`, and machine-limit stride-2 key context from `PrintConfig.cpp:9925-9928`, into `ares-core` as an internal helper for later non-diff base-config update milestones. Preserve expected-size zero clearing, empty-vector zero filling, one-value pair creation, odd-length repair, truncation, pair replication, and odd expected-size integer-division behavior. Defer `log_normalize_legacy_vector_size`, `update_non_diff_values_to_base_config`, `update_diff_values_to_child_config`, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m240-print-config-normalize-stride2-floats.md`.

## M241: DynamicPrintConfig non-diff base-config variant index setup
Port the setup and `variant_index` calculation prefix of `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9844-9894`, plus declaration context from `PrintConfig.hpp:666-668`, downstream `set_with_restore` context from `PrintConfig.cpp:9918-9963`, M240 normalization context from `PrintConfig.cpp:9789-9830`, and `ConfigOptionInts` / `ConfigOptionStrings` storage context from `Config.hpp`, into `ares-core` as an internal helper for later non-diff base-config update milestones. Preserve optional id loading, variant-list loading, target-sized `-1` initialization, missing-current-variant first-target fallback, id/vector length mismatch behavior, nested variant/id matching, unmatched `-1` sentinels, and malformed vector rejection. Defer `log_normalize_legacy_vector_size`, key iteration, `different_keys`, scalar/vector branching, `set_with_restore`, `update_diff_values_to_child_config`, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m241-print-config-non-diff-variant-index.md`.

## M242: DynamicPrintConfig non-diff base-config direct inheritance
Port the key-loop entry and non-`different_keys` direct inheritance branch of `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9896-9904`, plus setup context from `PrintConfig.cpp:9844-9894`, declaration context from `PrintConfig.hpp:666-668`, and deferred `different_keys` handling context from `PrintConfig.cpp:9905-9964`, into `ares-core` as an internal helper for later full non-diff update assembly. Preserve ordered key iteration, source/target presence checks, equality skip, `different_keys` skip, target-value cloning, repeated-key idempotence, unknown-present JSON key copying, and unrelated-key preservation. Defer scalar `different_keys` no-op handling, vector `set_with_restore`, stride-1/stride-2 normalization, `log_normalize_legacy_vector_size`, `update_diff_values_to_child_config`, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m242-print-config-non-diff-direct-inherit.md`.

## M243: DynamicPrintConfig non-diff different-key no-op classification
Port the `different_keys` no-op condition of `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9905-9909`, plus direct-inheritance context from `PrintConfig.cpp:9896-9904`, declaration context from `PrintConfig.hpp:666-668`, deferred restore context from `PrintConfig.cpp:9910-9964`, and scalar/vector option context from `Config.hpp`, into `ares-core` as an internal helper for later full non-diff update assembly. Preserve scalar target no-op behavior, key-set absence no-op behavior, vector restore-needed classification for `key_set1` and `key_set2` members, unknown scalar JSON classification, unknown array JSON classification, and mutation-free operation. Defer vector `set_with_restore`, child-greater-than-parent guard, stride selection, stride-1/stride-2 normalization, `log_normalize_legacy_vector_size`, `update_diff_values_to_child_config`, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m243-print-config-non-diff-different-key-noop.md`.

## M244: DynamicPrintConfig non-diff restore count guard
Port the child-greater-than-parent restore guard inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9910-9916`, plus no-op predicate context from `PrintConfig.cpp:9905-9909`, deferred stride/restore context from `PrintConfig.cpp:9918-9964`, declaration context from `PrintConfig.hpp:666-668`, and variant-count setup context from `PrintConfig.cpp:9844-9864`, into `ares-core` as an internal helper for later full non-diff update assembly. Preserve strict `cur_variant_count > target_variant_count` skip behavior, equal-count no-skip behavior, fewer-current no-skip behavior, and zero-count edge cases. Defer stride selection, expected-size calculation, vector `set_with_restore`, stride-1/stride-2 normalization, `log_normalize_legacy_vector_size`, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m244-print-config-non-diff-restore-count-guard.md`.

## M245: DynamicPrintConfig non-diff restore stride and expected size
Port the restore-branch stride selection and expected-size calculation inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9918-9923`, plus child-greater-than-parent guard context from `PrintConfig.cpp:9910-9916`, deferred stride-2 context from `PrintConfig.cpp:9925-9942`, deferred stride-1 context from `PrintConfig.cpp:9943-9963`, declaration context from `PrintConfig.hpp:666-668`, and variant-index setup context from `PrintConfig.cpp:9844-9894`, into `ares-core` as an internal helper for later full non-diff restore assembly. Preserve stride 1 default behavior, stride 2 for keys in `key_set2`, `expected_size = restore_n * stride`, duplicate-membership behavior, and zero restore-count behavior. Defer stride-2 float type checks, vector normalization, vector resizing, temporary target cloning, vector `set_with_restore`, `log_normalize_legacy_vector_size`, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m245-print-config-non-diff-restore-stride-size.md`.

## M246: DynamicPrintConfig non-diff stride-2 float type check
Port the stride-2 restore branch float-vector type check inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9925-9928`, plus stride and expected-size context from `PrintConfig.cpp:9918-9923`, deferred stride-2 restore context from `PrintConfig.cpp:9930-9942`, declaration context from `PrintConfig.hpp:666-668`, and `ConfigOptionFloats` storage context from `Config.hpp:812-870`, into `ares-core` as an internal helper for later full non-diff restore assembly. Preserve rejection when either source or target is not float-vector shaped and preserve the key-bearing `ConfigOptionFloats for stride=2` error. Defer source/target cloning, size logging, vector normalization calls, vector `set_with_restore`, stride-1 restore behavior, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m246-print-config-non-diff-stride2-float-type-check.md`.

## M247: DynamicPrintConfig non-diff stride-2 size mismatch detection
Port the stride-2 restore branch source/target size capture and mismatch predicate inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9933-9937`, plus float-vector type-check context from `PrintConfig.cpp:9925-9928`, deferred normalization and restore context from `PrintConfig.cpp:9939-9942`, logging helper context from `PrintConfig.cpp:9832-9841`, declaration context from `PrintConfig.hpp:666-668`, and `ConfigOptionFloats` storage context from `Config.hpp:812-870`, into `ares-core` as an internal helper for later full non-diff restore assembly. Preserve source size, target size, and `src_size != expected_size || dest_size != expected_size` mismatch behavior. Defer `log_normalize_legacy_vector_size`, vector normalization calls, vector `set_with_restore`, stride-1 restore behavior, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m247-print-config-non-diff-stride2-size-mismatch.md`.

## M248: DynamicPrintConfig non-diff stride-2 source and target normalization
Port the stride-2 restore branch source float access, target temporary float copy, and paired normalization calls inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9930-9941`, plus float-vector type-check context from `PrintConfig.cpp:9925-9928`, size mismatch context from `PrintConfig.cpp:9933-9937`, M240 normalization context from `PrintConfig.cpp:9789-9830`, deferred restore mutation context from `PrintConfig.cpp:9942`, declaration context from `PrintConfig.hpp:666-668`, and `ConfigOptionFloats` storage context from `Config.hpp:812-870`, into `ares-core` as an internal helper for later full non-diff restore assembly. Preserve source in-place normalization, target temporary-clone normalization, shared expected-size use, and existing M240 stride-2 normalization semantics. Defer vector `set_with_restore`, `log_normalize_legacy_vector_size`, stride-1 restore behavior, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m248-print-config-non-diff-stride2-normalize-pair.md`.

## M249: DynamicPrintConfig non-diff stride-2 set_with_restore mapping
Port the stride-2 restore mutation call inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9942`, plus `ConfigOptionVector<T>::set_with_restore(...)` semantics from `OrcaSlicer/src/libslic3r/Config.hpp:488-504`, M248 normalization context from `PrintConfig.cpp:9930-9941`, float-vector type-check context from `PrintConfig.cpp:9925-9928`, size mismatch context from `PrintConfig.cpp:9933-9937`, declaration context from `PrintConfig.hpp:666-668`, and `ConfigOptionFloats` storage context from `Config.hpp:812-870`, into `ares-core` as an internal helper for later full non-diff restore assembly. Preserve source backup, source replacement with target temporary values, invalid restore-index-size rejection, `-1` skip behavior, and restoration of selected stride-2 source pairs by restore index. Defer `log_normalize_legacy_vector_size`, stride-1 restore behavior, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m249-print-config-non-diff-stride2-set-with-restore.md`.

## M250: DynamicPrintConfig non-diff stride-1 vector size mismatch detection
Port the non-stride-2 restore branch vector access, source/target size capture, and mismatch predicate inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9943-9950`, plus stride-selection context from `PrintConfig.cpp:9918-9923`, stride-2 sibling context from `PrintConfig.cpp:9930-9942`, deferred resize/clone/restore context from `PrintConfig.cpp:9952-9963`, declaration context from `PrintConfig.hpp:666-668`, `ConfigOptionVectorBase` context from `Config.hpp:341-360`, and `ConfigOptionVector<T>::set_with_restore(...)` context from `Config.hpp:488-504`, into `ares-core` as an internal helper for later full non-diff restore assembly. Preserve source size, target size, and `src_size != expected_size || dest_size != expected_size` mismatch behavior for the stride-1/general vector branch. Defer `log_normalize_legacy_vector_size`, source vector resize, target clone/resize normalization, vector `set_with_restore`, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m250-print-config-non-diff-stride1-size-mismatch.md`.

## M251: DynamicPrintConfig non-diff stride-1 source vector resize
Port the non-stride-2 restore branch source resize inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9952-9953`, plus size-mismatch context from `PrintConfig.cpp:9943-9950`, deferred target clone/resize and restore context from `PrintConfig.cpp:9955-9963`, declaration context from `PrintConfig.hpp:666-668`, `ConfigOptionVectorBase::resize(...)` declaration/comment context from `Config.hpp:341-362`, and concrete `ConfigOptionVector<T>::resize(...)` behavior from `Config.hpp:632-664`, into `ares-core` as an internal helper for later full non-diff restore assembly. Preserve no-op matching sizes, zero-size clear, truncation, first-source-value extension for non-empty sources, and first-target/default-value extension for empty sources. Defer target clone/resize normalization, vector `set_with_restore`, `log_normalize_legacy_vector_size`, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m251-print-config-non-diff-stride1-source-resize.md`.

## M252: DynamicPrintConfig non-diff stride-1 target temporary resize
Port the non-stride-2 restore branch target clone, vector check, and target temporary resize inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9955-9961`, plus source resize context from `PrintConfig.cpp:9952-9953`, deferred restore context from `PrintConfig.cpp:9963`, declaration context from `PrintConfig.hpp:666-668`, `ConfigOptionVectorBase::resize(...)` declaration/comment context from `Config.hpp:341-362`, and concrete `ConfigOptionVector<T>::resize(...)` behavior from `Config.hpp:632-664`, into `ares-core` as an internal helper for later full non-diff restore assembly. Preserve cloned-temporary semantics, no-op matching sizes, zero-size clear, truncation, and first-target-value extension for undersized non-empty targets. Defer vector `set_with_restore`, `log_normalize_legacy_vector_size`, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m252-print-config-non-diff-stride1-target-temp-resize.md`.

## M253: DynamicPrintConfig non-diff stride-1 set_with_restore mapping
Port the non-stride-2 restore mutation call inside `DynamicPrintConfig::update_non_diff_values_to_base_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9963`, plus `ConfigOptionVector<T>::set_with_restore(...)` semantics from `OrcaSlicer/src/libslic3r/Config.hpp:488-504`, M250-M252 stride-1/general branch context from `PrintConfig.cpp:9943-9961`, declaration context from `PrintConfig.hpp:666-668`, and `ConfigOptionVectorBase` context from `Config.hpp:341-360`, into `ares-core` as an internal helper for later full non-diff restore assembly. Preserve source backup, source replacement with target temporary values, invalid restore-index-size rejection after replacement, `-1` skip behavior, and restoration of selected stride-1 source elements by restore index. Keep Rust files under 400 LOC by splitting staged restore-vector helpers if needed. Defer `log_normalize_legacy_vector_size`, full non-diff function assembly, `update_diff_values_to_child_config`, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m253-print-config-non-diff-stride1-set-with-restore.md`.


## M254: DynamicPrintConfig diff child-config variant index setup
Port the setup and `variant_index` calculation prefix of `DynamicPrintConfig::update_diff_values_to_child_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9972-10022`, plus declaration context from `PrintConfig.hpp:667-668`, deferred key-loop/mutation context from `PrintConfig.cpp:10024-10103`, and `ConfigOptionInts` / `ConfigOptionStrings` storage context from `Config.hpp`, into `ares-core` as an internal helper for later full diff update assembly. Preserve optional id loading, variant-list loading, current-sized `-1` initialization or missing-current `[0]` initialization, missing-target first-current fallback to zero, id/vector length mismatch behavior, nested current-to-target variant/id matching, and unmatched `-1` sentinels. Defer key iteration, scalar direct set, vector `set_only_diff`, nil inheritance, full diff function assembly, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m254-print-config-diff-child-variant-index.md`.

## M255: DynamicPrintConfig diff child-config direct set branch
Port the key iteration prefix, extruder id/variant key skip, source/target presence and inequality check, and direct `opt_src->set(opt_target)` branch inside `DynamicPrintConfig::update_diff_values_to_child_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10024-10037`, plus M254 variant-index setup context from `PrintConfig.cpp:9972-10022`, deferred vector branch context from `PrintConfig.cpp:10038-10045`, declaration context from `PrintConfig.hpp:668`, and scalar/vector option context from `Config.hpp`, into `ares-core` as an internal helper for later full diff update assembly. Preserve target/child key iteration, extruder metadata key skip, source/target existence checks, equality skip, scalar direct set, and vector-key direct set when absent from both key sets. Defer vector `set_only_diff`, stride selection, nil handling, full `update_diff_values_to_child_config`, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m255-print-config-diff-direct-child-set.md`.

## M256: DynamicPrintConfig diff vector set_only_diff branch
Port the vector branch stride selection and `opt_vec_src->set_only_diff(opt_vec_dest, variant_index, stride)` call inside `DynamicPrintConfig::update_diff_values_to_child_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10038-10045`, plus `ConfigOptionVector<T>::set_only_diff(...)` semantics from `OrcaSlicer/src/libslic3r/Config.hpp:561-580`, M254 variant-index setup context from `PrintConfig.cpp:9972-10022`, M255 direct-set branch context from `PrintConfig.cpp:10024-10037`, and declaration context from `PrintConfig.hpp:668`, into `ares-core` as internal helpers for later full diff update assembly. Preserve stride 1 default, stride 2 for keys in `key_set2`, invalid source-size rejection, `-1` no-op entries, selected target-to-source stride segment copying, and nil target slot skip behavior. Defer full `update_diff_values_to_child_config`, JSON option type dispatch, concrete nullable option classes, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m256-print-config-diff-vector-set-only-diff.md`.

## M257: DynamicPrintConfig diff child-config update assembly
Port the full staged body of `DynamicPrintConfig::update_diff_values_to_child_config(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9972-10048`, plus declaration context from `PrintConfig.hpp:667-668`, M254 variant-index setup from `PrintConfig.cpp:9972-10022`, M255 direct-set branch from `PrintConfig.cpp:10024-10037`, M256 vector `set_only_diff` branch from `PrintConfig.cpp:10038-10045`, and `ConfigOptionVector<T>::set_only_diff(...)` semantics from `Config.hpp:561-580`, into `ares-core` as an internal helper for later public/profile API wiring. Preserve variant-index computation, child-key iteration, extruder metadata skips, source/target existence and equality checks, direct scalar/non-restore-vector copy, restore-vector `set_only_diff` mapping with stride 1 or 2, and staged nil target skip behavior. Defer public API wiring, exhaustive JSON option type dispatch, concrete nullable option classes, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m257-print-config-diff-child-update-assembly.md`.

## M258: compute_filament_override_value long-retraction override defaults
Port the long-retraction special-case override-input substitution prefix of `compute_filament_override_value(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10051-10071`, plus declaration context from `PrintConfig.hpp:690-691`, enum context from `PrintConfig.hpp:183-188`, relevant option definitions from `PrintConfig.cpp:5077-5090`, and deferred clone/apply/change context from `PrintConfig.cpp:10073-10082`, into `ares-core` as an internal helper for later filament override resolution. Preserve replacing `long_retractions_when_cut` and `retraction_distances_when_cut` filament values with same-length nil arrays when `enable_long_retraction_when_cut != EnableFilament`, with the float branch documented as an upstream typo-intent inference, preserving filament values when it equals `EnableFilament`, and passing other keys through unchanged. Defer `ConfigOptionVector::apply_override`, changed-key insertion, `filament_overrides` mutation, full `compute_filament_override_value`, public API wiring, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m258-print-config-filament-override-long-retraction-defaults.md`.

## M259: ConfigOptionVector apply_override mapping
Port `ConfigOptionVector<T>::apply_override(...)` from `OrcaSlicer/src/libslic3r/Config.hpp:713-753`, plus caller context from `PrintConfig.cpp:10073-10076`, M258 long-retraction input-preparation context from `PrintConfig.cpp:10051-10071`, and deferred changed-key/output context from `PrintConfig.cpp:10077-10082`, into `ares-core` as an internal helper for later `compute_filament_override_value` assembly. Preserve non-nullable vector replacement and modified flag behavior; nullable override nil handling with `default_index[i] - 1`, fallback to first original machine value, resize-to-override-length behavior, and zero-overlap no-op behavior. Defer full `compute_filament_override_value`, changed-key insertion, `filament_overrides` mutation, concrete `ConfigOption` dispatch, public API wiring, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m259-config-option-vector-apply-override.md`.

## M260: compute_filament_override_value update assembly
Port the clone/apply/change/output suffix of `compute_filament_override_value(...)` from `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10073-10082`, plus M258 input-preparation context from `PrintConfig.cpp:10051-10071`, declaration context from `PrintConfig.hpp:690-691`, and M259 vector override semantics from `Config.hpp:713-753`, into `ares-core` as an internal JSON-vector helper for later full filament override resolution. Preserve cloning the new machine value, applying the prepared filament override, comparing the computed value against the old machine value, appending the key and storing the computed override value only when changed, and leaving outputs untouched when unchanged. Defer concrete `ConfigOption` hierarchy dispatch, scalar option override dispatch, public API wiring, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m260-print-config-filament-override-update-assembly.md`.

## M261: Print filament override key-loop assembly
Port the per-key filament override loop from `OrcaSlicer/src/libslic3r/Print.cpp:2976-2988`, plus equivalent call-loop context from `PrintApply.cpp:220-244`, M258-M260 `compute_filament_override_value(...)` context from `PrintConfig.cpp:10051-10082` / `PrintConfig.hpp:690-691`, and sorted retract-key context from `PrintConfig.cpp:7164-7195` / `PrintConfig.hpp:569-574`, into `ares-core` as an internal JSON-map helper for later print/config diff wiring. Preserve source-order `extruder_retract_keys` iteration, `filament_` prefix lookup, missing-prefixed-key skip behavior, unprefixed old/new machine value lookup for present filament overrides, and delegation to the staged M260 helper. Defer `Print::update_filament_maps_to_config` state mutation, printer-extruder expansion, config apply/apply_only, placeholder parser updates, full `PrintApply::print_config_diffs`, scalar/non-filament diffing, public API wiring, preset/profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m261-print-filament-override-key-loop.md`.

## M262: PrintApply print-config diff scalar branch
Port the scalar/non-filament diff branch of `print_config_diffs(...)` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:232-260`, plus return context from `PrintApply.cpp:262-264`, M261 filament override call-loop context from `PrintApply.cpp:240-244` / `Print.cpp:2976-2988`, and existing wipe tower option-definition context from `PrintConfig.cpp:6694-6708`, into `ares-core` as a private JSON-map diff helper for later full print config diff wiring. Preserve current-config key iteration, missing new option skip, filament override delegation for retract keys with present `filament_` values, plain changed-key insertion for non-filament changed options, and special `wipe_tower_x` / `wipe_tower_y` plate-index comparison semantics. Defer full `PrintApply::print_config_diffs` public wiring, `full_print_config_diffs`, placeholder parser updates, print config mutation, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m262-print-apply-print-config-diff-scalar-branch.md`.

## M263: PrintApply full print-config diff branch
Port `full_print_config_diffs(...)` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:269-294`, plus purpose comment context from `PrintApply.cpp:267-268`, M262 wipe-tower comparison context from `PrintApply.cpp:245-258`, and wipe tower option-definition context from `PrintConfig.cpp:6694-6708`, into `ares-core` as a private JSON-map helper for later full print config diff wiring. Preserve new-full-config key iteration, missing-old changed-key insertion, equal-value suppression, ordinary changed-key insertion, existing `diff_keys` append-only behavior, and `wipe_tower_x` / `wipe_tower_y` plate-index comparison semantics when old values exist. Defer public `full_print_config_diffs` wiring, public `PrintApply::print_config_diffs` wiring, print config mutation, config apply/apply_only, placeholder parser updates, profile loading, UI runtime, slicing, extrusion, G-code, crate, dependency, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m263-print-apply-full-print-config-diff-branch.md`.

## M264: PrintApply printable-filament change guard
Port the entry guard of `is_printable_filament_changed(...)` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:297-303`, plus deferred geometry context from `PrintApply.cpp:304-340` and `FilamentMapMode` option context from `PrintConfig.cpp:577-582`, `PrintConfig.cpp:2414-2428`, and `PrintConfig.hpp:424-428` / `PrintConfig.hpp:1335`, into `ares-core` as a private helper for later printable-area/extruder-area diff wiring. Preserve returning `false` when old and new polygons are equal, returning `false` when they differ but `filament_map_mode` is manual, and otherwise returning a staged `true` sentinel that represents the deferred geometry-comparison branch. Defer printable-area/extruder-area polygon construction, Clipper `diff`/`intersection`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m264-print-apply-printable-filament-change-guard.md`.

## M265: PrintApply printable-area polygon extraction
Port the printable-area and extruder-area polygon construction prefix of `is_printable_filament_changed(...)` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:304-315`, plus deferred geometry context from `PrintApply.cpp:316-340`, option default context from `PrintConfig.cpp:684-693`, and declaration context from `PrintConfig.hpp:1481-1482`, into `ares-core` as private JSON-to-`Point2` polygon extraction helpers for later printable-area/extruder-area diff wiring. Preserve required `printable_area` finite `[x, y]` point-pair parsing, optional `extruder_printable_area` finite `[x, y]` point-group parsing, printable point order, extruder group order, per-group point order, empty-default `extruder_printable_area` behavior, `SliceError::InvalidInput("printable_area must be an array of [x,y] points")` for malformed printable area, and `SliceError::InvalidInput("extruder_printable_area must be an array of point arrays")` for malformed extruder groups. Defer scaling to `coord_t`, Clipper `diff`/`intersection`, split polygon assembly, intersection-id comparison, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m265-print-apply-printable-area-polygon-extraction.md`.

## M266: PrintApply scaled printable-area polygons
Port the `Point(scale_(pt.x()), scale_(pt.y()))` conversion loops for printable-area and extruder-area polygons from `OrcaSlicer/src/libslic3r/PrintApply.cpp:306-315`, plus deferred geometry context from `PrintApply.cpp:316-340`, scaling context from `libslic3r.h:40-43`, `libslic3r.h:60-64`, `libslic3r.h:92-94`, `libslic3r.cpp:3`, and `Point.hpp:190-205`, into `ares-core` as private staged integer-coordinate polygon helpers for later printable-area/extruder-area diff wiring. Preserve default Orca scaling by `SCALING_FACTOR_INTERNAL = 0.000001`, integer rounding through `Point(double, double)`, printable point order, extruder group order, per-group point order, and negative/fractional coordinate handling by the same scale-and-round rule. Defer large-printer scaling-factor selection, Clipper `diff`/`intersection`, split polygon assembly, intersection-id comparison, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m266-print-apply-scaled-printable-area-polygons.md`.

## M267: PrintApply extruder diff first-result collection
Port the per-extruder `diff(printable_poly, poly)` loop and first-result collection from `OrcaSlicer/src/libslic3r/PrintApply.cpp:317-320`, plus deferred tail context from `PrintApply.cpp:323-340` and difference operation context from `ClipperUtils.hpp:429-433` and `ClipperUtils.cpp:676-679`, into `ares-core` as a private staged helper over scaled polygons and an injected difference callback. Preserve one diff call per extruder polygon in source order, `printable_poly` as subject, current extruder polygon as clip, empty-result skip behavior, `res[0]`-only append behavior, and append order. Defer Clipper `ctDifference`, fill rules, safety offsets, full split polygon assembly, all-extruder intersection, intersection-id comparison, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m267-print-apply-extruder-diff-first-results.md`.

## M268: PrintApply all-extruder intersection append
Port the all-extruder `intersection({printable_poly}, extruder_polys)` first-result append branch from `OrcaSlicer/src/libslic3r/PrintApply.cpp:323-324`, plus deferred tail context from `PrintApply.cpp:326-340` and intersection operation context from `ClipperUtils.hpp:496-508` and `ClipperUtils.cpp:702-703`, into `ares-core` as a private staged helper over scaled polygons and an injected intersection callback. Preserve one intersection call with a single printable subject and all extruder clips, empty-result skip behavior, `all_extruder_polys[0]`-only append behavior, and preservation of existing split polygon order before appending. Defer Clipper `ctIntersection`, fill rules, safety offsets, intersection-id comparison, final printable-filament changed result, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m268-print-apply-all-extruder-intersection-append.md`.

## M269: PrintApply find_intersections control flow
Port the local `find_intersections` lambda from `OrcaSlicer/src/libslic3r/PrintApply.cpp:326-333`, plus deferred final comparison context from `PrintApply.cpp:335-340` and polygon/polygon intersection backend context from `ClipperUtils.hpp:496-508` and `ClipperUtils.cpp:696-697`, into `ares-core` as a private staged helper over scaled polygons and an injected intersection callback. Preserve contour index-order traversal, one `intersection(poly, contours[i])` call per contour, empty-result skip behavior, non-empty index insertion, and sorted-set return semantics. Defer Clipper `ctIntersection`, fill rules, safety offsets, old/new id comparison, final printable-filament changed result, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m269-print-apply-find-intersections.md`.

## M270: PrintApply intersection-id set comparison
Port the old/new `find_intersections(...)` call pair and final set comparison from `OrcaSlicer/src/libslic3r/PrintApply.cpp:335-337`, plus required predecessor context from `PrintApply.cpp:326-333` and deferred surrounding context from `PrintApply.cpp:297-324` / `PrintApply.cpp:339-340`, into `ares-core` as a private staged helper over scaled old/new polygons, existing split polygons, and an injected intersection callback. Preserve computing old ids first, computing new ids second with the same split polygons, and returning whether the sorted id sets differ. Defer concrete Clipper `ctIntersection`, fill rules, safety offsets, printable-area parsing/scaling, split polygon assembly, full `is_printable_filament_changed(...)`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m270-print-apply-intersection-id-set-comparison.md`.

## M271: PrintApply printable-filament staged assembly
Port the full private `is_printable_filament_changed(...)` control-flow assembly from `OrcaSlicer/src/libslic3r/PrintApply.cpp:297-340`, using the already staged M264-M270 slices and boolean operation context from `ClipperUtils.hpp:429-433`, `ClipperUtils.cpp:676-679`, `ClipperUtils.hpp:496-508`, and `ClipperUtils.cpp:696-703`, into `ares-core` as a private helper over JSON config maps, old/new polygons, and injected diff/intersection callbacks. Preserve equal-polygon and manual-mode false exits, printable/extruder-area extraction and scaling from the new config, split polygon assembly from per-extruder diff first results plus all-extruder intersection first result, M269 `find_intersections` id-set construction, M270 `old_poly_ids != new_poly_ids` comparison, and final boolean return. Defer concrete Clipper `ctDifference`/`ctIntersection`, fill rules, safety offsets, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m271-print-apply-printable-filament-staged-assembly.md`.

## M272: PrintApply LayerRanges assign normalization
Port `LayerRanges` storage context and `LayerRanges::assign(...)` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:342-383`, with Orca tolerance context from `OrcaSlicer/src/libslic3r/libslic3r.h:52` and deferred lookup context from `PrintApply.cpp:385-395`, into `ares-core` as private interval normalization helpers over lightweight config identifiers. Preserve sorted-input traversal from `last_z = 0`, skipped covered ranges, negative-start clamping to zero, unconfigured gap insertion, configured interval insertion, Orca `EPSILON = 1e-4` comparisons, empty-input fallback to `[0, DBL_MAX]`, trailing unconfigured range extension to `DBL_MAX`, and trailing unconfigured range append after configured tails. Defer `LayerRanges::config(...)`, `DynamicPrintConfig`, `ModelConfig`, `ModelObjectStatus`, model-object apply logic, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m272-print-apply-layer-ranges-assign.md`.

## M273: PrintApply instance printable-filament invalidation
Port the changed-instance synchronization loop body from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1505-1511`, plus branch/bounding-box context from `PrintApply.cpp:1487-1504`, apply-status context from `PrintApply.cpp:1231-1234`, staged printable-filament predicate context from `PrintApply.cpp:297-340`, print-step names from `Print.hpp:78-88`, and invalidation API context from `PrintBase.hpp:606-612`, into `ares-core` as a private staged helper over minimal instance state and injected M271 geometry callbacks. Preserve evaluating printable-filament change before copying instance fields, returning staged `{psWipeTower, psGCodeExport}` invalidation steps only when the predicate is true, copying transformation/print-volume/printable fields after successful predicate evaluation, and propagating predicate errors without mutation. Defer full `Print::apply`, real print-state invalidation, bounding-box invalidation, concrete Clipper operations, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m273-print-apply-instance-printable-filament-invalidation.md`.

## M274: PrintApply LayerRanges config lookup
Port `LayerRanges::config(...)` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:385-395`, plus M272 storage/normalization context from `PrintApply.cpp:342-383` and Orca `EPSILON` context from `libslic3r.h:52`, into `ares-core` as a private lookup helper over normalized layer ranges. Preserve adjusted lower-bound lookup using `{range.first - EPSILON, range.second - EPSILON}`, missing-range `nullptr` behavior, boundary mismatch tolerance checks, and returning the matched config identifier including unconfigured matched ranges. Defer `DynamicPrintConfig`, `ModelConfig`, model-object apply wiring, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m274-print-apply-layer-ranges-config-lookup.md`.

## M275: PrintApply ModelObjectStatus state
Port `ModelObjectStatus` state declaration from `OrcaSlicer/src/libslic3r/PrintApply.cpp:407-440`, plus `ModelObjectStatusDB` operation context from `PrintApply.cpp:442-470`, into `ares-core` as private staged model-object status vocabulary and an id-keyed record. Preserve `Status` variants `Unknown`, `Old`, `New`, `Moved`, `Deleted`; `PrintObjectRegionsStatus` variants `Invalid`, `Valid`, `PartiallyValid`; constructor defaults; and id-based ordering semantics. Defer `ModelObjectStatusDB`, `PrintObjectStatus`, ref-counted regions, print object transformations, model-object apply wiring, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m275-print-apply-model-object-status-state.md`.

## M276: PrintApply ModelObjectStatusDB operations
Port `ModelObjectStatusDB` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:442-470`, plus M275 `ModelObjectStatus` state context from `PrintApply.cpp:407-440`, into `ares-core` as private staged id-keyed status database operations. Preserve duplicate rejection in `add`, insert-or-skip return semantics in `add_if_new`, missing-id failure in `get`, deleted-record rejection in `reuse`, and deterministic id-keyed uniqueness/order. Defer `PrintObjectStatus`, `PrintObjectStatusDB`, ref-counted regions, print object transformations, model-object apply wiring, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m276-print-apply-model-object-status-db.md`.


## M277: PrintApply PrintObjectStatus state
Port `PrintObjectStatus` state declaration from `OrcaSlicer/src/libslic3r/PrintApply.cpp:473-498`, plus `PrintObjectStatusDB` context from `PrintApply.cpp:500-540`, into `ares-core` as private staged print-object status vocabulary and an id-keyed record. Preserve `Status` variants `Unknown`, `Deleted`, `Reused`, `New`; constructor defaults for id-only records; and id-based ordering semantics. Defer `PrintObjectStatusDB`, real `PrintObject` pointers, concrete `Transform3d`, model-object apply wiring, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m277-print-apply-print-object-status-state.md`.


## M278: PrintApply PrintObjectStatusDB operations
Port `PrintObjectStatusDB` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:500-540`, plus M277 `PrintObjectStatus` state context from `PrintApply.cpp:473-498`, into `ares-core` as private staged multiset-like print-object status database operations. Preserve constructor insertion from print-object ids, duplicate-id multiset behavior, id-keyed `get_range`, `count`, sorted iteration, and `clear`. Defer real `PrintObject` pointers, concrete `Transform3d`, model-object apply wiring, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m278-print-apply-print-object-status-db.md`.

## M279: PrintApply model-volume solid-or-modifier predicate
Port `model_volume_solid_or_modifier(...)` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:542-546`, plus `ModelVolumeType` enum context from `OrcaSlicer/src/libslic3r/Model.hpp:340-348` and downstream filtering context from `PrintApply.cpp:667-695`, into `ares-core` as private staged model-volume vocabulary and a boolean predicate. Preserve upstream discriminant order, return `true` only for `ModelPart`, `NegativeVolume`, and `ParameterModifier`, and return `false` for `Invalid`, `SupportBlocker`, and `SupportEnforcer`. Defer real `ModelVolume`, mesh data, transformations, bounding boxes, print-object-region invalidation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m279-print-apply-model-volume-solid-or-modifier.md`.

## M280: PrintApply bbox transform composition
Port `trafo_for_bbox(...)` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:548-554`, plus transform alias context from `OrcaSlicer/src/libslic3r/Point.hpp:79-85` and downstream bbox context from `PrintApply.cpp:582-606`, into `ares-core` as a private staged 3D affine transform helper. Preserve `object_trafo * volume_trafo` composition order, zeroing composed translation X/Y after multiplication, preserving translation Z and linear terms, and returning f32 matrix values equivalent to Orca's `m.cast<float>()`. Defer full Eigen-compatible transforms, inverse/rotation comparison, mesh vertex transformation, bounding boxes, print-object-region invalidation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m280-print-apply-trafo-for-bbox.md`.

## M281: PrintApply transform Z-rotation/mirroring predicate
Port `trafos_differ_in_rotation_by_z_and_mirroring_by_xy_only(...)` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:556-580`, plus `Transform3f` / `Transform3d` alias context from `OrcaSlicer/src/libslic3r/Point.hpp:79-85`, `coordf_t` / `EPSILON` context from `OrcaSlicer/src/libslic3r/libslic3r.h:46-52`, and downstream invalidation context from `PrintApply.cpp:667-695`, into `ares-core` as a private staged transform predicate. Preserve Z-translation mismatch rejection, `m2.inverse() * m1` relative linear transform construction, relative Z-column validation, X/Y Z-component rejection, X/Y over-unit-length checks, and the final perpendicularity expression. Defer full Eigen-compatible transforms, robust general-purpose matrix algebra, mesh vertex transformation, bounding boxes, print-object-region invalidation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m281-print-apply-transform-z-rotation-mirroring-predicate.md`.

## M282: PrintApply transformed indexed-triangle bbox2d
Port `transformed_its_bbox2d(...)` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:582-593`, plus indexed-triangle context from `OrcaSlicer/deps_src/admesh/stl.h:42-44` and `stl.h:219-235`, `PrintObjectRegions::BoundingBox` context from `OrcaSlicer/src/libslic3r/Print.hpp:216-223`, `Transform3f` context from `OrcaSlicer/src/libslic3r/Point.hpp:84`, and `coordf_t` / `EPSILON` context from `OrcaSlicer/src/libslic3r/libslic3r.h:46-52`, into `ares-core` as private staged indexed-triangle and bounding-box helpers. Preserve empty-index assertion behavior, bbox initialization from the first transformed vertex, extension over every transformed triangle vertex, and final min/max expansion by `[offset, offset, EPSILON]`. Defer full mesh import/storage, face properties, z-range clipping, print-object-region invalidation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m282-print-apply-transformed-its-bbox2d.md`.

## M283: PrintApply transformed indexed-triangle bboxes in Z ranges
Port `transformed_its_bboxes_in_z_ranges(...)` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:595-661`, plus predecessor bbox context from `PrintApply.cpp:582-593`, indexed-triangle context from `OrcaSlicer/deps_src/admesh/stl.h:42-44` and `stl.h:219-235`, `t_layer_height_range` context from `OrcaSlicer/src/libslic3r/Slicing.hpp:150`, `PrintObjectRegions::BoundingBox` context from `OrcaSlicer/src/libslic3r/Print.hpp:216-223`, `Transform3f` / vector interpolation context from `OrcaSlicer/src/libslic3r/Point.hpp:84` and `Point.hpp:136-144`, and `coordf_t` / `EPSILON` context from `OrcaSlicer/src/libslic3r/libslic3r.h:46-52`, into `ares-core` as private staged per-Z-range transformed bbox helpers. Preserve output assignment per Z range, transformed triangle point reuse, edge-order clipping, lower/upper/two-bound intersection handling, bbox init/extend flags, and final min/max expansion by `[offset, offset, EPSILON]`. Defer full mesh import/storage, face properties, print-object-region invalidation, volume filtering, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m283-print-apply-transformed-its-bboxes-in-z-ranges.md`.

## M284: PrintApply keep reusable cached volume ids
Port `print_objects_regions_invalidate_keep_some_volumes(...)` cache-retention behavior from `OrcaSlicer/src/libslic3r/PrintApply.cpp:664-695`, plus model-volume sorting context from `OrcaSlicer/src/libslic3r/Model.hpp:1227-1230`, `ObjectID` ordering context from `OrcaSlicer/src/libslic3r/ObjectID.hpp:20-37`, `PrintObjectRegions::all_regions` / `cached_volume_ids` context from `OrcaSlicer/src/libslic3r/Print.hpp:291-296`, `ModelVolumeType` context from `OrcaSlicer/src/libslic3r/Model.hpp:340-348`, and M279 predicate context from `PrintApply.cpp:542-546`, into `ares-core` as private staged cache-retention helpers. Preserve clearing all regions, sorting old/new volumes by id, filtering new volumes to solid/modifier types, monotonic old-volume matching, transform-compatible reuse, cached-id forward scan with assertion, compaction of kept cached ids, and truncation of stale tail ids. Defer real `PrintObjectRegions`, `PrintRegion`, `ModelVolume` pointers, Eigen `isApprox`, bbox recomputation, layer-range region rebuilding, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m284-print-apply-keep-some-volume-cache.md`.

## M285: PrintApply find volume extents lookup
Port `find_volume_extents(...)` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:697-703`, plus `VolumeExtents` context from `OrcaSlicer/src/libslic3r/Print.hpp:224-228`, sorted `LayerRangeRegions::volumes` context from `OrcaSlicer/src/libslic3r/Print.hpp:271-278`, `ObjectID` ordering context from `OrcaSlicer/src/libslic3r/ObjectID.hpp:20-37`, lower-bound helper context from `OrcaSlicer/src/libslic3r/libslic3r.h:230-247`, and bounding-box context from `OrcaSlicer/src/libslic3r/Print.hpp:216-223`, into `ares-core` as a private staged lower-bound lookup over sorted volume extents. Preserve first-not-less-than lookup by volume id, exact-match-only bbox return, no-match `None` behavior, and first-equal duplicate behavior. Defer real `PrintObjectRegions::LayerRangeRegions`, `ModelVolume`, `ObjectID` wrapper types, bbox recalculation, modifier extents, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m285-print-apply-find-volume-extents.md`.

## M286: PrintApply find modifier volume extents
Port `find_modifier_volume_extents(...)` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:705-725`, plus predecessor `find_volume_extents(...)` context from `PrintApply.cpp:697-703`, `PrintObjectRegions::VolumeRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:229-240`, `LayerRangeRegions::volumes` / `volume_regions` context from `Print.hpp:271-282`, `ModelVolume::is_model_part()` context from `OrcaSlicer/src/libslic3r/Model.hpp:901-907`, and bounding-box context from `Print.hpp:216-223`, into `ares-core` as a private staged modifier bbox helper. Preserve current-region lookup, current volume extents assertion, output initialization from current bbox, model-part early return, modifier parent traversal, parent extent assertion, bbox extension through parent chain, and stop at the first model-part parent. Defer real `PrintObjectRegions::LayerRangeRegions`, real `VolumeRegion` pointers, `ModelVolume` pointers, `PrintRegion`, painted/fuzzy regions, region config merging, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m286-print-apply-find-modifier-volume-extents.md`.

## M287: PrintApply print region ref count helpers
Port `print_region_ref_inc(...)`, `print_region_ref_reset(...)`, and `print_region_ref_cnt(...)` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:729-731`, plus `PrintRegion::m_ref_cnt` and friend-helper context from `OrcaSlicer/src/libslic3r/Print.hpp:104-149`, and downstream reset context from `PrintApply.cpp:746-747`, into `ares-core` as private staged print-region ref-count helpers. Preserve default zero count, increment by one, reset to zero, read without mutation, and signed `int`-like staged count behavior. Defer real `PrintRegion`, `PrintRegionConfig`, config hash/equality, `PrintObjectRegions::all_regions`, region validation, merging/splitting, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m287-print-apply-print-region-ref-count.md`.

## M288: PrintApply verify-update region initialization
Port the initialization prefix of `verify_update_print_object_regions(...)` from `OrcaSlicer/src/libslic3r/PrintApply.cpp:743-747`, plus `model_volumes_sort_by_id(...)` context from `OrcaSlicer/src/libslic3r/Model.hpp:1227-1230`, `PrintObjectRegions::all_regions` context from `OrcaSlicer/src/libslic3r/Print.hpp:291-296`, and M287 `print_region_ref_reset(...)` context from `PrintApply.cpp:729-731`, into `ares-core` as a private staged verify-update initialization helper. Preserve sorting model volumes by ascending id before region processing, resetting every existing print-region ref count after sorting, duplicate-id behavior by id grouping only, and empty input behavior. Defer the layer-range loop, model-part/modifier filtering, model-volume lower-bound lookup, modifier override detection, config diff/apply, callback invalidation, painted/fuzzy painted regions, reslice return decisions, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m288-print-apply-verify-update-region-init.md`.

## M289: PrintApply verify-update volume-region matching
Port the `volume_regions` eligibility, sorted model-volume lookup, and first modifier visit detection prefix from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:755-766`, plus sorted model-volume context from `OrcaSlicer/src/libslic3r/Model.hpp:1227-1230`, `ModelVolume::is_model_part()` / `is_modifier()` context from `Model.hpp:905-907`, `VolumeRegion::model_volume` context from `OrcaSlicer/src/libslic3r/Print.hpp:229-240`, and `LayerRangeRegions::volume_regions` source-order context from `Print.hpp:271-282`, into `ares-core` as a private staged matching helper. Preserve skipping non-model-part/non-modifier regions, exact lower-bound lookup against sorted model volumes with assertion on missing ids, source-order output, and first-visit detection for modifier model volumes using last-visited modifier id only. Defer modifier parent-region creation checks, `next_region_id`, bbox lookup/intersection, config derivation, config diff/apply, callback invalidation, ref-count increment, painted/fuzzy painted regions, reslice return decisions, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m289-print-apply-verify-update-volume-region-match.md`.

## M290: PrintApply verify-update parent scan existing overrides
Port the first-modifier parent scan and existing override detection prefix after the current-modifier bbox lookup from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:766 and PrintApply.cpp:769-782`, plus `VolumeRegion::model_volume` / `parent` context from `OrcaSlicer/src/libslic3r/Print.hpp:229-240`, `LayerRangeRegions::volume_regions` source-order context from `Print.hpp:271-282`, and model-part/modifier predicate context from `OrcaSlicer/src/libslic3r/Model.hpp:905-907`, into `ares-core` as a private staged parent-scan helper. Preserve `next_region_id` initialization, descending parent scan, same-model-volume assertion, filtering to model-part/modifier parents, generated-region ordering assertion, and advancing `next_region_id` when an override for the current modifier and scanned parent already exists. Defer `find_volume_extents` / current-modifier bbox assertion from `PrintApply.cpp:767-768`, `find_modifier_volume_extents`, bbox intersection, config derivation/comparison/application, callback invalidation, ref-count increment, painted/fuzzy painted regions, reslice decisions, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m290-print-apply-verify-update-parent-scan-existing-overrides.md`.

## M291: PrintApply verify-update current modifier bbox lookup
Port the current first-visited modifier bbox lookup/assert from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:767-768`, plus M285 `find_volume_extents(...)` context from `PrintApply.cpp:697-703`, `VolumeExtents` context from `OrcaSlicer/src/libslic3r/Print.hpp:224-228`, and `LayerRangeRegions::volumes` context from `Print.hpp:271-278`, into `ares-core` as a private staged current-modifier bbox helper. Preserve exact lower-bound volume-id lookup, copied bbox return for matching ids, and assert/panic behavior for missing current modifier extents. Defer parent bbox lookup/intersection, `find_modifier_volume_extents` verify-update integration, config derivation/comparison/application, callback invalidation, ref-count increment, painted/fuzzy painted regions, reslice decisions, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m291-print-apply-verify-update-current-modifier-bbox.md`.

## M292: PrintApply verify-update parent bbox intersection gate
Port the missing-parent-override bbox intersection gate from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:783-789`, plus current modifier bbox context from `PrintApply.cpp:767-768`, `find_modifier_volume_extents(...)` context from `PrintApply.cpp:705-725`, `PrintObjectRegions::BoundingBox` / `VolumeExtents` / `VolumeRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:216-240`, and `LayerRangeRegions::volumes` / `volume_regions` context from `Print.hpp:271-282`, into `ares-core` as a private staged parent-bbox intersection gate. Preserve computing the candidate parent bbox via `find_modifier_volume_extents(...)`, closed-box intersection against the current modifier bbox, and staged output for the later config-comparison milestone. Defer `region_config_from_model_volume`, config comparison, `return false`, callback invalidation, ref-count increment, painted/fuzzy painted regions, reslice decisions, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m292-print-apply-verify-update-parent-bbox-intersection-gate.md`.

## M293: PrintApply verify-update missing override config gate
Port the missing-modifier-override config comparison/reslice gate from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:786-789`, plus `region_config_from_model_volume(...)` declaration context from `PrintApply.cpp:727` and implementation context from `OrcaSlicer/src/libslic3r/PrintObject.cpp:3430-3460`, into `ares-core` as a private staged config-difference gate. Preserve that a derived modifier config equal to the parent region config does not require reslice, while a differing derived config requires reslice for a newly needed missing override. Defer full `region_config_from_model_volume(...)` merge internals, `apply_to_print_region_config`, extruder clamping, sparse infill/fuzzy-skin normalization, callback invalidation, ref-count increment, painted/fuzzy painted regions, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m293-print-apply-verify-update-missing-override-config-gate.md`.

## M294: PrintApply verify-update existing region config change gate
Port the existing volume-region config change predicate from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:796`, plus derived config context from `PrintApply.cpp:793-795`, changed-region branch context from `PrintApply.cpp:798-806`, and `region_config_from_model_volume(...)` implementation context from `OrcaSlicer/src/libslic3r/PrintObject.cpp:3430-3460`, into `ares-core` as a private staged config-change gate. Preserve that equal derived/current configs do not enter the update/split branch, while differing configs do. Defer derived config source selection, real config merge internals, ref-count update/split behavior, callback invalidation, config diff/apply, painted/fuzzy painted regions, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m294-print-apply-verify-update-existing-region-config-change-gate.md`.

## M295: PrintApply verify-update existing region ref-count split gate
Port the changed existing-region ref-count decision from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:798-806`, plus M294 config-change context from `PrintApply.cpp:796`, M287 `print_region_ref_cnt(...)` context from `PrintApply.cpp:729-731`, and `PrintRegion::m_ref_cnt` / config mutation context from `OrcaSlicer/src/libslic3r/Print.hpp:104-149`, into `ares-core` as a private staged update action gate. Preserve that unchanged configs do not enter the branch, changed configs with zero ref count are eligible for in-place update, and changed configs with nonzero ref count require reslice. Defer config diff key collection, callback invalidation, config apply, ref-count increment, derived config source selection, real config merge internals, painted/fuzzy painted regions, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m295-print-apply-verify-update-existing-region-ref-count-split-gate.md`.

## M296: PrintApply verify-update existing region config diff keys
Port `t_config_option_keys diff = region.region->config().diff(cfg);` from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:801`, plus `ConfigBase::diff(...)` behavior from `OrcaSlicer/src/libslic3r/Config.cpp:518-528`, key/vector alias context from `OrcaSlicer/src/libslic3r/Config.hpp:73-75`, and update-in-place branch context from `PrintApply.cpp:798-803`, into `ares-core` as private staged config diff-key collection. Preserve current-config key order, intersection-only comparison, changed-value-only output, and no diff keys outside the M295 update-in-place action. Defer callback invalidation, config apply, real `ConfigBase` / `PrintRegionConfig`, option value typing, hashing, ref-count increment, derived config source selection, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m296-print-apply-verify-update-existing-region-config-diff-keys.md`.

## M297: PrintApply verify-update existing region invalidate callback
Port `callback_invalidate(region.region->config(), cfg, diff);` from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:802`, plus callback purpose/signature context from `PrintApply.cpp:734-741`, diff-key context from `PrintApply.cpp:801`, and config-apply-after-callback context from `PrintApply.cpp:803`, into `ares-core` as private staged invalidation callback event data. Preserve callback argument order, diff-key order, update-in-place-only emission, unchanged/reslice suppression, and empty-diff update-in-place callback emission. Defer real callback invocation, background-process cancellation, config apply, real `ConfigBase` / `PrintRegionConfig`, hashing, ref-count increment, derived config source selection, real `PrintRegion`, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m297-print-apply-verify-update-existing-region-invalidate-callback.md`.

## M298: PrintApply verify-update existing region config apply-only
Port `region.region->config_apply_only(cfg, diff, false);` from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:803`, plus `PrintRegion::config_apply_only(...)` context from `OrcaSlicer/src/libslic3r/Print.hpp:136-139`, `ConfigBase::apply_only(...)` behavior from `OrcaSlicer/src/libslic3r/Config.cpp:461-500`, M296 diff-key context from `PrintApply.cpp:801`, and M297 callback-before-apply context from `PrintApply.cpp:802`, into `ares-core` as private staged config apply-only state. Preserve invalidate-before-apply gating, diff-key order, copying derived values for matching keys, missing-derived-key no-op behavior, `ignore_nonexistent = false`, and staged hash refresh metadata. Defer vector `#` option handling, unknown-option exceptions, real `ConfigBase` / `PrintRegionConfig`, real config hash calculation, real `PrintRegion` mutation, ref-count increment, derived config source selection, real `PrintObjectRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m298-print-apply-verify-update-existing-region-config-apply-only.md`.

## M299: PrintApply verify-update existing region ref increment
Port `print_region_ref_inc(*region.region);` from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:809`, plus `print_region_ref_inc(...)` helper context from `PrintApply.cpp:729`, `PrintRegion::m_ref_cnt` / friend-helper context from `OrcaSlicer/src/libslic3r/Print.hpp:104-149`, changed-config branch context from `PrintApply.cpp:796-806`, M295 update-action context, and M298 apply-before-increment context from `PrintApply.cpp:803`, into `ares-core` as private staged ref-increment sequencing. Preserve unchanged-region increment, update-in-place increment only after staged config apply, requires-reslice no-increment behavior because upstream returns before line 809, and accumulated ref-count mutation through the M287 helper. Defer real `PrintRegion`, real `PrintObjectRegions`, loop integration, missing-override region creation, painted/fuzzy painted regions, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m299-print-apply-verify-update-existing-region-ref-increment.md`.

## M300: PrintApply painted region extruder config
Port the color-painted region config derivation prefix from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:813-820`, plus `PrintObjectRegions::PaintedRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:243-252`, `LayerRangeRegions::painted_regions` context from `Print.hpp:271-283`, parent `VolumeRegion` context from `Print.hpp:229-240`, and filament option fields from `OrcaSlicer/src/libslic3r/PrintConfig.hpp:1121,1154,1161`, into `ares-core` as private staged painted-region config derivation. Preserve copying parent config before overrides and assigning the painted extruder id to `wall_filament`, `solid_infill_filament`, and `sparse_infill_filament`, while preserving unrelated parent config fields. Defer comparison/update/reslice handling from `PrintApply.cpp:821-831`, ref-count increment from `PrintApply.cpp:834`, fuzzy painted regions from `PrintApply.cpp:837-856`, real `PrintRegionConfig`, real `PrintObjectRegions`, loop integration, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m300-print-apply-painted-region-filament-config.md`.

## M301: PrintApply painted region update gate
Port the color-painted region config comparison and ref-count update/reslice decision from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:821-831`, plus M300 derived painted config context from `PrintApply.cpp:813-820`, existing-region update gate context from `PrintApply.cpp:796-806`, `print_region_ref_cnt(...)` context from `PrintApply.cpp:729-731`, `PrintRegion::m_ref_cnt` / config mutation context from `OrcaSlicer/src/libslic3r/Print.hpp:104-149`, and `PrintObjectRegions::PaintedRegion` context from `Print.hpp:243-252`, into `ares-core` as a private staged painted-region update gate. Preserve unchanged config no-op behavior, changed zero-ref update-in-place eligibility, changed nonzero-ref reslice requirement, and comparison-result payload for later diff/callback/apply wiring. Defer concrete diff-key collection from `PrintApply.cpp:826`, invalidate callback from `PrintApply.cpp:827`, config apply-only from `PrintApply.cpp:828`, painted-region ref increment from `PrintApply.cpp:834`, fuzzy painted regions from `PrintApply.cpp:837-856`, real `PrintRegionConfig`, real `PrintObjectRegions`, loop integration, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m301-print-apply-painted-region-update-gate.md`.

## M302: PrintApply painted region config apply
Port the color-painted region update-in-place diff, invalidate callback, and config-apply sequence from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:826-828`, plus M300 derived painted config context from `PrintApply.cpp:813-820`, M301 update gate context from `PrintApply.cpp:821-831`, existing-region diff/callback/apply context from `PrintApply.cpp:801-803`, `PrintRegion::config_apply_only(...)` context from `OrcaSlicer/src/libslic3r/Print.hpp:136-139`, and `ConfigBase::apply_only(...)` behavior from `OrcaSlicer/src/libslic3r/Config.cpp:461-500`, into `ares-core` as private staged painted-region config apply state. Preserve update-in-place-only diff collection, current-config key order, callback-before-apply sequencing, `ignore_nonexistent = false`, staged hash refresh metadata, and no-op behavior for unchanged/requires-reslice actions. Defer painted-region ref increment from `PrintApply.cpp:834`, fuzzy painted regions from `PrintApply.cpp:837-856`, real `PrintRegionConfig`, real `PrintObjectRegions`, real callback execution, real config hash calculation, vector `#` option handling, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m302-print-apply-painted-region-config-apply.md`.

## M303: PrintApply painted region ref increment
Port `print_region_ref_inc(*region.region);` from the color-painted region loop in `verify_update_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:834`, plus M301 painted update gate context from `PrintApply.cpp:821-831`, M302 painted config-apply context from `PrintApply.cpp:826-828`, `print_region_ref_inc(...)` helper context from `PrintApply.cpp:729`, existing-region increment context from `PrintApply.cpp:809`, and `PrintRegion::m_ref_cnt` / helper access context from `OrcaSlicer/src/libslic3r/Print.hpp:104-149`, into `ares-core` as private staged painted-region ref-increment sequencing. Preserve unchanged-region increment, update-in-place increment only after staged config apply, requires-reslice no-increment behavior because upstream returns before line 834, and accumulated ref-count mutation through the existing staged helper. Defer fuzzy painted regions from `PrintApply.cpp:837-856`, real `PrintRegion`, real `PrintObjectRegions`, loop integration, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m303-print-apply-painted-region-ref-increment.md`.

## M304: PrintApply fuzzy painted region config
Port the fuzzy-skin painted-region config derivation prefix from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:837-842`, plus `PrintObjectRegions::FuzzySkinPaintedRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:255-266`, `LayerRangeRegions` parent collections from `Print.hpp:271-283`, parent lookup implementation from `OrcaSlicer/src/libslic3r/Print.cpp:4932-4947`, and `FuzzySkinType` variants from `OrcaSlicer/src/libslic3r/PrintConfig.hpp:50-57`, into `ares-core` as private staged fuzzy-painted config derivation. Preserve resolving volume-region and painted-region parents, copying parent config before mutation, changing any non-`Disabled_fuzzy` fuzzy-skin value to `All`, preserving disabled fuzzy skin, source-order output, and fuzzy region/parent metadata. Defer fuzzy-painted config comparison/update/apply from `PrintApply.cpp:843-853`, ref-count increment from `PrintApply.cpp:856`, real `PrintRegionConfig`, real `PrintObjectRegions`, loop integration, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m304-print-apply-fuzzy-painted-region-config.md`.

## M305: PrintApply fuzzy painted region update/apply
Port the fuzzy-skin painted-region config comparison and update/apply block from `verify_update_print_object_regions(...)` in `OrcaSlicer/src/libslic3r/PrintApply.cpp:843-853`, plus M304 derivation context from `PrintApply.cpp:837-842`, `PrintObjectRegions::FuzzySkinPaintedRegion` destination-region context from `OrcaSlicer/src/libslic3r/Print.hpp:255-266`, `PrintRegion` config/apply/ref-count context from `OrcaSlicer/src/libslic3r/Print.hpp:104-149`, and the shared update pattern from `PrintApply.cpp:786-800` and `PrintApply.cpp:821-833`, into `ares-core` as private staged fuzzy-painted update/apply state. Preserve config comparison, unchanged no-op behavior, zero-ref update-in-place behavior, referenced-region requires-reslice behavior, diff/invalidate/apply-only sequencing, diff key order inherited from current config values, fuzzy painted region/parent/destination metadata, and focused test-file split below the 400 LOC threshold. Defer fuzzy-painted ref-count increment from `PrintApply.cpp:856`, real `PrintRegionConfig`, real `PrintObjectRegions`, loop integration, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m305-print-apply-fuzzy-painted-region-update-apply.md`.

## M306: PrintApply fuzzy painted region ref increment
Port `print_region_ref_inc(*region.region);` from the fuzzy-skin painted region loop in `verify_update_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:856`, plus M304 fuzzy-painted derivation context from `PrintApply.cpp:837-842`, M305 fuzzy-painted update/apply context from `PrintApply.cpp:843-853`, `print_region_ref_inc(...)` helper context from `PrintApply.cpp:729`, existing-region increment context from `PrintApply.cpp:809`, color-painted increment context from `PrintApply.cpp:834`, and `PrintRegion::m_ref_cnt` / helper access context from `OrcaSlicer/src/libslic3r/Print.hpp:104-149`, into `ares-core` as private staged fuzzy-painted ref-increment sequencing. Preserve unchanged-region increment, update-in-place increment only after staged config apply, requires-reslice no-increment behavior because upstream returns before line 856, accumulated ref-count mutation through the existing staged helper, and a focused fuzzy-painted state module because the prior painted-region state file reached the 400 LOC split threshold. Defer region merge verification after `PrintApply.cpp:860`, real `PrintRegion`, real `PrintObjectRegions`, loop integration, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m306-print-apply-fuzzy-painted-region-ref-increment.md`.

## M307: PrintApply region merge verification
Port the final region-merge verification block from `verify_update_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:860-875`, plus ref-count prerequisite context from `PrintApply.cpp:809`, `PrintApply.cpp:834`, and `PrintApply.cpp:856`, ref helper context from `PrintApply.cpp:729-731`, and `PrintRegion::config()`, `config_hash()`, and `m_ref_cnt` context from `OrcaSlicer/src/libslic3r/Print.hpp:104-149`, into `ares-core` as private staged merge verification. Preserve positive-ref assertion for every all-regions entry, sorting by config hash before comparison, same-hash config equality requiring reslice, hash-collision unequal configs remaining valid, equal configs with different hashes not being compared by this upstream block, and empty/unique region success. Defer real `PrintRegion`, real `PrintObjectRegions`, loop integration, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m307-print-apply-region-merge-verification.md`.

## M308: PrintApply update_volume_bboxes volume order/cache ids
Port the model-volume ordering/filtering and cached-volume-id refresh shell from `update_volume_bboxes(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:884-893` and `PrintApply.cpp:946-950`, plus existing staged `model_volume_solid_or_modifier(...)` eligibility context, into `ares-core` as private staged volume-cache state. Preserve sorting model volumes by `ObjectID`, processing only solid-or-modifier volumes, replacing stale cached ids with the current sorted eligible ids, preserving duplicate ids rather than inventing deduplication, clearing stale ids for empty eligible input, and keeping new tests in a focused file below the 400 LOC threshold. Defer single-layer bbox reuse/computation from `PrintApply.cpp:895-907`, multi-layer bbox behavior from `PrintApply.cpp:908-941`, real meshes/transforms/bounding boxes, real `ModelVolumePtrs`, real `LayerRangeRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m308-print-apply-update-volume-bboxes-volume-order-cache-ids.md`.

## M309: PrintApply update_volume_bboxes single-layer extents
Port the single-layer `update_volume_bboxes(...)` branch from `OrcaSlicer/src/libslic3r/PrintApply.cpp:895-907`, plus M308 ordering/cache-id context from `PrintApply.cpp:884-893` and `PrintApply.cpp:946-950`, existing staged `StagedVolumeExtents` / `staged_find_volume_extents(...)` context, and staged `model_volume_solid_or_modifier(...)` eligibility context, into `ares-core` as private staged volume-cache state. Preserve processing only eligible model volumes, reusing old extents when an id is cached and present in old extents, inserting supplied new extents when an id is not cached, skipping cached ids missing old extents, preserving input model-volume order, and processing duplicate ids independently. Defer actual `transformed_its_bbox2d(...)`, real meshes/transforms/bounding boxes, multi-layer branch behavior from `PrintApply.cpp:908-941`, final cache-id refresh already staged in M308, real `ModelVolumePtrs`, real `LayerRangeRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m309-print-apply-update-volume-bboxes-single-layer.md`.

## M310: PrintApply update_volume_bboxes multi-layer old extents
Port the multi-layer old-volume setup from `update_volume_bboxes(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:908-917`, plus M308 ordering/cache-id context from `PrintApply.cpp:884-893` and `PrintApply.cpp:946-950`, M309 single-layer context from `PrintApply.cpp:895-907`, and existing staged `StagedVolumeExtents` context, into `ares-core` as private staged volume-cache state. Preserve clearing every layer's current volumes when cached ids are empty, capturing every layer's old volumes in layer order when cached ids are non-empty, emptying each layer for later output population, preserving empty layer-range input, and preserving per-layer volume order. Defer layer-height range expansion from `PrintApply.cpp:919-927`, cached multi-layer extent reuse from `PrintApply.cpp:928-936`, uncached bbox generation/insertion from `PrintApply.cpp:937-941`, final cache-id refresh already staged in M308, real meshes/transforms/bounding boxes, real `ModelVolumePtrs`, real `LayerRangeRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m310-print-apply-update-volume-bboxes-multi-layer-old-extents.md`.

## M311: PrintApply update_volume_bboxes multi-layer expanded ranges
Port the multi-layer range setup from `update_volume_bboxes(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:919-927`, plus M310 old-extents context from `PrintApply.cpp:908-917`, M308 ordering/cache-id context from `PrintApply.cpp:884-893` and `PrintApply.cpp:946-950`, into `ares-core` by reusing private staged layer-height range state. Preserve copying each layer height range, expanding lower bounds by subtracting `EPSILON`, expanding upper bounds by adding `EPSILON`, preserving layer order, preserving empty input, and not introducing an Ares-specific tolerance policy. Defer cached multi-layer extent reuse from `PrintApply.cpp:928-936`, uncached bbox generation/insertion from `PrintApply.cpp:937-941`, real bbox vector population, real meshes/transforms/bounding boxes, real `ModelVolumePtrs`, real `LayerRangeRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m311-print-apply-update-volume-bboxes-multi-layer-ranges.md`.

## M312: PrintApply update_volume_bboxes multi-layer cached reuse
Port the cached-volume reuse branch inside the multi-layer `update_volume_bboxes(...)` loop from `OrcaSlicer/src/libslic3r/PrintApply.cpp:928-936`, plus M310 old-extents context from `PrintApply.cpp:908-917`, M311 expanded-range context from `PrintApply.cpp:919-927`, M308 ordering/cache-id context from `PrintApply.cpp:884-893` and `PrintApply.cpp:946-950`, and existing staged `StagedMultiLayerVolumeCacheLayer`, `StagedVolumeExtents`, and `staged_find_volume_extents(...)` context, into `ares-core` as private staged volume-cache state. Preserve processing only eligible model volumes, appending per-layer old extents only for cached ids present in that layer's old extents, skipping missing old extents without fallback computation, doing nothing for uncached ids, preserving model-volume loop order and layer order, and processing duplicate cached model-volume ids independently. Defer uncached bbox generation/insertion from `PrintApply.cpp:937-941`, real `transformed_its_bboxes_in_z_ranges(...)`, real bbox vector population, real meshes/transforms/bounding boxes, final cache-id refresh already staged in M308, real `ModelVolumePtrs`, real `LayerRangeRegions`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m312-print-apply-update-volume-bboxes-multi-layer-cached-reuse.md`.

## M313: PrintApply update_volume_bboxes multi-layer uncached insertion
Port the uncached-volume insertion branch from the multi-layer `update_volume_bboxes(...)` loop at `OrcaSlicer/src/libslic3r/PrintApply.cpp:937-941`, plus M310 old-extents context from `PrintApply.cpp:908-917`, M311 range context from `PrintApply.cpp:919-927`, M312 cached reuse context from `PrintApply.cpp:928-936`, and M308 ordering/cache-id context from `PrintApply.cpp:884-893` and `PrintApply.cpp:946-950`, into `ares-core` as private staged multi-layer uncached bbox insertion state. Preserve filtering to solid-or-modifier model volumes, entering only uncached ids, appending only populated per-layer bboxes to corresponding layer outputs, skipping unpopulated per-layer bboxes, preserving existing output prefixes, preserving model-volume order, and processing duplicate uncached ids independently. Defer full integration with real `ModelVolumePtrs`, real mesh/transform/matrix orchestration, final cache-id refresh already staged in M308, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m313-print-apply-update-volume-bboxes-multi-layer-uncached-insertion.md`.

## M314: PrintApply generate_print_object_regions layer-range shell
Port the object reuse/new allocation and layer-range shell from `generate_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:953-993`, plus `PrintObjectRegions::LayerRangeRegions`, `all_regions`, `layer_ranges`, `trafo_bboxes`, and `cached_volume_ids` context from `OrcaSlicer/src/libslic3r/Print.hpp:271-296`, into `ares-core` as private staged print-object-region shell state in a new focused module. Preserve clearing all regions, reuse detection when an old object has non-empty layer ranges, reused count/range assertions, reused config refresh, clearing reused volume/painted/fuzzy region lists while preserving existing volumes/cached ids/old transform, and fresh transform/layer-range initialization. Defer `is_mm_painted` / `update_volume_bboxes(...)` from `PrintApply.cpp:995-996`, `get_create_region` from `PrintApply.cpp:998-1010`, volume-region construction from `PrintApply.cpp:1012-1054`, painting/fuzzy construction from `PrintApply.cpp:1056-1101`, real `PrintObjectRegions`, real configs and transforms, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m314-print-apply-generate-object-regions-layer-range-shell.md`.

## M315: PrintApply generate_print_object_regions update_volume_bboxes call
Port the MM-painted offset selection and `update_volume_bboxes(...)` call boundary from `generate_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:995-996`, plus M314 shell context from `PrintApply.cpp:953-993`, M308-M313 staged `update_volume_bboxes(...)` context, and `ModelVolume::is_mm_painted()` context from `OrcaSlicer/src/libslic3r/Model.hpp:1014`, into `ares-core` as private staged call-boundary state. Preserve `is_mm_painted = num_extruders > 1 && any(model_volume.is_mm_painted)`, preserve offset `0.0` for MM-painted otherwise `max(0.0, xy_contour_compensation)`, and preserve passing shell `trafo_bboxes`, cached volume ids, and layer ranges to the call record. Defer real `update_volume_bboxes(...)` orchestration, `get_create_region` from `PrintApply.cpp:998-1010`, volume-region construction from `PrintApply.cpp:1012-1054`, painting/fuzzy construction from `PrintApply.cpp:1056-1101`, real `PrintObjectRegions`, real configs/transforms, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m315-print-apply-generate-object-regions-update-volume-bboxes-call.md`.

## M316: PrintApply generate_print_object_regions region-set helper
Port the local `region_set` and `get_create_region(...)` helper from `generate_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:998-1010`, plus M314 shell context from `PrintApply.cpp:953-993` and M315 call-boundary context from `PrintApply.cpp:995-996`, into `ares-core` as private staged region-set state. Preserve lookup ordering by `PrintRegion::config_hash()` and `PrintRegion::config()` ordering, reuse of equal hash/config regions, new-region id assignment from current `all_regions.size()`, append order in `all_regions`, and sorted lookup insertion in `region_set`. Defer volume-region construction from `PrintApply.cpp:1012-1054`, painted/fuzzy construction from `PrintApply.cpp:1056-1101`, real `PrintRegionConfig`, real `PrintRegion`, real config diffing, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m316-print-apply-generate-object-regions-region-set.md`.

## M317: PrintApply generate_print_object_regions model-part volume region
Port the model-part `VolumeRegion` append branch from `generate_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:1012-1024`, plus eligibility context from `PrintApply.cpp:542-546`, `VolumeRegion` field context from `OrcaSlicer/src/libslic3r/Print.hpp:229-240`, M314 layer-range shell context, and M316 staged region-set context, into `ares-core` as private staged model-part volume-region construction. Preserve model-volume iteration order, filtering to solid-or-modifier volume types before branch handling, per-layer `find_volume_extents(...)` gating, parent `-1`, non-null region id creation/reuse through the staged region-set helper, and bbox/extent identity. Defer negative-volume branch from `PrintApply.cpp:1025-1027`, modifier branch from `PrintApply.cpp:1028-1054`, painted/fuzzy construction from `PrintApply.cpp:1056-1101`, real `region_config_from_model_volume(...)`, real `PrintRegionConfig`, real `PrintRegion`, real `ModelVolumePtrs`, real bbox pointers, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m317-print-apply-generate-object-regions-model-part-volume-region.md`.

## M318: PrintApply generate_print_object_regions modifier parent scan
Port the modifier parent-scan prefix from `generate_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:1028-1037`, plus M317 model-part volume-region context from `PrintApply.cpp:1012-1024` and existing staged `find_modifier_volume_extents(...)` behavior, into `ares-core` as private staged modifier parent-scan state. Preserve modifier-only entry, initial `added = false` and `parent_model_part_id = -1`, descending parent scan order, parent eligibility for model-part or modifier regions only, index-preserving parent-region adaptation for staged modifier extents, and intersection gating against the current modifier bbox. Defer config merge and changed-config append from `PrintApply.cpp:1038-1042`, fallback parent-model-part selection and unchanged modifier append from `PrintApply.cpp:1043-1050`, painted/fuzzy construction from `PrintApply.cpp:1056-1101`, real configs/regions, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m318-print-apply-generate-object-regions-modifier-parent-scan.md`.

## M319: PrintApply generate_print_object_regions modifier changed-config append
Port the changed-config append branch from `generate_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:1038-1042`, plus M316 staged region-set context from `PrintApply.cpp:998-1010`, M318 intersecting parent context from `PrintApply.cpp:1028-1037`, and `PrintObjectRegions::VolumeRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:229-240`, into `ares-core` as private staged modifier changed-config append state. Preserve modifier-only entry, explicit derived-config versus parent-config comparison, append only for changed configs, `added = true` only when an append is produced, parent region index propagation, region id creation/reuse through the staged region set, current modifier bbox preservation, and candidate-order appends. Defer fallback parent-model-part selection and unchanged modifier append from `PrintApply.cpp:1043-1050`, painted/fuzzy construction from `PrintApply.cpp:1056-1101`, real `region_config_from_model_volume(...)`, real configs/regions, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m319-print-apply-generate-object-regions-modifier-changed-config-append.md`.


## M320: PrintApply generate_print_object_regions modifier unchanged fallback
Port the unchanged modifier fallback branch from `generate_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:1043-1050`, plus M318 intersecting parent context from `PrintApply.cpp:1028-1037`, M319 changed-config `added` context from `PrintApply.cpp:1038-1042`, and `PrintObjectRegions::VolumeRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:229-240`, into `ares-core` as private staged modifier unchanged-fallback state. Preserve selecting only the first unchanged model-part parent, skipping modifier parents for fallback selection, appending only when no changed-config append was produced and a model-part parent was selected, reusing the selected parent region index, current modifier bbox preservation, and modifier-only entry. Defer painted/fuzzy construction from `PrintApply.cpp:1056-1101`, real `region_config_from_model_volume(...)`, real configs/regions, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m320-print-apply-generate-object-regions-modifier-unchanged-fallback.md`.


## M321: PrintApply generate_print_object_regions painted region append
Port the painted-region append loop from `generate_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:1056-1067`, plus M316 staged region-set context from `PrintApply.cpp:998-1010`, M317-M320 generated volume-region context, and `PrintObjectRegions::PaintedRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:243-251`, into `ares-core` as private staged generated painted-region append state. Preserve nested iteration order over layer ranges, painting extruders, and volume regions; preserve model-part/modifier parent eligibility; preserve parent config copy with wall, solid infill, and sparse infill filament values overwritten to the painted extruder id; preserve appended extruder id, parent volume-region index, and region id from the staged region set. Defer painted-region sorting from `PrintApply.cpp:1068-1072`, fuzzy painted construction from `PrintApply.cpp:1075-1101`, real `PrintRegionConfig`, real configs/regions/pointers, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m321-print-apply-generate-object-regions-painted-region-append.md`.


## M322: PrintApply generate_print_object_regions painted region sort
Port the painted-region sort comparator from `generate_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:1068-1072`, plus M321 painted-region append context from `PrintApply.cpp:1056-1067` and `PrintObjectRegions::PaintedRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:243-251`, into `ares-core` as private staged generated painted-region sort state. Preserve sorting each layer range's painted regions by parent volume region `print_object_region_id()` and then by painted `extruder_id`, while preserving painted-region fields and leaving layer order untouched. Defer fuzzy painted construction from `PrintApply.cpp:1075-1101`, real `PrintRegion` pointers, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m322-print-apply-generate-object-regions-painted-region-sort.md`.


## M323: PrintApply generate_print_object_regions fuzzy volume-region append
Port the fuzzy painted volume-region parent append loop from `generate_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:1075-1086`, plus M316 staged region-set context from `PrintApply.cpp:998-1010`, M317-M320 generated volume-region context, and `PrintObjectRegions::FuzzySkinPaintedRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:255-264`, into `ares-core` as private staged generated fuzzy volume-region append state. Preserve the `has_painted_fuzzy_skin` gate, volume-region parent iteration order, model-part/modifier parent eligibility, parent config copy with fuzzy skin changed to `All` unless disabled, appended parent type `VolumeRegion`, parent volume-region index, and region id from the staged region set. Defer painted-region parent fuzzy append from `PrintApply.cpp:1089-1095`, fuzzy painted sorting from `PrintApply.cpp:1097-1100`, real `PrintRegionConfig`, real configs/regions/pointers, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m323-print-apply-generate-object-regions-fuzzy-volume-region-append.md`.


## M324: PrintApply generate_print_object_regions fuzzy painted-region append
Port the fuzzy painted-region parent append loop from `generate_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:1089-1095`, plus M316 staged region-set context from `PrintApply.cpp:998-1010`, M321-M322 generated painted-region context, M323 generated fuzzy volume-region context, `PrintObjectRegions::PaintedRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:243-251`, `PrintObjectRegions::FuzzySkinPaintedRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:255-266`, and `FuzzySkinType` context from `OrcaSlicer/src/libslic3r/PrintConfig.hpp:50-57`, into `ares-core` as private staged generated fuzzy painted-region append state. Preserve the `has_painted_fuzzy_skin` gate, source-order iteration over existing painted-region parents, parent config copy with fuzzy skin changed to `All` unless disabled, appended parent type `PaintedRegion`, parent painted-region index, and region id from the staged region set. Defer fuzzy painted sorting from `PrintApply.cpp:1097-1100`, real `PrintRegionConfig`, real configs/regions/pointers, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m324-print-apply-generate-object-regions-fuzzy-painted-region-append.md`.


## M325: PrintApply generate_print_object_regions fuzzy painted-region sort
Port the fuzzy painted-region sort comparator from `generate_print_object_regions(...)` at `OrcaSlicer/src/libslic3r/PrintApply.cpp:1097-1100`, plus `PrintObjectRegions::FuzzySkinPaintedRegion` context from `OrcaSlicer/src/libslic3r/Print.hpp:255-266`, parent resolution context from `OrcaSlicer/src/libslic3r/Print.cpp:4932-4947`, and M323-M324 generated fuzzy painted-region context, into `ares-core` as private staged generated fuzzy painted-region sort state. Preserve sorting each layer range's fuzzy painted regions by resolved parent `print_object_region_id()`, resolving volume parents through volume regions and painted parents through painted regions, while preserving fuzzy-region fields and leaving layer order untouched. Defer real `PrintRegion` pointers, real parent lookup methods, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m325-print-apply-generate-object-regions-fuzzy-painted-region-sort.md`.


## M326: PrintApply apply normalization prelude
Port the `Print::apply(...)` normalization prelude from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1115-1127`, plus `normalize_fdm_1` / `normalize_fdm_2` declaration context from `OrcaSlicer/src/libslic3r/PrintConfig.hpp:629-631` and existing Ares normalization context, into `ares-core` as private staged apply-normalization state. Preserve materializing `print_settings_id`, `filament_settings_id`, and `printer_settings_id` before normalization, collecting `used_filaments` in source order from the caller and deriving a membership set, recording `normalize_fdm_1` before `normalize_fdm_2`, passing object count and used-filament vector length to `normalize_fdm_2`, and preserving changed-key output. Defer changed-key logging from `PrintApply.cpp:1127-1133`, support flag handling from `PrintApply.cpp:1134-1138`, scarf-seam handling, extruder variant expansion, real `DynamicPrintConfig`, real `Print`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m326-print-apply-apply-normalization-prelude.md`.

## M328: PrintApply apply support-used flag
Port the `Print::apply(...)` support-used assignment from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1134-1138`, plus the preceding apply prelude and changed-key logging context from `PrintApply.cpp:1115-1133` and existing `enable_support` option metadata context, into `ares-core` as private staged apply support-used state. Preserve querying exactly `enable_support`, assigning true only when the option exists and its bool value is true, assigning false when the option is missing or false, and returning an assignment record for later staged apply behavior. Defer scarf-seam handling from `PrintApply.cpp:1140+`, extruder variant expansion, real `ConfigOption`, real `DynamicPrintConfig`, real `Print::m_support_used` mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m328-print-apply-apply-support-used-flag.md`.

## M329: PrintApply apply scarf joint seam flag
Port the `Print::apply(...)` scarf joint seam detection and guarded config-set block from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1140-1154`, plus `SeamScarfType` enum mapping from `OrcaSlicer/src/libslic3r/PrintConfig.hpp:216-220` and `OrcaSlicer/src/libslic3r/PrintConfig.cpp:360-365`, into `ares-core` as private staged apply scarf joint seam state. Preserve detecting non-`None` `seam_slope_type` from object-level resolved config, volume override configs, and layer-range override configs; OR detection across all model objects; and conditional config-set intent for exact key `has_scarf_joint_seam` with value `true` only when found. Defer logging from `PrintApply.cpp:1155`, real logging backend, real `DynamicConfig`, `ModelObject`, `ModelVolume`, `ConfigOptionEnum<SeamScarfType>`, mutation of `new_full_config`, extruder variant expansion from `PrintApply.cpp:1157+`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m329-print-apply-apply-scarf-joint-seam-flag.md`.

## M335: PrintApply apply filament_map extraction
Port the local `filament_map` option extraction from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1178-1179`, plus downstream `print_config_diffs(...)` consumer context from `PrintApply.cpp:1184` and option-definition context from `PrintConfig.cpp:2401-2405` / `PrintConfig.hpp:1336`, into `ares-core` as private staged apply filament-map extraction state. Preserve source identity `new_full_config`, option key `filament_map`, absent-option empty-vector behavior, present empty-vector behavior, source-order integer copying, and duplicate/negative values without validation or deduplication. Defer the commented else branch from `PrintApply.cpp:1168-1176`, `print_config_diffs(...)`, full/full-object/region diff computation, filament-map mode mutation logic from `PrintApply.cpp:1190+`, real `DynamicPrintConfig`, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m335-print-apply-apply-filament-map-extraction.md`.

## M337: PrintApply apply filament_map_mode guard
Port the filament-map processing set setup and guard from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1190-1192`, plus the `print_diff` result context from `PrintApply.cpp:1184`, into `ares-core` as private staged apply filament-map-mode guard state. Preserve the BBS filament-map processing comment intent, `print_diff_set` construction from `print_diff`, unordered-set duplicate suppression, guard key `filament_map_mode`, and entry only when that key is absent. Defer map-mode lookup and `< fmmManual` branch from `PrintApply.cpp:1194-1204`, manual branch from `PrintApply.cpp:1205-1226`, `print_diff` reassignment from `PrintApply.cpp:1227-1228`, real config mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m337-print-apply-apply-filament-map-mode-guard.md`.

## M338: PrintApply apply filament_map auto-mode gate
Port the `filament_map_mode` lookup and auto-mode branch gate from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1194-1195`, plus `FilamentMapMode` ordering from `PrintConfig.hpp:424-428` and enum name-map context from `PrintConfig.cpp:577-582`, into `ares-core` as private staged apply filament-map auto-mode gate state. Preserve source config `new_full_config`, option key `filament_map_mode`, required lookup flag `true`, local value identity `map_mode`, and branch entry only for modes ordered before `fmmManual` (`fmmAutoForFlush` / `Auto For Flush`, `fmmAutoForMatch` / `Auto For Match`). Defer the auto-mode inner `filament_map` branch from `PrintApply.cpp:1196-1203`, manual branch from `PrintApply.cpp:1205-1226`, `print_diff` reassignment from `PrintApply.cpp:1227-1228`, real config lookup/mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m338-print-apply-apply-filament-map-auto-mode-gate.md`.


## M339: PrintApply apply auto filament_map diff prune
Port the auto-mode inner `filament_map` diff-prune branch from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1196-1203`, plus M337 `print_diff_set` context and M338 auto-mode gate context, into `ares-core` as private staged apply auto filament-map diff-prune state. Preserve branch entry only when `filament_map` is present in `print_diff_set`, active `print_diff_set.erase("filament_map")`, the commented `full_config_diff.erase("filament_map")` as a non-action, required `ConfigOptionInts` lookups for old/new `filament_map`, staged `old_opt->set(new_opt)`, and staged `m_config.filament_map = *new_opt`. Defer manual branch from `PrintApply.cpp:1205-1226`, `print_diff` reassignment from `PrintApply.cpp:1227-1228`, real config lookup/mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m339-print-apply-apply-auto-filament-map-diff-prune.md`.

## M340: PrintApply apply manual filament_map setup
Port the manual-mode branch setup from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1205-1208`, plus M337 `print_diff_set` context and M338 manual branch context, into `ares-core` as private staged apply manual filament-map setup state. Preserve branch entry as an explicit gate, active `print_diff_set.erase("extruder_ams_count")`, duplicate-suppressed diff-set membership after erasure, old map copy from `m_config.filament_map.values` to `old_filament_map`, and required `ConfigOptionInts` lookup of `new_full_config` key `filament_map` values to `new_filament_map`. Defer same-size comparison and same-map loop from `PrintApply.cpp:1210-1224`, `print_diff` reassignment from `PrintApply.cpp:1227-1228`, real config lookup/mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m340-print-apply-apply-manual-filament-map-setup.md`.

## M341: PrintApply apply manual filament_map same-map prune
Port the manual-mode same-size comparison, `same_map` loop, and conditional `filament_map` erase from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1210-1224`, plus M340 old/new map setup context and `used_filament_set` context from `PrintApply.cpp:1121-1122`, into `ares-core` as private staged apply manual filament-map same-map prune state. Preserve the equal-length guard, ordered index visits, continue-on-equal behavior, continue-on-unused-index behavior, break on the first used differing index, and erasing `filament_map` from `print_diff_set` only when `same_map` remains true. Defer `print_diff` reassignment from `PrintApply.cpp:1227-1228`, real config mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m341-print-apply-apply-manual-filament-map-same-map-prune.md`.

## M342: PrintApply apply print_diff set reassignment
Port the size-gated `print_diff` reassignment from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1227-1228`, plus M337-M341 staged `print_diff_set` context, into `ares-core` as private staged apply print-diff set reassignment state. Preserve the exact `print_diff_set.size() != print_diff.size()` gate, no reassignment when sizes are equal, reassignment from duplicate-suppressed staged set contents when sizes differ, and unspecified set iteration order semantics. Defer apply-status handling from `PrintApply.cpp:1231-1239`, lock acquisition from `PrintApply.cpp:1241-1242`, real config mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m342-print-apply-apply-print-diff-set-reassign.md`.

## M343: PrintApply apply status initial diff update
Port the initial `apply_status` setup, max-based `update_apply_status` helper semantics, and initial non-empty-diff changed update from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1231-1239`, plus M342 finalized `print_diff` context, into `ares-core` as private staged apply-status state. Preserve unchanged initialization, numeric max ordering, changed vs invalidated update mapping, any-diff gate over print/object/region diff lengths, and staged log metadata containing those diff sizes only when the gate fires. Defer lock acquisition from `PrintApply.cpp:1241-1242`, later print/object/region invalidation and status updates, real logging, real config mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m343-print-apply-apply-status-initial-diff-update.md`.

## M344: PrintApply print_diff config invalidation
Port the lock-ordered `print_diff` config invalidation block from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1241-1246`, plus M343 max-based status-update context, into `ares-core` as private staged print-diff config invalidation state. Preserve staged lock acquisition before the gate, no invalidation call when `print_diff` is empty, a staged `invalidate_state_by_config_options(new_full_config, print_diff)` call when non-empty, and max-based status aggregation from the invalidation boolean result. Defer placeholder parser/full-config handling from `PrintApply.cpp:1248-1265`, real mutex locking, real background processing stop, real invalidation, real config mutation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m344-print-apply-print-diff-config-invalidation.md`.

## M345: PrintApply full_config_diff placeholder entry
Port the `full_config_diff` placeholder-parser entry slice from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1248-1256`, plus M343 max-based status-update context, into `ares-core` as private staged full-config placeholder entry state. Preserve current extruder-count capture from `m_config.filament_diameter.size()`, `num_extruders_changed = false`, branch entry only when `full_config_diff` is non-empty, staged changed-branch log metadata, staged `invalidate_step(psGCodeExport)`, max-based status aggregation from that invalidation result, and staged `m_placeholder_parser.clear_config()` after invalidation. Defer placeholder preset assignments from `PrintApply.cpp:1257-1260`, placeholder `apply_config(filament_overrides)` from `PrintApply.cpp:1261-1263`, config mutation from `PrintApply.cpp:1264-1275`, extruder-count change handling from `PrintApply.cpp:1276+`, real logging, real invalidation, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m345-print-apply-full-config-placeholder-entry.md`.

## M351: PrintApply extruder count change handling
Port the extruder-count change handling slice from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1276-1279`, plus the pre-branch count capture and full print config assignment context from `PrintApply.cpp:1248-1275`, into `ares-core` as private staged extruder-count change state. Preserve previous-count identity `num_extruders`, current-count source `m_config.filament_diameter.size()`, the inequality branch, no assignment/no changed flag when counts match, and assignment of `num_extruders` to the current count plus `num_extruders_changed = true` when counts differ. Defer full-config branch exit from `PrintApply.cpp:1280`, `ModelObjectStatusDB` construction from `PrintApply.cpp:1282`, model-object synchronization from `PrintApply.cpp:1284+`, real config mutation, real vector storage, public API wiring, profile loading, UI runtime, slicing, extrusion, G-code, crates, dependencies, and Ares-owned pipeline behavior.

Exit criteria are tracked in `docs/milestones/m351-print-apply-extruder-count-change.md`.

## M852: OrcaSlicer source crate partition checkpoint
Record the source-structure checkpoint for crate partitioning using `OrcaSlicer/src/CMakeLists.txt`, `OrcaSlicer/src/libslic3r/CMakeLists.txt`, `OrcaSlicer/src/libvgcode/CMakeLists.txt`, and `OrcaSlicer/src/slic3r/CMakeLists.txt` as evidence. Confirm that the active Rust workspace remains `ares-core`, `ares-vgcode`, `ares-cli`, and `ares-wasm`; do not create `ares-geometry`, `ares-config`, `ares-gcode`, `ares-support`, or UI crates without a future source-cited milestone that proves the boundary and updates `Cargo.toml` plus `AGENTS.md`. This milestone is an architecture/documentation gate for the full `libslic3r`/`libvgcode` rewrite, not an Ares-owned pipeline feature.

Exit criteria are tracked in `docs/milestones/m852-orcaslicer-source-crate-partition-checkpoint.md`.

## Task 22O.1: Classic perimeter prelude

Port the first executable Classic generator slice from fixed OrcaSlicer v2.4.2
`LayerRegion.cpp::LayerRegion::make_perimeters`,
`PerimeterGenerator.cpp::process_classic` before the onion loop,
`process_no_bridge`'s `chbNone` path,
`generate_lower_polygons_series`, `Flow.cpp::Flow::with_width`,
`ShortestPath.cpp::chain_expolygons`, and `BoundingBox.hpp` into
`ares-core::project_slice::perimeters::classic` and
`ares-core::geometry::bounding_box`.

Included behavior is transactional activated-branch preflight, preserved Task
22N predecessor slots, typed 3MF Option consumption, precise spacing, smaller
external Flow, lower support masks and samples, arc-aware surface
simplification/union, center chaining, and loop-count preparation. Public
`slice_project` consumes the new state and continues to return
`ProjectSlicingIncomplete`. The obsolete opaque Task 22N synthetic binary
embedding is removed in favor of readable parser behavior construction.

The Package-A0 recovery documents are historical/non-blocking and are not
retried. This milestone does not claim complete Classic generation or G-code
parity. `split_top_surfaces`, onion shells, traversal, overhang splitting, gap
medial axes, variable-width entities, fill, seams, infill, motion planning,
writer behavior, metadata, post-processing, and normalized KSR byte parity
remain deferred. Exit criteria are tracked in
`docs/superpowers/specs/2026-08-01-ksr-fdmtest-v4-task22o1-classic-prelude.md`
and its matching plan.

## Task 22O.2: Classic dynamic top-one-wall split

Port fixed OrcaSlicer v2.4.2
`PerimeterGenerator.cpp::split_top_surfaces` plus the smallest source-exact
non-thin-wall `i == 0` external offset prerequisite and caller seam. The first
offset is included solely because upstream assigns it to `last` before calling
the split; later onion iterations remain deferred.

The Rust stage owns Task 22O.1 as its predecessor, preflights all typed 3MF
options before geometry, retains normal/smaller first-offset geometry, and
records the post-caller remaining area, top fills, fill clip, caller outcome,
and upper-source selection. It also ports the required Clipper bbox vertex
prefilter, ExPolygon area, polygon-clip difference, and automatic infill-width
seams. The public lifecycle executes the stage and remains intentionally
incomplete.

`i >= 1`, loop entities, hierarchy/traversal, thin-wall medial axes,
multi-region behavior, later bridge kinds, gap masks, overhang splitting, fill
remainder, seams, infill, motion, writer and post-processing remain deferred.
Exit criteria are tracked in
`docs/superpowers/specs/2026-08-01-ksr-fdmtest-v4-task22o2-dynamic-top-one-wall.md`
and its matching plan.

## Task 22O.3: Classic raw-onion iteration

Task 22O.3 ports fixed OrcaSlicer v2.4.2
`PerimeterGenerator.cpp:1304-1387` as a loop-back continuation after the
immutable Task 22O.2 depth-zero/top-split state. The
`project_slice::perimeters::classic::onion` successor validates typed effective
`sparse_infill_density` transactionally and converts it to the source local
`int`, selects depth-one and deeper spacing, applies source-exact `offset2_ex`
casts and fixed-coordinate safety terms, appends ordered gap masks before
termination, records raw normal/smaller shell depths, reduces the effective
count on collapse, retains final `last`, and executes the positive
converted-density gap-only pass. The public lifecycle executes this
stage and remains intentionally incomplete.

Hierarchy and nesting begin at `PerimeterGenerator.cpp:1388` and remain the next
source boundary. Traversal, extrusion entities, overhang behavior, gap medial
axes, fill remainder, seams, infill, motion, writer/post-processing, complete
Task 22O, and exact KSR G-code parity remain open. Exit criteria are tracked in
`docs/superpowers/specs/2026-08-01-ksr-fdmtest-v4-task22o3-onion-iteration.md`
and its matching plan.

## Task 22O.4: Classic loop hierarchy

Task 22O.4 ports fixed OrcaSlicer v2.4.2
`PerimeterGenerator.cpp:34-55,1353-1369,1388-1443` and exact
boundary-inclusive containment from `Polygon.hpp:66`, `Polygon.cpp:722-729`,
and Clipper v6 `PointInPolygon`. The
`project_slice::perimeters::classic::hierarchy` successor nests immutable O3,
materializes its raw normal/smaller loops without recomputation, and performs
the source destructive hole-first and contour first-parent searches. Roots and
diagnostic leftovers preserve source order. The public lifecycle executes O4
and remains intentionally incomplete.

Traversal around line 1450, extrusion entities, thin walls, overhang behavior,
wall ordering, gap medial axes, fill remainder, seams, infill, motion,
writer/post-processing, complete Task 22O, and exact KSR G-code parity remain
open. Exit criteria are tracked in
`docs/superpowers/specs/2026-08-01-ksr-fdmtest-v4-task22o4-loop-hierarchy.md`
and its matching plan.

## Task 22O.5: Classic traversal seeds

Task 22O.5 ports fixed OrcaSlicer v2.4.2
`PerimeterGenerator.cpp:100-151` and `PerimeterGeneratorLoop::is_internal_contour`
at `2537-2547` into `project_slice::perimeters::classic::traversal`. The O5
successor nests immutable O4 and iteratively builds ordered seed trees from O4
roots, preserving immediate-child loop classification, exact depth roles,
source flow/lower-series selectors, `f32` width, source `f64` layer height and
`f64 mm3_per_mm`,
and typed pending line-158 overhang predicate provenance. Public slicing
executes O5 and remains intentionally incomplete.

No fuzzy mutation or pending ordinary/overhang branch executes. Clipping,
extrusion paths/loops/entities, actual entity traversal/reordering, thin walls,
active overhang reversal, wall ordering, gaps/fill, seams, infill, motion,
writer, complete Task 22O, and exact KSR parity remain open. Exit criteria are
tracked in
`docs/superpowers/specs/2026-08-01-ksr-fdmtest-v4-task22o5-traversal-seeds.md`
and its matching plan.

## Task 22O.6: Exact open-path Clipper

Task 22O.6 ports the fixed Clipper v6 open-input and output state machine from
`clipper.cpp:756-949` and its output-affecting `IsOpen` branches, plus
OrcaSlicer `ClipperUtils.cpp:835-934`, into `geometry::clipper`. The exit
boundary includes open subjects, typed open PolyTree roots, exact scanline and
horizontal behavior, open fixup, source-order extraction, polygon closure, and
destructive four-case polyline recombination while preserving inherited closed
Clipper results and `f64` full-range predicates.

O6 remains the exact open-clipping dependency consumed by O7. Exit criteria
are tracked in
`docs/superpowers/specs/2026-08-01-ksr-fdmtest-v4-task22o6-open-clipper.md` and
its matching plan.

## Task 22O.7: Raw extrusion path materialization

Task 22O.7 ports fixed `PerimeterGenerator.cpp:153-207,218-224`, reached
`ExtrusionEntity.hpp:153-188,551-580`, and `Polyline.hpp:291-302` into
crate-private `project_slice::perimeters::classic::materialize`. It creates an
aligned successor nesting O5, dispatches solely on O5's pending branch, and
uses O2 bbox filtering plus O6 intersection-before-difference output. Ordinary
paths preserve exact polygon closure and seed flow; supported fragments retain
seed role/flow and layer height while remainder fragments use overhang role and
all overhang-flow numeric fields. Trees, error cleanup, and terminal sinking
are iterative. Public slicing executes O7 and remains
`ProjectSlicingIncomplete`.

O1-established inactive fuzzy skin and rejected active `overhang_reverse` make
the fuzzy, steep, and reverse branches unreachable, so O7 does not model them.
O8 owns lines 208-210 empty/start/chaining and line 227 loop construction. Exit
criteria are tracked in
`docs/superpowers/specs/2026-08-01-ksr-fdmtest-v4-task22o7-raw-extrusion-paths.md`
and its matching plan.

## Task 22O.8: Chained extrusion loops

Task 22O.8 ports fixed `PerimeterGenerator.cpp:208-210,227`, the reached
all-paths-reversible greedy specialization in `ShortestPath.cpp` with exact
`KDTreeIndirect.hpp` and `MutablePriorityQueue.hpp` semantics, and the reached
`ExtrusionLoopRole` / `ExtrusionLoop` ownership from `ExtrusionEntity.hpp`.
Only overhang-clipping records apply empty `continue` and start-near chaining;
ordinary records bypass both. O8 moves O7 path buffers zero-copy, retains the
boxed O5 predecessor, maps all loop roles, and transforms and drains arbitrary
depth trees iteratively. Public slicing executes O8 and remains
`ProjectSlicingIncomplete`.

O9 owns `PerimeterGenerator.cpp:230-280`; orientation is not moved earlier
because upstream applies it only after recursive entity selection. O8 exit criteria are
tracked in
`docs/superpowers/specs/2026-08-02-ksr-fdmtest-v4-task22o8-chained-extrusion-loops.md`
and its matching plan.

## Task 22O.9: Ordered entity collections

Task 22O.9 ports fixed `PerimeterGenerator.cpp:230-280`, caller setup/call
`1443-1450`, reached loop-only `ShortestPath.cpp:1026-1040`, and
`ExtrusionEntity.cpp:141-170`. Each recursive source group chains loop entities
from zero, clears loop reversal selections, recursively orders children,
applies exact Clipper orientation from aligned typed wall direction, sets
`inset_idx`, and emits a flat collection. The Rust implementation is iterative
and moves O8 buffers while retaining the boxed O5 predecessor.

The exact source compact-entity/original-loop indexing after line-208
`continue` is retained rather than repaired. Thin-wall append is inactive
because O1 rejects `detect_thin_wall=true`; active medial-axis generation,
`variable_width`, heterogeneous entity chaining, fuzzy skin, overhang
reorientation, wall-sequence changes, gaps/fill, seams, infill, motion,
G-code, writer/post-processing, complete Task 22O, and final parity remain
open. Exit criteria are tracked in
`docs/superpowers/specs/2026-08-03-ksr-fdmtest-v4-task22o9-ordered-entity-collections.md`
and its matching plan.

## Task 22O.10: Perimeter collection append

Task 22O.10 ports fixed `PerimeterGenerator.cpp:1451-1569`. O1 preflight
proves overhang reorientation, non-`InnerOuter` wall ordering, and active
layer-zero outer-brim reversal inactive before geometry moves, so O10 records
those typed operands without adding active algorithms or fallbacks. Each
nonempty O9 flat collection is moved as one nested perimeter collection;
empty collections are omitted and all entity order, fields, allocations, and
the boxed O5 predecessor remain intact. Public slicing executes O10 and stays
`ProjectSlicingIncomplete`.

Gap filling at `PerimeterGenerator.cpp:1573+`, active ordering/reorientation,
seams, infill, motion, G-code, writer/post-processing, complete Task 22O, and
final parity remain open. Exit criteria are tracked in
`docs/superpowers/specs/2026-08-03-ksr-fdmtest-v4-task22o10-perimeter-collection-append.md`
and its matching plan.

## Task 22O.11: Pre-medial Classic gap domain

Task 22O.11 ports fixed commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, exact primary boundary
`PerimeterGenerator.cpp:1573-1581,1583-1585`, and stops before line 1586.
The crate-private O11 stage reads aligned O3 gaps and prelude parameters through
O10's boxed O5 chain, transactionally stages exact opening, second offset,
ordinary difference, and in-place ExPolygon Douglas–Peucker results, then
moves O10 collections without changing their allocations. Empty gaps produce
typed `None`; retained predecessor trees are consumed iteratively on success and geometry error; public slicing reaches O11 and remains incomplete.

Exit requires focused direct, lifecycle, and in-memory KSR anchors, unchanged
boxed O5 and O10 nested allocations, stable geometry range errors, and Tier-1
portable Rust checks. The paired spec and plan are
`docs/superpowers/specs/2026-08-03-ksr-fdmtest-v4-task22o11-pre-medial-gap-domain.md`
and
`docs/superpowers/plans/2026-08-03-ksr-fdmtest-v4-task22o11-pre-medial-gap-domain.md`.
The next rewrite boundary begins at `PerimeterGenerator.cpp:1586` with medial
axis and actual ThickPolyline prerequisites. Gap extrusion, downstream G-code,
final KSR parity, and Orca end-to-end comparison remain deferred.

## Task 22O.12: ThickPolyline medial-axis prerequisite

Task 22O.12 ports fixed commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, exact source boundaries
`Line.hpp:15-19,202-212`, `Polyline.hpp:14-17,256-287`, and
`Polyline.cpp:637-679`, into crate-private geometry types. It preserves
ThickLine endpoint widths, ThickPolyline default/reverse/clear semantics,
ordered two-width segment projection, closed-ring rotation, and fixed-width
conversion.

O12 does not advance public slicing: O11 remains the terminal prefix and
`PerimeterGenerator.cpp:1586` remains the next unexecuted line. Exit requires
literal source-semantic tests, unchanged O11 behavior, no dependency or public
API change, Tier-1/WASM checks, and independent review. The paired documents
are
`docs/superpowers/specs/2026-08-03-ksr-fdmtest-v4-task22o12-thick-polyline-prerequisite.md`
and
`docs/superpowers/plans/2026-08-03-ksr-fdmtest-v4-task22o12-thick-polyline-prerequisite.md`.
The next milestone must port the actual Voronoi topology required by
`Geometry::MedialAxis`; a simplistic skeleton substitute or runtime Orca
oracle is not acceptable.

## Task 22O.13: Classic gap medial-axis extraction

Task 22O.13 ports fixed commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1` source boundaries
`ExPolygon.cpp:261-369`, `Geometry/MedialAxis.cpp:458-707`, and the reached
`Geometry/VoronoiOffset.cpp:646-971` annotation path. It adds the pinned
pure-Rust Boost-compatible segment Voronoi topology, source-order annotation,
validation and chaining, ExPolygon post-processing, and an aligned,
transactional O13 Classic lifecycle successor. Exit requires literal geometry,
topology, lifecycle and KSR structure tests, unchanged O11-O5 ownership,
Tier-1/WASM checks, strict Clippy/rustfmt, and independent review.

The parity boundary uses `std::round` (half away from zero) for Voronoi
`Point(double, double)` seed/growth and validation-Line sites, while the
endpoint-extension Eigen casts remain truncating. The adapter validates the
reached twin/site/face/rotation invariants and annotation uses Boost's 64-ULP
point comparison. Browser qualification names transitive `getrandom` 0.3.4
with `wasm_js` only on `wasm32`.

Public slicing remains `ProjectSlicingIncomplete`. Orca's invalid-diagram
repair path remains deferred.

## Task 22O.14: Classic variable-width gap extrusion

Task 22O.14 ports fixed commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1` source boundaries
`PerimeterGenerator.cpp:1604-1624`, `VariableWidth.cpp:99-234`, reached
`Flow` and extrusion-entity coverage, `ClipperUtils` open-polyline offset, and
Clipper 6 OpenButt generation/cleanup. It validates every aligned typed
`RegionOptions.filter_out_gap_fill` before geometry, applies the strict
source-length filter at Normal and LargeBed scales, converts retained
ThickPolylines into ordered fixed-coordinate GapFill paths/loops using the
aligned solid-infill flow, and subtracts their ordered covered-width polygons
from cloned onion `last` geometry.

The O14 lifecycle is transactional across the whole project: validation, keep
masks, conversion, two-level open-offset cleanup, and differences precede any
O13 ownership move. It preserves the boxed O5 and nested O11/O10/O13
allocations that survive filtering and uses iterative success/error sinks.
Public slicing reaches O14 exactly once and stays
`ProjectSlicingIncomplete`. Exit requires literal open-offset, variable-width,
direct option/geometry, ownership/lifecycle, and repeatable in-memory KSR
coverage plus focused O13-O5 regressions, workspace Nextest, strict Clippy,
workspace/native and both WASM checks, rustfmt, diff/LOC/forbidden audits, and
independent implementation review. The post-fix workspace run passed 5,491
Nextest tests with 2 skipped; strict Clippy, workspace/native and both WASM
checks, rustfmt, diff/LOC/forbidden/dependency/staging audits passed. Independent
Codex and OpenCode re-reviews both returned `VERDICT: APPROVE`. Post-O20
Tier-1 run `30900710846`, Windows job `91964102127`, later recorded the O11
closed-boolean-tree and O14 open-offset/variable-width constrained-stack tests
reaching the Windows 64 KiB floor, aborting at `86.033s` and `47.523s` with
`0xc00000fd` / OS error 1001 after 4,175 preceding passes. Raising only those
two let exact-SHA rerun `30904949178`, Windows job `91977766653`, advance to
O15, where aggregate-union and final-top-union cleanup hit the same floor.
All project-slice constrained-stack tests therefore share a test-only baseline
of 64 KiB on Unix and 256 KiB on Windows while retaining their 10,000-node
predecessor and iterative-cleanup assertions.

## Task 22O.15: Classic infill-boundary construction

Task 22O.15 ports fixed commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`
`PerimeterGenerator.cpp:1628-1691` and the reached `ExPolygon::simplify_p`,
Clipper offset/boolean, and internal-surface helpers. It derives inset and both
overlap values from aligned typed 3MF state, uses raw `m_scaled_resolution`,
preserves exact integer/floating halves and narrowing casts, builds ordered
internal fill and `fill_no_overlap` geometry, and retains the exact inactive
six-operand `apply_extra_perimeters` guard.

Numeric preflight and every simplification/Clipper result are staged for the
whole project before O14 ownership moves. The successor preserves O14 gap
entities and remaining geometry plus O13/O11/O10 and boxed O5 allocations;
success and failure cleanup remain iterative. Public slicing reaches O15 once
and remains `ProjectSlicingIncomplete`. The literal KSR checksum is
`136197013209006370081121271251125478104`; 49 focused O15 tests and geometry
regressions, 5,540 workspace Nextest tests with 2 skipped, strict Clippy, workspace/native and both
WASM checks, formatting, diff, LOC, forbidden-pattern, dependency, and staging
audits pass. The final independent six-dimensional implementation rereview and
OpenCode rereview both returned `VERDICT: APPROVE`.

The activated extra-perimeter body and Arachne-only helper beginning at
`PerimeterGenerator.cpp:1695` remain deferred.

## Task 22O.16: Layer-region perimeter outputs

Task 22O.16 ports the KSR-reached output seam at fixed commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1` from
`LayerRegion.cpp:82-142`, `Layer.cpp:185-226`, `Layer.hpp:50-61,72-74`, and
`Surface.hpp:159-166`. For the already validated one-compatible-region shape,
it materializes ordered `perimeters`, `thin_fills`, `fill_surfaces`, copied
`fill_expolygons`, and `fill_no_overlap_expolygons` after the completed Classic
perimeter generator.

The source many-to-one append consumes artificial per-surface wrapper vectors
while moving every nested collection entity, loop/path/point, gap-loop/path,
record-level fill/no-overlap, and boxed traversal allocation that survives in
LayerRegion state. `fill_expolygons` are value-equal and allocation-distinct
copies of fill-surface geometry. The inactive `process_no_bridge` return stays
typed by existing Classic preflight; its active body and the multi-compatible-
region merge/split branch remain deferred.

The literal KSR checkpoint is
`-169716507603417685621692788651154411580`, with totals
`[1, 460, 460, 2881, 5243, 2285, 1112, 1112, 1112]`. Fourteen focused O16
tests, 192 O1/O10-O16 regressions, and 5,554 workspace Nextest tests with 2
skipped pass together with strict Clippy, workspace/native and both WASM
checks, formatting, diff, LOC, forbidden-pattern, source-pinning, dependency,
and staging audits. The final independent six-dimensional implementation
review and OpenCode review both returned `VERDICT: APPROVE`. Public slicing
reaches O16 once and remains
`ProjectSlicingIncomplete`.

## Task 22O.17: Surface-type detection and clipped fill transfer

Task 22O.17 ports the first complete `PrintObject::prepare_infill` mutation at
fixed commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`: the KSR-active
`detect_surfaces_type` path at `PrintObject.cpp:1520-1923` and
`LayerRegion::slices_to_fill_surfaces_clipped` at `LayerRegion.cpp:63-80`.
It emits source-ordered typed slices, then rebuilds fill surfaces in numeric
kind order against unchanged O16 fill boundaries.

The implementation preserves the two-stage miter/3.0 opening, clip-only
10-unit safety, exact integer/float cast order, per-surface contour-then-hole
order, metadata clone/reconstruction rules, and the pinned crack-containment
overload that ignores its apparent safety argument. It stages the whole project
before moving O16 ownership and iteratively consumes every failure path.

The temporary early `enable_support` and `enforce_support_layers` capability
gates are removed only far enough to feed Orca's literal typed bottom-support
predicate. Interface shells and active external/all extra-bridge modes remain
O17 preflight errors; earlier spiral and counterbore errors retain precedence.
No support generation or completed prepare-infill lifecycle is claimed.

The literal KSR checkpoint is
`-126362407653399901571400348049652748978`, with totals
`[1, 460, 460, 2881, 5243, 2285, 1112, 1112, 5388, 519, 6, 666, 4197, 1294,
113, 6, 48, 1127, 5388, 517, 85886, 1294, 168, 46011]`. Forty-three focused
O17 tests, 178 O1-O17 regressions, and 5,597 workspace Nextest tests with 2
skipped pass with strict Clippy, workspace/native and both WASM checks,
formatting, diff, LOC, forbidden-pattern, source-pinning, dependency, and
staging audits. ZIP repack/non-slicing rename invariance and an exact component
X-scale relation with fixed elephant-foot compensation provide anti-hardcoding
evidence. The final independent six-dimensional implementation rereview and
OpenCode rereview both returned `VERDICT: APPROVE`.

Public slicing reaches O17 once and remains `ProjectSlicingIncomplete`.

## Task 22O.18: Fill-surface shell preparation

Task 22O.18 ports the slicing-state mutation in
`LayerRegion::prepare_fill_surfaces` at `LayerRegion.cpp:935-973`, called from
`PrintObject.cpp:587-592`. It consumes each aligned record's typed resolved
region options and performs three literal in-place kind passes: zero top shells
retag `Top` to `Internal`; zero bottom shells retag `Bottom` and
`BottomBridge` to `Internal`; strict `abs(density - 100) < 1e-4` retags
`Internal` to `InternalSolid`. The pinned static
`infill_only_where_needed = false` leaves `InternalVoid` deferred.

All alignment is validated before writing. O18 allocates no replacement state:
fill vectors, surface geometry and metadata, source order, typed slices,
perimeter/thin-fill outputs, boundaries, and boxed predecessor retain identity.
The early capability boundary now rejects typed global spiral mode before O17,
closing its threshold-masked record-local bypass; six obsolete unsupported-
spiral checkpoint-pinning tests were removed instead of retained as legacy
expectations. Tier-1 run `30900710846`, WASM job `91964102068`, confirmed the
stale six-pair browser matrix failed at its first spiral-activated pair with
`unsupported project feature: spiral_mode`; the browser N matrix now retains
only the supported alignment, signed-zero, and generator context pairs.

KSR's inactive 5/3/15% options preserve checksum
`-126362407653399901571400348049652748978`; totals are
`[1, 460, 460, 2881, 5243, 2285, 1112, 1112, 5388, 519, 6, 666, 4197, 1294,
113, 6, 48, 1127, 5388, 517, 85886, 1294, 168, 46011, 0, 0]`. Seventeen focused
O18 tests, 209 O10-O18 regressions, and 5,607 workspace tests with 2 skipped
pass with native, strict Clippy, both WASM, formatting, diff, LOC,
forbidden-pattern, dependency, pinning-removal, and staging gates. Typed global
and model-part override cases freeze nonzero literal transition counts and
prove record-aligned option provenance. The final independent six-dimensional
implementation rereview and OpenCode rereview both returned
`VERDICT: APPROVE`.

Public slicing reaches O18 once and remains `ProjectSlicingIncomplete`. The
next rewrite boundary is `PrintObject::discover_vertical_shells` beginning at
`PrintObject.cpp:595`. Horizontal shells, external-surface processing, fill
grouping/generation, seams, ordering, motion, G-code, and post-processing remain
deferred.

## Task 22O.19: Single-region vertical-shell cache

Task 22O.19 ports caller `PrintObject.cpp:595-596` and cache declarations,
gating, and single-region population at `PrintObject.cpp:2008-2027,2111-2149`.
For every aligned populated record, only `EnsureAll` expands typed top and
bottom/bottom-bridge slices by `(solid_infill_spacing as f32) * 0.05_f32` with
miter `3.0`; all other enum modes produce an empty cache. Fill expolygons
flatten contour then holes without union. The borrowed-expolygon offset adapter
preserves per-expolygon Paths order and the source conditional positive NonZero
union.

The whole-project stage completes before ownership moves. Its successor keeps
the exact O18 predecessor and object/record allocations and stores fresh cache
geometry in an aligned sidecar. One-region preflight keeps the aggregate branch
at `PrintObject.cpp:2028-2109` deferred. Public slicing reaches O19 once and
remains `ProjectSlicingIncomplete`. KSR freezes cache checksum
`-114359197324258778780701398534712718623`, parent-bound successor checksum
`148296943860974241781127169756103364063`, totals
`[1, 460, 0, 460, 572, 713, 1227, 60370, 2512]`, and first/later spacings
`[457079, 377079]`. Twenty-one focused tests, 310 O10-O19 regressions, and
5,630 workspace tests with 2 skipped pass with strict native, Clippy, WASM,
formatting, LOC, forbidden-pattern, dependency, source-pinning, and staging
gates. The final independent six-dimensional and OpenCode rereviews both
returned `VERDICT: APPROVE`. The next rewrite boundary is projection at
`PrintObject.cpp:2153`; horizontal shells, external surfaces, fill generation,
seams, ordering, motion, G-code, and post-processing remain deferred.

## Task 22O.20: Single-region vertical-shell projection gather

Task 22O.20 ports the release-observable projection gather in
`PrintObject::discover_vertical_shells` at `PrintObject.cpp:2153-2278`. It
starts each active populated layer with current cache holes, scans top before
bottom with exact count-or-thickness predicates, combines neighboring holes by
incremental NonZero Paths intersection, and combines top/bottom shells by
append-then-incremental NonZero Paths union. Planned-index existence controls
the windows: a neighboring `None` remains a visited empty cache that clears
holes and suppresses anchors, while current `None` stays aligned and defers a
transient proven dead at the next trim boundary.

If a positive shell count visits no layer and its stopped index exists, the
anchor expands current cache Paths by current aligned external-perimeter
spacing after the exact f32 cast, miter `3.0`, then intersects stopped-index
object `lslices` in contour-then-hole order. Existing CCW Positive and CW
Negative offset cleanup feeds final NonZero union; new Paths-only boolean
adapters preserve flat Clipper output without PolyTree or canonicalization.

The whole project validates alignment and stages while borrowing O19, then
moves the exact O19 predecessor, objects, caches, and nested allocations beside
fresh projection geometry. Every O20 geometry error uses stable text and
iterative cleanup. Public slicing reaches O20 once and remains
`ProjectSlicingIncomplete`.

KSR freezes parent-bound checksum
`-106767561006193260948265111057697183253`, totals
`[1, 460, 0, 460, 1688, 1224, 36512, 69033]`, and event totals
`[1830, 917, 1539, 749, 0, 0, 0, 0]`. Forty-five focused tests cover exact
combiners, CCW/CW anchors including an exact acute miter-3 witness, first/last
and strict epsilon windows, both `None` roles, current-versus-stopped spacing,
exhaustive alignment/identity rejection, recursive ownership of both
predecessor tree families, an active later-object transactional failure, all
error sites, constrained stacks, typed project/model-part mutations, ZIP/name
and scale metamorphism, and independent KSR parses. All 355 O10-O20 regressions
and 5,678 workspace tests with 2 skipped pass with strict Clippy, native
all-target, both WASM, formatting, LOC, forbidden-pattern, dependency,
source-pinning, and staging gates. Final independent six-dimensional and
OpenCode implementation rereviews both approve the identical final diff. The
pushed commit must pass the Tier-1 native matrix and complete browser-WASM job.

The next rewrite boundary is internal-surface trimming at
`PrintObject.cpp:2334`. Regularization, horizontal shells, external surfaces,
fill generation, seams, ordering, motion, G-code, and post-processing remain
deferred. O19/O20 sidecars are temporary source-compatibility representations,
not an Ares-owned pipeline.

## Task 22O.21: Single-region vertical-shell internal trimming

Task 22O.21 ports `PrintObject::discover_vertical_shells` lines 2334-2342. An
active populated record flattens reachable `Internal | InternalSolid` fill
surfaces in stable collection order, contour then holes. `InternalVoid` remains
explicitly deferred because the pinned static-false producer is not present in
the approved O17-O20 envelope; this task adds neither a variant nor a producer.

The O20 shell is intersected with the internal clip using the shared raw
path-by-path `10.0_f32` safety offset and miter `3.0`, then flat NonZero
`polygons_internal - holes` output is appended. Safety expands only the clip and
never pre-unions it. Both flat boolean results preserve Clipper path/point order
without PolyTree conversion, sorting, deduplication, or union. The accumulated
empty gate precedes the second source-order scan and verbatim
`InternalSolid` append, so nonempty records intentionally duplicate solid
geometry while fully erased records skip the append.

The whole project validates complete O20 alignment and stages while borrowing
O20 before moving its exact predecessor, surface, cache, and projection
allocations beside fresh non-aliasing trims. Inactive populated records are
`Some(empty trim)` without geometry, `None` stays aligned, every trimming error
uses one stable message and iterative O20 cleanup, and public slicing reaches
O21 once while remaining `ProjectSlicingIncomplete`.

KSR independently guards the frozen O19/O20 parent checksums, totals, and O20
events before freezing O21 checksum
`-86220837291247746226319093859583939318`, totals
`[1, 460, 0, 460, 7704, 104680]`, and ordered events
`[460, 460, 460, 460, 259]`. Forty-two focused O21 tests, 386 explicit
O10-O21 regressions, and 5,717 workspace tests with 2 skipped pass. Native
all-target check and strict all-feature Clippy are clean; final exact-diff
formatting/audits and Tier-1/review gates remain part of the ship gate.
Complete post-review mutation REDs exercise all 11 adapter, 10 record, and 21
integration tests before byte-exact restoration and GREEN reruns.

The next rewrite boundary is regularization at `PrintObject.cpp:2344`.
Horizontal shells, external surfaces, fill generation, seams, ordering, motion,
G-code, and post-processing remain deferred. O19-O21 remain temporary
source-compatibility sidecars, not an Ares-owned pipeline. Rollback restores O20
terminal consumption and removes only O21 state/wiring/tests/docs, its two
flat-Paths adapters, and the sibling visibility change for existing safety
constants.

## Task 22O.22: Single-region vertical-shell morphology regularization

Task 22O.22 ports `PrintObject::discover_vertical_shells` lines 2344-2367 and
stops before `object_volume` at line 2369. For each nonempty O21 trim, the
aligned typed solid-infill spacing is cast once to `f32`, multiplied by
`1.05_f32`, and used in the three literal source expressions for ensure,
sparse-gap, and overlap radii. O22 executes NonZero `union_ex`, both Square
`offset2_ex` stages, and Square shrink in nested source order with miter limit
`3.0`; an empty union still flows through the remaining operations. A minimal
inter-stage observer reuses the existing offset2 body without duplicating or
changing its ordinary production behavior.

The whole project is validated and staged while borrowing O21 before moving the
exact predecessor, surface, cache, projection, and trim allocations beside
fresh regularized ExPolygons. Every error uses one stable message and
iteratively cleans both 10,000-node predecessor tree families. Public slicing
reaches O22 exactly once while remaining `ProjectSlicingIncomplete`. Real typed
3MF tests cover active/inactive modes, model-part precedence, line-width to
spacing/radius/output provenance, ZIP/name invariance, and component scaling.

Parent-guarded repeated KSR capture freezes O22 checksum
`134936948052282121922360252649864225707`, totals
`[1, 460, 0, 460, 632, 632, 128, 34557]`, ordered operation totals
`[259, 259, 259, 259]`, and exact-radii digest
`-119839535044106185061007902266478724784` after reasserting all O19-O21
values. Eleven direct and 22 integration tests pass, as do 346 O10-O22
regressions and 5,750 workspace tests with 2 skipped. Strict
all-target/all-feature Clippy passes. Compiling post-implementation mutation
REDs fail 4 direct and 2 integration tests when the source `1.05_f32` factor is
removed; supplemental mutations fail all 5 alignment tests, public lifecycle,
and genuine later-slot transaction staging before byte-exact production
restoration and GREEN reruns.

The next boundary is `object_volume`, neighboring-layer volume accumulation,
and tiny-area filtering at `PrintObject.cpp:2369`. Horizontal shells, external
surfaces, fill generation, seams, ordering, motion, G-code, and post-processing
remain deferred. O19-O22 stay temporary source-compatibility sidecars rather
than an Ares-owned pipeline. Rollback restores O21 terminal consumption and
removes only O22 state/wiring/tests/docs and its inter-stage observer entry.

## Task 22O.23: Single-region vertical-shell tiny-island filtering

Task 22O.23 ports `PrintObject::discover_vertical_shells` lines 2369-2400 and
stops before `intersection_ex(polygonsInternal, regularized_shell)` at line
2402. Previous and next retained object `lslices` form a flat NonZero
intersection with lower Paths as subject and upper Paths as clip. Current
internal Paths use flat Miter-3 closing with the directly cast floating
`(1e-4_f64 / scale.factor()) as f32` epsilon. The area constants follow their
separate truncating coordinate path: selected-scale `scaled(1.5)` and
`scaled(8.0)` become `i64`, then `f32`, multiply the shared O22 minimum in
`f32`, and are promoted only for signed `f64` strict-`<` comparisons.

The complete lazy source predicate retains the conditional visibility
difference, candidate Miter-3 expansion, and literal flat path-count protection
comparison. Survivors are fresh deep clones in stable O22 order. O23 validates
all inherited alignment before geometry, stages the whole project while
borrowing O22, moves the exact predecessor only after success, and iteratively
disposes both 10,000-node predecessor tree families on success, failure, and
the public incomplete boundary. Public slicing reaches O23 exactly once and
remains `ProjectSlicingIncomplete`.

Parent-guarded repeated KSR capture freezes checksum
`-41564956609250807593946297629749369320`, totals
`[1, 460, 0, 460, 632, 554, 78, 554, 128, 33815]`, threshold digest
`-167664109034474951983490568976349754300`, and ordered event totals
`[259, 259, 259, 632, 66, 80, 80, 259]` after reasserting O19-O22. Eighteen
direct and 29 integration tests pass; 393 O10-O23 regressions and 5,797
workspace tests with 2 skipped pass. Native, strict Clippy, four WASM checks,
optimized browser-WASM/export audit, and two 9-test Playwright runs are green.
All ten required compiling behavioral mutations fail their intended witnesses
before byte-exact restoration and GREEN reruns. Rust LOC, formatting, diff,
dependency, forbidden-pattern, and staging audits pass.

The next boundary starts at
`intersection_ex(polygonsInternal, regularized_shell)` in
`PrintObject.cpp:2402`. Fill-surface mutation and every later horizontal-shell,
external-surface, fill, toolpath, G-code, and post-processing stage remain
deferred. O19-O23 are temporary source-compatibility sidecars, not an
Ares-owned pipeline. Rollback restores O22 terminal consumption and removes
only O23 state/wiring/tests/docs and its two restricted sibling visibility
changes.

## Task 22O.24: Single-region vertical-shell fill-surface assignment

Task 22O.24 ports `PrintObject::discover_vertical_shells` lines 2402-2432 and
completes that function's constrained single-region state transition. It adds
`InternalVoid = 8` as source vocabulary with exhaustive non-bridge semantics,
feeds flat internal Paths directly into a mixed Polygon/ExPolygon NonZero
intersection, then executes Internal and InternalVoid differences against the
same pre-mutation collection. After whole-project staging, active records
stably retain Top/Bottom/BottomBridge and append fresh Internal, InternalVoid,
and InternalSolid groups in source order with default metadata. Empty-filter
records are allocation-exact no-ops.

All inherited O23 alignment, including typed printable-area scale selection,
is rejected before geometry. Later-record failures cannot expose a partial
mutation; exact O23 ownership is retained and both 10,000-node predecessor tree
families are disposed iteratively across success, every geometry failure, and
the public-incomplete path. Synthetic topology covers multiple ordered
subjects, holes, nested islands, full cover, and InternalVoid participation in
the preceding O23 closing/protection path, while real KSR correctly has no
InternalVoid producer.

Repeated parent-bound KSR capture freezes checksum
`-117597382518472843802490205604634875775`, kind totals before/after
`[113, 6, 48, 1127, 0, 0]` / `[113, 6, 48, 1281, 575, 0]`, geometry totals
`[1294, 168, 46011]` / `[2023, 270, 73848]`, 460 total records, 161 active,
299 no-op, and 299 unchanged records. Structural digest tags delimit
object/slot, record/surface, contour/hole role and index, point counts, and end
markers. Record and exact event sequence digests are `-65994586923856785425316699963519338136` and
`-110138798119262824097709645699717637653`; ordered operation totals are
`[161, 161, 161]` and InternalVoid counts are `[0, 0]`.

Thirty-one focused tests, 149 O21-O24 regressions, and 5,827 workspace tests
with 2 skipped pass across direct, integration, provenance, metamorphic, transaction, cleanup,
lifecycle, and parent-regression coverage. Native, WASM, browser, and review
gates form the remaining release criteria. Thirteen planned compiling behavioral mutations plus the retained-scale review mutation are killed;
role-only intersection reversal is documented as an equivalent commutative
control rather than a false RED. Formatting, LOC, forbidden-pattern,
dependency, staging, rollback, byte-exact restoration, both independent review
paths, and exact pushed-SHA Tier-1 must all be green.

The next rewrite boundary is `PrintObject::prepare_infill` line 618 and
`PrintObject::discover_horizontal_shells` at `PrintObject.cpp:3955-4161`.
Bridge-over-infill, external-surface processing, fill generation, seams,
ordering, motion, G-code, and post-processing remain deferred. O19-O24 are
source-compatibility state only. Rollback restores O23 terminal consumption and
removes only O24 state/wiring/tests/docs, the mixed adapter, InternalVoid
vocabulary updates, and shared helper selection.

## Task 22O.25: Horizontal-shell extra-solid promotion

Task 22O.25 ports `PrintObject::discover_horizontal_shells` lines 3955-3972,
stopping before the EnsureAll gate at line 3974. It consumes each aligned
record's resolved raw `extra_solid_infills`, preserves the exact empty-string
short circuit, uses the shared typed parser and one-based matcher against the
zero-based planned array index, and retags every and only Internal
`fill_surfaces` entry to InternalSolid in place. The operation has no stored-ID
or sparse-density gate and never changes `slices`, metadata, geometry, order,
or allocation identity.

The shared option boundary accepts positive signed-`i32` components and uses
checked arithmetic for explicit ranges and one-based matching, yielding the
same stable invalid-pattern error on native and browser-WASM. Complete O24
alignment is validated before any visit; all record decisions are staged before
mutation, so later parse errors roll back the exact O24 graph. Success retains
the boxed predecessor, object records, all O19-O24 sidecars and nested
allocations; cleanup delegates iteratively through O24. Public slicing invokes
O25 once after O24 and remains incomplete.

Repeated parent-bound KSR capture freezes checksum
`58727684244877231975278290246623082466`, record digest
`160750122870413723145549886803558415603`, event digest
`95826544899519698779358289371798515623`, and unchanged surface digest
`-107673730348313625723619859456104452971`. All 460 records are unchanged;
kind totals remain `[113, 6, 48, 1281, 575, 0]`, geometry totals remain
`[2023, 270, 73848]`, and event totals are `[460, 0, 0, 0, 0]` with zero
commits and exactly one prepare/disposal. A normal typed archive mutation
promotes 1,281 Internal surfaces in 460 records and preserves the complete
allocation graph.

Forty-two focused O25/shared-option tests, 191 explicit O21-O25 regressions,
and 5,856 workspace tests with 2 skipped pass. Native/strict Clippy, four WASM
checks, optimized export audit, two 10-test Playwright runs, 14 killed compiling
mutations with byte-exact restoration, formatting, LOC, forbidden-pattern,
dependency, and rollback gates are green. Both six-dimensional review paths
are approved; exact pushed-SHA Tier-1 remains the exit gate.

The next bounded rewrite starts with the EnsureAll early return at
`PrintObject.cpp:3974-3976`; all later horizontal-shell geometry,
external-surface processing, fill generation, toolpaths, and G-code remain
deferred. O19-O25 remain temporary source-compatibility state. Rollback restores
O24 terminal consumption and removes only O25 state/wiring/tests/docs and its
crate-private raw-parser seam.

## Task 22O.26: Horizontal-shell propagation

Task 22O.26 ports the complete executable remainder of
`PrintObject::discover_horizontal_shells` at `PrintObject.cpp:3974-4150`. It
preserves the EnsureAll gate; Top, Bottom, BottomBridge source order; directional
count-or-strict-thickness windows; record-local typed options and flow values;
serial visibility of earlier neighbor rebuilds; flat path topology; exact
Clipper opening, safety, filtering, and repair order; and metadata-complete
collection reconstruction.

The milestone validates all inherited identity, alignment, slot-presence, and
printable-area-derived scale invariants before cloning. It stages work in a
whole-project clone and commits only records whose rebuild path executed after
all objects succeed. A failure leaves O25 unchanged, untouched records preserve
allocation identity, and a geometry-equal rebuild remains a dirty commit. The
successor owns the exact O25 graph and existing sidecars, adds no durable public
state, cleans up iteratively, and runs once in public project slicing before the
existing incomplete result.

The real KSR EnsureAll capture retains surface digest
`-107673730348313625723619859456104452971`, freezes event digest
`55157732452648897477979936233453742487`, and visits/skips all 460 aligned
records without source visits, geometry, or commits. The typed Moderate archive
freezes surface/event digests
`55371787254720044626064449746884984931` and
`71433667081695804905700384637078674080`, with raw event totals
`[460, 460, 0, 1380, 1010, 547, 143]` for fill clones, record visits,
EnsureAll skips, source kinds, neighbors, rebuilds, and dirty commits. All 547
rebuilds follow nonempty intersections and commit 143 distinct dirty records;
the capture also freezes 5,469 ordered geometry events.

Forty-five final O26-focused tests, six asymmetric-opening tests, one surface
template test, and the complete 5,908-test workspace pass with 2 skipped.
Thirty-three compiling behavioral mutations cover traversal, gates, windows, flow
provenance and casts, opening/filter/repair geometry, carried-solid state,
rebuild semantics, metadata, absent records, original-before-failure rollback,
public/cleanup bypass, and dirty-only commit. Controlled serial, all-site
transaction fingerprint, full sidecar/clean-geometry ownership,
geometry-equal production rebuild, resolved option/flow/scale, and non-slicing
rename witnesses are green. Formatting,
native all-target checks, strict all-feature Clippy, WASM/export/browser gates,
LOC, forbidden-pattern, dependency, staging, and rollback audits form the local
exit evidence. Final independent six-dimensional and default-model OpenCode
reviews approve O26; exact pushed-SHA Tier-1 remains required before release.

The next bounded rewrite is the pinned `prepare_infill` call to
`PrintObject::process_external_surfaces` after horizontal shells
(`PrintObject.cpp:624-642`). External-surface processing and its owning
`PrintObject`/`LayerRegion` source must be specified before implementation.
Infill combination, fill generation, toolpaths, seams, motion, G-code, and
post-processing remain deferred. O19-O26 are temporary source-compatibility
state. Rollback restores O25 terminal consumption and removes only O26
state/wiring/tests/docs, the path-opening adapter, and narrow surface-template
seam.

## Task 22O.27: Region-expansion direct wave propagation

Task 22O.27 ports the first bounded prerequisite of the active external-surface
stage: Clipper 6 `etClosedLine` and `etOpenRound`,
`Algorithm::RegionExpansion::RegionExpansionParameters::build`, and the direct
`propagate_waves(const WaveSeeds &, ...)` overload with its wavefront helpers.
The implementation uses only the ARD-0024 indexed Clipper kernel. It does not
port ClipperZ seed discovery, expansion merging, `LayerRegion` or `PrintObject`
external-surface orchestration, and it adds no lifecycle wiring.

Offset input preserves exact-equality versus strict positive
`ShortestEdgeLength` filtering, mixed ClosedPolygon/ClosedLine orientation,
strict near-zero handling, one-point joins, two-sided ClosedLine output, and
OpenRound side/cap order. Region parameters preserve the source `f32`/`f64`
expression order and explicit retained coordinate scale. Direct propagation
keeps contiguous `(boundary, src)` groups, raw endpoint closure, one persistent
configured offsetter, inflated-bbox contour-before-hole trimming, staged Round
offsets, Clipper-operation-order orientation, clockwise sign and reversal,
Positive/Positive clipping, and ordered paths and IDs. Errors remain direct
`ClipperError` values.

Twenty-one O27-focused tests cover six end-type cases, five parameter cases,
nine direct-propagation cases, and the new group-bbox constructor. Complete
ordered vectors come from out-of-tree pinned-source diagnostics; no oracle code
or payload is committed. Twenty-eight compiling behavioral mutations are
killed, including end selection, filtering, near-zero and orientation branches,
f32/f64 precision, scale substitution, reassociation, step counts, persistent
offset configuration, fill rule, IDs, hidden sort/regroup, OpenRound versus
ClosedPolygon, staged versus one-shot expansion, bbox trimming, clockwise
sign/reversal, and eager/error-reordered access. The final
offset/RegionExpansion/bbox regression runs 77 tests.
The current full workspace runs 5,929 tests with 2 skipped; native all-target
check, strict workspace Clippy, four wasm32 checks, optimized WASM/export
audit, two 11-test Playwright runs, formatting, LOC, dependency, forbidden
pattern, lifecycle, and rollback audits are green. The independent
six-dimensional reviewer approved after one repair/re-review loop, and the
separate default-model OpenCode reviewer also returned `VERDICT: APPROVE`.
Exact pushed-SHA Tier-1 remains the release gate.

Public slicing still consumes O26 and returns `ProjectSlicingIncomplete`; O27
is a crate-private geometry prerequisite only. O28 takes the next
ClipperZ-backed `RegionExpansion.cpp::wave_seeds` boundary: expanded/opened Z
paths, Z-fill intersection collection, split reconciliation, source/boundary ID
recovery, and the closed-seed AABB fallback. Source-taking propagation,
expansion merge helpers, external-surface processing, fill generation,
toolpaths, seams, motion, G-code, and post-processing remain deferred.
Mechanical rollback removes only O27 RegionExpansion/end-type code, tests, and
documentation while retaining the exact O26 lifecycle.

## Task 22O.28: ClipperZ wave-seed discovery

Task 22O.28 ports pinned `Algorithm::wave_seeds` from
`Algorithm/RegionExpansion.cpp:88-391`, `ClipperZUtils.hpp:14-139`, the reached
bundled ClipperZ sites, the four-direction `Polyline.hpp` merge, and the local
AABB build/traversal behavior. It extends only the existing ARD-0024 indexed
Clipper kernel with geometry-private per-vertex Z provenance. It introduces no
second geometry engine, dependency, public API, option, persisted state, or
project lifecycle stage.

Ordinary kernel equality and all geometry predicates remain XY-only, existing
2-D adapters assign zero Z, and existing ordered 2-D outputs discard metadata.
The Z path preserves pinned endpoint-priority `SetZ`, direction-sensitive
horizontal and strictly-simple fills, output/fixup/join survivor metadata, an
execution-local sorted and deduplicated intersection table, and optional
PolyTree Z sidecars. Seed discovery preserves contour/hole expansion signs,
ExPolygon-level IDs, NonZero intersection and point order, repeated endpoints,
four-direction split merging, swap-pop/reprocessing, four recovery branches,
and the documented release-only drops.

The closed-seed fallback builds lazily from contour-only inflated boxes. It
retains both coordinate-scale epsilons, literal `min + max / 2` centroid
arithmetic, longest-axis X ties, source QuickSelect order, left-first first-hit
traversal, and exact outer/hole containment. `sorted=true` applies the accepted
ARD-0024 MSVC STL 14.44 control flow to an index permutation with comparator
`(boundary, src)` only; no stable sort, host sort, geometry tie-break, or index
tie-break enters production.

Twenty-five focused Z tests, 39 focused wave-seed tests, 211 full Clipper tests,
and 53 full RegionExpansion tests compare complete ordered metadata, IDs, and
paths. Focused release filters freeze first-two collector behavior, invalid
front/valid back continuation, and containment drops. The unchanged O27 handoff
and inactive public lifecycle are tested. All 23 planned mutations plus a
strict shortest-edge mutation are killed, restored, and rerun GREEN. Pinned
C++ debug/`NDEBUG` captures cover inside, crossing, hole, split, multiple-ID,
overlapping-fallback, and release-only shared-vertex behavior; original
compiling-RED chronology is unavailable and is disclosed rather than
reconstructed.

Both final-state workspace commands pass 5,994 tests with 2 skipped. Native
all-target check, strict all-feature Clippy, formatting, four wasm32 checks,
two optimized WASM builds, export/syntax audits, and two 11-test Playwright
runs pass. Static audits confirm the exact file allowlist, all Rust files under
400 LOC, every new shard at most 300 LOC, empty staging, no manifest/lockfile,
dependency, lifecycle, adapter, or forbidden-pattern change. A disposable
worktree reproduced the exact O28 state, rolled it mechanically back to clean
predecessor `f361bb73b558b4e50bfa4fa712afcd63df44ba9f`, and proved the primary
diff, file list, staging state, and digests unchanged. Final documented-state
independent six-dimensional and default-model OpenCode reviews both return
`VERDICT: APPROVE`. Implementation commit `7eb0d27` and documentation commit
`be33437` are pushed; exact-SHA Tier-1 run `31156094839` passed Linux, macOS,
Windows, formatting, WASM, export, and both browser executions at
`be334375be871eb12ca98c98d889b65a92d13a37`.

Public slicing still consumes O26 and returns `ProjectSlicingIncomplete`; O28
changes no KSR checkpoint and makes no G-code parity claim. The next bounded
rewrite is the source-taking
`propagate_waves(const ExPolygons &, const ExPolygons &,
const RegionExpansionParameters &)` overload and its scalar overload at
`Algorithm/RegionExpansion.cpp:463-478`, which compose O28 seed discovery with
unchanged O27 propagation. `propagate_waves_ex`, expansion merge helpers,
external-surface orchestration, fill generation, toolpaths, seams, motion,
G-code, and post-processing remain deferred. Mechanical rollback removes only
O28 Z/seed/AABB code, tests, and documentation while retaining all O27 code and
the exact O26 lifecycle.

## Task 22O.29: source-taking RegionExpansion propagation

Task 22O.29 locally ports pinned
`Algorithm/RegionExpansion.cpp:463-466,468-477` and
`Algorithm/RegionExpansion.hpp:74-83` into two crate-private destinations:
`geometry::region_expansion::propagate_waves_from_sources` and
`propagate_waves_from_sources_with_steps`. The parameter entry invokes O28
`wave_seeds` with literal `sorted=true` and sends the complete ordered result to
unchanged O27 propagation. The scalar entry builds parameters exactly once and
delegates exactly once with the same retained explicit coordinate scale.

The five-test composition shard freezes complete compact and
sorted-versus-unsorted results plus complete 16-point Normal and 128-point
LargeBed scalar vectors. Focused composition passes 5/5 and the full
RegionExpansion regression passes 58/58. Ten runtime mutations are killed and
restored, one signature-shape mutation is rejected at compile time, and the
separate structural audit truthfully verifies one scalar build followed by one
delegation without claiming behaviorally equivalent inlining as a mutation.
The frozen six-argument scalar signature has one function-scoped, reasoned
`#[expect(clippy::too_many_arguments)]` because the workspace threshold is five;
no lint `allow` was added. Final LOC are 172, 55, 150, 5, and 264 for
`propagate.rs`, `region_expansion.rs`, `geometry.rs`, the RegionExpansion test
root, and `composition.rs`, respectively.

The chronological RED log is the earlier eight-test
`/tmp/task22o29-red-focused-all.txt`: seven empty-stub assertions failed and
`scalar_scale_outputs_differ` passed while both wrapper stubs returned empty
because it compared explicit pipelines. The final shard was later consolidated
and strengthened to five tests, including valid discovery before propagation
failure. No chronological RED exists for that exact final list. Mutation
kills/restored GREEN are post-hoc recurrence evidence, not original RED.

O29 changes no public API, lifecycle, KSR checkpoint, G-code byte, option,
persisted state, or ARD; ARD-0024 is unchanged and public slicing still uses
O26 and returns `ProjectSlicingIncomplete`. Mechanical rollback removes only
the O29 wrappers, private reexports/signature assertions, composition
shard/registration, and O29 documentation while retaining O27, O28, and the O26
lifecycle. The restored final local state passes composition 5/5,
RegionExpansion 58/58, O26 lifecycle 3/3, workspace 5,999/5,999 with 2 skipped,
native all-target check, warning-denying Clippy, rustfmt, four WASM checks, two
optimized WASM builds, export/syntax audits, two 11/11 Playwright runs, static
audits, and disposable rollback. Final documented-state independent
six-dimensional and default-model OpenCode rereviews both return literal
`VERDICT: APPROVE`. O29 was released as implementation commit `55c2c23` and
documentation commit `118f6a7`; exact-SHA Tier-1 run `31168584784` passed all
format, WASM/browser, Linux, Windows, and macOS jobs at
`118f6a72b33926efe41ced1c931f9a51b26b2945`.

The next bounded rewrite boundary is the direct supplied-seed
`propagate_waves_ex` at `Algorithm/RegionExpansion.cpp:480-503`. Its source
scalar overload, `expand_expolygons` and expansion merge helpers,
external-surface orchestration, fill generation, toolpaths, seams, motion,
G-code, and post-processing remain deferred.

## Task 22O.30: direct RegionExpansionEx wave output

Task 22O.30 locally ports pinned `Algorithm/RegionExpansion.cpp:480-503` and
`Algorithm/RegionExpansion.hpp:85-92`. It adds crate-private
`RegionExpansionEx` and direct supplied-seed `propagate_waves_ex`. The entry
completes unchanged O27 propagation before its debug-only boundary-first,
source-second sorted assertion, groups only adjacent expanded records by both
IDs, directly wraps singleton contours, and uses the existing ARD-0024
`union_ex` with NonZero fill for multi-polygon groups.

The six-test shard freezes complete singleton, natural one-seed hole,
multi-island, adjacent source/boundary transition, comparator-conflict,
release-unsorted, zero-output, and error-before-assertion behavior. Its real
compiling RED had five failures and one valid zero-output pass. Final debug and
release focused runs pass 6/6, RegionExpansion 64/64, PolyTree 6/6, O26
lifecycle 3/3, and workspace 6,005/6,005 with 2 skipped. Sixteen runtime
mutations are killed, one signature mutation is compiler-rejected, and two
semantic survivors are disclosed: valid O27 output makes Positive equivalent
to NonZero in the probed hole, while repeated union preserves three probed
singleton vectors. Native check, warning-denying Clippy, rustfmt, four WASM
checks, two optimized builds, export/syntax audits, and two 11/11 Playwright
runs are green. Final allowed-Rust LOC are 74, 218, 62, 156, 6, and 263.

O30 changes no public API, Option, lifecycle, checkpoint, persisted state, KSR
golden expectation, G-code byte, or ARD; public slicing remains on O26 and
returns `ProjectSlicingIncomplete`. Exact static and disposable rollback gates
are green; final independent six-dimensional and default-model OpenCode
implementation reviews both return literal `VERDICT: APPROVE`. O30 was
released as implementation commit `0a19939` and documentation commit `6ccb145`;
exact-SHA Tier-1 run `31184069746` passed all five jobs at
`6ccb145dbb1867e5724538fb071795a7fd4179f0`.

## Task 22O.31: source-taking RegionExpansionEx composition

Task 22O.31 locally ports pinned `Algorithm/RegionExpansion.cpp:506-520` and
`RegionExpansion.hpp:94-100`. Its crate-private scalar entry builds
`RegionExpansionParameters` once, discovers sorted O28 seeds once with the
built tiny expansion and the same explicit `CoordinateScale`, then delegates
once to unchanged O30. Discovery and propagation/union errors escape directly;
there is no empty shortcut, rescaling, regrouping, lifecycle wiring, or public
export.

The five-test shard freezes builder precedence, empty behavior, a natural hole,
complete sorted source/boundary IDs and topology, Normal/LargeBed complete
vectors, and discovery-before-propagation error order against the explicit
build/discover/O30 pipeline. Chronological RED failed 5/5 against the compiling
empty stub; focused debug/release pass 5/5 and RegionExpansion passes 69/69.
Nine runtime mutations are killed, one signature mutation is compiler-rejected,
and the discovery-call scale substitution is truthfully recorded as a focused
witness survivor while exact same-scale forwarding is fixed structurally.

O31 changes no public API, Option, lifecycle, checkpoint, persisted state, KSR
golden expectation, G-code byte, or ARD. Public slicing remains on O26 and
returns `ProjectSlicingIncomplete`. Workspace Nextest passes 6,010 with 2
skipped; native/WASM/static/rollback gates and both final reviews are green.
The local host lacked Chromium runtime libraries, while the exact CI WASM job
installed them and passed the browser suite twice. O31 was released as commits
`7113f7c`/`1f89dd3`; exact-SHA Tier-1 run `31196271880` passed all five jobs at
`1f89dd34c9226a96b92ddc1711c317ff6ce7b7b0`.

## Task 22O.32: group expanded polygons by source ExPolygon

Task 22O.32 locally ports pinned `Algorithm/RegionExpansion.cpp:522-534` and
`RegionExpansion.hpp:102-108`. Its crate-private `expand_expolygons` allocates
one polygon-vector slot per source, calls the O29 scalar source propagation
once with unchanged expansion, step, maximum steps, and explicit scale, then
moves each complete polygon into `src_id`'s slot. It discards boundary IDs but
preserves per-source relative order and all empty source slots; it performs no
union, sorting, compaction, rescaling, validation, or lifecycle wiring.

The five-test shard has a compiling chronological RED of 0/5 and focused
debug/release GREEN of 5/5. It freezes empty/precondition behavior, one source
with multiple raw polygons, boundary-first flat records redistributed into
source-index slots with leading/interior/trailing empties, complete Normal and
LargeBed vectors, and direct discovery/propagation errors. RegionExpansion is
74/74. Thirteen runtime mutations are killed, two type-shape mutations are
compiler-rejected, and allocation/ownership equivalences are recorded as
structural survivors. Initial independent and default-model OpenCode
implementation reviews both approve. After tuple-packing one test helper for
the five-argument Clippy threshold, workspace Nextest passes 6,015/6,015 with 2
skipped; native, WASM build/export/syntax, static, and exact-O31 rollback gates
are green. The local host lacks Chromium's `libglib-2.0.so.0`, so both browser
runs remain an exact-SHA CI requirement. Final independent six-dimensional
and default-model OpenCode reviews both approve. Implementation/documentation
commits `2e7168f`/`699f02b` were pushed; exact-SHA Tier-1 run `31213611275`
passed all five jobs, including both browser runs, at
`699f02b2bbc3d797f53edf5f8c65dd2614830ecb`. O32 is released.

## Task 22O.33: merge region expansions into source ExPolygons

Task 22O.33 locally ports pinned `Algorithm/RegionExpansion.cpp:536-587` and
`RegionExpansion.hpp:110-111`. Its crate-private helper uses O28's fixed-MSVC
index permutation to group movable expansion records by source ID, preserves
untouched source ExPolygons, accumulates expansion polygons followed by the
source contour and holes, applies Orca's fixed unscaled 10/Miter/3 safety-offset
union, and retains the source-connected component through the O28 AABB sampler
with explicit `CoordinateScale`. It performs no lifecycle or public slicing
integration.

The compiling chronological RED ran 11 tests with ten meaningful failures and
one behavioral-equivalence pass. Initial review found missing zero-result test
evidence and a moved-buffer defect in the temporary C++ oracle. The repaired
suite directly reaches the zero branch, kills its panic mutation, and the
corrected C++ harness now gives byte-identical debug/`NDEBUG` vectors. Focused
debug/release pass 13/13 and RegionExpansion passes 87/87. Thirteen runtime
mutations are killed, one signature mutation is compiler-rejected, and four
structural/equivalent survivors are disclosed. Repaired independent and
default-model OpenCode initial implementation reviews approve. A test-only
function-pointer alias repair resolved the first full run's Clippy finding;
the complete exact candidate was rerun. Focused debug/release 13/13, AABB 8/8,
O32 5/5, RegionExpansion 87/87, PolyTree 6/6, offset 58/58, lifecycle 3/3,
workspace 6,028/6,028 with 2 skipped, check, warning-denying Clippy, rustfmt,
four WASM checks, two optimized builds, export, and JavaScript syntax gates are
green. Both local Playwright attempts stop before test code because Chromium
cannot load `libglib-2.0.so.0`; exact-SHA CI must pass both runs. Disposable
exact-O32 rollback proves candidate/primary byte identity and passes
RegionExpansion 74/74, PolyTree 6/6, and lifecycle 3/3. Final review found
exact oracle-input and stale status defects; both were repaired, the entire
suite and rollback were refreshed, and final independent/default-model OpenCode
rereviews both approve. Implementation/documentation commits
`b9e65fd`/`0f6f801` were pushed; exact-SHA Tier-1 run `31228800274` passed all
five jobs, including both browser runs, at
`0f6f80130d28c0cc629e8561e46d187b137a8206`. O33 is released.

O33 adds no Option, public API, lifecycle, checkpoint, persistence, adapter,
KSR golden expectation, or G-code byte. Public slicing still consumes O26 and
returns `ProjectSlicingIncomplete`.

## Task 22O.34: compose source wave expansion and ExPolygon merge

Task 22O.34 locally ports pinned `Algorithm/RegionExpansion.hpp:113` and
`Algorithm/RegionExpansion.cpp:589-594`. Its crate-private
`expand_merge_expolygons` borrows the complete source vector for exactly one O29
source-propagation call, propagates discovery/propagation errors through `?`,
then moves the original sources and complete ordered records into exactly one
O33 merge call with the same explicit `CoordinateScale`. It adds no builder,
seed discovery, sorting, cloning, rescaling, shortcut, validation, fallback,
error mapping, public export, or lifecycle wiring.

The historical compiling stub run reported 0/5, with four genuine failures and
one deleted witness that failed in direct O29 setup before reaching O34. The
replacement successful non-empty O29-to-O33 handoff is classified only as
post-body recurrence/GREEN evidence. Focused debug/release pass 5/5 and
RegionExpansion passes 92/92. Six runtime mutations are killed, one signature
mutation is compiler-rejected, and two equivalent scale substitutions plus an
unreachable-through-valid-O29 O33-error swallowing mutation are disclosed as
truthful survivors. Post-mutation restoration and the initial static audit pass.
The default-model OpenCode initial review approved, while independent review
required physical placement after O33 and non-vacuous multiple-source,
multiple-hole ordering/ownership evidence. Both repairs are present and
verified. The repaired exact candidate passes focused debug/release 5/5, O29
5/5, O33 13/13, RegionExpansion 92/92, PolyTree 6/6, offset 58/58, lifecycle
3/3, workspace 6,033/6,033 with 2 skipped, check, warning-denying Clippy,
rustfmt, four WASM checks, two optimized builds, export, and JavaScript syntax.
Both local Playwright attempts fail before test code only because Chromium
cannot load `libglib-2.0.so.0`; exact-SHA CI retains both runs. Disposable
exact-O33 rollback proves candidate/primary byte identity and passes
RegionExpansion 87/87, PolyTree 6/6, offset 58/58, and lifecycle 3/3. The
repaired candidate's independent six-dimensional and default-model OpenCode
rereviews both approve. Implementation/documentation commits
`f499058`/`25460c2` were pushed; exact-SHA Tier-1 run `31259140846` passed all
five jobs, including both browser runs, at
`25460c2abfc5bf94104f41b05df5af2dfac419ee`. O34 is released.

O34 adds no Option, public API, lifecycle, checkpoint, persistence, adapter,
KSR golden expectation, or G-code byte. Public slicing still consumes O26 and
returns `ProjectSlicingIncomplete`.

## Task 22O.35: expand and merge one external-surface kind

Task 22O.35 locally ports pinned `LayerRegion.cpp:147-163,166-171,439-484` and
`ClipperUtils.hpp:19,27,407-408`. Its inactive crate-private helper moves one
selected `RegionSurfaceKind` into ordered sources, propagates each ordered zone
through O29, rebases boundary IDs with 32-bit wrapping behavior, merges once
through O33, applies explicit Miter/3 morphological closing, trims only zones
with successful expansions, and materializes default surface metadata with the
supplied bridge angle. Matching records retain metadata with empty moved
geometry; nonmatching records and point buffers remain untouched.

The authoritative stub RED compiled 13 tests, with two truthful equivalent
passes and 11 intended failures after two test-only pre-RED repairs. The frozen
candidate passes focused debug/release 13/13, offset 62/62, O29 5/5, O33 13/13,
O34 5/5, and RegionExpansion 92/92. Fourteen runtime mutations are killed, one
signature mutation is compiler-rejected, and four behaviorally equivalent
miter/rebasing/scale mutations are disclosed as survivors. Exact byte
restoration, warning-denying focused Clippy, rustfmt, LOC/visibility/forbidden
audits, and both initial independent/default-model OpenCode reviews pass.

The complete documented implementation candidate passes focused debug/release
13/13, offset 62/62, O29 5/5, O33 13/13, O34 5/5, RegionExpansion 92/92,
PolyTree 6/6, lifecycle 3/3, workspace Nextest 6,046/6,046 with 2 skipped, all
native/static/WASM/build/export gates, and exact-O34 rollback with
5/92/6/58/3 baseline suites. Both local Playwright attempts fail before test
code only because Chromium cannot load `libglib-2.0.so.0`; they are not passes
and exact-SHA CI retains both runs. Final independent six-dimensional and
default-model OpenCode implementation reviews both approve with no required
changes.

Implementation/documentation commits `984bc01`/`c6f23ce` were pushed;
exact-SHA Tier-1 run `31269521736` passed all five jobs and both browser
executions at `c6f23ce1a9350ca76241d007f804f3fcfa22c352`. O35 is released but
remains inactive. It adds no Option, public API, lifecycle, adapter, golden
expectation, or G-code byte. Its partial mutations are safe only on a future
staged owned working copy. Public slicing still consumes O26 and returns
`ProjectSlicingIncomplete`.

## Task 22O.36: compose bridge anchors and expansions across ordered zones

Task 22O.36 locally ports pinned `LayerRegion.cpp:353-356,358-393`: the
translation-unit-local `ExpansionResult` and `expand_expolygons` helper,
distinct from released O32. The inactive crate-private helper visits O35 zones
in source order, calls O28 sorted discovery then O30 ExPolygon propagation,
rebases anchor/expansion boundary IDs by every prior zone's full ExPolygon
count with 32-bit wrapping behavior, commits `expanded_into`, and move-appends
the two complete streams.

The compiling empty-stub RED failed 0/6 at the O36 seam. The frozen candidate
passes focused debug/release 6/6, O35 13/13, O28 39/39, O30 6/6, O31 5/5,
RegionExpansion 92/92, external surfaces 15/15, PolyTree 6/6, offset 62/62,
and lifecycle 3/3. A disposable exact pinned-Orca CLI E2E sliced the KSR 3MF
to a nonempty G-code, and a linked original-helper oracle emitted byte-identical
Debug/NDEBUG vectors matching the complete Rust anchor/expansion/hole/ID/flag
literals. Thirteen runtime mutations are killed, two API/result mutations are
compiler-rejected, and the `sorted=false`/hard-coded-scale equivalent survivors
are disclosed structurally. Exact restoration, rustfmt, LOC/private visibility,
and both initial independent/default-model reviews pass. Final review required
one test-only repair that now explicitly proves nonempty O28 seeds before both
O30 propagation errors; existing flag/no-partial assertions and production are
unchanged, and the shard remains bounded at 295 LOC.

The repaired complete candidate passes O36 debug/release 6/6, all focused
regressions 13/39/6/5/92/15/6/62/3, workspace 6,052 passed with 2 skipped,
check, warning-denying Clippy, rustfmt, four WASM checks, two optimized builds,
export/JavaScript audits, and exact-O35 rollback 13/92/6/62/3. Both local
Playwright runs fail before test code only because `libglib-2.0.so.0` is absent;
they are not passes. Both final implementation rereviews approve.

Implementation/documentation commits `b546e6f`/`3e927ed` were pushed;
exact-SHA Tier-1 run `31280579891` passed all five jobs and both browser
executions at `3e927ed569d3db8d6f5c08b7843fb049fcc86412`. O36 is released but
remains inactive. It adds no Option, public API, lifecycle, adapter, golden
expectation, or G-code byte. Public slicing still consumes O26 and returns
`ProjectSlicingIncomplete`. The next bounded slice is `Bridge`, `group_id`, and
`get_grouped_bridges` at `LayerRegion.cpp:174-260`; bridge direction/merge
orchestration, fill, toolpath, seam, motion, G-code, and post-processing remain
deferred.

## Task 22O.37: group bridge regions by overlapping expansions

Task 22O.37 locally ports pinned `LayerRegion.cpp:174-260`: source-shaped
`Bridge`, parent traversal `group_id`, and `get_grouped_bridges`. The inactive
crate-private helper consumes source ExPolygons into ordered bridge records,
keeps the expansion-end index sentinel and `None` angle, scans only adjacent
boundary-ID windows, caches contour-only bounding boxes, and processes ordered
pairs through equal-source, exact inclusive bbox, and one fallible NonZero
contour intersection before attaching the higher root to the lower. It returns
the raw parent forest without normalization; holes remain irrelevant.

The compiling initialization-only stub ran ten tests with six body-dependent
failures and four disclosed stub-equivalent passes. A disposable exact pinned-
Orca CLI E2E produced a nonempty G-code and removed it without content
inspection; the linked original helper passed 45 assertions and emitted byte-
identical Debug/`NDEBUG` vectors matching complete Rust source/order/group/
sentinel/angle literals. The frozen candidate passes O37 debug/release 10/10,
O36/O35/O28/O30/RegionExpansion/external-surface/PolyTree/boolean-path/offset/
O26 regressions 6/13/39/6/92/25/15/11/62/3, warning-denying Clippy, and
rustfmt. After a review-required private pair-helper extraction, the exact-byte
mutation campaign again kills thirteen runtime mutations, compiler-rejects two,
and truthfully reports the strict/inclusive bbox comparison substitution as an
equivalent survivor. All hashes restore; body/test shards are 96/289 LOC; both
repaired initial implementation reviews approve.

Implementation/documentation commits `a0caa5a`/`4d83d15` were pushed;
exact-SHA Tier-1 run `31291016394` passed all five jobs and both browser
executions at `4d83d15832c7905d7ea9727d14c07c5a75eb7312`. O37 is released but
remains crate-private and inactive. It changes no Option, lifecycle, adapter,
golden expectation, or G-code byte. Public slicing still consumes O26 and
returns `ProjectSlicingIncomplete`.

## Task 22O.38: direct bridge-direction selection

Task 22O.38 locally ports the independent
`detect_bridging_direction(const Lines &, const Polygons &)` helper at pinned
`BridgeDetector.hpp:75-119`, its `PrincipalComponents2D.hpp:12-20` /
`PrincipalComponents2D.cpp:8-138` dependency, `Line.hpp:180`, and the cited
Eigen 5.0.1 scalar-normalization behavior. The inactive crate-private API takes
explicit `CoordinateScale`, preserves signed ordered `f32` PCA accumulation and
mixed-width eigensystem casts, and preserves the nonempty-edge `(dy,-dx)`,
normalization, quantization, cost, strict-minimum, and `(y,-x)` rotation order.
A private platform-neutral adapter reproduces the audited MSVC STL 14.44
FNV/unique-emplace/bucket/rehash iteration target without host hash order or a
platform branch.

The compiling return-only stub ran 18 tests with 17 body-dependent failures and
one shape-equivalent pass. A fresh exact pinned-Orca CLI E2E produced a nonzero
6,338,289-byte G-code and deleted it without content inspection. The standalone
pinned helper emits byte-identical Debug/`NDEBUG` complete vectors, while an
independent audited MSVC model freezes duplicate, occupied-bucket, 8-to-64
rehash, and distinguishing rehash-group order. After one test-only named-alias
Clippy repair, the campaign kills fourteen runtime mutations, compiler-rejects
one signature mutation, and truthfully reports four bounded equivalent
survivors. Exact production hashes restore; focused debug/release pass 18/18,
O37/O36 pass 10/6, complete geometry passes 442/442, warning-denying Clippy and
rustfmt pass, and both repaired initial implementation rereviews approve.

The exact documented candidate passes focused debug/release, complete geometry
and bounded predecessor regressions, workspace Nextest 6,080/6,080 with two
skipped, all-target check, warning-denying Clippy, rustfmt, four WASM checks,
two optimized builds, bindgen/export/JavaScript audit, static audit, and exact-
O37 rollback. Both local Playwright attempts failed before test code because
Chromium could not load `libglib-2.0.so.0`; neither was treated as a pass.
Implementation/documentation commits `04920e0`/`2d6154d` were pushed;
exact-SHA Tier-1 run `31303115603` passed all five jobs and both browser
executions at `2d6154d401c3c954bed69de6ba631a53af05f1a3`. O38 is released but
remains crate-private and inactive. It adds no Option, lifecycle, adapter,
fixture branch, golden expectation, or G-code byte. Public slicing still
consumes O26 and returns `ProjectSlicingIncomplete`.

## Task 22O.39: detect grouped bridge directions

Task 22O.39 locally implements the exact pinned source slice
`detect_bridge_directions` at `LayerRegion.cpp:262-308`, with direct conversion
and geometry dependencies at `ExPolygon.hpp:228-242,300-307`,
`Polyline.hpp:180-193`, `libslic3r.h:52,93-96`, and
`ClipperUtils.hpp:19,23-27,373-376,457` / `ClipperUtils.cpp:837-845,908-909`.
The inactive crate-private entry composes released O36/O37/O38 records and
helpers while preserving the one-way supplied-order anchor cursor,
source-width ID casts, contour/hole order, double-to-float scaled epsilon,
Miter-3 offset, non-recombining open-path difference, unchanged-scale O38 call,
sequential error commits, and `PI + atan2(y,x)` angle assignment.

The repaired fresh implementation cycle records 11 body-dependent RED failures
and two stub-equivalent passes, followed by 14/14 debug/release GREEN. The
byte-identical original-Orca Debug/`NDEBUG` helper passes 12 assertions and
covers multiple bridges plus an unmatched boundary; reviewed literals,
contour/hole pointer identity, M01-M28 mutation coverage, exact restoration,
and both implementation rereviews pass. Complete exact-final-byte native,
WASM, static, and exact-O38 rollback gates pass, including workspace Nextest
6,094/6,094 with two skipped and warning-denying Clippy. Both local Playwright
attempts failed before test execution on missing `libglib-2.0.so.0`; neither
was treated as a pass. Implementation/documentation commits
`2038e93491de89e33f12ecb5379132a013bfc996` /
`c84119ee6871a176ec94117bc16f7e402c9caf96` were pushed, and exact-SHA Tier-1
run `31317150231` passed all five jobs and both browser executions at the
documentation SHA. O39 is released but inactive and does not complete KSR
slicing.

## Task 22O.40: merge bridge groups

Task 22O.40 locally implements the exact pinned `merge_bridges` slice at
`LayerRegion.cpp:310-351`, with the directly used `Bridge`/`group_id`,
`RegionExpansionEx`, ExPolygon conversion, `BottomBridge` surface, and flat
polygon closing definitions cited in its spec. The crate-private function
associates sorted expansion runs with their source bridges, resolves groups,
moves bridge contours and holes into per-root polygon soups, closes each group
independently with Miter-3 positive/negative offsets, and emits default bottom-
bridge surfaces with the root angle. It uses an owned Rust vector plus temporary
index ranges instead of retaining Orca iterator state.

The compiling stub RED failed the first observable surface assertion. Eight
focused behavior tests now pass, including an independently compiled pinned-
Orca Debug/`NDEBUG` flat-closing vector whose outputs are byte-identical and a
three-bridge oracle that distinguishes per-group from global closing. The
O35-O40/effective-config focused regression set passes 69/69; workspace
Nextest passes 6,101/6,101 with two skipped; warning-denying all-target/all-
feature Clippy, rustfmt, native and wasm32 checks, diff/LOC/include audits pass.
The ignored normalized KSR golden remains an expected progress RED at the CLI
contract because `--options` is still required. O40 is crate-private, inactive,
unreleased, and independently approved after its initial rustfmt, coverage,
and citation findings were repaired and every gate rerun. It adds no Option,
adapter, lifecycle, fixture branch, or G-code byte. The next bounded source
slice is
`expand_bridges_detect_orientations` at `LayerRegion.cpp:395-437`; active
external-surface orchestration and all downstream fill/toolpath/motion/G-code
work remain deferred.

## Task 22O.41: expand bridges and detect orientations

Task 22O.41 locally ports pinned
`LayerRegion.cpp:395-437::expand_bridges_detect_orientations`. The inactive
crate-private helper extracts bottom-bridge geometry, composes O36/O37/O39/O40,
preserves the upstream anchor and expansion sort orders, emits merged bridge
surfaces, and trims only zones marked `expanded_into`. The first nonempty
tracer recorded a compiling RED against an empty stub; six focused behavior
tests are now GREEN for the complete sorted composition, no-source no-op, mixed
surface preservation, default metadata, selective zone clipping, and first and
later expansion-error mutation order.

O41 adds no Option, public API, lifecycle activation, adapter, fixture branch,
or G-code byte. Public slicing still returns `ProjectSlicingIncomplete`;
`LayerRegion::process_external_surfaces` at `LayerRegion.cpp:486-623` and all
downstream work remain deferred. The first independent review requested direct
sorting and error-ledger coverage; those tests were added and the affected
gates rerun. Workspace Nextest passes 6,107/6,107 with two skipped; workspace
warning-denying Clippy, rustfmt, diff, LOC, and include audits pass. The same
six-dimensional reviewer then approved the repaired bounded slice with no
remaining findings.

## Task 22O.42: active external-surface processing

Task 22O.42 ports the complete pinned active body of
`LayerRegion.cpp:486-623::LayerRegion::process_external_surfaces` and the
behavior-relevant caller order at `PrintObject.cpp:610-641`. The new deep
crate-private successor consumes O26 records, reads composed region options,
scaled Classic prelude values, per-record model rotation, global spiral mode,
and coordinate scale, then rebuilds fill surfaces in the upstream
solid/sparse/bridge/bottom/top order. Lower-layer cache arguments unused by the
pinned body, fill generation, toolpaths, motion, G-code, and the project-only
CLI contract remain deferred.

The real 460-record KSR lifecycle and 15 direct arithmetic/angle/area/error
cases pass. O42 is 19/19, all external-surface coverage is 72/72, and the
O24-O26/O40-O42 regression band is 119/119. Workspace Nextest passes
6,126/6,126 with 27 slow and two skipped; warning-denying workspace Clippy,
rustfmt, WASM checks, diff, LOC, and include audits pass. Review repairs add
observable public activation/disposal, all-record traversal markers, real-3MF
option-driven output, source-shaped allocation behavior, exhaustive geometry
error mapping, and complete LargeBed arithmetic coverage. The normalized
golden remains the expected RED at the unchanged CLI `--options` requirement.
The final independent standards, specification, and upstream-parity re-review
returned unconditional approval with no findings. O42 is approved; the
broader KSR pipeline remains incomplete.

## Task 22O.43: gather internal-bridge candidates

Task 22O.43 ports the first coherent section of pinned
`PrintObject.cpp:2467-2591::PrintObject::bridge_over_infill`. The intervening
`clip_fill_surfaces` call is recorded as the pinned program's disabled identity
operation, with no invented Option or shallow lifecycle wrapper. The new deep
crate-private successor scans every effective object region for Lightning,
builds lower unsupported and solid masks, applies the exact policy-dependent
Clipper morphology and signed-area gates, and retains owned candidates by
stable object/layer/region/surface indices after O42.

The compiling stub gave the intended candidate RED. Review-driven REDs then
repaired object-wide Lightning provenance and aligned empty-lower handling.
Thirty-five focused tests cover all filter policies, exact density and scale
boundaries, source and hole topology, signed thresholds, morphology and cast
order, per-object provenance, lifecycle ownership, and first/later geometry
errors. The O24-O26/O40-O43 band passes 154/154; workspace Nextest passes
6,161/6,161 with 27 slow and two skipped. Warning-denying workspace Clippy,
rustfmt, WASM, diff, LOC, include, fixture, and source-pin audits pass.

O43 advances public slicing through candidate discovery but still returns
`ProjectSlicingIncomplete`. The ignored normalized KSR golden remains the
expected RED at the unchanged CLI `--options` requirement. Lightning and
CrossHatch anchor generation, candidate clustering/depth/angle selection,
surface commit, fill/toolpath/motion/G-code generation, and CLI activation
remain source-cited future slices; the broader KSR pipeline is incomplete.
Both final independent standards and specification/upstream reviews approve
O43 unconditionally with no remaining findings.

## Task 22O.44: connect infill to its boundary

Task 22O.44 ports the reusable pinned
`FillBase.cpp:323-398,420-842,995-1241,1243-1252,1263-1269,1432-1566,
1580-1588,1594-1614,1690-1818::Fill::connect_infill` dependency into the deep
crate-private `fill::connect` module. It preserves endpoint association,
boundary working-copy splits, occupied-boundary collision trimming, fixed MSVC
ordering at both active sort sites, root-preserving path merges, limited hooks,
remaining-endpoint handling, dual-scale arithmetic, and first checked error
behavior. Stable indices replace Orca pointer links without exposing the
working graph.

The compiling empty-output stub produced the intended exact-vector RED. Final
O44 coverage passes 41/41, its geometry/fixed-sort band 76/76, and the
O24-O26/O40-O44 band 194/194. Workspace Nextest passes 6,201/6,201 with
27 slow and two skipped. Workspace warning-denying Clippy, rustfmt, wasm32
core/adapter checks, diff, LOC, source-splitting, fixture-read, pinned-Orca
restoration, and independent source/specification and standards reviews pass.
Two disposable pinned-Orca harnesses freeze comparator-distinct Debug/Release
vectors; reversible mutations distinguish both sort adapters, threshold/tie,
scale, collision, and trace-order decisions before exact restoration.

O44 is a dependency-first slice and deliberately adds no `PreparedPost...`
stage, Option, parser default, public API, platform branch, or G-code byte.
Public slicing still consumes and disposes O43 and returns
`ProjectSlicingIncomplete`; the ignored normalized KSR golden remains the
expected RED at the unchanged CLI `--options` requirement. Complete
`FillCrossHatch`, `group_fills`, the lower-layer anchor map, Lightning
generation, bridge clustering/depth/direction/commit, fill/toolpath/motion/
G-code generation, and CLI activation remain source-cited future work.

## Task 22O.45: generate and connect CrossHatch fill surfaces

Task 22O.45 ports pinned `FillBase.cpp:105-119`, complete
`FillCrossHatch.cpp:28-232`, the KSR-active connector dispatch at
`FillBase.cpp:1820-1823,1827-1829`, and multiline-one return at
`FillBase.cpp:2712-2715`. The resulting dependency-first crate-private
`fill::cross_hatch` transaction owns the public half-spacing inset, exact
CrossHatch repeat/transform lattice, open clipping, strict remnant filtering,
O44 boundary connection, and per-component rotate-back. It adds only the
source-shaped open-polyline Intersection geometry dependency and does not call
the legacy Ares infill scaffold.

The public Orca and supplemental pattern-order harnesses freeze byte-identical
Debug/Release output; the arithmetic harness freezes pinned stdout. Their
output hashes are respectively
`17b755322c8d1e586e29145836f04ea728f4fdd846cce965430f8af1fea8691f`,
`bda674683e3990477401aeba3dcb3deec1a817f98d1fae049bc9b73744071f84`,
and `42434f1fad069e70c09e5538da1e173e2ce8919fe225c4b5fff8897608b10ea7`.
The supplemental public f32-repeat-ratio harness has byte-identical repeated
Debug/Release output
`e9b62afdc6fe0f7b03e4baf86d9c0e13e4692398f5ac89b6d8850bc82bd01aa2`.
The LargeBed inputs were corrected to match source `scaled<coord_t>`
truncation toward zero, after which all four LargeBed cases pass without a
production change. Eight arithmetic, six composition, and two f32-repeat-ratio
mutations are observably RED, and the four production files were restored
byte-for-byte.

Focused O45 coverage passes 34/34, the Clipper/open-intersection/O44 band
passes 305/305, and the O24-O26/O40-O45 band passes 228/228. Workspace Nextest
passes 6,235/6,235 with 30 slow and two skipped. Rustfmt, workspace
warning-denying Clippy, wasm32 core/adapter checks, and static audits pass. The
ignored normalized KSR golden remains the expected RED at the unchanged
missing `--options` boundary. Final independent source/specification and
standards reviews unconditionally approve this implemented and gate-verified
state.

O45 adds no prepared-project lifecycle successor, public option, API, or
G-code byte. Public slicing still consumes and disposes O43 and returns
`ProjectSlicingIncomplete`. O46 is scheduled to port the public Layer anchoring
result with its exact-corpus bridge-angle/pattern grouping plus nominal sparse
Flow/angle projection. Complete
generic `Fill.cpp::group_fills`, the transaction-local lower-layer anchor map,
Lightning generation, bridge clustering/depth/direction/commit, fill/toolpath/
motion/G-code generation, and CLI activation remain source-cited future work.

## Task 22O.46: generate sparse infill polylines for anchoring

Task 22O.46 ports pinned
`Layer.hpp:194-196::Layer::generate_sparse_infill_polylines_for_anchoring` and
`Fill/Fill.cpp:1377-1504`, keeping the reached grouping dependency private. The
crate-private borrowed Layer view performs exact four-kind projection, explicit
decreasing f32 bridge-angle/pattern-rank ordered coalescing, contour-before-hole
priority union/difference with raw-prior accumulation, post-priority Internal
selection, nominal non-first sparse Flow projection, and O45 CrossHatch
generation. It returns only ordered owned polylines and leaves input unchanged.

The strict global fixed-MSVC proof rebuilds 209 affected objects per mode from
pinned source and confirms byte-identical Debug/Release results: 103 O45 calls,
1,507 endpoint records with zero ties, 1,439 arc records with 2,700 ties across
30 calls and 82 classes, 186 paths, 5,941 points, and ordered digest
`917adc6ea02ad7cd7af79e45d90db6f4c1497bf5c8716d7f2f49b7de4b2070ef`.
The exact per-layer table has SHA-256 `bf531afc...`; clean source restoration is
recorded in `/tmp/task22o46-global-msvc-full-proof.zzdoO5`. The previous Linux,
hybrid, and captured-input results remain diagnostic or rejected.

O46 remains deliberately unwired. It adds no prepared-project map or successor,
public option/API, or G-code byte; public slicing remains terminal at O43.
Complete generic grouping, rotation-template language, multi-region,
InternalVoid/narrow-solid postpasses, other patterns, adaptive/Lightning state,
the `PrintObject.cpp:2725-2761` map, downstream bridge commit, extrusion,
motion, G-code, and CLI activation remain deferred without fallback. The final
focused 6/6, dependency 625/625, and workspace 6,241/6,241 Nextest runs pass;
rustfmt, warning-denying workspace Clippy, core/browser wasm32, diff/LOC/static
checks, and the unchanged missing-`--options` ignored golden probe pass at their
intended boundary. The 18-case serial reversible mutation audit is fully
killed with byte-exact restoration. Independent source/specification and
standards rereviews approve unconditionally.

## Task 22O.47: gather deep sparse bridge area

Task 22O.47 ports pinned
`PrintObject.cpp:2819-2846::gather_areas_w_depth` as an unwired crate-private
borrowed operation. It preserves the source's second `0.9f` target-height
factor, immediate-lower-layer exception, descending depth stop, per-layer
`Internal` density and unconditional `InternalVoid` classification, independent
union/closing, and final sparse-minus-non-sparse geometry.

Eight focused source-shaped tests cover f32 depth and epsilon arithmetic,
immediate-layer inclusion, density/kind projection, flat topology and
union/closing/subtraction at both scales, empty success, and range-error
nonmutation. The 18 candidate-layer KSR regression is repeatable and preserves
its complete borrowed layer/config inputs; it freezes 115 flat Polygons, 5,641
points, 91,464 serialized bytes, and ordered SHA-256
`f28db7dd3fc63155752ba5c33d4cd6338b2e311d83eb973c473d7f65268aa92a`.

Final O47 verification passes focused 9/9, dependency 590/590, and workspace
6,250/6,250 Nextest, rustfmt, warning-denying workspace Clippy, wasm32, diff,
LOC, and structural audits. The first independent six-axis review rejected a
hierarchical-result mismatch and weak arithmetic/closing discriminators; the
main thread repaired the operation to return source-faithful flat paths, added
exact f32 and both-scale closing tests. Two re-review rounds inspected the
repairs; the final independent six-axis verdict is unconditional approval.

O47 adds no prepared successor, public option/API, or G-code byte. Public
project slicing remains terminal at O43. Candidate clustering, sequential
lower-bridge subtraction, current-layer expansion, transaction-local O46 map
ownership and line-3203 consumption, direction, anchored polygon construction,
surface commit, extrusion, motion, G-code, and CLI activation remain
source-cited future work.

## Task 22O.48: resolve thick solid-infill bridge Flow

Task 22O.48 ports pinned
`LayerRegion.cpp:31-61::LayerRegion::bridging_flow(frSolidInfill, true)` and its
reached `PrintRegion.cpp` role selector plus `Flow.hpp`/`Flow.cpp` circular
thread semantics. The crate-private resolver consumes typed embedded
`internal_solid_filament_id`, `bridge_line_width`, `bridge_flow`, and nozzle
values, returning the existing Flow model with exact f32/f64 cast order,
0.05-mm bridge spacing, and circular volume.

Seven focused tests freeze default KSR bits, percent/absolute/zero widths,
selected and element-zero fallback nozzles, f64 percent and sqrt cast order,
nonpositive ratios, invalid selected nozzles, spacing, volume, errors,
repeatability, and nonmutation. O47's real-KSR test now obtains its
caller-side target height through O48 and retains the exact 115-path /
5,641-point digest
`f28db7dd3fc63155752ba5c33d4cd6338b2e311d83eb973c473d7f65268aa92a`.

O48 also factors the existing thick overhang Flow branch through the shared
private source operation. Final verification passes focused 7/7, combined
O47/O48 16/16, dependency 597/597, and workspace 6,257/6,257 Nextest,
warning-denying workspace Clippy, rustfmt, core/browser wasm32, diff, LOC, and
static audits. Independent six-axis review first rejected missing boundary and
cast-order discriminators; the main thread repaired them, and two read-only
re-reviews ended in unconditional approval with no remaining finding. It
introduces no new Flow type, lifecycle stage, public
option/API, geometry, or G-code output. Candidate clustering and the remaining
bridge transaction stay pending.

## Task 22O.49: apply the internal bridge angle override

Task 22O.49 is the next dependency-first rewrite slice. It ports pinned
`PrintObject.cpp:3253-3267`, consuming typed embedded
`internal_bridge_angle`, `relative_bridge_angle`, and
`align_infill_direction_to_model` plus the already-retained object transform
rotation. The private operation preserves exact degree-to-radian and branch
arithmetic while returning only the final angle value.

The implemented milestone remains unwired and adds no geometry, map,
scheduler, prepared successor, or G-code byte. Five focused tests freeze exact
source-order radians, relative/absolute/alignment ownership, nonfinite
behavior, no normalization, repeatability, and nonmutation. Three real-KSR
tests traverse all 43 O43 candidates and prove default pass-through plus
separately prepared absolute/aligned and relative archives with a retained
pi/2 object rotation. Final verification passes focused 8/8, dependency
605/605, and workspace 6,265/6,265 Nextest, warning-denying workspace Clippy,
rustfmt, core/browser wasm32, diff, LOC, and static audits. Independent
six-axis review requested two repeatability hardening fixes; the main thread
applied them and final re-review approved with no remaining finding.

Automatic direction detection, clustering, transaction-local O46/O47
composition, anchored polygon construction, surface commit, extrusion, motion,
G-code, and CLI activation remain deferred.

## Task 22O.50: nearest anchor-line AABB tree

Task 22O.50 ports the balanced indexed-line `LinesDistancer` reached by pinned
`PrintObject.cpp:2849-2930::determine_bridging_angle`, with exact
`AABBTreeLines.hpp` / `AABBTreeIndirect.hpp` build, QuickSelect, traversal,
projection, pruning, and equal-distance ownership. The rendering-neutral Rust
geometry seam borrows lines, owns only implicit tree nodes, and returns the
nearest original line index, squared distance, and integer nearest point.

Five focused tests freeze actual pinned-C++ literals for implicit layouts,
projection branches, containment and bbox tie ownership, non-power-of-two and
equal-centroid QuickSelect behavior, coordinates above 2^53, `HI_RANGE`,
repeatability, and input preservation. Verification passes focused 8/8,
dependency 613/613, and workspace 6,273/6,273 Nextest, warning-denying
workspace Clippy, rustfmt, core/browser wasm32, diff, LOC, and static audits.

This dependency-first milestone adds no direction aggregation, candidate map,
prepared successor, or public behavior. Automatic direction windows/pattern
adjustments, O49 composition, clustering, anchored polygon construction, and
the remaining bridge/G-code/CLI transaction stay deferred.

## Task 22O.51: automatic bridge-angle vote

Task 22O.51 ports pinned
`PrintObject.cpp:2849-2932::determine_bridging_angle`. It reuses O50 to sample
bridge-area segments against anchor lines, preserves exact ordered orientation
buckets and periodic sliding-window voting, and applies the source fallback
plus Hilbert/Octagram adjustments.

Nine focused pinned-C++ literal tests cover both integer scale thresholds,
polygon-local/reset/no-closing sampling, Eigen normalization plus f32 step
casts, nearest ownership and orientation folding, closed/wrapped windows,
strict ties, numeric-key equivalence, fallback, exhaustive typed patterns, and
nonmutation. Verification passes focused 9/9, dependency 622/622, and workspace
6,282/6,282 Nextest, warning-denying workspace Clippy, rustfmt, core/browser
wasm32, diff, LOC, and static audits.

The operation stays crate-private and unwired. Candidate anchor construction,
O46/O47 ownership, O49 composition, O43 mutation, clustering, anchored polygon
construction, surface commit, extrusion, motion, G-code, and CLI activation
remain deferred.

## Task 22O.52: indexed line intersections and contour outside

Task 22O.52 ports pinned
`AABBTreeLines::LinesDistancer::intersections_with_line<true>` and `outside` on
the existing O50 borrowed tree, preserving left/right traversal, segment
intersection casts, fixed-MSVC equal-key sorting, shared-vertex ray ownership,
and X/Y parity fallback.

Eight focused tests freeze actual pinned-C++ literals, source-safe boundary behavior, complete input preservation, the normative greater-than-32 equal-key permutation, and high-coordinate sort-key bits. Five reversible mutations are killed and restored byte-exact. Verification passes focused 8/8, dependency 630/630, workspace 6,290/6,290, warning-denying Clippy, core/browser wasm32, rustfmt, diff, LOC, and static audits.

This does not yet construct anchored polygons or alter candidates. Rotation,
scanline generation, section tracing, safety union, transaction commit,
extrusion, motion, G-code, and CLI activation remain deferred.

## Task 22O.53: anchored bridge polygon construction

Task 22O.53 ports pinned `PrintObject.cpp:2939-3111::construct_anchored_polygon`. It composes existing O50-O52 indexed line queries with exact Flow scaling, rotation, center-shifted scanlines, anchor extension, section merge/fixed-MSVC ordering, trace reconstruction, flat safety union, and inverse rotation.

The crate-private borrowed-input/owned-output operation introduces no option. A pinned-C++ harness with fixed-MSVC replay freezes normal/large scales, rotated and axis-aligned complete output, empty output, multi-section ordered paths, and flat union topology. Nineteen reversible mutations are killed and restored byte-exact. Verification passes focused 20/20, dependency 650/650, workspace 6,310/6,310, warning-denying Clippy, wasm32, both Windows architectures, both macOS architectures, rustfmt, diff, LOC, static, and clean-Orca checks.

The call sites and bridge transaction remain deferred: boundary/anchor assembly, O46/O47/O48/O49/O51 composition, collision rerun, postprocessing, candidate commit, lifecycle successor, extrusion, motion, G-code, and CLI parity are not part of O53.

## Task 22O.54: bridge candidate-layer clustering

Task 22O.54 ports pinned `PrintObject.cpp:2763-2818`, the complete
candidate-layer clustering prepass for `bridge_over_infill`. The private
operation constructs each layer's source-order flat coverage from 7-mm
inflated candidate bounding boxes, then groups ascending layers with the exact
strict Z-gap and previous-tail intersection rules.

An actual-source/fixed-MSVC-order oracle freezes normal/large coverage and
cluster literals. Final gates pass focused 11/11, dependency 661/661, workspace
6,321/6,321, warning-denying Clippy, wasm32, Windows/macOS, formatting/static
checks, and fifteen killed/restored mutations, including independent O48
raw-nozzle, ignored-width, and ignored-flow-ratio bypasses.

The milestone reuses O43 candidate geometry and the current layer's region-zero
O48 bridge Flow height without activating a lifecycle successor. The source
TBB/time-limit/debug-output adapter, candidate expansion, O46-O53 transaction
composition, collision rerun, surface commit, extrusion, motion, G-code, and
CLI parity remain deferred.

## Task 22O.55: bridge candidate ordering

Task 22O.55 ports pinned `PrintObject.cpp:3127-3153`, the complete
per-layer candidate presort reached after O54 clustering. It preserves the
non-stable fixed-MSVC minimum-X/minimum-Y sort, then the source stable tail
ordering by distance from the first candidate's maximum corner.

Pinned-dependency/fixed-MSVC literals match exactly. Final gates pass focused
12/12, dependency 673/673, workspace 6,333/6,333, Clippy, wasm32,
Windows/macOS, formatting/static checks, and thirteen killed/restored
mutations.

The private operation consumes and returns owned O43 candidates without
cloning polygon payloads or activating a successor. TBB/time-limit/debug
adapters, deep-area gathering, lower-layer subtraction, anchor/angle/polygon
composition, collision handling, postprocessing, candidate commit, surface
rewrite, prepared successor/lifecycle activation, extrusion, motion, G-code,
and CLI parity remain deferred.

## Task 22O.56: lower-cluster bridge subtraction

Task 22O.56 ports pinned `PrintObject.cpp:3160-3179` from the reviewed
3160-3187 window, the complete block
that removes bridge polygons already filled by earlier jobs in the same O54
cluster from the current deep sparse area. The private operation preserves
exact bottom-Z arithmetic, newest-to-oldest break behavior, candidate/polygon
flattening order, and one unconditional flat Clipper difference.

Pinned-source/fixed-MSVC-order literals match exactly. Final gates pass focused
10/10, dependency 683/683, workspace 6,343/6,343, Clippy, wasm32,
Windows/macOS, formatting/static checks, and ten killed/restored mutations.

O47/O48 caller composition, deep-area expansion, current-layer area and anchor
gathering, O46/O49/O51/O53 composition, collision handling, postprocessing,
candidate commit, surface rewrite, prepared successor/lifecycle activation,
extrusion, motion, G-code, and CLI parity remain deferred.

## Task 22O.57: current-layer bridge expansion context

Task 22O.57 ports pinned `PrintObject.cpp:3181-3205`, the complete
current-layer context block after O56. The private operation preserves exact
spacing promotion/cast order, source-ordered Top/Internal/InternalSolid/all-fill
and Lightning collection, scale-dependent epsilon closing, deep intersection,
lower-layer infill-line anchor clipping, and unsupported-area shrinking.

Pinned actual-source ordered literals match exactly. Final gates pass focused
15/15, dependency 698/698, workspace 6,358/6,358, strict Clippy, wasm32,
Windows/macOS, formatting/static checks, and nineteen killed behavioral
mutations including both operation-order transformations.

O47/O48/O46 provenance composition, debug-only 3206-3210, the candidate block
at 3211-3308 (loop 3213, expansion 3215), O49/O51/O53 composition, collision handling, postprocessing, candidate commit,
surface rewrite, prepared successor/lifecycle activation, extrusion, motion,
G-code, and CLI parity remain deferred.

## Task 22O.58: candidate bridge area filtering

Task 22O.58 ports pinned `PrintObject.cpp:3215-3224`, the complete per-candidate
prefix after O57: candidate expansion, deep intersection, ordered per-polygon
unsupported filtering, limiting union, and the source union-before-empty gate.
The private seam consumes composer-owned O43/O55/O57 geometry and
candidate-region O48 scaled spacing without inference.

Removed actual-source ordered literals match exactly. Fifteen behavioral
mutations, including repeated-union and two competing-error-order variants,
were killed with byte-exact restoration. Final gates pass focused 10/10,
dependency 708/708, workspace 6,368/6,368, strict Clippy, wasm32,
Windows/macOS, formatting/static checks, clean Orca, no staged files, and
independent six-axis re-review approval.

Loop/Flow provenance at 3213-3214, source continue at 3226-3227, boundary
construction at 3229-3233, angle/anchor composition, lightning and collision
handling, postprocessing, candidate commit, surface rewrite, prepared
successor/lifecycle activation, extrusion, motion, G-code, and CLI parity
remain deferred.

## Task 22O.59: candidate boundary polylines

Task 22O.59 ports pinned `PrintObject.cpp:3226-3233`: the O58 survivor-empty
continue and the two ordered boundary expansions/conversions. The private seam
consumes the O58 result plus composer-supplied O48 `scaled_spacing()` and
`spacing()` values, preserving the distinct f64 arithmetic and total-before-
limiting polyline order without option inference.

Removed actual-source literals match exactly. Nineteen mutations, including
explicit ascending output sorting, were killed with byte-exact restoration. Final gates pass focused 10/10, dependency
718/718, workspace 6,378/6,378, strict Clippy, wasm32, x86_64/aarch64 Windows
and macOS, formatting/static checks, clean Orca, and no staged files.

Candidate loop/Flow provenance, angle and anchor composition, lightning and
collision handling, postprocessing, candidate commit, surface rewrite,
prepared successor/lifecycle activation, extrusion, motion, G-code, and CLI
parity remain deferred.

## Task 22O.60: candidate bridge angle composition

Task 22O.60 ports pinned `PrintObject.cpp:3242-3267`: branch on the source outer
anchor container, flatten the selected anchor or fallback boundary polylines in
`Polyline.hpp::to_lines` order, call O51 with the sparse or neutral Line
pattern, then feed its exact detected angle through O49. The private seam
consumes only O57/O58/O59 results, typed candidate-region options, retained
object rotation, and coordinate scale supplied by the future composer.

Candidate iteration/provenance, line 3268 anchor append, Lightning clipping,
O53 construction, collision handling, postprocessing, candidate/surface commit,
prepared successor/lifecycle activation, extrusion, motion, G-code, and CLI
parity remain deferred.

O60 now passes focused 7/7, dependency 2,354/2,354, workspace 6,385/6,385,
strict lint/format, wasm32 and four desktop cross-target checks. Nineteen
mutations are killed; the source-derived line/order/numeric oracle is repeatable.
Final independent six-axis re-review approved with no remaining repair item.

## Task 22O.61: candidate anchored bridge

Task 22O.61 ports pinned `PrintObject.cpp:3268-3272`: source-ordered anchor
append, conditional Lightning overlap clipping with exact `scale_(10)`, exact
Polyline-to-Line conversion, and one O53 anchored-polygon call. The private
operation consumes only O57/O58/O59/O60/O48 state plus retained coordinate
scale supplied by the future composer and returns owned post-clip boundaries
plus initial bridge polygons.

Collision reconstruction, postprocessing, expansion/candidate/surface commit,
prepared successor/lifecycle, second bridge pass, extrusion, motion, G-code,
and CLI parity remain deferred. Implementation starts after independent
ADR/spec/plan approval.

O61 now passes focused/KSR 9/9, dependency 2,363/2,363, workspace
6,394/6,394, strict lint/format, five portability builds, and twenty-three
mutation kills. Final independent six-axis implementation review approved
unconditionally with no remaining repair item.

## Task 22O.62: candidate collision reconstruction

Task 22O.62 will port pinned `PrintObject.cpp:3274-3288`: expand the initial
O61 bridge area by exact `3.0 * flow.scaled_spacing()`, traverse caller-provided
prior-completed surfaces in future-composer append order, select the first
colliding surface's angle, and rerun O53 exactly once only on collision. Their
`new_polygons` must be postprocessed at source lines `3292-3297` and appended
at `3304-3305`, never raw/pre-expansion O43 candidate geometry. Producing that
history remains deferred. The private operation consumes O61 owned output plus
borrowed completed O43-shaped records, exact O48 Flow, original candidate area,
and retained coordinate scale without activating the transaction lifecycle.

Opening/closing and limiting/total-fill/top-area postprocessing, expansion-area
mutation, candidate/per-layer commit, composer and prepared successor,
extrusion, motion, G-code, CLI, and full golden parity remain deferred. Start
behavioral RED only after independent ADR/spec/plan approval; exit requires
focused/dependency/workspace, mutation, portability/static, and independent
six-axis implementation approval.

O62 implementation now passes focused 8/8, dependency 2,371/2,371, workspace
6,402/6,402 with two skipped, strict lint/format, five portability builds, and
26/26 mutation kills. Final independent six-axis implementation review approved
unconditionally with no remaining repair item.

## Task 22O.63: bridge postprocessing geometry

Task 22O.63 will port pinned `PrintObject.cpp:3290-3298`: active fine-detail
opening at `0.75 * flow.scaled_spacing()`, one-spacing closing, limiting and
total-fill intersections, total-top subtraction, and subtraction of the final
bridge from expansion area. The private operation consumes O62 owned state and
borrows caller-provided layer areas without activating the transaction.

Debug drawing, candidate append, per-layer replacement, history-producing
composer, prepared successor/lifecycle, second bridge pass, region rewrite,
extrusion, motion, G-code, CLI, and full golden parity remain deferred.

O63 implementation now passes focused 7/7, dependency 2,378/2,378, workspace
6,409/6,409 with two skipped, strict lint/format, five portability builds, and
25/25 mutation kills with byte-exact restoration. Final independent six-axis
implementation review approved unconditionally with no remaining repair item.

## Task 22O.64: bridge candidate commit history

Task 22O.64 will port pinned `PrintObject.cpp:3304-3310`: append each successful
postprocessed candidate with original identity, final polygons, and angle; carry
the owned expansion state to the next candidate; then swap the completed vector
into the current layer and clear the original inventory.

Reserve/orchestration/debug, map and cluster traversal, the second pass at
`3315+`, region rewrite, lifecycle, extrusion, motion, G-code, CLI, and full
golden parity remain deferred. Start RED only after independent source-boundary
approval; exit requires focused/full, mutation, portability/static, and six-axis
implementation approval.

O64 implementation now passes focused 6/6, dependency 2,384/2,384, workspace
6,415/6,415 with two skipped, strict lint/format, five portability builds, and
16/16 mutation kills with byte-exact restoration. Final independent six-axis
implementation review approved unconditionally with no remaining repair item.

## Task 22O.65: bridge rewrite-area collection

Task 22O.65 will port pinned `PrintObject.cpp:3318-3319,3322-3336`: distinguish absent
current/upper candidate-map keys, flatten current bridge polygons into the
layer cut set, and sequentially build one-spacing ensuring rings from each
upper committed candidate using its already-resolved Task 22N normal
solid-infill Flow. Traversal/timeouts `3315-3317`, layer retrieval `3320`, and
source-to-record projection remain composer-owned.

Parallel/map/layer traversal, per-region rewrite at `3338+`, second bridge pass,
composer/lifecycle, extrusion, motion, G-code, CLI, and full golden parity
remain deferred. Start RED only after independent source-boundary approval;
exit requires focused/full, mutation, portability/static, and six-axis
implementation approval.

O65 implementation now passes focused 9/9, dependency 2,393/2,393, workspace
6,424/6,424 with two skipped, strict lint/format, five portability builds, and
24/24 compiling mutation kills with byte-exact restoration. Final independent
six-axis implementation review approved unconditionally.

## Task 22O.66: region bridge ensuring-area preparation

Task 22O.66 will port pinned `PrintObject.cpp:3341-3343`: flatten every region
fill surface, safety-union the complete set, derive the one-solid-spacing
near-perimeter ring, and intersect O65 upper ensuring areas with that ring. The
private operation receives already-resolved Task 22N normal solid-infill Flow
and retained coordinate scale from the future composer.

Internal infill subtraction at `3345+`, bridge retagging, solid recomposition,
region mutation, layer/map traversal, second bridge pass, composer/lifecycle,
extrusion, motion, G-code, CLI, and full golden parity remain deferred. Start
behavioral RED only after independent ADR/spec/plan approval; exit requires
focused/full, mutation, portability/static, and six-axis implementation
approval.

O66 implementation now passes focused 12/12, dependency 776/776, workspace
6,436/6,436 with two skipped, strict lint/format, five portability builds, and
30/30 compiling mutation kills, including safety difference/intersection, with
byte-exact restoration. Final independent
six-axis implementation review approved unconditionally.

## Task 22O.67: internal infill rebuild

Task 22O.67 will port pinned `PrintObject.cpp:3345-3350`: stably select the
current region's Internal surfaces, subtract O65 cut geometry, subtract O66
ensuring geometry, and create fresh default-metadata Internal surfaces.

Bridge conversion at `3352+`, solid recomposition, region replacement, second
pass, composer/lifecycle, extrusion, motion, G-code, CLI, and full golden parity
remain deferred. Start RED only after independent source-boundary approval;
exit requires focused/full, mutation, portability/static and six-axis approval.

O67 now passes focused 6/6, dependency 782/782, workspace 6,442/6,442 with two
skipped, strict lint/format, five portability builds, and 18/18 compiling
mutation kills with byte-exact restoration. Final independent six-axis review
approved unconditionally.

## Task 22O.68: internal bridge surface conversion

Task 22O.68 ports pinned `PrintObject.cpp:3352-3367` and the directly owned
`Surface.hpp` internal-bridge vocabulary: match O64 candidates to current-region
InternalSolid source surfaces by stable source index, default-NonZero union each
candidate's final polygons once, clone source metadata, retag InternalBridge,
replace the angle, and emit fresh surfaces in candidate/engine order.

Solid recomposition at `3368+`, region replacement, second pass, composer and
prepared lifecycle, extrusion, motion, G-code, CLI, and complete golden parity
remain deferred.

O68 passes focused 6/6, dependency 788/788, workspace 6,448/6,448 with two
skipped, strict lint/format, five portability builds, and 14/14 compiling
mutation kills with byte-exact restoration. Independent six-axis review
approved without repair items.

## Task 22O.69: internal solid recomposition

Task 22O.69 ports pinned `PrintObject.cpp:3368-3374`: stable InternalSolid
selection, ordered O66 ensuring append, one no-safety difference against the
O65 cut set, intact ExPolygon topology forwarding, one safety union, and fresh
default-metadata InternalSolid surfaces in engine order.

Debug-only `3376-3383`, region commit at `3385-3386`, composer and second pass,
prepared lifecycle, extrusion, motion, G-code, CLI, and complete golden parity
remain deferred.

O69 passes focused 6/6, dependency 794/794, workspace 6,454/6,454 with two
skipped, strict lint/format, five portability builds, and 26/26 compiling
mutation kills with byte-exact restoration. Independent six-axis review
approved with no blockers or major repairs.

## Task 22O.70: region bridge surface commit

Task 22O.70 ports pinned `PrintObject.cpp:3385-3386` plus the directly owned
stable removal and copy-append semantics: remove prior InternalSolid/Internal
records while preserving all other original surfaces and order, then append
the complete O67/O68/O69 rebuilt sequence in caller order.

Composer wiring, the second internal-bridge pass, prepared lifecycle,
extrusion, motion, G-code, CLI, and complete golden parity remain deferred.

O70 now passes focused 3/3, workspace 6,457/6,457 with two skipped, strict
lint/format, five portability builds, and 15/15 compiling mutation kills with
byte-exact restoration. Final independent six-axis review approved after the
complete topology snapshot repair, with no remaining item.

## Task 22O.71: first internal bridge transaction

Task 22O.71 ports pinned `PrintObject.cpp:2725-2761,3114-3389` as one deep
production transaction. It generates lower-layer anchors before clustering,
executes O54-O64 with ordered cluster history, then executes O65-O70 against
unchanged region snapshots and activates the post-bridge prepared lifecycle.

The current scope is the fixture-reachable single-region, non-Lightning
CrossHatch path. Lightning/adaptive/support-cubic and generic other-pattern
generation, the optional second internal-bridge pass at `3393+`, combine-infill,
extrusion, motion, G-code, CLI, and complete golden parity remain deferred.
Start from a real-KSR layer-15 `InternalBridge` RED; exit requires focused and
workspace tests, strict/portability/static gates, current golden checkpoint,
and unconditional independent six-axis approval.

O71 is now implemented and lifecycle-active. Active adaptive/support-cubic
octree states and other unported anchoring behavior fail explicitly, while
source-inactive density-zero or empty-fill adaptive states remain no-ops. The
KSR checkpoint freezes 47 bridge surfaces, 15,689 ordered topology points,
and the 17-layer digest
`c547cb34b8d5d27d572a166f13a16741f75f7f9d34f15db59ddac8575b5a33b9`;
the independent Orca G-code witness has the same 17 bridge layers and 30
feature runs. Focused O71 16/16, bridge dependency 240/240, workspace
6,473/6,473 with two skipped, strict lint/format/diff, all five portability
checks, LOC/static gates, and final independent reviews pass. Full G-code
golden parity remains deferred beyond the current incomplete lifecycle.

## Task 22O.72: infill-combination identity gate

Task 22O.72 ports the admitted branch of pinned
`PrintObject.cpp:673-680,3701-3706,4163-4287` as
`prepare_infill::combine_infill::{prepare, dispose, PreparedPostInfillCombination}`.
After O71, every region with disabled combination or exactly zero sparse
density passes unchanged; any enabled, nonzero-density region returns
`UnsupportedProjectFeature("infill_combination")` before mutation. Public
slicing advances to `consume_post_infill_combination`, disposes the successor,
and remains `ProjectSlicingIncomplete`.

The milestone must not reuse the legacy path-level
`infills::combination`/`InfillOptions` scaffold. The optional internal-bridge
pass at `PrintObject.cpp:3393-3546` gets no placeholder lifecycle stage because
O17/O71 reject every activating mode. The active combination algorithm at
`4176-4287`, fill grouping, extrusion, motion, G-code, CLI, and complete golden
parity remain deferred.

O72 is implemented and lifecycle-active. The public enabled/zero RED also
closed the preceding source case: O43 candidates remain intact, O71 projects
zero-density sparse anchor generation to an empty line set, then continues
boundary anchoring and commits InternalBridge surfaces before O72 applies its
identity branch. Tests do not clear candidate state to reach this behavior.

Exit requires disabled/nonzero and enabled/zero identity tests,
enabled/nonzero global/object/part override rejection and ownership tests,
public lifecycle coverage, gate-mutation kills, and an exact unchanged KSR O71
checkpoint: 47 bridge surfaces, 15,689 points, 17 bridge-bearing layers, SHA-256
`c547cb34b8d5d27d572a166f13a16741f75f7f9d34f15db59ddac8575b5a33b9`.
Final validation passes focused 14/14, prepare-infill 255/255, and workspace
6,486/6,486 with two configured skips. Six compiling mutations (including the
source `0.00011f` threshold) were killed and byte-exactly restored. Strict
lint/format, WASM core and adapter, both Windows targets, both macOS targets,
LOC/static/diff/no-staged gates, and a clean pinned Orca worktree pass;
unconditional independent six-axis review is the final release gate.

O72 handed the next source boundary to O73 at pinned
`Fill/Fill.cpp:216-346,829-1067,1213-1224` for `SurfaceFillParams`,
`SurfaceFill`, and base `group_fills`; O73 now implements that boundary as a
lifecycle-inactive module. O74 owns the remaining full-group tail at
`Fill/Fill.cpp:349-827,1069-1186`, including the KSR-active narrow-solid
splitting at `1152-1186`, before grouped fills may be considered for a later
production lifecycle successor.

## Task 22O.73: base fill grouping

Task 22O.73 is the dependency-first port of pinned
`Fill/Fill.cpp:216-346,829-1067` and the directly reached Flow, surface,
configuration, and Clipper behavior. It introduces one graph-native,
crate-private deep module:

```rust
project_slice::group_fills::group_fills_base(
    &PreparedPostExternalSurfaces,
    object_index,
    layer_index,
) -> Result<BaseGroupedFills, SliceError>
```

The module borrows the smallest prepared graph common to both source callers
and returns owned ordered base groups, LockedZag sidecars, and the observed
InternalVoid continuation bit. It hides exact surface/role Flow projection,
one-based filament and nozzle selection, source f32 cast points, the layer-wide
sticky params record, explicit pattern/role ranks, comparator-equivalent
interning, source-order coalescing, representative metadata, and raw-priority
safety union/difference.

Grouping identity follows only `SurfaceFillParams::operator<`: decreasing f32
bridge angle followed by the exact source fields. It is not the source
`operator==`, derived Rust equality/order, `total_cmp`, or hashing. Flow
spacing, Flow bridge state, `mm3_per_mm`, and source `idx` remain excluded;
`params.bridge` and `flow.bridge` remain independent. Grouped ExPolygons are
the authoritative geometry, so the Rust result carries representative surface
metadata without a moved-from placeholder ExPolygon.

O73 is not a lifecycle successor and must not alter the O72 incomplete sink.
The current public `multi_region_layer_slices` capability gate continues to
own multi-region rejection; its graph representation, ordered region joining,
and no-overlap union are deferred rather than replaced with region zero.
Nonempty rotation templates remain explicitly unsupported until their pinned
grammar/PRNG is ported, with no legacy simple-list or host-RNG fallback.

O73 also does not replace the current O46 reduced private grouping. Both O46
and future `Layer::make_fills` may move to the shared implementation only after
O74 ports `Fill/Fill.cpp:349-827,1069-1186`, including the KSR-active narrow
internal-solid continuation.

The pre-narrow KSR exit contract preserves all 460 ordered layer slots and
requires 477 groups, 1,882 fill ExPolygons, 174 fill holes, 2,056 fill paths,
107,540 fill points, and 2,547 no-overlap ExPolygons, with metadata SHA-256
`a091ca0a63e45dc81712223571b1dfe888ab256bec2437ea564f386783f77900`
and canonical geometry SHA-256
`062fab2bbcb683df778ac024a8f6abed7960f3ebac3d55f13124617694d7e2af`,
plus layer-table SHA-256
`ebd74a25609827e4affda26a21d9cd3b10dca08778f56f394b5170f74ecdf721`.
These values replay O38's fixed-MSVC bridge-direction order; the complete
Linux PRE/POST triplets recorded in the O73 ADR and specification remain
nonnormative provenance. The fill totals exclude the no-overlap section; the
canonical geometry digest includes both. Canonicalization is oracle-only;
production Clipper order is preserved. Explicit `assert_ne!` witnesses reject
the distinct O74 aggregate totals and each of its three hashes; O74 remains a
negative boundary, not an O73 success target.

Final exact-tree validation passed focused `task22o73` 19/19 with 6,451
skipped, prepare-infill 277/277 with 26 slow and 6,193 skipped, and workspace
6,508/6,508 with 27 slow and two configured skips. Strict workspace
all-target/all-feature Clippy with `-D warnings`, rustfmt, diff, all six Tier-1
checks, zero-staged/Cargo-unchanged/forbidden-production/lifecycle-static
checks, and clean pinned Orca at
`8500fcdccaa10b5099ac20d252af3a7c560046f1` passed. The maximum changed/new
Rust file was `project_slice.rs` at 381 LOC and the maximum new production
shard was `group_fills/params/projection.rs` at 369 LOC. Thirty-one compiling
behavioral mutations were killed and byte-exactly restored; one additional
compiling contour/hole insertion-order mutation was an equivalent survivor on
normalized valid ExPolygons and was not counted as a kill. The nine restored
production hashes matched the exact manifest in the O73 ADR, specification,
and plan. Independent source/specification and standards rereviews closed
unconditionally. O73 remains crate-private and lifecycle-inactive; it handed
the remaining grouping tail to the O74 implementation recorded below.

## Task 22O.74: full fill grouping

Task 22O.74 implements pinned `Fill/Fill.cpp:349-827,1069-1186`, reusing
O73's verified `216-346,829-1067` port behind one crate-private graph-native
seam:

```rust
project_slice::group_fills::group_fills(
    &PreparedPostExternalSurfaces,
    object_index,
    layer_index,
) -> Result<GroupedFills, SliceError>
```

The implementation removes the callable `_base` seam and `BaseGroupedFills`
rather than wrapping them. `GroupedFills` owns only ordered surface fills and
LockedZag sidecars. `SurfaceFillParams` carries the source `idx`: comparator-
ordered base groups receive their source ordinal, the comparator excludes it,
and an appended partial narrow group copies its original group's identity even
though its vector position differs.

The apparent InternalVoid repair at `1069-1150` is not reachable from the
same source function: voids are observed but excluded at `855-861`, excluded
again during group materialization at `1028-1051`, and then searched only
inside the already filtered groups at `1086-1097`. O74 ports the observable
no-op, removes O73's continuation bit, and does not pull raw void geometry
from the graph or claim active repair.

The implemented behavior is the complete line/non-line narrow split at
`349-827` and its option-gated mutation/append at `1152-1186`. It preserves the
original-count snapshot, source vibration state and quirks, all-narrow
pattern-only mutation, partial append order, source-default appended
representative metadata, copied region/no-overlap/`idx`, and unchanged lock
sidecars. O73 behavior tests cross the full seam with
`detect_narrow_internal_solid_infill = false`; the full 460-layer KSR POST
oracle crosses it with the option true.

The false-option full-seam regression retains O73's all-460-layer PRE totals
and fixed-MSVC metadata, canonical-geometry, and layer-table digests. It is a
required disabled-option behavior witness; POST remains O74's success target.

The normative POST contract replays the fixed-MSVC predecessor order:
536 groups, 2,218 fill ExPolygons, 152 holes, 2,370 paths, 110,610 points,
2,928 no-overlap ExPolygons, metadata
`cd4aa18a831dd4672e3e394944e496b8d349b5e21990672a7f14868cc2b3b387`,
canonical geometry
`c149d65f5e5ddb89643b78314861ac2343707ddf76decc1e6aa2f88901331f6c`, and
layer table
`8d9845b22e38857dbb0840b2527286436a6b9c684c8662d925f8fd4873cef5b2`.
The Linux POST triplet remains nonnormative provenance, canonicalization is
oracle-only, and source-pinning or instrumentation hashes alone are not
acceptance.

The disabled-option PRE digests remain
`a091ca0a63e45dc81712223571b1dfe888ab256bec2437ea564f386783f77900`,
`062fab2bbcb683df778ac024a8f6abed7960f3ebac3d55f13124617694d7e2af`,
and `ebd74a25609827e4affda26a21d9cd3b10dca08778f56f394b5170f74ecdf721`.
Raw-order POST evidence pins layer-1 metadata
`b466abfd76770f5e776b9df3866cf12b07b836bee2a8a7ba721c66ae1f2851bf`,
layer-1 geometry
`0938758d43750be165712735f6f5e1b6a1ae8fbb52a7f551b101118e1083c856`,
and ordered layer-45/layer-70 geometry
`33bf737e3d836096a20a821fcf1ace79dccda10973203408ba87ddee5ee25d64` /
`7a8e9ec6e0aa2b1a8cd6bd8d1e9c261719b77168427f113fa051e7f5c551be71`.
The fixed-MSVC source-backed table rows are:

```text
1\t2\t29\t0\t723\t5,5\t0,29\t5,5
45\t4\t75\t15\t29423\t6,5,0,4\t0,29,1,20\t10,5,6,4
70\t8\t70\t0\t626\t2,6,6,6,6,6,5,4\t0,0,0,0,0,0,29,20\t9,10,10,10,10,10,5,4
```

The layer-45/layer-70 geometry hashes above use the same source-backed ordered
raw records, not canonical-sort substitutes.

The source-backed oracle grammar deliberately does not add
`Flow::mm3_per_mm`. Rust-only focused tests assert its exact `f64::to_bits()`
values, including the partial-split copy
`0x3fbb_4fc3_4000_0000`; these invariants do not alter the C++ grammar or the
aggregate PRE/POST hashes.

The public-seam corpus killed the vibration-filter identity substitution,
`4 mm -> 3 mm`, maximum skips `2 -> 1`, exact two-skip `>= 2 -> > 2`,
removal depth `> 5 -> >= 5`, exact `4 mm` `< -> <=`, touch-back removal,
final normal expansion `0.5 * spacing -> 0`, a zero non-line closing delta,
and hard-coded Normal scale. The KSR checkpoint specifically killed the
filter/threshold/skip/depth/touch-back/final-expansion subset; graph-native
focused tests killed exact-4-mm, zero-closing-delta, and hardcoded-scale
changes. The two skip
mutations produced 2,223 / 2,375 / 110,582 and 2,217 / 2,369 / 110,597
fill-ExPolygon/path/point totals. Next-section reset removal,
inclusive-Y-to-strict-Y, the source `558-559` correction, `candidates_begin`
correction, early-closure removal, reconnection `< -> <=`, one-coordinate-unit
non-line spacing, and premature f32 scale/cast changes survived;
pinned-source/static review retains them and they are not claimed as kills.
FIFO/LIFO pending-order and duplicate-queue cases are
monotone-closure/static-review cases.

`crates/ares-core/src/project_slice.rs` changes only the inactive-module
reason. O74 adds no prepared lifecycle state, O46 wiring, public API, or Cargo
activation. O46's reduced private grouping remains. Its future replacement
source is `Fill.cpp:1394-1407`, where sparse anchoring calls full
`group_fills` before selecting `stInternal`; a later source-cited milestone
must wire that caller and delete the compatibility grouping atomically.
Fill-generator dispatch, `FillConcentricInternal`, extrusion, motion, G-code,
CLI, and complete golden parity remain later slices.

### Final evidence — pending

O74 is implemented, but exact focused/dependency/workspace command counts,
strict lint/format/Tier-1/diff/static gate results, and unconditional
independent source/specification and standards approval remain a deliberately
unfilled final-evidence placeholder. Do not infer those results from the
implemented status.

## Task 22O.75: full-grouping sparse anchoring

Task 22O.75 ports the pinned caller relationship at
`Fill/Fill.cpp:1394-1407`: sparse anchoring now calls the complete O74
`group_fills` seam and filters its owned result for `stInternal` before the
existing KSR-active CrossHatch continuation. The transaction passes the
prepared external-surface graph plus aligned object/layer indices, so grouping
uses only the effective options and geometry derived from the 3MF.

The temporary O46 `sparse_anchoring/grouping.rs`, its three-pattern comparator,
priority copy, caller-built `SparseAnchoringLayer`, and direct reduced tests are
deleted without a wrapper or fallback. Returned percentage density is converted
with source expression `float(0.01 * density)` before CrossHatch filling.

The fixed-MSVC KSR 18-layer oracle remains exact at 186 paths, 5,941 points,
and aggregate SHA-256
`917adc6ea02ad7cd7af79e45d90db6f4c1497bf5c8716d7f2f49b7de4b2070ef`.
Focused anchoring, grouping, and bridge transaction runs passed 1/1, 35/35,
and 17/17. Workspace Nextest passed 6,516/6,516 with 27 slow and two configured
skips; core strict Clippy, rustfmt, diff, static deletion, and sub-400-LOC gates
passed.

O75 adds no lifecycle stage or public API. Unsupported filler generators,
`Layer::make_fills`, extrusion, motion, G-code, CLI success, and normalized KSR
golden parity remain later source-cited work.

## Task 22O.76: CrossHatch fill entities

O76 ports the first bounded `Layer::make_fills` slice from pinned
`Fill/Fill.cpp:1213-1224,1234-1357` and `FillBase.cpp:133-184`. A graph-native
crate-private seam calls complete `group_fills`, selects configured CrossHatch
groups, iterates authoritative ExPolygons in source order, and turns generated
polylines into owned extrusion collections.

Each path retains the grouped extrusion role and Internal sparse Flow exactly;
the focused KSR-shaped witness freezes `mm3_per_mm`, width, and height bits as
`0x3fb4d7aca0000000`, `0x3ee66666`, and `0x3e4ccccd`. Non-CrossHatch groups do
not run through a fallback. Three focused graph tests pass for metadata/order,
repeatability/immutability, non-fallback, and atomic range errors. Strict
workspace Clippy, rustfmt, diff, and sub-400-LOC gates pass.

O76 is lifecycle-inactive and deliberately does not claim complete KSR fill
entities. Remaining filler classes, adjusted solid/bridge flow, gap/thin fill,
ordering, motion, G-code, and normalized golden parity remain later slices.

## Task 22O.77: rectilinear vertical segmentation

O77 ports pinned `FillRectilinear.cpp:357-496,759-993` as the first dependency
slice for KSR's Monotonic and MonotonicLine fillers. The private Rust module
builds source outer/inner offset contours, rotates fixed coordinates, intersects
source-ordered contours with equally spaced vertical lines using rational
arithmetic, classifies outer/inner low/high intersections, and removes duplicate
vertices.

Three focused tests pass for rectangle order/kinds, holes and offset identities,
rational rounding, rotation, repeatability/immutability, and coordinate range
errors. Strict core Clippy, rustfmt, diff, and sub-400-LOC gates pass. Link graph,
monotonic traversal, complete fillers/entities, lifecycle, motion, and G-code
remain later source-cited slices.

## Task 22O.78: rectilinear contour links

O78 ports pinned `FillRectilinear.cpp:994-1214` over O77 vertical sections.
Each intersection now owns source previous/next contour link index, horizontal
or vertical direction, and valid/invalid/too-long quality. Adjacent candidate
selection follows contour/kind identity and source segment-distance order;
same-line opposite kinds may replace adjacent links. Don't-connect and maximum
length gates update quality without changing geometry.

Two O78 focused tests and all three O77 regressions pass. Strict core Clippy,
rustfmt, diff, and LOC gates pass. Pinch handling, monotonic region chaining,
complete fillers/entities, lifecycle, motion, and G-code remain later slices.

## Task 22O.79: rectilinear pinch intersections

O79 ports pinned `FillRectilinear.cpp:1216-1312`. Disconnected adjacent
InnerHigh/InnerLow runs receive source-midpoint phony OuterHigh/OuterLow pairs;
current and neighboring link indices are remapped in place. Nonpinched sections
remain identical.

Two O79 tests and all five O77/O78 regressions pass. Strict core Clippy,
rustfmt, diff, and LOC gates pass. Monotonic regions/chaining, complete filler
entities, lifecycle, motion, and G-code remain later source-cited slices.

## Task 22O.102: Arachne transition middle generation

O102 ports `generateTransitionMids` from the pinned
`SkeletalTrapezoidation.cpp:788-850` boundary. The inactive slice generates
ordered threshold transitions on upward central edges with O99 strategy
values, wide integer interpolation, and owned weak-reference storage. One
focused test passes; transition filtering, transition ends, ribs, toolpaths,
and G-code remain deferred.

## Task 22O.80: monotonic region generation

O80 ports pinned `FillRectilinear.cpp:1590-1629,1711-1931`. O79 linked
sections are scanned in source seed order and exclusive adjacent overlaps extend
owned left/right monotonic boundaries. Source consumed state prevents duplicate
regions and line-count parity determines `flips`.

Two focused tests pass for odd/even rectangular parity, repeatability, and input
immutability. Strict core Clippy, rustfmt, diff, and LOC gates pass. Neighbor
scattering/path lengths, ant chaining, polylines/entities, lifecycle, motion,
and G-code remain later slices.

## Task 22O.81: monotonic region neighbors

O81 ports pinned `FillRectilinear.cpp:2079-2179`. O80 region boundaries are
mapped through O78 horizontal links into sorted unique left/right neighbor
indices with bidirectional symmetry.

Two focused tests and all 1,179 task22o core regressions pass. Strict core
Clippy, rustfmt, diff, and LOC gates pass. Region path lengths, ant chaining,
polylines/entities, lifecycle, motion, and G-code remain later slices.

## Task 22O.82: rectilinear contour context

O82 ports the retained ownership boundary of pinned
`FillRectilinear.cpp:357-457,759-993`. One owned slice now keeps the rotated
source, ordered outer/inner offset contours, and O77 lines whose contour and
segment indices address that inventory.

Two focused tests pass; strict core Clippy, rustfmt, diff, and LOC gates pass.
Contour path measurement/emission, region lengths/chaining, filler entities,
lifecycle, motion, and G-code remain later slices.

## Task 22O.83: rectilinear perimeter primitives

O83 ports pinned `FillRectilinear.cpp:38-116,459-685`. O82 contour indices now
drive directed/wrapped arc distance, exact f64 length, and forward/reverse
perimeter vertex emission for adjacent and same-line intersections.

A RED same-segment oracle caught an incorrect full-loop append. Two O83 focused
and all seven O77-O79 regression tests pass. Strict core Clippy, rustfmt, diff,
and LOC gates pass. Corrected source link selection/quality, region
costing/chaining, entities, lifecycle, motion, and G-code remain later slices.

## Task 22O.84: source rectilinear links

O84 replaces O78 approximations with pinned `FillRectilinear.cpp:994-1214`.
O82 retained contours and O83 directed distances now select strict first-tie
adjacent/same-line links, invalidate skipped-inner/same-side arcs, apply exact
contour-length quality gates, and mirror invalid vertical quality.

Compile RED rejected the old bare-lines seam. Two focused and all 15 O77-O83
regressions pass. Strict core Clippy, rustfmt, diff, approximation-removal, and
LOC gates pass. Region costing, ant chaining, polyline/entity output, lifecycle,
motion, and G-code remain later slices.

## Task 22O.85: monotonic region costs

O85 ports pinned `FillRectilinear.cpp:1989-2077,2179-2188`. Both O80 boundary
orientations traverse corrected O84 vertical runs and horizontal arcs, retaining
source f32 accumulation, half perimeter cost, straight split-gap distance,
coordinate unscaling, and common-minimum subtraction.

Compile RED proved the missing seam. Two focused and both O84 regressions pass.
Strict core Clippy, rustfmt, diff, and LOC gates pass. Inter-region path matrix,
ant chaining, polyline/entity output, lifecycle, motion, and G-code remain later
slices.

## Task 22O.86: monotonic path matrix

O86 ports pinned `FillRectilinear.cpp:1590-1709`. A dense `2N × 2N` matrix
lazily caches exact f32 endpoint length/visibility for all region orientations,
while initial-deposit reset changes pheromone only.

Compile RED proved the missing module. Two focused and both O85 regressions pass.
Strict core Clippy, rustfmt, diff, and LOC gates pass. Ant simulation/RNG,
pheromone evolution, path selection, polylines/entities, lifecycle, motion, and
G-code remain later slices.

## Task 22O.87: monotonic ant chain

O87 ports pinned `FillRectilinear.cpp:2190-2582`. Standard default MT19937-64,
precedence queue, greedy deposit, bounded ant rounds, source probabilities,
local diversification, global pheromone evolution, strict best replacement,
and the pinned no-op 3-opt now emit owned region/orientation chains.

Compile RED proved missing modules. Three focused and both O86 regressions pass.
Strict core Clippy, rustfmt, diff, and LOC gates pass. Path-to-polyline emission,
filler entities, lifecycle, motion, and G-code remain later slices.

## Task 22O.88: monotonic polyline emission

O88 ports pinned `FillRectilinear.cpp:2584-2753`. O87 chains now emit source-
ordered outer endpoints, vertical runs, retained contour arcs, disconnected
splits, scale-aware filtering, duplicate removal, and phony-pinch merging.

Compile RED proved the missing emitter. Two focused and all three O87 regressions
pass. Strict core Clippy, rustfmt, diff, and LOC gates pass. Full
`fill_surface_by_lines` spacing/alignment/rotation, extrusion entities,
lifecycle, motion, and G-code remain later slices.

## Task 22O.89: monotonic surface filler

O89 ports pinned `FillBase.cpp:255-324` and
`FillRectilinear.cpp:2755-2908,3404-3421`. Explicit source parameters now drive
direction/layer alternation, offsets, density and adjusted-solid spacing,
retained scanlines, O79-O88 graph generation, and inverse rotation. O82 contour
retention and line population are separated without recomputing offsets.

Compile RED proved the missing module. Two focused and five O77/O88 boundary
regressions pass. Strict core Clippy, rustfmt, diff, and LOC gates pass. Grouped
extrusion entities, lifecycle, motion, and G-code remain later slices.

## Task 22O.90: monotonic fill entities

O90 ports the Monotonic/MonotonicLine part of pinned `Fill.cpp:1213-1374` and
`FillBase.cpp:133-155`. The graph-native layer pass now derives all O89 inputs
from grouped effective state, applies the pinned dense `3 × spacing` link gate
and MonotonicLine zero-anchor policy, and emits ordered role/flow collections.

Compile RED proved missing dispatch. Two focused, three O76, and two O89
regressions pass. Strict core Clippy, rustfmt, diff, and LOC gates pass. Remaining
fillers/thin fills, lifecycle, motion, and G-code remain later slices.

## Task 22O.91: layer fill entity stage

O91 ports pinned `Fill.cpp:1213-1384` ownership. Post-combination state now
materializes every object/layer through O76/O90 transactionally and advances the
public lifecycle before the explicit incomplete sink. Full traversal also ports
pinned O77 endpoint-overlap classification, O79 any-side vertical connectivity,
and the O80 zigzag reachability invariant assumed by source assertions.

Three O91 and O79/O80/O90 regressions pass. Strict core Clippy, rustfmt, diff,
and LOC gates pass. Thin fills, perimeter/fill ordering, motion, and G-code
remain later slices.

## Task 22O.92: thin fill append

O92 ports pinned `Fill.cpp:1376-1384`. O91 now moves each retained thin-fill
path/loop after generated fill collections, preserving source order, 3D points,
roles, flow metadata, and single ownership.

Compile RED proved missing ownership. The KSR oracle freezes 2,285 entities,
2,285 paths, and 5,401 points; all three O91 tests and strict core Clippy,
rustfmt, diff, and LOC gates pass. Island ordering, motion, and G-code remain
later slices.

## Task 22O.93: layer-region extrusion ownership

O93 ports pinned `Layer.hpp:43-76`. Each O92 layer output now owns retained
perimeter collections beside generated fills and moved thin fills, preserving
source tree/path/role/flow order and draining predecessor ownership.

Compile RED proved missing ownership. KSR freezes 2,881 collections, 5,243
loops, 5,483 paths, and 111,933 points. Three lifecycle/repeatability tests and
strict core Clippy, rustfmt, diff, and LOC gates pass. Island sorting/chaining,
motion, and G-code remain later slices.

## Task 22O.94: extrusion island assignment

O94 ports pinned `GCode.cpp:4970-5048` for KSR's single region/tool. Generated
fills, appended thin fills, then perimeter collections are assigned by first
point to ordered `lslices` through increasing bbox area, half-open bounds, and
contour containment, retaining the source fallback island.

KSR freezes 3,350 total and 2,881 nonempty islands, zero nonempty fallbacks,
1,658/2,285/2,881 fill/thin/perimeter entities, and a deterministic 1,835
perimeter-only / 1,046 mixed split. Three tests and strict core gates pass.
Multi-region/tool/wiping, chaining, motion, and G-code remain later slices.

## Task 22O.95: island print phase order

O95 ports pinned `GCode.cpp:5434-5470,6131-6148`. O94 islands flatten into
owned perimeter/fill/thin print entities; layer zero is always wall-first and
later layers use 3MF-derived `is_infill_first` (false for KSR).

Focused tests cover all phase branches. KSR freezes 3,350 islands, 2,881
nonempty/perimeter-first islands, and exact 2,881/1,658/2,285 entity counts.
Four tests and strict core gates pass. Infill greedy chaining/reversal,
multi-region/tool behavior, motion, and G-code remain later slices.

## Task 22O.96: pure infill chaining dependencies

O96 ports pinned `ShortestPath.cpp:15-40,92-393,1026-1069`,
`ExtrusionEntityCollection.cpp:65-72,87-96`,
`ExtrusionEntityCollection.hpp:78-123`, and `FillBase.cpp:161-185`. The reached
classic shortest-path seam now supports explicit-cursor constrained reversal,
source fallback ordering, fill/gap endpoint and reverse operations, and pure
`chained_path_from`. Monotonic variants own no-sort while CrossHatch remains
sortable.

Four O96 entity tests and all ten shortest-path regressions pass. KSR freezes
782 no-sort and 876 sortable collections with valid endpoints; strict core
gates pass.

O96 deliberately does not advance the public lifecycle or invent a cursor.
O95 activation with its real current position, multi-region role filtering,
motion, and G-code remain later slices.

## Task 22O.97: external seam candidate topology

O97 ports pinned `GCode/SeamPlacer.hpp:42-108`,
`GCode/SeamPlacer.cpp:229-273,406-592,1014-1038`, and
`ExtrusionEntity.hpp:507-512` as a pure source-native perimeter seam. External
loops, including mixed external/overhang loops, retain source `collect_points`
order and their corresponding region flow width; candidate polygons normalize
counter-clockwise and retain original winding through signed 0.4 mm-arm vertex
angles.

Five focused/KSR tests pass; KSR freezes 3,272 perimeters, 62,094 candidates,
and checksum `11805973356074762675`. Strict core gates pass. Runtime
visibility/penalties/selection/alignment/placement/clipping, cursor, O96
activation, motion, and G-code remain later slices.

## Task 22O.98: Arachne extrusion-line primitives

O98 ports pinned `Arachne/utils/ExtrusionJunction.hpp` and
`Arachne/utils/ExtrusionLine.hpp/.cpp:21-275`. Crate-private Rust types retain
junction width/perimeter identity, line metadata and mutation, source integer
length and thick-width layout, clockwise contour/area conventions, and the
source simplification and extrusion-area guards across active coordinate
scales.

Ten focused tests cover the accepted primitive boundary and strict core gates
pass. O98 is inactive:
half-edge/skeletal topology, beading, `WallToolPaths`,
`FillConcentricInternal`, variable-width entity materialization, lifecycle,
motion, and G-code remain later source-cited slices.

## Task 22O.99: Arachne beading strategies

O99 ports the pinned base, distributed, redistribute, widening, limited, outer-
inset, and factory files under `Arachne/BeadingStrategy/`. The crate-private
strategy stack preserves source integer/f32 rounding, odd/even thresholds,
transition metadata, thin-wall widening, fixed outer widths, zero-width limit
markers, signed optional inset, and source factory order across the active
coordinate scale. Ten focused source-worked tests, including full-expression
conversion order and the KSR-style 0.42 mm factory stack, and strict core gates
pass.

O99 remains inactive. Half-edge/skeletal topology, `WallToolPaths`, extrusion-
line production, `FillConcentricInternal`, variable-width entities, lifecycle,
motion, and G-code remain later source-cited slices.

## Task 22O.100: Arachne skeletal graph

O100 ports the pinned half-edge templates and skeletal edge, joint, and graph
sources into an inactive crate-private stable-index arena. It preserves weak
payload storage, twin/chain/incident topology, upward and local-maximum walks,
rib/node insertion, both small-edge collapse shapes, removal holes, and stable
identity. Thirteen source-worked tests cover payload lifetime, recursive
traversal, large-coordinate projection, insertion, collapse thresholds and
rewiring cap, and removal identity; strict core gates pass.

O100 remains inactive. The full skeletal trapezoidation builder and transition
stages, `WallToolPaths`, `FillConcentricInternal`, variable-width entities,
lifecycle, motion, and G-code remain later source-cited slices.

## Task 22O.101: Arachne skeletal trapezoidation builder

O101 ports the pinned `SkeletalTrapezoidation` constructor through initial
central/bead filtering. The inactive crate-private slice preserves ordered
polygon segment sites, inside-cell ranges, rounded Voronoi transfer,
point/segment and point/point discretization, O100 graph links and collapse,
pointy-node separation, central angles, recursive filters, and O99 beading
strategy calls at both active coordinate scales. Six source-worked tests are
written; focused nextest, formatting, and strict workspace clippy pass.

O101 stops before `generateTransitioningRibs`. Transition generation and later
skeletal stages, `WallToolPaths`, `FillConcentricInternal`, variable-width
entities, lifecycle, motion, and G-code remain later source-cited slices.

## Task 22O.103: Project G-code emission lifecycle

O103 activates the typed `.3mf` project route through a crate-private emitter
bounded by Orca `FillConcentricInternal.cpp`, `GCode.cpp`, and `GCodeWriter`
sources. It materializes concentric groups through the existing geometry kernel,
retains ordered prepared entities until emission, writes resolved header/config
and machine metadata, and emits perimeter/fill/thin entity paths. Focused
project lifecycle tests pass. Full WallToolPaths, placeholder evaluation,
seams, arcs, timing, motion parity, and exact golden output remain deferred.

## KSR FDM Test V4 complete G-code parity

Complete `slice_project` through OrcaSlicer 2.4.2 `GCode.cpp:4539-7110`,
`GCodeWriter.cpp`, `GCode/SeamPlacer.cpp`, the configured arc-fitting path, and
`GCode/GCodeProcessor.cpp:1100-1140`. Work proceeds as independently committed
option-driven slices: finite volumetric extrusion and role speeds; acceleration,
travel, retraction/lift/wipe; seam placement and arcs; object/layer/end templates;
timing/progress/statistics; then removal of obsolete internal source-pinning
oracles. Exit requires normalized byte-for-byte parity for the KSR project (only
the Ares generator name/timestamp may differ), workspace nextest, strict Clippy,
rustfmt, LOC/macro gates, and an approved independent six-axis runtime review.

## Task 22O.124: Monotonic configured direction

O124 corrects the active monotonic fill direction against OrcaSlicer 2.4.2
`FillBase.cpp:275-319` and `FillRectilinear.hpp:39-54`. Monotonic fill now
inherits the rectilinear zero `_layer_angle` behavior instead of applying the
generic odd-layer 90-degree alternation; resolved bridge angles remain active.
The focused direction contract and KSR project motion smoke test pass. Remaining
surface assignment, path ordering, gap fill, lifecycle, timing, and exact G-code
differences remain later source-cited parity slices.
