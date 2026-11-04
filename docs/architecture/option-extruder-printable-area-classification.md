# extruder_printable_area: bounded grouped-point coverage classification

Test-harness slice. No production port or full geometry-domain acceptance is claimed.

## Upstream boundary

Pinned OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `src/libslic3r/PrintConfig.cpp:692-696` defines
  `extruder_printable_area` as `coPointsGroups`, empty by default, with no
  explicit numeric minimum/maximum. `PrintConfig.hpp:1491` stores it as
  `ConfigOptionPointsGroups` in the print configuration.
- `Config.hpp:1605-1717::ConfigOptionPointsGroups` owns the grouped `Vec2ds`
  value: serialization separates points with commas, coordinates with `x`,
  and polygon groups with `#`; `vserialize()` returns one string per group.
  This is a polygon-list type, not a scalar numeric interval.
- `PrintConfig.cpp:11481-11496::get_shared_poly` intersects the groups in
  order; `:11498-11544::get_bed_shape` uses those groups when `use_share` is
  true and nonempty, otherwise the ordinary `printable_area`.
- `Print.cpp:3256-3279::get_extruder_printable_polygons` and
  `get_extruder_unprintable_polygons` construct per-extruder printable regions
  and bed-minus-region exclusions. `GCode.cpp:2166-2173` passes the print
  validity context to the processor; these consumers identify adjacent
  geometry behavior, not permission to implement it in this slice.

## Current Rust boundary and defect

Core already stores the typed value in
`options/printer_options/remaining.rs::PrinterRemainingOptions` as
`Point2dGroups`, implemented by `options/config_types/point.rs` and exported
through the existing printer wire/config paths. The processor consumes it in
`gcode_processor/printable_area.rs::check` for multiple nozzles. No change to
these core consumers, public APIs or emitted G-code is included.

The destination is CLI test harness
`crates/ares-cli/tests/orca_parity/option_coverage/`:

- `domains/probes.rs` already authors a single full-bed group for this key.
- `domains.rs::plan` excludes `coPointsGroups` from the authored free-form
  type classification. Consequently a real successful Native/Ares comparison
  is recorded INCOMPLETE with the unrelated explanation “schema bounds
  rejected by upstream validation”.
- `numeric_metadata_tests.rs` currently explicitly locks this key to that
  incomplete classification. Its unrelated numeric/width/baseline controls
  must remain intact.

Latest actual full matrix at `acf100550` is 621 PASS /28 INCOMPLETE /1 FAIL.
This spec does not retroactively change that evidence or promise a fresh
650-domain PASS result.

## Included bounded contract

Correct classification for this one authored grouped-point key, with actual
Native requested/effective admission and complete output equality. Coverage
means the stated finite serialization/admission cases in the existing Ender-3
0.4mm, one-nozzle, centered cube10 context; it does not mean all possible
polygons, all printers or the shared-bed intersection algorithm are covered.
Keep this qualification in the test header and coverage evidence.

Use three deterministic valid polygon requests, each retaining the placed
model within the printable regions:

1. One full-bed group: `0x0,220x0,220x220,0x220` (existing probe).
2. One inset group: `10x10,210x10,210x210,10x210`.
3. Two overlapping groups: full-bed plus the inset group. This is a grouped
   value serialization/admission control on one nozzle, not a two-nozzle
   routing witness. Requested/effective proof must retain two distinct groups.

Generate requested JSON as one string per polygon according to upstream
`vserialize`, not an array with an embedded `#` presented as one group.
Reuse the existing case/export/application/cache path. If the authored probe
representation needs splitting for this key, make the smallest key-local
change at `domains`/`bounds::case`; do not replace generic request equivalence
or weaken application verification. If Native rejects or rewrites a proposed
case, retain that attempt and report it before changing the admitted scope.

Only this key may lose the numeric omission and receive the bounded authored
coverage classification. Other numeric probes, locked widths, baseline-only
keys and unproven grouped options remain unchanged. Existing Ares scaffold
here is the test admission adapter around the cited PrintConfig type; this
slice corrects its metadata and uses the existing upstream port.

## RED and GREEN evidence

Add a dedicated active CLI E2E file
`option_coverage/extruder_printable_area.rs`, registered only in its owning
option-coverage module. Reuse `OrcaRunner` and `slice_case`; Native artifacts
stay in external `ARES_ORCA_CACHE_DIR`/artifact roots. No committed Native
output, new Native build, custom cache or test-only production entry point.

RED must first export/slice the existing admitted full-bed request at runtime,
verify requested/effective value and group count, and compare complete ordered
Native/Ares bytes with only existing generated-by normalization. After those
checks succeed, assert the coverage outcome is PASS with no numeric omission;
current harness must fail specifically on its INCOMPLETE classification.
This is a real Native-backed metadata RED, not an invented motion discrepancy.
The additional one/two-group cases must use the same admission and full-output
checks; none may pass solely because config/footer bytes match.

GREEN changes the narrow classification and grouped case authoring only.
Adjust the old metadata test so it retains all other incomplete-key assertions
and adds a positive assertion for this explicitly admitted bounded key. No
comparison tolerance, omitted lines, ignore, skip invention or production fix.
Missing Native follows shared `tests/support/native_environment`: optional
visible SKIPPED, required FAIL. Evidence records commit, producer, requested /
effective groups, input SHA, full output comparison and fresh/cache-hit status.

Acceptance: required cold/hot dedicated Native cases; fresh single-key sweep
with no value filter reports all admitted cases strictly compared; fresh
seeded base/candidate comparison confirms common Ares raw G-code unchanged;
default-environment workspace, control targeted tests, strict clippy, fmt and
wasm check. Preserve previous full-matrix rows as latest-observed, not a new
full execution.

## Deferred

Empty/degenerate/self-intersecting polygons and permissive malformed-coordinate
parsing; disjoint intersection behavior (upstream directly uses result[0]);
multi-nozzle group indexing, filament/physical maps, shared-bed arrangement,
per-head excluded zones, -102 emitted-path rejection, wrapping/timelapse,
SEMM/tool ordering, numeric G3/G2 domain work, and CLI printer_technology.
Any fresh production divergence blocks this classification-only slice and is
reported with its source boundary. Type2 enable=1 is a separate boundary.
