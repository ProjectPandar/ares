# Merged profile option storage

## Scope

This is a representation-only maintenance change to
`crates/ares-core/src/profiles/inheritance.rs::MergedProfile`, based on
`b065c0e856011f468e1d88c192d17330ef228f64`. It adds no upstream behavior.
The existing profile resolution and composition boundaries remain unchanged.

## Observed failure

The wasm32 core/WASM all-targets Clippy command with `-D warnings` fails on
both `5a0516dc6` and `b065c0e85`: Process contains at least 2588 bytes,
Filament at least 1780 bytes, and MergedProfile at least 2592 bytes.
The same command passes at `a4ae7e0c6` and fails at its child `b8c613f6d`,
which materializes process speed vectors and removes the enum's previous
lint expectation. At that child, Process is 2556 bytes versus Filament's
1744 bytes. This identifies a pre-existing main regression, not a new
vendor-profile default-resolution regression.

## Storage decision

Box the options of all three variants. A Process-only boxing probe still
fails on wasm32: Filament is 1780 bytes versus Machine's 1364 bytes. Boxing
only those two would leave Machine oversized relative to the boxed options.
Composition moves each pointee back into the existing ProjectSettings fields.

## Constraints

- Introduce indirection for oversized option storage; do not change metadata,
  builder overlays, default resolution, inheritance, or profile selection.
- Move owned options out at the existing composition boundary; do not clone
  option values merely to adapt storage.
- Add no lint allow/expect, target-specific representation, compatibility
  fallback, or configuration behavior.
- Boxing has an allocation cost. This change does not claim zero allocation
  or benchmark-proven performance improvement.
- MergedProfile is a public enum: boxed fields change its Rust construction
  representation, although resolved values and slicing behavior are unchanged.

## Failure inventory and checks before implementation

Possible failures are missed constructors, incorrect ownership conversion at
composition, a new size imbalance after boxing, or changed merged values.
Existing `profile_public_api`, `profile_bundle_compat`, `profile_settings`,
`profile_entry_overrides`, and `profile_entry_processed` integration tests
cover public typed borrowing, merging, composition, and entry semantics.
Retain their before/after results. Use the reproduced wasm32 lint failure as
the failing representation check, then require the identical command to pass.
Also require rustfmt and native workspace/all-targets Clippy without warnings.
No new unit test or STL repair implementation belongs to this change.
