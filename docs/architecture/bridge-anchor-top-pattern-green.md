# C2-T candidate: blocked by existing loaded-config regression

Implemented PLAN `a5f445512` (carried `718c7cf1e`) on main `023b742f2`.
Local RED `df184e73b` carries to `e3f203dc6`; global-role RED `f73fdbc97`
carries to `6032cc5c5`. Rebase range-diff is 5/5 `=` before this repair.

## Upstream boundary and implementation

`PrintConfig.cpp:10400-10421` validates four global pattern subsets through
`Config.hpp:2498-2503::has_enum_value`. The existing Rust pattern validator
is now merged into the ordered error map in
`ProjectSettingsRaw::loaded_range_options`. This is the raw global 3MF
boundary; it does not receive effective local object/part/range metadata.

`Config.hpp:2084-2092` decodes local enums from the full keymap;
`PrintObject.cpp:3664-3709` applies local overrides, and
`Fill/Fill.cpp:858-902,1394-1505` selects/generates by actual surface role.
Only the top-pattern compatibility guard in anchor_projection.rs is deleted.
The internal-solid guard remains for C2-S; numeric D behavior is deferred.
There are no changes to Fill geometry, error rendering or comparisons.

## Verification

Official CPU13 and seeded CPU14, outer cargo/nextest taskset, jobs2/threads2,
fresh cache and artifact root per leg, both 238/238 PASS, zero FAIL/TIMEOUT:

- official `/tmp/worker2-c2t-all-directed-official.vRqVjm`.
- seeded `/tmp/worker2-c2t-all-directed-seeded.VEtTtK`.

Each leg includes finite7 (local object/part Grid, no-candidate control,
exact Native global config rejection), full option_coverage, existing
bridge/anchor regressions. Existing assertions remain unchanged. The first
filter accidentally omitted profile_entry, so loaded-config cases were not
in those 238 tests. Group counts are finite7/coverage220/bridge-anchor11.

Strict clippy (workspace/all-targets/all-features, -D warnings), fmt check,
and wasm32 core/vgcode/wasm check all exit0, CPU13, standard env:
`/tmp/worker2-c2t-engineering.r52IyT`.
Joined external evidence `/tmp/worker2-c2t-green-review-evidence.json` contains
per-group counts, exact test names, global diagnostic witnesses and receipts.
No Native artifacts are committed. Full remains pending coordinator FULL-GO;
only the coordinator merges main.

## Required stop after supplemental loaded-config regression

Fresh supplemental runs on candidate `320fb0757` matched the actual
`runner::profile_entry::{loaded_config_*,preset_vector_loading}` modules:

- official `/tmp/worker2-c2t-loaded-official.tWhUM2`: 16 PASS, 1 FAIL.
- seeded `/tmp/worker2-c2t-loaded-seeded.3NOeCp`: 16 PASS, 1 FAIL.

The unchanged `o3_matrix_sweep_loaded_config` fails in both modes.
Anycubic Kobra S1 Max 0.25/0.4 and Kobra X 0.4 acquire an extra Ares
`internal_solid_infill_pattern: invalid value zig-zag` error; Native reports
only the existing numeric errors. Per PLAN approval condition, implementation
and all experiments stop here. Assertions are unchanged; no expanded repair,
no FULL-READY, no full, no merge. External evidence is updated with this
failure and the filter correction. Await coordinator ruling.
