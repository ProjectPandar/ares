# Task 20A.2: Typed Filament Variant-Aware Inheritance

## Status and objective

This specification is a draft. No implementation plan or production change may
begin until the exact spec bytes receive the independent Codex and default-model
OpenCode approvals required at the end of this document.

Task 20A.2 is the next bounded Task 20A slice after released commit
`e0c50564283744b3dd3388eeaa10f624a492ff1f` (Tier 1 run `29488449752`). It
replaces Task 20A.1's whole-vector behavior for the exact filament
variant-aware option family with source-faithful, concrete typed inheritance.
The oldest filament profile is resolved once against typed defaults; every
descendant then applies sparse non-variant fields by whole replacement and the
variant family by slot mapping.

This task does not wire profiles into `slice_project` or advance geometry/G-code.
The released Task 19C path continues returning `ProjectSlicingIncomplete`.

## Fixed upstream rewrite boundary

All upstream citations refer to OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `src/libslic3r/PrintConfig.cpp:8375-8415` defines the exact 37-key
  `filament_options_with_variant` family.
- `src/libslic3r/Preset.cpp:922-945` selects
  `filament_extruder_variant`, that 37-key stride-1 family, no filament
  extruder-ID key, and an empty stride-2 family for filament presets.
- `src/libslic3r/Preset.cpp:231-278::extend_default_config_length` derives the
  filament cardinality from `filament_extruder_variant`, optionally resets
  upstream-`is_nil` non-override vectors from defaults, and resizes every
  present member of the variant family.
- `src/libslic3r/PrintConfig.cpp:63-84` defines the exact 16
  `filament_extruder_override_keys`; they coincide exactly with the 16 typed
  `FilamentRetractOverrideOptions` fields in this task.
- `src/libslic3r/Preset.cpp:1679-1685` starts an inherited preset from its
  already concrete parent, extends the child config to default lengths, and
  applies `update_diff_values_to_child_config`; lines `1693-1697` apply a root
  to the concrete default preset. Therefore an omitted root option contributes
  its typed default to later child diffing rather than remaining sparse.
- `src/libslic3r/PrintConfig.cpp:10209-10297` builds the current-to-child
  variant mapping; lines `10265-10267` skip an equal source/target field before
  assignment, identity keys are skipped, and lines `10281-10293` whole-assign
  the child vector when source length does not equal mapping length times stride.
- `src/libslic3r/Config.hpp:558-580` defines `set_only_diff`: a mapped child
  slot replaces its source slot only when the child slot is not nil.
