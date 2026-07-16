# Task 20A.1: Typed Profile Fragment, Inheritance, and Composition Migration

## Status and objective

This specification is a draft until its exact bytes receive the independent
Codex and default-model OpenCode approvals required below.

Task 20A.1 is the first bounded slice of Task 20A in the approved
`ksr_fdmtest_v4` parity program. It replaces the remaining dynamic JSON profile
fragment, inheritance, and composition consumers with sparse concrete typed
option groups. It removes exactly the 29 approved migration-baseline
fingerprints owned by `crates/ares-core/src/profiles/fragment.rs` and
`crates/ares-core/src/profiles/composition.rs`.

This task does not connect profiles to `slice_project`. The current project
path already obtains its typed `ProjectSettings` directly from the 3MF, and it
must continue doing so. A valid project must still return
`ProjectSlicingIncomplete` after the already-released Task 19C config-block
writer runs. Geometry, layer planning, and G-code output remain later tasks.

### Pre-implementation review contract

The independent approvals that freeze this specification are design reviews
performed before a Task 20A.1 implementation plan or implementation exists.
Reviewers must verify the fixed upstream claims, sparse-presence semantics,
typed destination, exact dynamic-debt boundary, error behavior, and acceptance
criteria.

A `REVISE` verdict must identify a concrete spec defect. Implementation
conformance is a later review gate. No production implementation may begin
until both this spec and its later implementation plan receive literal
`VERDICT: APPROVE` from an independent Codex agent and the required default
OpenCode model.

## Aggregate-plan ownership and ordering

The approved aggregate plan has the strict dependency chain:

```text
19C -> 20A -> 20B -> 20C -> 20D -> 20E
    -> 21A -> 21B -> 21C -> 22
```

Task 20A owns dynamic consumers under `options` and `profiles`. Task 20B owns
PrintApply, Task 20C owns retained STL geometry/planning consumers, Task 20D
owns retained G-code consumers, and Task 20E removes the final dynamic
`SliceOptions` compatibility shell. Task 22 owns
`SlicingParameters::create_from_config`, layer-profile generation, and project
mesh slicing.

Consequently this task must not introduce `SlicingParameters`, modify retained
STL planning, or connect the project path to `planning.rs`, `segments.rs`,
`contours.rs`, `print.rs::build_print_domain`, or `gcode.rs::format_gcode`.
Those changes would cross already-approved ownership and dependency gates.

The profile pair is the smallest closed Task 20A ownership set. Migrating only
`fragment.rs` or only `composition.rs` would require a new dynamic bridge
between them. Both files therefore belong to this one review unit.

## Superseded legacy contracts

The older M7 and M8 profile milestones predate the approved strict typed
project configuration. They required unknown-key preservation, a transparent
`SliceOptions` map, and JSON-level scalar/array flattening for multiple
filaments. Those requirements conflict with the later approved parity spec,
which requires concrete typed fields, unknown-key rejection, and an empty
dynamic-value baseline.

Task 20A.1 explicitly supersedes only those conflicting M7/M8 requirements:

- unknown profile option keys are rejected at the input boundary;
- no unknown-value side map is retained;
- no scalar/array shape is inspected dynamically;
- merged and composed output is not `SliceOptions`;
- profile tests no longer exercise the retained STL API through a dynamic
  composed map.

Same-kind inheritance, deterministic selection, child overrides, profile
identity accessors, filesystem independence, and the public in-memory profile
workflow remain supported through typed replacements.

## Fixed upstream rewrite boundary

All upstream citations in this task refer to OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `src/libslic3r/Preset.hpp:22-24` defines the `filament`, `process`, and
  `machine` profile kinds.
- `Preset.hpp:43-65` defines profile JSON identity and metadata keys including
  `version`, `name`, `type`, `from`, `setting_id`, `filament_id`, `inherits`,
  and `instantiation`.
- `src/libslic3r/Preset.cpp:491-504` removes keys outside the selected preset
  kind's default config and reports them to the caller.
- `Preset.cpp:1476-1494` defines the process, filament, and printer ownership
  lists; printer ownership includes the printer, machine-limit, and nozzle
  option sets.
- `Preset.cpp:1622-1703` loads one profile JSON config, obtains its metadata,
  finds a same-collection parent, starts from the parent or default config,
  applies child-present values, and normalizes the result.
- `Preset.cpp:3112-3140` resolves a direct parent and recursively resolves the
  base preset without accepting self-parenting.
