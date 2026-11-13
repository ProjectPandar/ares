# Loop-inward retained-arc endpoint clipping

## Upstream boundary

Authority: OrcaSlicer 2.5.0, commit `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

1. [`GCode.cpp:8583`](https://github.com/SoftFever/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/src/libslic3r/GCode.cpp#L8583) calls `loop.clip_end(clip_length, &paths)` before emission and inward sampling.
2. [`ExtrusionEntity.cpp:330–345`](https://github.com/SoftFever/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/src/libslic3r/ExtrusionEntity.cpp#L330-L345) implements `ExtrusionLoop::clip_end`. It copies paths, removes consumed tail paths, and clips the last remaining polyline.
3. [`Polyline.cpp:692–731`](https://github.com/SoftFever/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/src/libslic3r/Polyline.cpp#L692-L731) implements `Polyline3::clip_end`. It linearly clips points, trims retained fitting data, clips the final arc, and writes its projected endpoint into `points.back()` while preserving Z.
4. [`GCode.cpp:8763–8824`](https://github.com/SoftFever/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/src/libslic3r/GCode.cpp#L8763-L8824) consumes the prepared paths in `wipe_on_loops_destination`.
5. [`GCode/WipePathHelpers.cpp:920–960`](https://github.com/SoftFever/OrcaSlicer/blob/40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19/src/libslic3r/GCode/WipePathHelpers.cpp#L920-L960) samples forward and backward from clipped geometry, derives an angle, and rotates the inward offset point.

## Rust boundary and decision

Destination: `crates/ares-core/src/project_slice/gcode_emit/motion/loop_paths/inward.rs::clipped_last`, called by `post_loop_destination`.

The helper's linear tail reconstruction is a temporary compatibility shell around upstream `Polyline3::clip_end`. Retained arc endpoint projection belongs to that clipping boundary and must precede inward sampling.

Reuse `motion::arc::clip_fitting_end`, which ordinary emission calls in `motion/path.rs`. Supply emission's `CoordinateScale` and owned temporary XY points and fitting data. Copy a changed final XY endpoint into the clipped path without changing its Z. Do not introduce another projection helper or duplicate circle arithmetic.

## Included behavior

- Project a clipped planar last-path endpoint through retained fitting before inward path-length measurement and backward sampling.
- Preserve the existing fitting-trim and arc-validity semantics, including the linear result when a retained arc cannot be clipped.
- Preserve the original shared paths and fitting, untrimmed borrowed points, and z-contoured behavior.
- Preserve loop eligibility, consumed-path removal, orientation, epsilon, sampling order, inward rotation, cursor recording, and downstream wipe arithmetic.

## Deferred behavior

- `WallToolPaths.cpp::fixSelfIntersections` near-self-intersection correction.
- Arachne outline preparation, raw junction generation, stitching, and seam selection.
- New arc fitting, z-contoured clipping changes, general clipping refactors, and new geometry APIs.
- Configuration, initialized preset defaults, browser transport, viewer data, and lifecycle changes.
- Comparator changes, additional normalization, missing-output equality, fixture or golden regeneration, and Native rebuilding.

## Failure modes and verification contract

- Native startup failure or unstable reference replay prevents a meaningful product comparison.
- A clipped retained-arc endpoint that bypasses projection changes the geometry consumed by inward sampling and can change wipe extrusion allocation.
- Additional CONFIG_BLOCK or unrelated output differences remain outside this clipping correction.
- Skipped comparisons, empty or missing output, and comparator relaxation cannot establish output parity.

The existing thin-gap seam-embedding Arachne E2E and its Classic control exercise the canonical byte-in slicing API and CLI. Their complete-output comparison retains only the existing generator-banner allowance. No private or unit-only witness replaces that public boundary.

## Platform boundary

Core remains portable, memory-only, and UI-neutral for browser WASM, Windows, macOS, and Linux. This correction introduces no filesystem or terminal behavior, unsafe code, dependency, global state, feature flag, or public API change.
