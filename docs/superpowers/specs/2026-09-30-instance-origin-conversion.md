# Integer print-instance origin restoration

## Upstream boundary and Rust destination

Port `PrintApply.cpp:150`'s separately scaled instance translation,
`PrintObject.cpp:97-110,126-130`'s scaled raw model center and instance
shift addition, and `GCode.cpp:5385,5408,5442`'s
`set_origin(unscale(instance.shift))` into
`ares-core/src/project_slice/gcode_emit/footprint.rs`, initial emitter
setup, and per-object layer emission. Reuse the existing source-cited
`raw_intersections::raw_center` implementation.

## Observed failure

Rolohaun Delta Flyer Refit's complete output differs only at an internal
solid infill endpoint: Y6.014 versus Y6.013. Integer geometry agrees.
The raw world bounding-box center used at each layer is
2.3967849999999995; the unscaled integer instance shift is
2.3967849999999999. With local Y=3616715 the latter reaches the upstream
6013.5 formatting input. This is not a geometry or formatting tolerance.

## Scope and constraints

Restore emitter origins by adding separately truncated integer instance
translation and raw model center, then unscaling once. Preserve existing
local path/cursor representation as a temporary compatibility shell. Do
not replace the origin with a rounded world bounding-box midpoint.

Select the initial source object from the resolved printable objects, not
the first source-file object. Select translation from a printable instance,
as `PrintApply.cpp:145` does; nonprinting source objects/instances must not
supply an emission origin. An empty resolved set retains a zero initial
origin. This does not expand multiple printable instances into new paths.

World-space bounds used for labels, scheduling, skirt footprint and
templates remain unchanged in this slice. General multi-instance transform
expansion and other printer/option differences are deferred. This is an
upstream instance-origin rewrite slice, not an Ares-owned pipeline feature.

Known adjacent limitations exposed by the nonprinting-prefix regression
remain deferred: world-bounds-dependent start templates can reject this
layout, and the existing label scheduler confuses resolved and source
indices. That fixture clears start G-code and checks emitted movement,
not complete G-code parity or correct object labeling. The normal printer
smoke comparisons still check the complete official output.

## Acceptance

- Official complete-output Rolohaun smoke fails before and passes after.
- No tolerance or source-structure pinning tests; complete outputs retained.
- Validate positive/negative translation, separate center quantization,
  nonprinting source prefixes and existing multi-object/extruder-offset behavior.
- fmt, Clippy, workspace Nextest, platform compilation and independent
  six-aspect review; commit/push code and docs together after approval.
