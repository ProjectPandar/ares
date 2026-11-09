# Print::apply post-region normalization

## Upstream boundary

This slice rewrites OrcaSlicer `libslic3r` at
`345efec64c7b95a05ce5cf96df0dde66a158835c`, not an independently designed
slicing pipeline.

- `PrintApply.cpp:1350-1369`: retain the unnormalized configuration and the
  keys changed by the first `normalize_fdm_2` pass.
- `PrintApply.cpp:2208-2216`, `Print::apply`: after constructing regions,
  obtain `extruders(true)`, normalize the retained configuration again,
  append the first pass's changed keys, remove keys equal to the applied
  value, and `apply_only` the remaining keys.
- `Print.cpp:686-803`: region, volume, height-range, support/current-tool,
  current-plate custom-tool and selected tower-filament membership used by
  `extruders(true)` within the represented model/configuration boundary.
- `PrintConfig.cpp:10513+`, `DynamicPrintConfig::normalize_fdm_2`: the existing
  normalization call and its typed changed-key vocabulary.

The Rust destination is `project/effective_config/phases.rs` and the existing
region/configuration carriers in `project/effective_config`. The current
pre-region-only late-normalization scaffold is a temporary compatibility
shell and must gain the upstream post-region reconciliation. It must not
become an Ares-owned pipeline contract.

## Required behavior

1. Retain the pre-pass values needed to normalize the original configuration
   again. Do not normalize the already-disabled prime-tower value a second
   time and thereby prevent restoration.
2. Compute the second used-filament set after final regions exist, including
   the represented sources of `extruders(true)` in their upstream order.
3. Merge keys changed by both normalization passes, remove keys whose new
   values equal the applied values, and update only the remaining keys.
4. Preserve all unrelated materialized values, process-variant indices,
   filament maps, object/region options and configuration serialization.
5. Reuse the existing typed normalization-key enum. Keep this private and
   platform-neutral: no new public API, filesystem, terminal or viewer work.

The two currently represented changed keys are `enable_prime_tower` and
`independent_support_layer_height`. Match them exhaustively rather than
adding a generic configuration mutation framework. Avoid rebuilding regions
unless a cited changed field actually requires it.

## Public failure-first case

Use the stock-profile two-filament Ender-3 project already retained as
`tests/parity/semm-ramming/R1.3mf`. Construct the new input at runtime:

- keep `enable_prime_tower=1` and both configured filaments;
- assign the model objects to filament 1;
- request sparse infill from filament 2 with the stock positive density;
- preserve the stock model geometry, instance transforms and valid tower
  placement, rather than commit another archive or fitted position;
- preserve unrelated stock settings and ZIP payloads.

The Native run must succeed, export nonempty G-code and contain actual tower
motion. Before this slice Ares omits the tower because its shell-only usage
is filament 1. Compare the complete Native G-code against the public
validated API and CLI outputs using the existing ordered generator-only
comparator. Retain every byte and comparison result before asserting. The
normalization slice alone must turn this new test from failing to passing;
no footer, timing, role, motion or configuration waiver is permitted.

A debugger observation is diagnostic only and cannot replace this test.
The unchanged-input outside-bed rejection remains a separate failing test
until the later `Print.cpp:1650-1733` printable-clearance slice.

## Verification

Before collecting the baseline, make the worktree's existing profile path
resolve to the pinned profile resources without changing code. Record the
link, resolved resource tree and hashes, and distinguish the previous missing
resource failures from executable behavioral controls.

Run the entire `orca_parity` selection in official and seeded modes before
and after the change, excluding exactly
`runner::cases::estimated_tower::tiq2_brim_corner_five_native_premises`.
That one-shot prerequisite already has a separate historical receipt; do not
repeat it or represent it as execution on this base. No other exclusions.
Match all selected test identities, require zero passing-to-failing
transitions, and enumerate every failing-to-passing transition. Retain all
other failures and raw residuals, including Native result JSON.

Bind the actual executed binaries, source, inputs, resources and catalog.
Run rustfmt, diff checks and strict workspace/all-targets/all-features Clippy.
Keep changed modules below 400 lines. Commit the specification, then the
failing public test, then the source rewrite.

## Deferred adjacent behavior

Do not include printable-clearance geometry, generated tower mesh checks,
PartPlate fitting/relocation, warning/proximity handling, exclusion/wrapping
geometry, cone rejection, new painted/mixed material representations or
independent changes to normalization predicates. Existing represented
normalization behavior is called at the correct upstream phase; unsupported
adjacent representation is not silently expanded by this caller slice.
