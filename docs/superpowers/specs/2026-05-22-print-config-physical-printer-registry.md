# M27 Spec: PrintConfig physical printer option registry slice

## Goal
Port the physical-printer common parameter definitions from `libslic3r::PrintConfigDef::init_common_params` into `ares-core` option registry metadata, while doing the minimum registry-table split needed to keep Rust files under 400 LOC.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:83-85`: `AuthorizationType` enum values.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:155-159`: `AuthorizationType` enum key map (`key`, `user`).
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:786-894`: physical-printer common parameters inside `PrintConfigDef::init_common_params()`.

Rust destination boundary:

- `crates/ares-core/src/options/registry.rs`: public registry API and value-kind vocabulary.
- `crates/ares-core/src/options/registry/definitions.rs`: sorted source-cited definition table.
- `crates/ares-core/src/options/registry/tests.rs`: registry-table tests split from definitions to keep both files under 400 LOC.
- `crates/ares-core/src/options/tests/registry_helpers.rs`: public lookup/count coverage.

## Included option definitions

Add registry metadata for these exact upstream options and default values:

- `preset_names` (`coStrings`, default empty, lines 786-790 and duplicate lines 872-876)
- `bbl_use_printhost` (`coBool`, default `false`, lines 792-797)
- `printer_agent` (`coString`, default empty, lines 799-804)
- `print_host` (`coString`, default empty, lines 806-814)
- `print_host_webui` (`coString`, default empty, lines 816-821)
- `printhost_apikey` (`coString`, default empty, lines 823-829)
- `printhost_port` (`coString`, default empty, lines 831-837)
- `printhost_cafile` (`coString`, default empty, lines 839-845)
- `printhost_user` (`coString`, default empty, lines 849-854)
- `printhost_password` (`coString`, default empty, lines 856-861)
- `printhost_ssl_ignore_revoke` (`coBool`, default `false`, lines 863-870)
- `printhost_authorization_type` (`coEnum`, default `key`, lines 878-888)
- `preset_name` (`coString`, default empty, lines 890-894)

## Functional requirements

1. Split the registry definition table out of `registry.rs` so no modified Rust file exceeds 400 LOC.
2. Preserve public API: `OptionValueKind`, `OptionDefinition`, `option_definitions()`, and `option_definition(key)` remain available from `ares_core::options::registry` and re-exported from `ares-core`.
3. Extend `OptionValueKind` only as needed for `coStrings` (`Strings`).
4. Add the included options to the sorted definition table.
5. Preserve binary-search lookup and sorted/no-duplicate test coverage.
6. Preserve `SliceOptions` unknown-value storage and current public slicing API.
7. Do not add typed parsing/accessors or behavior for these physical printer options.
8. Do not add a new pipeline stage, crate, dependency, filesystem behavior, network behavior, or G-code behavior.
9. Update M27/M28 roadmap and milestone docs so E2E parity moves to M28.

## Deferred behavior

- Actual print-host upload, authentication, network access, certificate handling, and UI visibility behavior are deferred.
- Typed accessors for the physical-printer options are deferred.
- FFF-specific options from `PrintConfigDef::init_fff_params()` are deferred except options already covered by prior milestones.
- Full option registry parity with every OrcaSlicer option remains incremental.

## Acceptance checks

- Registry tests prove new keys, kinds, default values, duplicate `preset_names` source accounting, and source line references.
- `known_definition_count()` counts newly registered keys while preserving unknown options.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
