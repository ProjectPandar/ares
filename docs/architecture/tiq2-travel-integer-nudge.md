# TiQ2 near-boundary travel: integer nudge conversion

## Upstream ownership and Rust destination

The owning rewrite slice is `libslic3r/GCode/AvoidCrossingPerimeters.cpp`,
`avoid_perimeters_inner` (:563-584), specifically the closest-boundary retry
and its integer epsilon displacement. The destination is
`crates/ares-core/src/project_slice/gcode_emit/motion/path/avoid_crossing/router.rs`,
`avoid_perimeters` retry construction. This is a correction of that upstream
port, not a new routing policy or endpoint cleanup heuristic.

## Source and Native conclusion

After the retained-fan correction, the exact TiQ2 runtime oracle and Ares agree
through 22563 lines. Native next travels directly to X169.401 Y169.401 Z8.8;
Ares inserts boundary waypoints near X169.4 Y169.4 before the same endpoint.
Line offsets identify this runtime observation only, not a test contract.

Existing seeded Native binary debugging reproduces the entire oracle with
only generated-by banner normalization. At the affected inner-wall approach,
`GCode::travel_to` invokes AvoidCrossingPerimeters before and after wiping.
The post-wipe local start is (-13729879,4425418), endpoint (4401461,4401461).
Native returns exactly those two points. The cached internal boundary is the
square with corners (±4400000,±4400000); it is not empty. The pre-wipe start
is (-14405860,5162338). Both calls return direct paths. Thus the endpoint
itself and the retained support-derived boundary are not the discrepancy.

Upstream constructs retry endpoints as integer point plus/minus a separately
integer-cast displacement:

```cpp
new_end_point = new_end_point +
    (unit_direction * double(coord_t(SCALED_EPSILON))).cast<coord_t>();
```

Rust instead casts the sum of the integer coordinate and fractional
displacement. These operations differ for a negative fractional displacement
added to a positive coordinate. For the post-wipe approach, the closest end
is (4400000,4400000). Its displacement is approximately (99.9999,-0.1402):
upstream truncates displacement to (99,0), ending at (4400099,4400000).
Rust truncates the coordinate sum, ending at (4400099,4399999).
That one-unit downward displacement crosses the top boundary and invents a
retry intersection. The original segment and Native retry remain outside
that boundary. Closest-line extension and protected routing points then
preserve the invented detour; removing formatted duplicate G-code would
hide the source error rather than repair it.

## Included and deferred behavior

Correct both retry start and end to cast the displacement before integer
subtraction/addition, matching Eigen's upstream expression. Existing routing,
intersection collection, closest-line ordering, protected-point simplification,
spacing, support-layer geometry and cache lifecycle remain unchanged.
The current Rust retry arithmetic is replaced, not retained as a fallback.
No coordinate-specific special case, output deduplication or tolerance is
introduced. Other geometry residuals and complete TiQ2 closure remain
unproven beyond the explicitly recorded next residual.

## Bounded verification conclusion

The complete-byte runtime RED fails before the change at line22564. With
only the four retry-coordinate expressions corrected, all bytes agree
through33323 lines; the next first difference is footer emission at33324:
Native emits `; filament end gcode`, while Rust emits
`G1 X-19 F3000 ; home X axis`. No output is filtered and complete TiQ2 parity
is not claimed. This is a bounded source-arithmetic slice; the still-failing
whole-byte test is not registered. The six smooth Type2 groups and exhaustive fan command
sequence comparison pass after the correction.

## Verification boundary

The independent route RED uses the Native-captured square and exact post-wipe
endpoints. It fails with five route points before the correction and passes
for both travel directions afterward. Existing crossing-detour controls remain
unchanged; avoidance and cooling directed controls pass29/29.

Fresh seeded v2 base/candidate gates on977c6e923 pass758/758 on both sides,
with0 skipped. All605 common runtime streams are accounted for:604 Ares raw
streams are identical, and only the target TiQ2 travel stream changes. All605
Native streams remain identical after generated-by banner normalization.
The target stream's next first difference remains the footer documented above.

Debugging receipts and generated streams remain runtime artifacts only.

