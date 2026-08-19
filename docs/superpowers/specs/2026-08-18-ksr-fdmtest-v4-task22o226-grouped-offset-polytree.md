# Spec: KSR FDM Test V4 task22o226 grouped offset PolyTree execution

## Observable contract

Two-stage ExPolygon offsets submit every intermediate contour and hole to one `ClipperOffset` execution before PolyTree cleanup. Negative second-stage offsets therefore preserve the same root traversal and coordinates as OrcaSlicer instead of independently cleaning each path and unioning the results afterward.

The contract is geometry-driven for arbitrary disconnected ExPolygons. Production code does not inspect fixture names, reference G-code, coordinates, digests, or known output text.

## Upstream boundary

This slice ports OrcaSlicer 2.4.2 `src/libslic3r/ClipperUtils.cpp:578-610` `offset2_ex` / `closing_ex` and the grouped `offset_paths<ClipperLib::PolyTree>` execution implemented by `deps_src/clipper/clipper.cpp:3460-3520`. It corrects the closing operation consumed by `src/libslic3r/LayerRegion.cpp:576-580` `expand_merge_surfaces`.

Included: grouped positive/negative PolyTree offset execution, negative root order, and first-layer external-surface closing geometry. Deferred: rectilinear intersection arithmetic, remaining arc numerics, travel/retraction and wipe ordering, cooling, timing/M73, and later G-code parity.
