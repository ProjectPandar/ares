# Catalog-backed CLI project filament vector witnesses

## Upstream boundary

Authority: OrcaSlicer 2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

The outline witnesses exercise the Native CLI project route, not
`PresetBundle::full_fff_config` or Always preset selection:

- `OrcaSlicer.cpp:1758`, `Format/bbs_3mf.cpp:2806`: load the embedded project
  configuration. `PrintConfig.cpp:11403–11410` leaves the ordinary two-filament
  self-index layout unchanged; the authored air-filtration vector remains `[1]`.
- `OrcaSlicer.cpp:2780–2816`: resolve the retained project filament system
  presets and insert successfully resolved configs in filament-slot order.
  Resolver misses keep imported values; this is not unconditional broadcasting.
- `OrcaSlicer.cpp:3510–3570,3670–3701`: assemble resolved filament configs,
  including retained variant-bearing keys through `set_with_restore_2`.
  A read-only watchpoint on `activate_air_filtration_during_print` confirms
  its first length-two transition at `:3699`'s destination `resize(2, source)`,
  immediately before `:3701`'s restore. This precedes `Print::apply` and export.
- `OrcaSlicer.cpp:4259`: normalize the assembled full config. The witness now
  carries `[1,1]`; `GCode::append_full_config` consumes that assembled state.

## Rust destination and existing implementation

`ares slice input.3mf --project-preset-catalog catalog.json` enters
`ares-cli/src/project_presets.rs`, which submits a CLI `from_project` request
with `uptodate=false` to the public core profile-slicing API. It does not use
Always selection.

`ProjectPresetRefresh::load`'s CLI arm first calls
`profiles/project_refresh/preset.rs::refresh(..., selected=None)`, then calls
`apply_retained_preset_defaults(raw, None)`. That `selected=None` means no
explicit preset replacement; it does not mean missing filament configs.
`preset.rs::refresh` resolves retained filament configs from the supplied
catalog and passes `Vec<Option<ProjectSettingsRaw>>` to
`profiles/project_refresh/filament.rs::apply` and `filament/slot.rs::apply`.
A filament entry is `None` only on a missing identity or resolver miss. The
existing assembler pads retained ranges and restores variant values. These
functions, not an Always helper, are the destination if an equivalent-route
test exposes a product defect.

`profiles::apply_retained_preset_defaults` only materializes retained
machine/process defaults. It is not the catalog resolver or filament
assembler. Direct byte-only `slice_project_plate` has no catalog and cannot
stand in for a Native CLI run that resolves resource presets. Its load-time
scaffolding remains a temporary compatibility shell, not an independently
designed full-config composition rule.

## Witness correction and acceptance

The three `runner::native_exclude_outline_migration` witnesses currently
invoke byte-only `ares_core::slice_project_plate` while their Native runner
receives the shared resource/catalog supply. Correct them to use the existing
`runner::application::slice_project_plate` adapter, which sends the same
catalog through the public `slice_with_profiles` API and uses the same CLI
project policy as `ares slice --project-preset-catalog`.

This changes the producer entry, not the comparator. Preserve complete
ordered lines and line count, the generated-by-only exception, all hull and
tolerance checks, M73, whitespace and CONFIG_BLOCK. Preserve input/output
hashes and artifacts. Check the actual public CLI against the same identity
input and supplied catalog as an additional route verification.

Sequence:

1. Retain the clean-c2a byte-only baseline: all three witnesses fail in both
   official and seeded modes at air-filtration `1` versus `1,1`.
2. Correct the tests to the equivalent catalog-backed public entry, then run
   them before any product change. If they already pass, report an asymmetric
   witness repair; do not invent an implementation change or claim an
   unobserved failure.
3. Only if the equivalent entry fails, port the missing upstream
   resolver/assembly behavior at the existing Rust CLI refresh boundary.
4. Run relevant config-block and project-selection regressions plus fmt,
   all-targets clippy and wasm32. Pin CPUs 16–19, four Cargo build jobs,
   nextest four test threads, required Native availability, and canonical
   official/seeded launchers. Seeded Native uses CPU16 explicitly.

## Included and deferred behavior

Included: ordinary retained multi-filament CLI composition, a singleton
boolean first witness, preservation of explicit rows and retained variant
values through the existing assembler, and equivalent resource supply.

Deferred: Always-mode preset composition, catalog discovery, missing-preset
fallback behavior, Print::apply, configuration footer serialization, thumbnail
generation, and counterbore processing. No raw singleton broadcast, filesystem
access in core, or legacy fallback is introduced.
