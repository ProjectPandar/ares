# AdaptiveCubic no-op modifier: missing sparse paths

Status: SPEC-REVIEW-REQUEST for diagnosis/public RED first. No production fix is
proposed as established yet; a localized source boundary must be reported and
approved before GREEN. Docs only during the FULL16 freeze.
Base: `98a2bfc325b2a06ba4e8b95b9719acd827e2f8f1`; rebase to the next merged main
before any runtime work. Upstream: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Existing reproduced difference

W3's accepted STOP `3be5e7af1` records an equal-config modifier control A-C1:
Native accepts it in both modes. Its official public API/CLI run on exact
main98a2 admitted but failed the unchanged whole-byte comparator. A-C0, the
same body without a modifier, passed that comparison.

Frozen input root: `/tmp/worker3-object-spacing-native-1tgobnaj/inputs`.
A-C0 SHA256 `d6ed45dca46d2b9f7a395df577083f37fb3f747e89b69be70eb956b9868484dd`;
A-C1 SHA256 `7fd8549eacaff99b71fe3b5eaea58d365a545d831174255cdc2bec1a69005619`.
A-C1 is one 20x20x10 Classic ModelPart with a rightmost-quarter parameter
modifier. Both regions request AdaptiveCubic, density30%, width0.45, multiline1.
This is the identity case for W3's region-spacing average: changing the mean
alone cannot account for the missing paths.

Preserve the real first whole-file difference: byte454 / line18,
Native `M73 P0 R12` versus Ares `M73 P0 R10`; 5024 versus3376 lines.
The first motion difference is Native line609 versus Ares line612:
the Native Inner wall XY/E line versus an added Ares `G1 F1180`.
That is not evidence that the wall geometry itself differs.

Offline parsing of these existing streams, with no new Native/API run, gives:

| Input | Positive Inner wall XY/E | Positive Outer wall XY/E | Positive Sparse XY/E |
| --- | --- | --- | --- |
| A-C0 Native / Ares | 200 /200, ordered equal | 200 /200, ordered equal | 1207 /1207, ordered equal |
| A-C1 Native / Ares | 200 /200, ordered equal | 200 /200, ordered equal | 1207 /0 |
| A-C2 outside modifier Native / Ares | 200 /200, ordered equal | 200 /200, ordered equal | 1207 /0 |

The diagnostic classification removes F only from the counted positive XY/E
records; it does not replace or relax any public comparator. Native's first
Sparse extrusion is physical layer4/Z0.8, raw line632 for A-C1. Missing sparse
paths could explain the earlier time/feedrate changes, but that downstream
causal link is an inference, not a localized production root cause.
Original report: `public-preflight.json` in the W3 root. Derived offline tables:
`/tmp/worker6-modifier-existing-stream-classification.json` and
`/tmp/worker6-modifier-positive-extrusion-classification.json`.
Only official public Ares comparison was run by W3; do not claim a seeded
public RED from Native-only seeded admission.

## Owning upstream and Rust rewrite boundary

`PrintObjectSlice.cpp:401-426` builds per-volume region slices, intersects a
modifier with its actual parent, subtracts the stolen area from that parent,
and retains the modifier source for its next parent record when needed.
At :448-472 it sorts by region/volume identity, merges equal-region fragments
with the existing closing operation, and publishes each region's slice.
This owns modifier split/recomposition; do not replace it with a new region
pipeline or use a first region as a proxy for all regions.

`LayerRegion::make_perimeters`, LayerRegion.cpp:82 onward, receives compatible
regions and owns generated fill_surfaces/fill_no_overlap. `Layer::make_perimeters`,
Layer.cpp:183-248, merges compatible slice groups before wall generation;
:250-260 intersects the resulting fills back into each contributing region.
The slice will preserve these boundary contracts rather than changing the
wall geometry algorithms or compatibility option policy.

