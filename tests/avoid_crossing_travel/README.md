# Travel reference fixtures

The three committed 3MF inputs exercise OrcaSlicer's
`AvoidCrossingPerimeters::travel_to` / `extend_for_closest_lines` and
`GCode::extrude_loop` / `extrude_infill` logical cursor semantics. Upstream
boundaries and migration contract are in
[the travel reference spec](../../docs/architecture/native-artifact-cleanup-travel-reference-spec.md).

Five active tests live in
`crates/ares-cli/tests/orca_parity/runner/native_travel_reference_migration.rs`:

| Input | Contract |
| --- | --- |
| `anchor.3mf` | Every ordered XY travel command, including repeated rounded corners; complete layers Z2.58/2.72/8.04 with required inward move and sparse-infill markers |
| `one-wall.3mf` | Exactly 436 ordered XY commands including duplicates; complete G-code string equality after only one strictly validated generator/date line normalization |
| `two-walls.3mf` | Complete layers Z0.34/0.76/1.18 |

Native references are generated at runtime by the existing OrcaRunner
`slice_case` cache (`ARES_ORCA_CACHE_DIR`). No Native output is tracked here.
The shared Native dependency policy applies: `ARES_REQUIRE_ORCA=1` makes missing
Native dependencies fail; optional runs report visible SKIPPED. Cache hits do
not bypass dependency validation.

Ares output still comes from public CLI `slice -o <output> <input.3mf>`.
`ARES_TRAVEL_BIN` selects an independently built diagnostic binary when set;
otherwise tests use Cargo's Ares executable. Raw outputs, hashes and receipts
are retained outside the repository under `ARES_PARITY_ARTIFACT_ROOT`.