- `src/libslic3r/Config.hpp:624-665` defines vector resize: zero clears,
  shrinking truncates, and growth duplicates the first value (or the supplied
  default's first value when the vector is empty).
- `src/libslic3r/Config.hpp:282-284,837,1015,1878` makes empty float, integer,
  and bool vectors `is_nil()` even when non-nullable; strings retain the base
  false result. This is observable during root default reset.
- `src/libslic3r/Config.hpp:921-931,1217` defines type-directed vector equality:
  nullable float and percent values use `is_approx`, while non-nullable floats
  use exact vector equality. `src/libslic3r/libslic3r.h:52,306-310` fixes
  `EPSILON` at `1e-4` and makes the comparison boundary strict.

Ares ports only the filament, no-ID, stride-1 subset, including root/child
normalization. An external empty vector that would trigger an upstream assert
or missing first-element read returns `SliceError::InvalidInput` with its key.

## Exact typed inventory

The inventory is one mapping identity plus 36 data fields. It is fixed by the
upstream set and grouped by the four existing Ares typed filament owners.

### `FilamentGCodeSourceOptions` (1 identity + 9 data)

Identity: `filament_extruder_variant`. Data: `filament_flow_ratio`,
`filament_max_volumetric_speed`, `long_retractions_when_ec`,
`retraction_distances_when_ec`, `filament_flush_volumetric_speed`,
`filament_flush_temp`, `filament_cooling_before_tower`,
`volumetric_speed_coefficients`, `filament_adaptive_volumetric_speed`.

### `FilamentPrintSourceOptions` (7 data)

`nozzle_temperature_initial_layer`, `nozzle_temperature`,
`activate_air_filtration`, `activate_air_filtration_during_print`,
`activate_air_filtration_on_completion`, `during_print_exhaust_fan_speed`,
`complete_print_exhaust_fan_speed`.

### `FilamentRegionSourceOptions` (4 data)

`filament_ironing_flow`, `filament_ironing_spacing`,
`filament_ironing_inset`, `filament_ironing_speed`.

### `FilamentRetractOverrideOptions` (16 data)

`filament_retraction_length`, `filament_z_hop`, `filament_z_hop_types`,
`filament_retract_lift_above`, `filament_retract_lift_below`,
`filament_retract_lift_enforce`, `filament_retract_restart_extra`,
`filament_retraction_speed`, `filament_deretraction_speed`,
`filament_retraction_minimum_travel`,
`filament_retract_when_changing_layer`, `filament_wipe`,
`filament_wipe_distance`, `filament_retract_before_wipe`,
`filament_long_retractions_when_cut`,
`filament_retraction_distances_when_cut`.

No option outside this inventory gains variant-slot semantics in this task.

## Required inheritance semantics

### Concrete normalized root

The deterministic same-kind chain resolver remains authoritative. Machine and
process chains keep Task 20A.1 whole-field overlay. For a filament chain:

1. Start from existing typed `FilamentOptions` defaults, whole-apply the
   oldest/root sparse builder, and resolve once to a concrete accumulator.
2. Let `N` be that result's `filament_extruder_variant` length.
3. Normalize the identity and all 36 data vectors to `N`: `N == 0` clears;
   excess elements are truncated; a nonempty short vector grows by duplicating
   its first element.
4. Before resize, reset a non-override field to its typed default when Orca's
   type-directed `is_nil()` is true. This includes every all-`Nil` nullable
   vector (including empty) and an empty non-nullable float, integer, or bool
   vector. In this fixed inventory, empty `filament_max_volumetric_speed` and
   every empty Print-owner field therefore reset from defaults.
5. The 16 override vectors—the complete Retract owner inventory above—never
   reset and retain their current `Nil` or empty representation. String vectors
   such as `volumetric_speed_coefficients` are not `is_nil()` merely because
   they are empty.
6. If `N > 0` and a vector remains empty after the permitted default reset,
   return `SliceError::InvalidInput` naming the option instead of asserting or
   reading a missing first element.

Thus omitted root fields participate as normalized typed defaults. An explicit
all-nil non-override root vector does not erase its default, while an all-nil
Retract override remains an inheritance marker exactly as upstream requires.
At positive cardinality, an empty root string or Retract vector is invalid.

### Sparse descendant pre-normalization

Apply each descendant sparse builder in chain order without resolving it. An
omitted field does nothing. A present non-variant field retains Task 20A.1
whole-field replacement. Before mapping a descendant:

- let `M` be its explicit `filament_extruder_variant` length, or one when the
  identity is omitted;
- normalize every present member of the exact 37-key family, including a
  present identity, to `M` by the same clear/truncate/duplicate-first rule;
- do not reset descendant all-nil vectors from defaults
  (`set_nil_to_default == false` upstream);
- if `M > 0` and a present vector is empty, return
  `SliceError::InvalidInput` naming it. A short nonempty vector is grown, not
  rejected, and an overlong vector is truncated. This includes child fields
  whose empty root counterpart would have reset from typed defaults.

An explicit zero-length identity gives `M == 0` and clears present family
vectors. It is an `InvalidInput` only when slot application would read missing
child slot zero after `source.len() == mapping.len()`; the source-length
fallback is tested first and does not read a child slot. No descendant default
may be materialized for an omitted field.

Before mapping or assignment, directly compare every normalized child-present
non-identity field with its accumulated concrete source. Vector lengths must
match. For every fixed-inventory `Vec<Nullable<OrcaFloat>>` and
`Vec<Nullable<Percent>>`, `Nil` equals only `Nil`; two concrete slots are equal
exactly when `abs(source - child) < 1e-4`. A delta exactly `1e-4` is unequal.
Every other fixed-inventory type uses exact equality, including non-nullable
`OrcaFloats`, `SpaceTuple` strings, and nullable integer, bool, and enum vectors.
This type-directed comparison is local to variant inheritance and must not
change the option types' global derived `PartialEq`. An equal field is a no-op;
only an unequal field proceeds to replacement, fallback, or slot mapping.

### Mapping identity

After pre-normalization, build one mapping from the accumulated/root identity
to the descendant's explicitly present identity:

- with `N > 0` accumulated variants, create `N` initially unmatched slots;
- with no accumulated variants, create the upstream singleton mapping `[0]`;
- when the descendant identity is omitted or explicitly empty, map only source
  slot zero to child slot zero;
- otherwise, for each accumulated variant in source order, select the first
  equal descendant variant in child order; a source variant with no equal child
  remains unmatched.

There is no filament extruder-ID tie-breaker in this task. Matching is exact
string equality. Reordering therefore redirects child slots into accumulated
source order. Unmatched source variants retain their accumulated values;
child-only variants are ignored.

The descendant identity is mapping input only and never overwrites the
accumulator. Every later descendant and the final merged result therefore keep
the normalized root identity.

### Variant data application

For each unequal, explicitly present one of the 36 data fields:

1. Let `source` be the concrete accumulated vector and `mapping` the mapping
   above. The included filament family has stride one.
2. If `source.len() != mapping.len()`, whole-replace source with the normalized
   child vector per `PrintConfig.cpp:10281-10293`.
3. Otherwise, an unmatched mapping keeps source. A mapped child slot replaces
   it according to the nullable rule below.
4. For a nullable field, `Nullable::Nil` at the mapped child slot inherits the
   accumulated source slot. `Nullable::Value` replaces it. For a non-nullable
   field, the mapped concrete child slot replaces it.

For root cardinality `N > 0`, child-only variants have no effect and root
normalization makes the source-length fallback unreachable through a
well-formed public chain. The typed helper nevertheless retains that fallback
and receives a direct crate-private test.

For `N == 0`, root normalization clears the identity and all family vectors,
while mapping remains the upstream singleton `[0]`. A descendant-present data
field therefore first sees `source.len() == 0 != 1` and whole-copies its
pre-normalized child vector; identity remains empty and omitted fields remain
accumulated. Later descendants still map from the empty identity. A source
field whose length differs from one whole-copies again; one whose length is one
uses slot-zero nil/concrete semantics. Thus an explicit empty child identity
does not read a missing slot when fallback applies, but is invalid when source
and mapping lengths are equal and slot zero is absent.

The existing `present_nullable_vector_replaces_the_whole_parent_vector` test is
superseded: its three-element root field is first truncated to the one-element
default identity, and its two-element child field is first truncated to the
implicit child cardinality one. The mapped child `Nil` then inherits the root's
first value. Rename or rewrite it to expect
`[Nullable::Value(OrcaFloat(0.9))]`; it must no longer assert whole-child
replacement. The neighboring
`omitted_nullable_vector_retains_the_parent_value` regression is also
superseded: its two-element root field is truncated to the same implicit
one-element identity before the child omission retains it. Rename or rewrite
it to expect `[Nullable::Value(OrcaFloat(0.9))]`. JSON type/shape errors remain
`SliceError::InvalidInput`.

## Typed implementation constraints

The implementation must operate on the existing concrete option types and
sparse builders. Compile-time field declarations or concrete owner methods may
generate the repetitive assignments, but production behavior must not use:

- a runtime key registry or string-key dispatch loop;
- `serde_json::Value`, another value-erased map, or a dynamic adapter;
- serialization/deserialization or config-token round-trips;
- equality against typed defaults to infer sparse presence. Sparse `Some`
  proves presence first; the required direct accumulated-source versus
  normalized-child equality short-circuit is permitted only afterward;
- the KSR fixture name, fixture values, reference G-code, or fixture hashes as
  behavioral branches.

The option-group declaration machinery may add a zero-cost operation that
applies only `Some` builder fields into an existing concrete group. Each typed
filament owner must remove its variant fields from that ordinary present-field
application and apply them through the rules above. The identity must likewise
be consumed as mapping input without ordinary assignment.

No public API signature changes are required. No executable test may pin an
OrcaSlicer source path, source text, or line range; the upstream pin belongs in
this reviewed specification and implementation evidence only.

## Exact production and test scope

The implementation manifest is restricted to these tracked production files:

- `crates/ares-core/src/options.rs`
- `crates/ares-core/src/options/option_group.rs`
- `crates/ares-core/src/options/filament_options.rs`
- `crates/ares-core/src/options/filament_options/gcode_source.rs`
- `crates/ares-core/src/options/filament_options/print_source.rs`
- `crates/ares-core/src/options/filament_options/region_source.rs`
- `crates/ares-core/src/options/filament_options/retract_overrides.rs`
- `crates/ares-core/src/profiles/inheritance.rs`
- `scripts/dynamic_value_baseline.txt`

The test manifest may modify or add only:

- `crates/ares-core/src/profiles/tests/mod.rs`
- `crates/ares-core/src/profiles/tests/inheritance.rs`
- `crates/ares-core/src/profiles/tests/filament_variant_inheritance.rs`

The existing inheritance file changes only to replace the two superseded
one-element normalization expectations described above; new coverage belongs
in the new focused test module.

The following obsolete files are deleted exactly:

- `crates/ares-core/src/options/update_diff_values_to_child_config.rs`
- `crates/ares-core/src/options/update_diff_values_to_child_config/tests.rs`
- `crates/ares-core/src/options/update_diff_values_to_child_config/tests/full_update.rs`

The deleted module is private dynamic scaffolding: `options.rs` declares it,
but no non-test production caller imports or invokes its private functions.
Only its two test files exercise those functions. The new typed profile path
supersedes that scaffold, so adapting or retaining it would create the dynamic
fallback this task forbids.

No other production or test path is in scope. If compilation proves an
additional path indispensable, the spec and both spec approvals must be
revised before implementation touches it. Rust files must remain below 400
physical lines; use existing file boundaries rather than creating a generic
variant subsystem. At the frozen base, the largest touched Rust files are the
existing inheritance test (353 lines), `options.rs` (319), inheritance logic
(252), and GCode owner (231); new coverage stays in the focused test file so
the listed boundaries have sufficient headroom.

## Exact dynamic-debt closure

Before production implementation, remove exactly these eight sorted baseline
rows (current lines 395-402):

```text
crates/ares-core/src/options/update_diff_values_to_child_config.rs#apply_diff_values_to_child_config@1|path|serde_json::Value::Array
crates/ares-core/src/options/update_diff_values_to_child_config.rs#crate@1|use|serde_json::Value
crates/ares-core/src/options/update_diff_values_to_child_config.rs#json_vector@1|type|&serde_json::Value
crates/ares-core/src/options/update_diff_values_to_child_config.rs#json_vector@1|type|Result<Vec<serde_json::Value>,SliceError>
crates/ares-core/src/options/update_diff_values_to_child_config.rs#nullable_json_vector@1|type|&serde_json::Value
crates/ares-core/src/options/update_diff_values_to_child_config.rs#nullable_json_vector@1|type|Result<Vec<Option<serde_json::Value>>,SliceError>
crates/ares-core/src/options/update_diff_values_to_child_config.rs#optional_int_vector@1|type|Option<&serde_json::Value>
crates/ares-core/src/options/update_diff_values_to_child_config.rs#optional_string_vector@1|type|Option<&serde_json::Value>
```

Their canonical UTF-8/LF bytes, including the final LF, have SHA-256
`93ee0515d6afb622094a9d7ca4b24753f63e15e822e01d3c6c6222ecb3a87fb0`.
The syntax-aware audit must then fail RED with exactly those eight actual
findings while the source module still exists. GREEN deletes the scaffold and
leaves exactly 675 baseline rows. Every other baseline row stays byte-identical.
`scripts/dynamic_value_allowlist.toml` remains unchanged and empty.

## TDD acceptance

### RED tests

Fresh tests must fail against Task 20A.1 and cover at least:

- a multi-variant root with an omitted family field expanding its typed default
  before a reordered child is mapped;
- an all-nil non-override root vector resetting from typed defaults, and an
  all-nil Retract override root vector retaining nil;
- root and child grow-by-first, truncate, zero-cardinality, and positive-target
  empty-vector `InvalidInput` behavior;
- at positive root cardinality, empty non-nullable GCode float plus
  representative Print int/bool fields reset from typed defaults, while empty
  `SpaceTuple` and Retract fields are invalid; corresponding child empties stay
  invalid rather than resetting;
- a public `N == 0` root → child → later-descendant chain: identity stays
  empty, a present nullable field transitions `[] -> [Value(1.2)]` by fallback,
  then child `Nil` retains `[Value(1.2)]`, while an omitted family field stays
  at its accumulated value;
- omitted descendant data retaining the concrete parent value;
- reordered identities mapping values into accumulated/root order while the
  descendant identity does not overwrite that order;
- equality short-circuit under reversed identities: exact-equal data retains
  source order; representative nullable float and percent deltas below `1e-4`
  also retain source order, while a delta exactly `1e-4` is unequal and maps;
- a representative non-nullable `OrcaFloats` delta below `1e-4` remains exact,
  proceeds through mapping, and therefore proves it did not use approximation;
- unmatched source and child-only variants;
- representative non-nullable mapping for GCode and Print owners, plus mapped
  nullable `Nil` inheritance and concrete override for Region and Retract;
- a structural assertion that the exact 1+36 inventory is owned;
- a present non-variant filament field retaining Task 20A.1 whole replacement;
- a later descendant still mapping against the retained root identity;
- the two rewritten one-element normalization regressions replacing the
  present-child whole-vector and omitted-child unnormalized-parent assertions;
- the source-length mismatch whole-child fallback through a direct
  crate-private typed-helper test.

Tests use small synthetic profile bytes. They must not call OrcaSlicer, inspect
the KSR reference G-code, or encode fixture-specific branches.

### GREEN and regression gates

Focused GREEN requires:

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/filament_variant_inheritance/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/profile/)'
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
cargo +1.91.0 nextest run -p ares-core config_export
cargo +1.91.0 nextest run -p ares-core project
```

The audit must report 675 retained rows and no allowlist addition. Structural
review must prove that the new production path contains no dynamic value,
runtime key-registry, serde round-trip, or fixture hardcode. The released KSR
config block remains 49,004 bytes with SHA-256
`b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`,
and valid project slicing still ends at `ProjectSlicingIncomplete`.

After focused GREEN, run the full release matrix:

```powershell
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets --all-features -- -D warnings
cargo +1.91.0 check --workspace --all-targets --all-features
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
```

The existing release-WASM, fresh wasm-bindgen, npm audit, and real-project
headless Chromium gates also remain mandatory.

## Explicit deferrals

Task 20A.2 defers all of the following without authorizing a fallback:

- process and printer variant-aware profile inheritance;
- every stride-2 option family;
- all `apply_extruder` and extruder-remapping composition branches;
- project-to-profile application and `slice_project` profile wiring;
- the remaining Task 20A slices and Tasks 20B, 20C, 20D, and 20E;
- geometry, layer planning, extrusion, G-code generation, metadata byte parity,
  and complete `ksr_fdmtest_v4` parity.

## Independent review, documentation, and release gates

Subagent-Driven implementation must split the accepted behavior into bounded
TDD assignments. Each assignment receives fresh independent spec-compliance
and code-quality review until both return literal `VERDICT: APPROVE`. The
frozen whole implementation then requires independent whole-spec,
whole-quality, and default-model OpenCode implementation approvals.

Only after whole-implementation approval may
`docs/architecture/option-parity-v4.md` and `docs/roadmap.md` be updated. A
fresh documentation reviewer must return literal `VERDICT: APPROVE`, after
which the complete release matrix is rerun from the approved documentation
bytes.

Stage only the frozen manifest, create a reviewed Conventional Commit, push
`codex/ksr-fdmtest-v4-parity` without force, verify local/tracking/direct remote
SHA equality and a clean worktree, and require every Tier 1 job green for that
exact pushed SHA before Task 20A.2 is released.

**Status: DRAFT — an implementation plan is forbidden until independent Codex
and default-model OpenCode spec reviewers both return literal
`VERDICT: APPROVE` for these exact bytes.**