- `src/libslic3r/PresetBundle.cpp:3884-4165` implements the selected FFF preset
  composition. Task 20A.1 ports only the `apply_extruder=false` and no
  `filament_maps_new` subset: typed defaults and selected profiles, typed
  vector append for multiple filaments, removal of colliding profile-local
  fields, selected-profile IDs, filament IDs/map/self-index, and ordered
  inheritance/compatibility groups.
- `PresetBundle.cpp:68-242::construct_full_config` is a different calibration
  path used at `src/slic3r/Utils/CalibUtils.cpp:937`. It does not have the same
  multi-filament metadata or machine-inheritance behavior and is explicitly
  not an owner for this task.
- `src/libslic3r/PrintConfig.hpp:610-682` is the dynamic upstream profile-load
  shell whose value-erased behavior is being replaced at the Rust boundary.
- `PrintConfig.hpp:695-914,916-1666` defines the concrete typed FFF config
  owners represented by Ares `PrinterOptions`, `ProcessOptions`,
  `FilamentOptions`, `ProjectRuntimeOptions`, and `ProjectSettings`.

The Rust rewrite includes only the in-memory behavior already exposed by Ares:
byte-supplied profile fragments, the current whole-field same-kind parent-chain
subset, selection of one machine and process plus one or more filaments, and
the cited `full_fff_config(false, std::nullopt)` composition subset into
existing typed groups. It does not port filesystem discovery, preset
collections, vendor bundles, cloud/UI state, or arbitrary upstream dynamic
config APIs.

## Current debt boundary and exact RED set

The committed `scripts/dynamic_value_baseline.txt` contains 712 findings before
this task. The Task 20A.1 ownership set is exactly the 29 sorted entries at
current baseline lines 684 through 712:

- 18 entries rooted at `profiles/composition.rs`;
- 11 entries rooted at `profiles/fragment.rs`.

The committed baseline is already in the audit's Rust
`BTreeSet<String>` ordinal order. The canonical ownership bytes are baseline
lines 684 through 712 encoded as UTF-8 without a BOM, joined with LF, and
terminated by one LF. Their SHA-256 is:

```text
373e1a695854439c94e33220b1fdd47c74bad5842fef4489ccc03408ced0fe55
```

Locale- or culture-sensitive sorting such as PowerShell `Sort-Object` is not
the canonical algorithm and must not be used to recompute this identity.

Before implementation, delete exactly these 29 baseline entries and run the
syntax-aware dynamic audit. RED is valid only when the audit fails and its
sorted actual findings equal this exact ownership set. Every other baseline
entry must remain byte-identical and covered. The implementation may not move
any finding to another file, wrapper, adapter, or crate.

GREEN requires all 29 findings to disappear from production syntax while the
remaining 683 baseline entries are unchanged. No allowlist entry is permitted.

## Rust destination and public API

The primary destination remains `crates/ares-core/src/profiles/`, split below
400 physical lines per Rust file. Expected ownership is:

- `profiles/fragment.rs`: public fragment identity and byte parser;
- `profiles/fragment/metadata.rs`: concrete profile-local metadata;
- `profiles/fragment/payload.rs`: private sparse kind-specific payload;
- `profiles/inheritance.rs`: index, cycle detection, and parent-first overlay;
- `profiles/composition.rs`: selection and public composed result;
- `profiles/composition/filament.rs`: typed multiple-filament append;
- `profiles/composition/metadata.rs`: typed cumulative output metadata;
- `profiles/mod.rs` and `lib.rs`: revised exports only.

Equivalent smaller file boundaries are allowed. The task may modify the
existing crate-private typed builders under `options/` only to add
presence-preserving clone/overlay/append operations. It must not create a
second set of option declarations or a flat 650-field config.

The public contract is equivalent to:

```rust
pub enum ProfileKind {
    Process,
    Filament,
    Machine,
}

pub struct ProfileFragment { /* identity + sparse typed payload */ }

pub struct MergedProfileMetadata { /* selected identity + inherited config metadata */ }

pub enum MergedProfile {
    Machine {
        metadata: MergedProfileMetadata,
        options: PrinterOptions,
    },
    Process {
        metadata: MergedProfileMetadata,
        options: ProcessOptions,
    },
    Filament {
        metadata: MergedProfileMetadata,
        options: FilamentOptions,
    },
}

pub fn merge_profile_fragments(
    fragments: &[ProfileFragment],
    target_kind: ProfileKind,
    target_name: &str,
) -> Result<MergedProfile, SliceError>;

pub struct ComposedProfile { /* ProjectSettings + typed profile metadata */ }

pub fn compose_profile_fragments(
    fragments: &[ProfileFragment],
    selection: &ProfileSelection,
) -> Result<ComposedProfile, SliceError>;
```

