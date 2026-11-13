# FDM feature-filament normalization in OrcaSlicer 2.5.0

## Source boundary

This is a Rust rewrite slice of `src/libslic3r/PrintConfig.cpp`, not an
Ares-designed filament-selection policy. Upstream commit
`f4bfb4660f0b97d60416e0bea6788197311db484` removes the feature-filament
propagation blocks from `DynamicPrintConfig::normalize_fdm` (parent lines
10374–10391) and `DynamicPrintConfig::normalize_fdm_1` (parent lines
10471–10488). At the final 2.5.0 revision
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, the functions begin at lines
10348 and 10426; each proceeds directly from legacy `extruder` handling to
spiral-mode normalization at lines 10376 and 10454, respectively.

## Decision and Rust destination

Remove the corresponding block from
`crates/ares-core/src/options/project_fdm_normalization/stage1.rs::normalize`,
called by `project_fdm_normalization::normalize_fdm_1`. Do not replace it with
a fallback. A positive `sparse_infill_filament_id` must no longer populate
`internal_solid_filament_id`. Likewise, internal-solid, top-surface, and
bottom-surface IDs must no longer populate one another. Existing explicit
values, including zero, survive this normalization stage unchanged.

These values are filament selection IDs, not shell-layer counts. This
change preserves their downstream interpretation rather than choosing a
new interpretation. The existing typed project-settings carrier remains a
temporary compatibility shell where it does not match DynamicPrintConfig;
this slice only removes its obsolete normalization behavior. Core remains
platform-neutral, with no filesystem, UI, or runtime-specific dependency.

## Included and deferred behavior

Included: the removed sparse-to-solid and inter-surface propagation in the
typed project normalization path. Preserve spiral-mode, resolution, tower,
legacy `extruder`, and downstream tool-selection behavior without unrelated
changes. The JSON `SliceOptions::normalize_fdm` compatibility API uses older
non-ID keys and does not contain this removed feature-ID block; redesign of
that API and adjacent legacy normalization parity are deferred.

Deferred: other 2.5.0 changes, timing/order changes, tool-template assertion
updates, comparator changes, crash repair, and workspace-wide parity runs.
The prime-tower blank line after `G4 S0` depends on the upstream
`34c8e78e`/`39408fa1f` wipe-tower newline port.
Assertions, expected values, and comparison tolerances remain unchanged.

## Authored dual-filament fixture inputs

Upstream `f4bfb4660f0b97d60416e0bea6788197311db484` removes the implicit
sparse-to-solid/surface propagation on which these two shared fixtures relied:

- `runner/compatible_regions/tool_templates/fixture.rs::configured_project`:
  explicitly author `internal_solid_filament_id`, `top_surface_filament_id`,
  and `bottom_surface_filament_id` as 2 alongside sparse ID 2. Its zero-sparse-
  density project needs solid/surface extrusion on the second filament to
  exercise the existing two-nozzle template lifecycle, cooling, heating,
  retraction, and statistics assertions. Native 2.5.0 otherwise schedules
  only the first filament, leaving those assertions without their premise.
- `runner/compatible_regions/modifier_travel.rs::outputs_with_cooling`:
  explicitly author the same three IDs as 2. Solid/surface tool use must
  remain on the second filament while the modifier activates sparse infill;
  this restores the dual-filament travel and final-use-cooldown premise that
  previously depended on the propagation removed by `f4bfb4660`.

These are author-input corrections, not output-derived expectations. Native
2.5.0 determines the result. Keep the inherited-filament fixture unchanged:
its zero-density baseline intentionally excludes sparse second-tool use.
Keep the stock sparse-tool prime-tower fixture unchanged so it continues to
exercise independent zero-valued solid/surface IDs. Shared fixture consumers,
including the three shared-fixture cases, retain all assertions.

## Verification contract

The 30 affected cases cover independent feature IDs in exported settings,
sparse-tool prime-tower activation, and dual-filament template lifecycle,
cooling, heating, retraction, statistics, travel, and final-use cooldown.
Three additional shared-fixture cases cover modifier-density activation of
an inherited second filament, mapped-tool load/change delays, and shared-
hotend unload/load delays. Whole-output comparisons against Native 2.5.0
and the existing behavior assertions remain unchanged.

Failure modes: obsolete sparse-to-solid or inter-surface propagation,
unintended overwrite of explicit IDs including zero, authored inputs no
longer exercising dual-filament behavior, whole-output differences masking
correct feature IDs, missing reference coverage, missing inputs, and
launcher/catalog setup failures.

Run `cargo fmt --check` and
`cargo clippy --locked --offline --workspace --all-targets -- -D warnings`.
