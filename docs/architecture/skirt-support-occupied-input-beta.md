# Skirt support-occupied input: beta qualification and RED

## Decision

Do not repair skirt seam selection or offsetting against unequal occupied
outlines. This record qualifies the input boundary and admits a public RED;
it does not approve a production port.

The source boundary is OrcaSlicer beta revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `Print.cpp:4025-4080`, `Print::_make_skirt`: collect object outer contours
  and support-fill points through skirt height, then construct a local hull.
- `ExtrusionEntity.cpp:67-73`, `ExtrusionPath::collect_points`: append each
  existing polyline point's XY coordinates, without inflation or arc sampling.
- `Print.cpp:4112-4131`: offset an occupied hull and simplify skirt loops.
- `GCode.cpp:5829-5854,5935-5938`, `Skirt::find_start_point` and
  `GCode::generate_skirt`: choose the first skirt's desired start point.

The Rust destination boundary is
`project_slice/gcode_emit/skirt/occupied.rs::items`, which consumes
`InstalledSupportLayer::support_fills` through `for_each_path`; downstream
consumers are `skirt/loop_generation.rs` and `skirt/geometry.rs`. These modules
are compatibility shells around the cited upstream concepts, not an
independently specified skirt pipeline. Their presence does not establish
that the upstream occupied-outline inputs have been reproduced.

## Real fixture and public witness

`tests/parity/skirt-support-occupied-input/support-overhang.3mf` is the existing
organic-support overhang project with support ironing configured as gyroid.
Its exact container SHA-256 is
`58bf8c9a7a28cc19d7ee61c45458ae319dbce6882036cc7a7f6dde06d60c288d`.
It is retained unchanged from the qualified runtime capture; no mesh or
preset rewrite is introduced for this RED.

On base `f4ea41332cdcb7eadf9adbc7f7c1936c0ad7e9c5`, first-layer travel to
`TYPE:Skirt` is:

- Native line 60: `G1 X95.517 Y99.935`.
- Rust line 61: `G1 X94.881 Y99.083`.

The unchanged complete-output comparator fails earlier, at byte 1084,
line 40: Native emits `G1 X2.3 F5000`, while Rust inserts `M73 P1 R31`.
That timing/progress prefix remains a separate, unassigned discrepancy.
Neither its removal nor motion-only comparison is an acceptance exception.
The RED retains the complete ordered comparator, feeds, progress, metadata,
and footer, and requires two runtime Native captures with the skirt premise.

## Measured input qualification

Read-only seeded Native observations and temporary Rust observations show:

| Boundary | Native | Rust | Qualification |
| --- | --- | --- | --- |
| Highest included skirt Z | 0.4 | 0.4 | Equal |
| Object-contour input | 12 points | 12 points | Ordered integer coordinates equal |
| Additional support-fill XY input | 603 points | 585 points | Unequal |
| Local occupied hull | 41 vertices | 39 vertices | Unequal coordinates, not merely order |
| First offset distance | 2188539.75 | 2188539.75 | Equal |
| Second offset distance | 2565619.25 | 2565619.25 | Equal |
| Generated skirt loops | Captured | Captured | Unequal |

The differing support collections are inputs to the hull computation; a
raw point-order difference alone is not claimed as the geometric cause.
The unequal hull coordinates establish that collection order alone cannot
explain the result. No equality of support entities, support layer geometry,
or their earlier producers is claimed.

Native's emitted loop and computed desired point were captured; its
start-angle local was optimized out. Rust's generated loops were captured,
not its later seam-selection locals. Equal seam inputs are therefore not
qualified, and the earlier generated loops already differ. Likewise, the Native spacing local was unavailable; the two
offset distances were captured directly, not substituted from Rust.

The Native observer outputs reproduce the public Native output except for
the generator comment. The unmodified base produces complete bytes equal
to the earlier Rust capture. Temporary Rust observations must be removed,
rebuilt, and checked for complete-byte restoration before delivery.

## Included and deferred behavior

Included: the source-owned occupied-outline collection boundary, its measured
object/support split, and a runtime Native/API/CLI complete-output RED on
real geometry. The RED must preserve Native replay and API/CLI byte checks;
its existence does not localize the earlier M73 prefix to skirt code.

Deferred:

- Production repair of support-fill generation or its collection. First name
  and measure the actual owning support producer and its Rust destination;
  this record does not authorize a speculative support rewrite.
- Changes to hull construction, loop offset/simplification, start-angle
  arithmetic, projection, or seam selection. Equal caller inputs are not
  established for any such repair.
- Variant/filament-map materialization, ironing arc output, timing/progress,
  CornerSmoothing, 3D honeycomb, flow, presets, and footer behavior.

Do not compensate for the support discrepancy by modifying skirt distances,
seam targets, or point rounding. Do not add a legacy path or fallback.

## Review gate

ARD and RED are separate commits. Submit the measured caller boundary,
unchanged strict-comparator result, Native replay, API/CLI byte identity,
probe removal/restoration, formatting, and strict all-target Clippy for
review. A later port proposal must identify the first differing support
producer before implementation; passing this RED through an unrelated
prefix change is not proof of occupied-input parity.
