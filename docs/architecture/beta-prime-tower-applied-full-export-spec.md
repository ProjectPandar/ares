# Beta prime-tower applied-full export authority

## Upstream rewrite boundary

This slice rewrites the applied configuration export contract of OrcaSlicer
`libslic3r` at `345efec64c7b95a05ce5cf96df0dde66a158835c`.
It removes a temporary v2.4.2 compatibility shell rather than adding a separate
Ares export rule.

| Upstream owner | Included behavior |
|---|---|
| [PrintConfig.cpp:10513–10541](../../OrcaSlicer/src/libslic3r/PrintConfig.cpp#L10513-L10541), `DynamicPrintConfig::normalize_fdm_2` | The effective prime-tower value comes from normalization using actual usage and the existing timelapse, wrapping, mixed-filament and print-sequence conditions. |
| [PrintApply.cpp:2208–2216](../../OrcaSlicer/src/libslic3r/PrintApply.cpp#L2208-L2216), `Print::apply` | Apply the late normalization's changed keys to the full configuration. |
| [PrintApply.cpp:2256–2261](../../OrcaSlicer/src/libslic3r/PrintApply.cpp#L2256-L2261), `Print::apply` | Synchronize changed keys into `m_ori_full_print_config` before publishing the applied full configuration. The checkpoint must not resurrect the preset's unnormalized prime-tower value. |
| [Print.cpp:4656–4751](../../OrcaSlicer/src/libslic3r/Print.cpp#L4656-L4751), `Print::update_filament_maps_to_config` | Write the derived maps into the saved configuration and rebuild `m_full_print_config` from that normalized checkpoint. The rebuild block follows the map-change conditional and executes independently of it. |
| [GCode.cpp:8244–8247,8342–8352](../../OrcaSlicer/src/libslic3r/GCode.cpp#L8244-L8247) | Serialize the applied full configuration. |

The v2.4.2 `Print.cpp:3166–3170` raw-resurfacing model is not the beta contract.
A changed filament map does not authorize restoring a raw process option.

## Rust destination and existing scaffold

`options/config_export/writer.rs::write_config_block` already receives
`ProjectConfigViews::full`, the applied dynamic-full snapshot used by this export.
`options/config_export/transform.rs::apply_cli_oracle_state` nevertheless compares
an imported filament map with the applied map and replaces the resolved
`enable_prime_tower` with a value read from the raw project settings.
`writer.rs::ExportOverrides` supplies that raw boolean and raw map token.
This is the compatibility shell being removed.

The production boundary is limited to:

- `crates/ares-core/src/options/config_export/transform.rs`.
- `crates/ares-core/src/options/config_export/writer.rs`.

Remove `apply_cli_oracle_state` and its invocation. Remove
`ExportOverrides.enable_prime_tower`, `raw_filament_map_token`, and their raw
lookup plumbing. If no consumer remains, delete the orphaned `ExportOverrides`
container and constructor, including fields retained only by that container.
Remove imports made unused by these deletions.

`transformed_for_export` continues to clone the applied snapshot and perform the
existing flush-matrix scaling. `write_config_block` continues to collect entries
from that transformed applied configuration. Keep its raw-settings parameter for
the unrelated full-config export uses that still need it.

Do not add a replacement override, a map-change special case, a hardcoded false
value, a raw-token echo, or a second export snapshot.

## Architectural constraints

1. `enable_prime_tower` must retain the effective value already present in the
   applied full configuration. This holds when the value is true as well as when
   normalization disables it.
2. Preserve the existing normalization rules, usage collection, phase ordering,
   immutable applied map and `refresh_filament_map(checkpoint, map, current)`.
   Rust refresh starts from `current.clone()` and refreshes filament fields.
   Its checkpoint's earlier process fields do not justify a speculative phase or
   checkpoint refactor for this export-only removal.
3. Preserve the existing producer and comparison of all three maps. This slice
   must not change logical-filament, physical-head, nozzle-group or volume-type
   selection.
4. Preserve every flush-matrix calculation, multiplier selection, rounding,
   cardinality check and error ordering. Keep the exact diagnostic
   `Flush volumes matrix do not match to the correct size!`.
5. Keep registry defaults and membership, printer/process/filament materialization,
   nullable-vector selection, acceleration/jerk behavior and normalization slices
   unchanged.
6. Add no core filesystem, terminal, platform, rendering or dependency behavior.
   Export remains portable across browser WASM, Windows, macOS and Linux.

The applied-validation map isolation documented in
[print-applied-filament-map-lifecycle.md](print-applied-filament-map-lifecycle.md)
is unchanged. That document pins a v2.4.2 lifecycle slice. It does not authorize
raw prime-tower restoration under the beta sources above.

## Public E2E boundary and failure modes

The primary failing test is:

`option_coverage::print_extruder_identity::print_process_identity_repeated_ids_select_matching_variants`

Preserve its original project construction, Native materialized-ID premises and
complete-output assertion. Its raw preset enables the tower and requests map 1.
The applied map becomes 2. The beta Native output disables the tower while the
obsolete restoration can emit the raw enabled value.

Use fresh Native execution and replay with the validated public byte API and CLI.
Record the imported map, applied three maps, raw and effective prime-tower values,
input and binary hashes, complete commands, output bytes and unrelated differences.
Expected values come from successful runtime Native output, not fixed footer
literals or private implementation state.

Failure modes:

- A changed map restores raw true after normalization selected false.
- Removing the restoration changes an unchanged-map case that already matched.
- Export forces false when the applied configuration correctly retains true.
- Removing orphaned plumbing affects matrix scaling or diagnostic precedence.
- A source-column or earlier whole-output difference is incorrectly hidden to
  label the prime-tower field recovery as complete-output parity.

The old H2D/X2D map-change coverage is mandatory. Reuse the original smoke export
recipe in `smoke.rs::assert_printer_smoke` and
`smoke/sweep.rs::run_printer_sweep` through `export_selection_case` for these
bounded rows:

- BBL/Bambu Lab H2D 0.2, 0.4, 0.6 and 0.8 nozzle.
- BBL/Bambu Lab X2D 0.2, 0.4, 0.6 and 0.8 nozzle.

These are the original historical smoke rows, including H2D map 1-to-2 and X2D
unchanged-map coverage. Use the existing cube, preset selection and smoke overrides.
Do not force Manual mode, change geometry or invent a new preset to recover the
historical branch. Observe and retain the current beta map and tower values.
A changed Native premise must be reported without changing or skipping it.
Do not run the vendor sweep or rewrite its historical summary.

Existing complete-output tests stay intact. A field-specific public observation
may discriminate prime-tower export while another entry still fails first.
It must retain that first failure and all remaining mismatches. Such an observation
cannot establish whole-output parity or reachability of assertions not executed.

## Checks

Record the repeated-ID failure and the historical H2D/X2D behavior with both
official and seeded Native supplies. Comparisons use the same test and smoke-row
identities and the same public entry points.

Prime-tower export must equal fresh Native output. Public API and CLI bytes must
match when metadata matches. Neither original assertions nor scoped field controls
may change from passing to failing. Unchanged-map and applied-true cases exercise
both effective boolean values. A remaining unrelated difference still means the
complete-output assertion fails.

Check the exact commit with `cargo fmt --check`,
`cargo clippy --locked --workspace --all-targets -- -D warnings` and the
applicable public browser-WASM export cases. Changed modules remain at most
400 lines. Raw artifacts include source, binary and input bindings.
A historical pass, equal tree or cached build is not a check of the new commit.

## Excluded behavior

This slice does not implement missing normalization conditions, change checkpoint
ownership, expand map solvers or physical multi-nozzle behavior, or repair other
footer entries. It does not change motion, geometry, time or statistics.
The three remaining filament-volume-map cases are outside this slice.
Prime-tower field equality alone does not establish whole-output parity.
