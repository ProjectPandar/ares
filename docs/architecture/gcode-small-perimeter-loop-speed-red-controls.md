# Small-loop independent RED control admission

Approved spec: `6cf6cde7a37f72a9c92cc713dae2525338b6b1af`.
Control base: main `3b6e0ebdd4143bb7559a480da916cab2bae58e7b`.
This plan is frozen before Native execution; no production implementation.

Public E2E seam: Native CLI export/slice plus `ares_core::slice_project_plate`
on the same exported byte input. The existing complete-byte comparator is
unchanged; only generated-by identity is normalized. M73/footer remain enabled.
No emitter-private API, test-only production API, mocks, or ignored tests.

Failure inventory: a short bare path or open multipath may incorrectly receive
loop speed; a short fragment of a long loop may override its parent's speed;
true small loops may lose the override; explicit caller speed may be replaced;
percentage may use inner instead of outer speed; zero may fail to select the
half-outer fallback; threshold equality may use the wrong comparison/units;
loop clipping may alter the eligibility length; unrelated export, geometry,
flow, travel or CLI rejection can prevent reaching the intended boundary.
Preserve all such evidence, distinguish them from the intended RED, and do
not patch those other boundaries or tune the fixture to obtain a desired result.

## Frozen geometry and configuration

All prisms have height0.6mm; three0.2mm layers. STL construction uses welded
adjacent occupied cells with no duplicate internal faces. No supports, infill,
top/bottom shell, overhang/curl slowdown, arcs, resonance or volumetric/cooling
limits; outer width0.4mm, outer speed120, inner200, seam gap0. M73 retained.
Settings are verified against the exported input before either comparison.

| Case | Shape / owning source | Threshold / small speed | Native non-first-layer witness |
|---|---|---|---|
| bare-path | classic thin wall0.3×8; PerimeterGenerator.cpp:230–232 variable_width | 5 / 30% | Outer F7200; bare entity must be confirmed, not inferred from TYPE alone |
| open-multipath | Arachne thin wall0.3×8; traverse_extrusions:548–566 | 5 / 30% | Outer F7200; one-member open multipath remains nonloop |
| true-loop-percent | classic square8×8, centerline perimeter30.4mm | 5 / 30% | Outer F2160 (outer120×30%, not inner200×30%) |
| true-loop-zero | same square | 5 / 0 | Outer F3600 if CLI admits zero; exact rejection means fallback remains unreachable/uncovered at this seam |
| long-loop-fragments | Arachne two8×8 squares joined by4×1.2 neck (overall20×8) | 5 / 30% | Outer F7200; full closed loop / short fragment premise still requires topology evidence |
| threshold-below | classic square | 4.8383102699936167 / 30% | Outer F7200 |
| threshold-equal | classic square | 4.8383102699936176 / 30% | Outer F2160 |
| threshold-above | classic square | 4.8383102699936185 / 30% | Outer F2160 |
| explicit-skirt | square, skirt1, height3, distance2, support_speed80, skirt_speed0 | 100 / 30% | non-first-layer Skirt F4800 |

The threshold literals come from **scaled** upstream
`libslic3r.h:84 SMALL_PERIMETER_LENGTH(r)=((r/1e-6)*2*PI)` for an axis-aligned
30,400,000-unit closed centerline. IEEE64 evaluation gives respectively
30,399,999.999999996;30,400,000;30,400,000.000000004. Do not label them actual
endpoint coverage unless the exported fixture's complete loop length is
confirmed to be30,400,000 scaled units. These are source-derived declared
samples, not a claim to cover every legal value.

## Explicit incoming speed reachability

Actual upstream caller found: `GCode.cpp:4438,4441 generate_skirt` invokes
`extrude_loop(loop,"skirt",support_speed,...)`. Corresponding Rust path is
`motion/skirt_brim.rs::emit_skirt_loop`; preserve its explicit-role behavior.
`GCode.cpp:4503,5395` also dispatches brim with explicit support speed;
`:6224` has support-entity loop dispatch. The skirt control covers a genuine
non-default loop caller, not a synthetic custom G1.

Ordinary perimeter dispatch (`GCode.cpp:6144`) passes default `-1`. The
explicit-speed **eligible perimeter-role** branch is not claimed covered
by the skirt case, whose role is not eligible for small-perimeter speed.
No new public or test-only production seam is added to force that branch.

## Executed classification (pre-implementation)

Evidence `${TMP_ROOT}/ares-tiq2-loop-red/audit-final.json`: eight input/output/producer
Slice receipt bindings and model-byte invariance, audit exit0. Final registered
controls have three intended REDs and five byte-PASS controls; no TIMEOUT.

