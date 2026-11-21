# Concentric thick-polyline rotation

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):

- `Fill/FillConcentric.cpp:130–194` handles full-density, adjustable standard
  concentric fill with Arachne. It converts each extrusion line, requests a
  start at the junction nearest the previous endpoint when `is_closed` is
  set, then clips and reorders paths.
- `Arachne/utils/PolylineStitcher.hpp:115–155,166–209` identifies a closure
  by geometry. `Arachne/WallToolPaths.cpp:655–670` marks the result closed
  and copies the first junction onto the end only when the endpoint
  coordinates differ within the stitch distance. Equal coordinates do not
  imply equal widths. `Arachne/utils/ExtrusionLine.hpp:212–230` flattens the
  junctions into per-segment width pairs without reconciling endpoint widths.
- `Polyline.cpp:657–667` in release rotates only for an interior index
  when both coordinates and widths match at the two ends. Rotation moves
  the width pairs with the segments. Otherwise it leaves the path unchanged.

## Rust destination

`geometry/polyline.rs::ThickPolyline::start_at_index` ports that release
condition. The nearest-point index and width-pair layout remain caller
contracts. Diagnostic assertions are not part of the release behavior for
an Arachne line whose endpoint widths differ.

On a synthetic X1 Carbon project with concentric internal solid infill, the
release implementation receives a seven-point loop with matching endpoint
coordinates and widths `338242` and `353389`, the same width pair reported
by the Rust assertion. Neither path conversion nor Arachne needs to
normalize those widths. Standard concentric finalization should continue
with the original start; the narrow internal concentric caller already
requires matching endpoints before requesting rotation.

The scope excludes changes to wall generation, nearest-point selection,
fill dispatch, end clipping, and final path ordering. Verify through Native
release and the public project slicing interfaces, including K1 and MK4
with the same pattern. Compare complete ordered output without ignoring
any difference beyond the generator line.