Exact private names may differ. The observable API requirements are fixed:

1. `ProfileFragment::from_json_bytes` remains the untrusted byte boundary.
2. `ProfileKind`, fragment name, direct parent, and supported metadata remain
   accessible without exposing builders or dynamic values.
3. `merge_profile_fragments` no longer returns `SliceOptions`; it returns a
   public tagged result. A caller can exhaustively match its kind and read the
   corresponding concrete `PrinterOptions`, `ProcessOptions`, or
   `FilamentOptions` plus merged metadata. An opaque all-private result is not
   compliant.
4. `ComposedProfile` exposes `settings()` / `into_settings()` returning
   `ProjectSettings`, selected profile-name accessors, and typed composition
   metadata. The old `options()` / `into_options()` map contract is removed.
5. No project, CLI, or WASM public slicing signature changes in this task.

## Direct typed fragment parsing

### Order-independent streaming decode

JSON object member order is not semantic, but kind-specific option dispatch
needs the `type` field. The parser therefore performs two bounded streaming
serde passes over the caller-provided bytes:

1. a metadata pass reads concrete profile-local fields and skips option
   payloads with `IgnoredAny`;
2. a kind-specific pass re-reads the same input and dispatches every non-local
   key directly into the matching concrete sparse builder.

The fragment stores neither the input bytes nor a generic syntax tree. It may
not use `serde_json::Value`, `serde_json::Map`, `RawValue`, `from_value`,
`json!`, a catch-all value enum, `Any`, a typed-erased map, or JSON
serialize/deserialize round-trips. Tests may use JSON literals as input bytes.

Both passes are subject to the existing project JSON nesting, string, and
resource limits where applicable. No filesystem access is introduced.

### Concrete metadata

Profile metadata is represented with explicit typed fields, not an option map.
`ProfilePresetMetadata` owns loader-local identity:

- required `type` and non-empty `name`;
- optional `from`, `version`, `setting_id`, `instantiation`, `description`,
  `url`, and `renamed_from`;
- optional `filament_id` for filament identity.

`ProfileConfigMetadataPatch` (or an equivalent private sparse typed struct)
owns the profile-config fields needed by inheritance and composition:

- optional `inherits`, where a missing or empty string means a root profile;
- for process profiles, sparse `compatible_printers` and
  `compatible_printers_condition`;
- for filament profiles, those two printer fields plus sparse
  `compatible_prints` and `compatible_prints_condition`;
- machine profiles accept none of the four compatibility fields.

Profile-local fields are not injected into an option builder. Duplicate local
fields, a non-string scalar where a string is required, an invalid string
vector, unsupported profile kind, or empty required name returns
`SliceError::InvalidInput`.

`inherits` is the selected fragment's direct-parent identity used by both the
inheritance resolver and the later cumulative group; it is not inherited from
the parent. Missing and explicit empty `inherits` are equivalent roots and
contribute an empty positional slot during composition.

The four compatibility fields are config metadata, not loader-local identity.
They participate in the same sparse parent-to-child overlay as concrete option
fields: child omission retains the parent value, while an explicit empty string
or empty vector clears it. The loader-local fields do not participate in that
overlay. For a filament fragment with a parent, the merged filament identity
uses the resolved parent's `filament_id`, matching the user-preset loader at
`Preset.cpp:1658-1684`; only a root retains its own `filament_id`.

`renamed_from` is parsed as typed metadata but alias lookup through it is
deferred. `version` remains the exact loader string; upstream Semver acceptance
and `instantiation` visibility behavior are preset-management concerns deferred
with alias lookup. `different_settings_to_system`, timestamps, vendor data,
and other preset-management fields are outside the current public Ares profile
workflow and are rejected unless an independently source-cited plan revision
adds a concrete typed field. They are not preserved as unknown data.

### Kind-specific ownership

After local metadata is consumed, every option key must belong to the selected
profile kind:

- `machine` dispatches only to sparse `PrinterOptions` builders;
- `process` dispatches only to sparse `ProcessOptions` builders;
- `filament` dispatches only to sparse `FilamentOptions` builders.

