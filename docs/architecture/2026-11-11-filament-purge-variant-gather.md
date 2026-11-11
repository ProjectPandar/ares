# Gather the source-owned filament purge variant

## Source boundary and destination

This follow-up completes the static filament payload gathering of upstream
`345efec64c7b95a05ce5cf96df0dde66a158835c`. It is not an Ares-owned configuration
pipeline or an exporter correction:

- `PrintConfig.cpp:10210` includes `filament_minimal_purge_on_wipe_tower` in
  `filament_options_with_variant`.
- The static branch of `PrintApply.cpp:1435–1455` passes that owning set plus
  `filament_self_index` to
  `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments`.
- `PrintConfig.cpp:11957–11979` dispatches each owned payload; `:11973` gathers
  `coFloats` through the existing variant indices, using the shared gather at
  `:11862–11882`.
- `Print.cpp:3171–3195` reapplies the selected filament payloads during refresh.

The only Rust implementation destination is
`crates/ares-core/src/options/project_variants/filament.rs::materialize`.
Add the existing `filament.gcode.filament_minimal_purge_on_wipe_tower` field to
its existing `select_wrapped_field` dispatcher, using the supplied source and
`indices`. That dispatcher remains a temporary typed compatibility shell for
the upstream owning-set gather. Both initial materialization and refresh call
it; neither caller nor the gather/index algorithm changes.

## Why owning-list membership alone was insufficient

The independent membership slice at `db98fe657` completed the63-member owning
list and corrected all six Native-first public import vector witnesses. Its
exact-tip25-case official/seeded replay nevertheless ran8 passes and17 failures
per mode. The new C tests reached strict comparison after their public import
assertions passed. Purge output retained three slots instead of Native's one;
affected H2C output retained18 slots instead of Native's three.

The purge field already exists as `OrcaFloats` with default15 in
`options/filament_options/gcode_source.rs:112`, and its serialization already
exists. The typed gather dispatcher includes the other owning members but
omits purge. The source audit records62/63 member field accesses; lexical
coverage itself is not behavioral proof of the other selectors. Membership
normalization now exposes that missing gather rather than fixing it.

The actual executed pre-fix tip remains `db98fe657`; this specification does
not relabel an unexecuted documentation commit as a runtime result. Evidence:

- `c-variant-members-r1/post-diagnostic-r1/OBSERVATION.json`, SHA-256
  `dbac91bc4c17593ac05adae8d63ff7bb64dd3340fe3339b0c5e3574cf7674fb3`.
- `c-variant-members-r1/post-diagnostic-r1/SOURCE-AUDIT.json`, SHA-256
  `0006ab94adade39b12079f4d08db514cdadff09a207495a3c6e5dd732950d6ad`.

Paths are relative to the retained nozzle evidence root. Original inputs,
Native/API/CLI emissions, public observations, comparison reports, source and
binary bindings remain unchanged. A first comparator difference does not
establish equality of the remainder of an output.

## Non-negotiable implementation limits

Gather the existing typed vector through the existing selected indices. Do not
truncate to a count, take an arbitrary prefix, replace repeated values, or
patch the exported string. No new field, helper, public API, dependencies,
filesystem/runtime behavior, or legacy fallback is needed.

Keep owning lists, types, defaults, nullable semantics, normalization, refresh
algorithms, selection/index algorithms, serializers/exporters, tower behavior,
physical/logical/firmware identities, and all fixtures/comparators unchanged.
Dynamic inventory, dynamic slot expansion, sequential scheduling, and adjacent
upstream behavior remain deferred. This is a narrow authorized exception to
the earlier membership spec's destination restriction, not permission to alter
its other boundaries.

## Test-first evidence and exit criteria

The coordinator explicitly accepts the already executed17 strict failures per
mode as this follow-up's pre-fix behavioral evidence. No new test or nonuniform
witness is added: retain frozen21 plus A plus the three independent C cases,
exactly25 per mode. Preserve the original Native premises, public import
observations, generator-only ordered complete-output comparator, raw API/CLI
equality, and every retained control's original contract. Retained12 still
contains four shared strict triplets, not twelve.

Commit this source-cited specification before the gather call. Bind the
existing failed execution and verify production/test source equivalence across
the documentation-only stage; then add only the missing typed gather. Require
all25 tests to pass in both official and seeded modes, including every case
that previously passed. Require fmt, locked workspace/all-target Clippy with
`-D warnings`, fresh core/WASM checks, and changed Rust files no larger than400LOC.

The owning-list correction `db98fe657` causes regressions by itself and must
be integrated together with this gather correction, never independently.
Rebase only after explicit B54 main landing confirmation, repeat final exact-tip
qualification, and submit one combined nozzle review. Acceptance, assembly
qualification, and main landing remain distinct.
