# CLI profile parity route alignment

## Boundary and decision

These runtime comparisons observe OrcaSlicer's CLI project application, not an
embedded-only core slice. Their Rust calls must use the existing supplied-profile
application adapter rather than bypassing it.

The upstream boundary is OrcaSlicer 2.5.0 `OrcaSlicer.cpp`:

- `2786–2817`: resolve embedded BBL filament identities into CLI profile inputs;
- `3510–3545`, `3639–3701`, `3753–3759`, `3823–3845`: materialize selected
  filament variant segments and reconstruct `filament_self_index`.

The existing Rust rewrite boundary is
`ares-core/src/project/domain/profile_input/overlay.rs` and its variant helper.
The selected-profile variant algorithm is specified in
`cli-filament-variant-overlay-spec.md`; it is a prerequisite, not reimplemented
by this change.

The destination of this slice is only these CLI parity test modules:

- `ares-cli/tests/orca_parity/runner/native_jerk_semm_migration.rs`;
- `ares-cli/tests/orca_parity/runner/native_bbl_boundary_migration.rs`.

They must import `runner::application::slice_project_plate`, whose existing
supplied-catalog path calls public `ares_core::slice_with_profiles`. The Native
side already uses the matching supplied system tree through `slice_case` and
`application::native_arguments`. Both sides retain identical project bytes,
plate selection, generation metadata and overrides.

This is a test-adapter correction around an existing upstream rewrite boundary,
not a new slicer pipeline or a replacement for the embedded-only API. No new
adapter, profile loader, resource catalog, fallback or emitter behavior is added.

## Included behavior

The original complete ordered-line and line-count comparisons remain the E2E
witnesses:

1. `e5max_klipper_jerk_switches_match_native`;
2. `semm_toolchange_m104_matches_native`;
3. `bbl_object_boundaries_match_native`;
4. `bbl_toolchange_boundary_matches_native`.

`overhang_wall_jerk_matches_native` is the adjacent passing control because it
shares the jerk/SEMM test helper. Existing role-jerk and temperature controls
remain intact.

The observed configuration and executable boundaries stay inside those whole
streams. In particular, filament variant cardinality must not be hidden in
`CONFIG_BLOCK`, and BBL preheat-dependent markers and time/footer output must
not be filtered. Their upstream emitter owners remain `GCode.cpp` and
`GCodeWriter.cpp`, as cited by the existing jerk/SEMM and BBL boundary specs.
Those emitters are not modified to compensate for different application inputs.

Only the existing generated-by-banner normalization is permitted. No changed
fixtures, expected G-code snapshots, epsilon, precision changes, command-only
comparison or removed assertions are part of this slice.

## Test-first verification

The existing public-API E2E tests already expose the incorrect route; do not add
duplicate tests of private helpers or inspect import text as a substitute for
behavior.

1. Commit this specification before changing either route.
2. Run the unchanged four witnesses and overhang control on the selected-profile
   overlay prerequisite. Retain per-case inputs, complete streams, manifests and
   comparison JSON for both established runtime profiles.
3. Change only the two slicing imports to the existing application adapter.
4. Repeat exactly the same test identities. Report the full base-to-tip matrix,
   including any residual failure, with no passing-to-failing transition.
5. Run formatting and workspace all-targets clippy. Confirm the patch contains no
   production or platform-specific behavior changes.

Evidence must identify the supplied catalog and actual Native producer. Two
runtime profiles are not necessarily two independent Native executables. Native
resources, archived baselines and producer binaries are not rebuilt or changed.

## Deferred behavior

- The core embedded-only BBL refresh gap remains the separate boundary described
  in `cli-filament-variant-overlay-spec.md`; routing a CLI parity test through the
  existing profile application is not an implementation of that gap.
- TiQ2 planning/entry/cooling, pause-source admission and other printer-specific
  residuals remain outside this slice.
- Public core APIs, WASM adapters and renderer-neutral data are unchanged. The
  existing platform-neutral profile API remains usable on all Tier1 platforms.