A key owned by another kind is an invalid misplaced profile option, not an
unknown value to retain. A key unknown to all three concrete owners is also an
input error. Duplicate concrete option assignments are rejected by the same
field-specific builder checks used by strict project deserialization.

Upstream `Preset::remove_invalid_keys` removes a misplaced key, reports it, and
continues loading. Ares has no substitution/report channel on this byte API;
returning `SliceError::InvalidInput` is the intentional stricter boundary
diagnostic. The accepted typed result is otherwise the same: the invalid key
never enters the profile or composed config.

Typed legacy key/value conversion may be shared only through the existing
compile-time typed conversion machinery. It may not materialize a dynamic
value. The three source-only cumulative legacy keys remain represented by the
typed composition metadata described below, not by reintroducing their old
dynamic spellings.

## Sparse inheritance semantics

Each fragment owns a sparse typed payload whose fields remain optional until
the complete same-kind chain is assembled. Resolving each fragment to defaults
before overlay is forbidden because child defaults would overwrite explicit
parent values.

For `(kind, name)` resolution:

1. Build a deterministic unique index over all fragments.
2. Treat missing or empty `inherits` as a root; otherwise follow only a
   same-kind parent link.
3. Reject a missing target, missing parent, cross-kind-only parent, duplicate
   `(kind, name)`, self-parent, or any longer cycle.
4. Overlay the oldest parent first and selected child last.
5. For each concrete field, replace the accumulated value only when that field
   is explicitly present in the child sparse payload.
6. Resolve defaults exactly once, after the final child overlay.

Input fragment order must not affect output. Overlay is generated from the
same compile-time field declarations as deserialize/resolve; it must not use a
runtime key registry, field-name lookup, serialization, or equality against a
default value to infer presence.

The selected fragment's loader-local identity remains the public merged
identity. The four compatibility fields follow the same presence overlay as
concrete options. Direct-parent identity remains the selected child's
`inherits`, not an accumulated chain rewritten into a scalar.

This is the whole-field inheritance behavior supported by the existing Ares
profile API. Upstream `update_diff_values_to_child_config` additionally maps
variant-indexed vector slots and treats nullable child elements as per-element
inheritance markers. That deeper nil/variant diff behavior is not silently
approximated here; it is explicitly deferred with the other profile variant
work. A child vector that is present in Task 20A.1 replaces the complete parent
vector while preserving each concrete nil/value element exactly.

## Typed full composition

### Group assembly

`compose_profile_fragments` resolves the selected profiles independently, then
constructs a `ProjectSettings` from existing concrete owners:

1. existing typed owner defaults provide the fixed FFF defaults;
2. selected process contributes `ProcessOptions`;
3. selected machine contributes `PrinterOptions`;
4. selected filaments contribute one composed `FilamentOptions`;
5. `ProjectRuntimeOptions` starts from its concrete defaults and receives only
   the selected-profile fields explicitly listed below;
6. the three ordinary project `PresetMetadata` fields are not populated from
   profile-local identity unless an existing typed destination has the same
   upstream meaning.

The process, machine, and filament Rust owners are disjoint, so construction
does not simulate upstream application order with a shared map. Profile-local
collisions are held separately and never enter those owners. No option moves
between owners. No field is converted to JSON or a string token for
composition.

Project-config application at `PresetBundle.cpp:3892` is deferred because the
existing in-memory profile API accepts no project config and the active 3MF
project path already has its own typed resolver. This task must not invent an
optional project-map argument.

### Multiple filament composition

For one filament, use its fully inherited typed `FilamentOptions` directly.

For multiple filaments, iterate the compile-time filament field declarations.
Every vector-valued filament option is appended in `ProfileSelection` order
using its concrete element type. Scalar fields, if any exist in the fixed typed
owner, use the first selected filament as in the cited upstream scalar branch.
Empty vectors remain empty contributions; nullable elements retain nil/value
identity; enums retain their concrete type. No value is flattened by runtime
shape inspection.

The append implementation is generated or written beside the concrete field
declaration. A generic value vector, serde serializer, config-token serializer,
runtime key iteration, or reference-derived table is forbidden.

Extruder-variant reshaping and printer-extruder remapping in the cited upstream
functions are deferred from this profile API. This task implements only the
`apply_extruder=false` append branch. Existing Task 19B project variant
materialization remains unchanged, and no profile composition result is wired
into it during Task 20A.1.

