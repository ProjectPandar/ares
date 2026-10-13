# R addendum: current-region outer-wall speed during emission

Status: SCOPE-REVIEW requested; no emission production change yet.
R approved spec: [layer ranges](layer-config-ranges-slicing-spec.md).
RED `556df7df2` is frozen. Local raw-slicing intermediate `ab0b36090` is not
GREEN or merge-ready. Its library/public E2Es compile; the private raw unit
fixture helper still needs a structural signature adapter before all-target
compilation (no assertion reversal planned).

## Actual post-admission result

Outer-pinned fresh run on current production working tree:
`/tmp/worker6-range-green-pinned.nZyWBl/{official,seeded}/{log,exit}`.
Each: 4 PASS/6 FAIL, exit100. Root launcher and each nextest process record
CPU affinity11 in `launcher-affinity` and per-mode `nextest-affinity`.
Every Native/test/cargo process inherits the outer taskset11; test runner also
pins11. Earlier unpinned-parent run `worker6-range-green.aDssVL` is trial only.

PASS: explicit density, explicit width, valid-profile region-only density,
model-part volume/range width precedence. FAIL: explicit speed, profile mixed,
profile overlap/gap/epsilon, modifier speed range. All layer-Z/HEIGHT assertions
pass. Whole-byte comparisons remain unchanged and FAIL as recorded.

| Case | First whole-output difference | Actual motion difference |
|---|---|---|
| speed | line25 M73 P0 R8 vs R7 | Z1.1, Native line614 G1 F2100 vs Ares line615 F7200 |
| profile-mixed | line85 M73 placement | Z3.2, Native line1820 F2100 vs Ares line1822 F7200 |
| profile-overlap | line55 M73 placement | Z3.2, Native line1990 F2100 vs Ares line1992 F7200 |
| profile-gap | line85 M73 placement | Z3.2, Native line1820 F2100 vs Ares line1822 F7200 |
| profile-epsilon | line85 M73 placement | Z1.05, line681 F2100 vs F7200 |
| modifier-range | line809 M73 placement | Native line1540 F5840 vs Ares line1541 F4086; modifier/cooling attribution still to verify |

The first-difference details above are observed official output; initialized
also fails the same six cases. External motion-only diagnostic:
`/tmp/worker6-range-green-pinned.nZyWBl/motion-diagnostic.json`. This inspection
is not an acceptance normalization: it only finds the later motion symptom.
Local wrapper SHA256 is
`1689a2ff46564cf723ad90c663051b58115920c3cce639bd2961340955cebbf3`.
These are candidate deterministic differences, not Native Variant A or ND.

## Additional upstream/Rust boundary requested

`GCode.cpp:6131-6145`, `GCode::extrude_perimeters`, applies the current
PrintRegionConfig at :6136 before perimeters. `GCode.cpp:6149-6175`,
`extrude_infill`, applies it at :6163 before emitting a region's infills.
Outer-wall speed is consumed at :6523 and relative small-perimeter speed uses
that current outer-wall reference at :5811-5813.

Ares `gcode_emit/motion/options/build/context.rs:35-43` takes the first object,
first layer candidate and first model part for initial motion options.
`motion/options/build/region.rs` resolves outer-wall/small-perimeter values
from that initial region. `gcode_emit/motion/layer.rs` already iterates current
region IDs but refreshes only density, infill-first and small-area gate;
`layers/object_entry.rs:82-97` has the actual current-region options lookup.
Outer-wall speed consequently remains 120 instead of the range's 35 mm/s.

Requested included port: refresh current-region outer_wall_speed and its
relative small_perimeter_speed dependency at regional perimeter/infill/ironing
phase transitions, using the existing region ID lookup, not any first region
as a proxy. Reuse existing absolute-speed semantics. Proposed Rust targets:
`gcode_emit/motion/layer.rs`, `layers/object_entry.rs`, and a small private
`motion/options` helper (each ≤400 LOC). No public API, new registry, runtime
flag, comparator change, post-hoc M73 patch or obsolete guard substitution.

Adjacent fields such as unrelated region fans, jerk, tool mapping and support
config are deferred unless a retained RED demonstrates they are required;
report any additional mismatch rather than silently expanding. The modifier
case must be traced through the same per-region emission/cooling path; if it
has an independent cause, report that before another production change.

Existing raw test helper `project_slice/tests/raw_support.rs::intersections`
requires passing default FilamentRegionSourceOptions and logical_count1 to the
expanded private raw intersection signature; no body assertion or expected
result changes. Any further callback test adapters will preserve all assertion
text. Actual expectation reversals/retirements require advance approval.

## Acceptance remains R's contract

Keep all ten RED cases and byte/layer/motion-change assertions. No failed
speed case is removed or recast as PASS. Re-run fresh official/seeded directed
on CPU11 with outer taskset for cargo/nextest/Native; record affinity.
Also verify existing profile/height/wall_loops/modifier controls, then run
strict/fmt/wasm and default workspace in authorized slots, canonical full only
with explicit FULL-GO. Preserve existing missing-height rejection and external
crash evidence; it never becomes an acceptance case.