`group_fills`, Fill.cpp:1016-1049, maps each region surface to its interned
parameters and appends each eligible expolygon into the correct SurfaceFill;
region_id_group and no_overlap geometry retain all contributing regions.
`Layer::make_fills`, Fill.cpp:1213-1248, selects the effective filler/object tree;
:1274,1326-1336,1353-1356 retains the actual region owner and emits every
expolygon's paths into that region's fills. A no-op modifier must not cause
all sparse geometry to disappear at these transitions.

Rust targets are the corresponding existing adapters:

- `region_slices/complex.rs::{partition_modifier,write_regions}` and existing
  volume-region identities, only if the first observed loss is in composition.
- `perimeters/groups/redistribute.rs::apply`, only the generated-fill split;
  no counterbore or wall-generator algorithm change.
- `group_fills.rs`, `group_fills/params.rs` and `group_fills/coalesce.rs`, for
  source-owned surface/parameter/region membership retention.
- `fill_entities/generate.rs`, only collection-owner/range handoff if the
  generated sparse paths are present before it and lost there.

These are localization candidates, not authorization to change them all.
Observe the first sparse layer's region polygons, post-perimeter fill surfaces,
projected/coalesced SurfaceFill entries and emitted collections in that order.
The first unequal value must name one exact upstream/Rust function before
GREEN review. If all these values agree and the loss occurs inside Adaptive
filling/tree construction, STOP and report outside this slice's ownership.
No new production behavior is authorized from output counts alone.

## Confirmed ownership and deferred behavior

Coordinator assigned W6 only the modifier sparse-loss boundary. W3 retains
adaptive object-region spacing, SupportCubic/Slice B and touching.rs/S-C0.
Do not edit `fill_entities/adaptive.rs`, its spacing/preparation helpers,
`fill/adaptive`, or the connector. Runtime tree/spacing inspection may only
classify an out-of-scope cause; it does not authorize changing it.

Also defer counterbore/perimeter algorithms and region-compatibility option
policy, brim/support/ironing, modifier wall-loop admission, parser/config
resolution, gcode cooling/time algorithms and non-Adaptive pattern algorithms.
Wall positive XY/E is already equal in the known witness; speed/time differences
remain in the strict comparator and may not be filtered from acceptance.
Existing adapters are temporary shells around the cited PrintObject/LayerRegion
and SurfaceFill concepts, not an independently designed Ares pipeline.

## Proposed Native-first reproduction and RED

After approval AND freeze release, CPU11 outer taskset, jobs1/threads2,
canonical official/seeded environments, fresh per-mode roots/caches. No full
or workspace. First rerun frozen A-C0/A-C1 once per mode with public validated
API and CLI against Native; preserve every outcome. All equal means NO RED
FOUND/STOP, not a search for replacement geometry. Native refusal/rewrite is
reported before changing an input.

If reproduced, observe one first-loss case using only existing Native/Ares
binaries and read-only gdb. No Native rebuild, inferior writes/calls or added
production probes. Keep ordinary/debugger streams and verify observer
neutrality with the same complete-output comparator. A missing/optimized
symbol is recorded rather than reconstructed as an observed local.

Public RED uses authored geometry/placement and runtime canonical presets;
retain the body/inside equal-config modifier pair and a Rectilinear counterpart
pair as controls. The existing tracked modifier fixture's authored model
packaging may supply geometry; runtime-exported projects and profile carriers
remain outside Git. Use existing fixture adapters and OrcaRunner cache, no
second oracle/cache. Report any difference from the frozen reproducer before
substituting a newly generated case as its RED.

All selected requests must have runtime Native admission and retain the strict
whole ordered bytes, generated-by normalization only, exact line count/layer
schedule, public API=CLI raw, plus Native actual sparse activity. The positive
XY/E counts are additional diagnostics, never a polygon/footer-only substitute.
Any real RED is committed separately and reviewed before a production amendment.
Files<=400 LOC; no new guards/ignore/allow/fallback, no assertion retirement or
Native products. Subsequent GREEN gates require both-mode frozen/control
comparisons, option_coverage, strict/fmt/wasm at exact hash; full/workspace remain
coordinator-only. No runtime work accompanies this SPEC request.