### Typed composition metadata

`ProfileGroupMetadata` (or an equivalent explicitly named struct) stores
profile-only results that are not members of the 650 real project option
groups. Each group is a concrete optional string-vector field so absence can
be distinguished from a present positional vector. It includes:

- `inherits_group` in process, selected-filament, machine order;
- `compatible_machine_expression_group` in process then selected-filament
  order;
- `compatible_process_expression_group` in selected-filament order.

The vectors preserve one positional slot per cited upstream contributor. They
are absent as a serialized/exported group only when every slot is empty; they
are not compacted by removing interior empty strings. This preserves the
upstream relationship between entries and selected profiles.

Existing real typed project fields are set directly:

- `print_settings_id` is the selected process name;
- `printer_settings_id` is the selected machine name;
- `filament_settings_id` is the selected filament-name vector;
- `filament_map` contains one `1` per selected filament;
- `filament_ids` preserves one slot per selected filament in selection order,
  including an empty string when a selected profile has no ID;
- `print_compatible_printers` receives the selected process vector when
  at least one slot is non-empty, preserving every slot, and otherwise remains
  its concrete empty default;
- `filament_self_index` contains one-based selected-filament identities: for a
  single filament every `filament_extruder_variant` slot is `1`; for multiple
  filaments each selected index is repeated for that filament's complete
  variant-vector length after typed append.

Profile-local `inherits`, compatibility conditions, compatible lists, and
identity fields do not remain duplicated inside machine/process/filament option
owners. `different_settings_to_system` computation is deferred because it
requires preset collection dirty-diff state not supplied by this API.

The post-composition support/wipe/feature-filament clamps, bundle-owned
`extruder_ams_count`, and forced `printer_technology=FFF` tail are outside this
bounded profile-consumer migration. They are recorded as explicit deferrals,
not copied into `profiles/` as partial normalization logic.

## Error and atomicity contract

All failures are `SliceError::InvalidInput` with stable context identifying a
profile or option category; tests must not freeze incidental serde wording.

Parsing, inheritance, and composition are atomic. An error returns no partial
fragment, sparse builder, merged group, `ProjectSettings`, or metadata. Public
input slices and fragments are not mutated.

The following must fail:

- malformed or non-object JSON;
- missing/duplicate/invalid `type` or `name`;
- duplicate concrete or local fields;
- unknown or misplaced option keys;
- malformed concrete typed values;
- duplicate fragment identity;
- missing/cross-kind parent and inheritance cycle;
- missing selected machine/process/filament;
- empty process, machine, filament list, or filament name.

## Required TDD acceptance

### Baseline RED

1. Freeze the exact 29-entry Rust-ordinal/LF ownership bytes and their
   SHA-256.
2. Delete exactly those rows from `scripts/dynamic_value_baseline.txt`.
3. Run `cargo +1.91.0 nextest run -p ares-core --test
   no_unapproved_dynamic_values`.
4. Require nonzero exit and exact finding-set equality with the frozen set.

### Behavioral RED

Before implementation, active tests must fail to compile or fail behaviorally
against the old map API. They cover:

- metadata-first and `type`-last JSON member order;
- process, filament, and machine direct typed decode;
- missing and explicit empty `inherits` both resolving as roots and later
  contributing positional empty slots;
- unknown, misplaced, duplicate, and malformed-key rejection;
- grandparent/parent/child presence overlay and child override;
- inherited value retained when the child omits it;
- child explicit fixed default still overriding a different parent value;
- present child vector replaces the whole parent vector while preserving typed
  nullable elements;
- child omission retaining inherited compatibility lists/conditions, while an
  explicit empty value clears the parent value;
- child-with-parent filament identity follows the resolved parent's
  `filament_id`, while a root retains its own ID;
- exhaustive `MergedProfile` matching exposing identity, merged metadata, and
  the kind-correct concrete option group;
- input-order independence and every inheritance error;
- single-filament typed composition;
- two-filament append for numeric, bool, string, enum, nullable, empty, and
  special typed vector representatives;
- selected IDs/map and non-compacted positional metadata groups;
- empty and all-empty `filament_id` slot preservation;
- single- and multi-filament `filament_self_index` from concrete variant-vector
  cardinalities;
- proof that composed output is `ProjectSettings`, not `SliceOptions`;
- proof that profile production modules contain no dynamic/erased value path.

Tests may build input bytes with JSON text, but they may not inspect private
builders or use the reference G-code as a behavioral oracle.