| Case | Native / Ares non-first feedrate | Classification |
|---|---|---|
| classic bare thin wall | 7200 / 2160 | intended loop-applicability RED |
| Arachne open multipath | 7200 / 2160 | intended loop-applicability RED |
| true square loop percentage | 2160 / 2160 | complete-byte PASS |
| threshold below | 7200 / 7200 | complete-byte PASS |
| scaled threshold equal | 2160 / 7200 | intended upstream scaled `<=` boundary RED |
| threshold above | 2160 / 2160 | complete-byte PASS |
| explicit skirt | 4800 / 4800 | complete-byte PASS |
| unchanged existing overhang loft | role-speed12000 witness; byte-identical | byte-PASS control; long-loop **short eligible fragment** coverage remains INCOMPLETE |

The coordinator explicitly accepts equality as part of the source slice:
port the scaled-coordinate macro/comparison in the loop caller. Do not
change/tune mm values, add an epsilon, or patch geometry/numerical code.
The existing path-length mm comparison is not the source-owned predicate.

The additional loft is an unchanged copy of
`runner/initialized_overhang_intersections/model.rs::loft`: previous/current
closed outlines across z0/0.1/0.3/0.4; classic generator, outer width0.42,
outer200, overhang50/30/20/15, threshold5 and small30%. It is a separate
predeclared source fixture, not a modified Dumbbell. Generated STL SHA:
`a5f86fc0e6628b18c7f3fa0b05319bc4ab849138234e2a05aeed2ea6b0152b03`.
Its PASS does not prove a short Inner/Outer fragment of a long loop exists:
short segments might have only overhang/bridge roles (excluded upstream).
No third geometry search or sample tuning is credited.

### Unreachable zero and transport evidence

Fresh zero Export stderr is exactly:
`Param values in 3mf/config error: \nsmall_perimeter_speed: 0 not in range [1.000000,340282346638528859811704183484516925440.000000]\n`.
`PrintConfig.cpp:2145` sets min1, despite `extrude_loop`'s zero branch and
zero/auto tooltip. Thus this CLI seam cannot exercise the zero fallback.
The evidence is retained; no permanently failing fallback test or testing-only
production API is registered, no definition/minimum is changed, and zero
fallback coverage is **UNREACHABLE/INCOMPLETE** rather than PASS.

Initial smoke overrides replaced requested thin-wall/Arachne values and
Native export serialized all three17-digit thresholds as4.83831. Input
application guards rejected those products before Slice. Corrected transport
removes the two conflicting overrides and restores the **unchanged declared
threshold string only** in project_settings after export, before both slicers.
All other archive member payloads remain unchanged; final input is receipt-bound.
The corrected-transport round reused the two valid original percentage/skirt
references with observed Export/Slice cache hits, not rerun. Final lint-only
test restructuring also introduced the fixed loft overhang50/30/20/15 keys
in common settings (disabled for all other cases): model bytes and active
parameters remain unchanged. Consequently final verification has seven new
input/producer-bound cache misses (one Native Slice each) and only the loft
cache hit, not the predicted all-hit round. These actual statuses are preserved
in `audit-final.json`; the directory name `final-cached-verification` is not
a claim of eight hits or fresh Native provenance for the loft. Three intended
REDs remain RED; this round did not tune models/active samples to get GREEN. The first parser erroneously counted
CHANGE_LAYER rather than actual LAYER_CHANGE; original failed logs remain
and corrected parsing uses the same Native outputs. Rust1.96 admission failures
from external cwd are preserved separately; pinned1.99 fixes that driver,
without Native retries. The shared-target lock attempt is also excluded.

### Separate Dumbbell time discrepancy / handoff ARD

The frozen Dumbbell's non-first Outer speed is7200 on both sides; only the
first layer has multiple widths. It did **not** demonstrate the intended
short eligible fragment predicate. Its original complete-byte DIVERGENT is
retained, not relaxed or converted to PASS. First difference:
byte1718, line81: Native `M73 P67 R0` precedes
`G1 X119.69 Y113.69 E.22677`, whereas Ares emits that move before M73.

Owning independent boundary is time-estimation/progress scheduling
(`GCodeProcessor` post-processing and Rust
`project_slice/gcode_emit/processor/{time,schedule}`), not small-loop caller
selection. This record identifies first evidence, not a completed diagnosis
of every downstream difference. The coordinator will assign it separately.
Do not change time estimation, M73 placement/footer, comparator or the
Dumbbell fixture in this slice. It is removed from this slice's registered
RED controls; original model/input/complete streams/receipt remain under
`corrected-transport/03-long_loop_fragments/`. 本片 RED 不依赖该差异。

## Directed execution and records

Fresh isolated caches; explicit Native/resources identity; seed0 and CPU0;
ARES_ORCA_RUNS1. One attempt/case, shared whole-case deadline300s with owned
process/descendant termination and partial captures; TIMEOUT overrides PASS.
Only one heavy task at a time, build/runtime/test limits16 threads.

Persist input, source model, settings request, Native channels/metadata,
producer/cache receipts, complete Native/Ares streams and comparator detail.
Non-first-layer feedrate witnesses must pass independently of comparison;
wrong witness or unreachable topology is INCOMPLETE coverage, never a
substitute for RED. Preserve every original failed run without a reference retry.
TiQ2's complete-byte receipt and line439 travel are excluded from this
independent control slice and await worker2 merge/rebase.
