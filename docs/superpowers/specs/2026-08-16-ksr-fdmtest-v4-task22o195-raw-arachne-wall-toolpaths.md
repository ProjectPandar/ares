# Spec: KSR FDM Test V4 task195 raw Arachne wall toolpaths

## Observable contract

A prepared polygon outline plus scaled wall/fill spacing, inset limit, layer height, thin-feature limits, and transition options produces raw variable-width wall lines through the skeletal trapezoidation module. Beading thresholds derive from rounded-rectangle extrusion widths; the strategy uses the requested outer/inner spacing, transition distance/angle, thin-wall limits, maximum bead count, inset, and distribution count.

A 10 mm rectangle configured like concentric-internal fill yields non-empty positive-width lines whose inset indices respect the requested limit. No fixture identifiers or golden constants enter production code; files remain below 400 LOC.

## Upstream boundary

This slice ports the parameter assembly and `SkeletalTrapezoidation` invocation in OrcaSlicer 2.4.2 `Arachne/WallToolPaths.cpp:482-554` into `arachne/wall_toolpaths.rs`. Outline preprocessing, stitching, small-line removal, inner-contour separation, simplification, concentric-internal conversion, cooling, timing, and remaining exact G-code differences are deferred.