### GREEN gates

Focused GREEN requires:

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/profile/)'
cargo +1.91.0 nextest run -p ares-core config_export
cargo +1.91.0 nextest run -p ares-core project
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
```

The dynamic audit must pass with exactly 683 retained baseline rows and no
allowlist addition. A structural scan must prove production files under
`profiles/` contain none of:

```text
serde_json::Value
serde_json::Map
RawValue
from_value
json!
SliceOptions
.values()
BTreeMap<String, Value>
Any
```

The scan is syntax-aware where an ordinary text match would report comments or
tests. New production code may use `BTreeMap` only for the typed fragment index
whose value is an integer position; it may not store option payloads.

Task 19C's committed KSR config block remains exactly 49,004 bytes with SHA-256
`b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`.
The project caller still returns `ProjectSlicingIncomplete`.

## Explicit deferrals

Task 20A.1 defers:

- every remaining dynamic consumer under `options/` to later Task 20A slices;
- PrintApply typed diffs to Task 20B;
- retained STL planning/geometry consumers to Task 20C;
- retained STL G-code consumers to Task 20D;
- final `SliceOptions` map/deserializer/parser deletion to Task 20E;
- coordinate, polygon, and Clipper ports to Tasks 21A-21C;
- `SlicingParameters`, layer profiles, and project mesh slicing to Task 22;
- project profile application and profile-to-3MF caller wiring;
- project-config overlay at `PresetBundle.cpp:3892` because this in-memory API
  has no project-config input;
- non-default `filament_map`, `filament_maps_new`, and partial resize
  preservation;
- every `apply_extruder=true` branch, including printer/process/filament
  `update_values_to_printer_extruders`, map-selected variants, and first-value
  multi-filament vector projection;
- per-element nullable inheritance and variant-indexed
  `update_diff_values_to_child_config` mapping beyond the included whole-field
  sparse overlay;
- `different_settings_to_system` dirty-diff computation and its positional
  group;
- post-composition support/wipe/feature-filament selector clamps;
- bundle-owned `extruder_ams_count` and forced printer-technology tail;
- the behaviorally different calibration-only `construct_full_config` path;
- filesystem resource discovery, vendor bundle loading, aliases/renames,
  substitutions, compatibility expression evaluation, cloud/UI state,
  dirty-setting computation, SLA profiles, and preview rendering;
- geometry, extrusion, G-code assembly, statistics, post-processing, and final
  KSR byte parity.

Deferral does not authorize a dynamic fallback for behavior included above.

## Verification and review gates

After focused GREEN, the implementation must pass:

```powershell
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets --all-features -- -D warnings
cargo +1.91.0 check --workspace --all-targets --all-features
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
```

The existing release-WASM, fresh wasm-bindgen, npm audit, and real-project
headless-browser gate also run before release. Fixture hashes, source-pinning
cleanup, forbidden hardcoding/reference scans, diff checks, and the under-400
physical-LOC rule remain mandatory.

Subagent-Driven implementation uses fresh TDD implementers for bounded slices:

1. sparse typed builder overlay primitives;
2. direct typed fragment parsing and inheritance;
3. typed filament append and full composition;
4. obsolete map-contract test/API cleanup and exact baseline closure.

Each slice receives an independent spec-compliance review and code-quality
review. Any finding is fixed with a fresh failing or mutation test where
applicable and reviewed again until literal `VERDICT: APPROVE`.

After all slices pass, fresh independent whole-spec and whole-code reviews and
the required default-model OpenCode implementation review must approve the
same frozen implementation bytes. Review evidence records a complete manifest
and hashes; it does not add executable source-pinning tests.

## Documentation, commit, push, and release

Only after whole implementation approval may tracked architecture/roadmap
documentation be updated. A fresh independent documentation reviewer must
return literal `VERDICT: APPROVE`. Then rerun the complete release gate from
the approved documentation bytes.

Stage only the frozen manifest, use a reviewed Conventional Commit, push the
current `codex/ksr-fdmtest-v4-parity` branch without force, verify local,
tracking, and direct remote SHA equality plus clean status, and require every
Tier 1 job green for that exact pushed SHA before Task 20A.1 is released.

The release record must state that the 29 profile fingerprints are gone while
the remaining compatibility baseline, Task 20A option consumers, Tasks
20B-20E, geometry, slicing, G-code, and complete KSR parity remain open.
