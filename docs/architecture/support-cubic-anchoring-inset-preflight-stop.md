# S-C0 priority preflight STOP: missing anchoring entry inset

## Disposition

This is a real earlier data difference, not equivalent bbox definitions.
Object-region spacing stays blocked and Inner-wall is deferred.

## Matched calls and first unequal intermediate

Read-only gdb compared the exact frozen S-C0 case on the public Ares binary
and release official Native. The readers make no inferior calls/writes.
Both have zero inspection errors; Native exits0, Ares exits101.

The first connector invocation is NOT the ordinary Z0.8 emission call.
It is the preparatory bridge-over-infill sparse-anchoring call at Z9.2,
layer45. Ares's actual stack is:
`adaptive::anchoring_lines -> filler::fill_surface -> connect_infill`.
Native's first adaptive filler call likewise has layer_id45 and z9.2;
its source-owned route is `Layer::generate_sparse_infill_polylines_for_anchoring`
through `Fill::fill_surface` into `Filler::_fill_surface_single`.

The input contour is identical on both sides:
min(-9223228,-9223228), max(9223233,9223233), ordered vertices identical.
Native's spacing is0.40707963705062866mm and overlap0.0.
The next stage differs:

| Stage at Z9.2 | Native | Ares |
| --- | --- | --- |
| Input to the base fill operation | original contour above | same contour passed directly to filler |
| Adaptive filler input min | (-9019688,-9019688) | (-9223228,-9223228) |
| Adaptive filler input max | (9019693,9019693) | (9223233,9223233) |

Native completes `float(scale_(0 - 0.5*spacing))` as -203539.8125 coordinate
units; the actual Clipper result shrinks each side by203540 units
(0.20354mm). Ares skips the offset stage on this helper route, so it enters
the adaptive filler with the original geometry. The connector subsequently
uses different cropped line endpoints and a different grid bbox. This is
observable geometry, not an equivalent bbox representation.

The ordinary Z0.8 emission route is a useful matched control: Ares's append
has overlap0, the same spacing and the same original contour, then enters
its filler with min(-9019688,-9019688), max(9019693,9019693), identical to
Native after its base fill operation. This isolates the missing entry
operation to sparse anchoring rather than the offset kernel itself.

## Exact source ownership

Upstream references under `OrcaSlicer/src/libslic3r/`:

- `PrintObject.cpp:2744-2754` generates the preceding sparse layer's anchoring
  polylines before bridge-over-infill processing.
- `Fill/Fill.cpp:1394-1405`, `Layer::generate_sparse_infill_polylines_for_anchoring`,
  selects internal surfaces; 1440-1453 creates/configures the adaptive/support
  filler and supplies layer id, print_z and the corresponding octree.
- `Fill/Fill.cpp:1492-1498` resets spacing, moves the original expolygon into
  the surface and calls **f->fill_surface**, rather than the pattern's
  `_fill_surface_single` directly.
- `Fill/FillBase.hpp:181-187`, `Fill` constructor, initializes overlap to0.
  The anchoring setup above does not replace that value.
- `Fill/FillBase.cpp:105-117`, `Fill::fill_surface`, offsets the surface by
  `float(scale_(overlap - 0.5*spacing))` before dispatching each component.
- `Fill/FillAdaptive.cpp:1321-1326` receives that already-offset expolygon;
  1378 crops lines to it, and 1401/1410 applies hooks then the connector.

Rust source under `crates/ares-core/src/project_slice/`:

- `fill_entities/adaptive.rs:248-282`, `anchoring_lines`, loops over
  `fill.expolygons` and passes each **unmodified expolygon** to
  `fill::adaptive::filler::fill_surface` at268-270. This bypasses the cited
  base fill entry operation.
- `fill_entities/adaptive.rs:49-65`, `append`, already applies the offset and
  iterates its components before invoking that same pattern-level filler.
- `prepare_infill/bridge_over_infill/sparse_anchoring/arms.rs::dispatch`
  routes AdaptiveCubic/SupportCubic to `adaptive::anchoring_lines`.

## Required ordering

The first missing operation above must be fixed before changing the
later `add_hook` collision limits, and no out-of-bbox assert should be
removed to conceal these inputs.
A follow-up would port the source base-fill entry inset for the
anchoring helper with Native's default overlap0 and completed-float delta,
iterate all offset components, then re-observe its generated polylines.
Do not alter ordinary append output, unrelated patterns, octree coordinates,
sorting, public contracts or comparison requirements as part of that fix.

The required Rust destination is `fill_entities/adaptive.rs::anchoring_lines`,
which shares its file with the `build_object_octree` coordinate work
(and `fill/adaptive/octree.rs`).

The input SHA remains9580fd4f17fad55829f561c13ee0ae336b447016c7c9682ce301f79afc9d0bc1.
Native output matches the public official reference line-for-line except
its generator date line2; no motion/configuration change.
