# Third-party notices

This file identifies source and license provenance for source-cited Rust
rewrites and linked dependencies in Ares. It does not set the license of the
repository or of unrelated code.

## AGG support-grid raster cell accumulation

`ares-core/src/geometry/agg_raster/cells.rs` rewrites the cell accumulation
used by `SupportMaterial.cpp::rasterize_polygons`, following
`OrcaSlicer/deps_src/agg/agg_rasterizer_cells_aa.h::set_curr_cell/line` and
`agg_rasterizer_scanline_aa.h::sweep_scanline`. The repair preserves repeated
coordinate contributions in the existing aggregate map.

Anti-Grain Geometry - Version 2.4
Copyright (C) 2002-2005 Maxim Shemanarev (http://www.antigrain.com)

Permission to copy, use, modify, sell and distribute this software
is granted provided this copyright notice appears in all copies.
This software is provided "as is" without express or implied
warranty, and with no claim as to its suitability for any purpose.

## Organic Tree collision/avoidance/wall caches, radius and settings rewrite

`ares-core/src/project_slice/support_material/tree/model_volumes.rs` and its modules
and `tree/organic/` rewrite `OrcaSlicer/src/libslic3r/Support/TreeModelVolumes.cpp/.hpp`
collision/avoidance/wall caches and precalculation, radius quantization and
`TreeSupportCommon.hpp` Organic settings. The upstream
files identify Thomas Rahm's tree supports, based
on CuraEngine, copyright (c) 2021 Ultimaker B.V., under AGPLv3 or higher.
OrcaSlicer's bundled license is retained in `LICENSES/AGPL-3.0.txt`.
This attribution also applies to the integrated Organic generator and shared
Tree geometry consumers described in ARD0086; verified output scope is recorded
in its plan, not inferred from license provenance.

The borrowed two-subject union used by these avoidance kernels follows
`ClipperUtils.cpp::union_(Polygons, Polygons)`'s subject order and NonZero
operation, implemented using the existing safe Clipper rewrite below.

## Regional perimeter and fill handoff rewrite

The regional perimeter handoff in `project_slice/fill_entities.rs` follows
`Layer.cpp:139-291` and `Fill/Fill.cpp:1361-1369`, retaining regional borrowed
views without duplicating path geometry. The OrcaSlicer AGPLv3 attribution and
retained license below apply to this rewrite slice as well.
The selected-region fill append ranges in `fill_entities/generate.rs` and
`types.rs` follow `Fill/Fill.cpp:1274,1354-1356`; the existing ironing shell
records its actual region owner (`Fill.cpp:1507-1724`).
All-region projection and global grouping in `group_fills.rs`, `params.rs`
and `coalesce.rs` follow `Fill/Fill.cpp:828-1053`, including shared parameter
state, LockedZag maps and first-owner selection with no-overlap union.
The regional support input adapter in `support_material/project.rs` follows
`Support/SupportMaterial.cpp:592-604,1162-1171,1430,1613,1784-1788,3177-3190`
for all-region surfaces, bridge detection/heights, contacts and trim views.
Its bottom-interface ratio follows `Support/SupportParameters.hpp:85-97`
over `Print.hpp:423-424` object printing regions.
Selected support-grid island flattening in `geometry/support_grid.rs`
follows `Support/SupportMaterial.cpp:739-820` and `ExPolygon.hpp:415-422`,
moving each contour and every hole in source order. The same OrcaSlicer
AGPLv3 attribution and retained license apply to this repair.

## Tree smart-overhang detection rewrite

`ares-core/src/project_slice/support_material/tree/overhangs.rs` and its
modules rewrite enabled first-pass detection, sequential sharp-tail propagation
and Tree-local overhang clusters in
`OrcaSlicer/src/libslic3r/Support/TreeSupport.cpp:705-1052`. Nearest-vertex
distance follows `MultiPoint.hpp::distance_to`, and growth rounding follows
`Point.hpp::Point(double,double)`. OrcaSlicer's README identifies the repository
license as GNU AGPLv3; its license is retained in `LICENSES/AGPL-3.0.txt`.
The integrated detector and generators retain this source attribution;
complete-output acceptance evidence is recorded separately in the Tree plan.

`tree/overhangs/finalize.rs` rewrites the geometry/type pass at
`TreeSupport.cpp:1050-1125`, using the existing bounding-box prefilter from
`ClipperUtils.cpp:684-686` for sharp-tail difference. Actual annotation
projection and downstream Tree generation are described in ARD0086. The same
OrcaSlicer AGPLv3 attribution and retained license apply.

`tree/remove_bridges.rs` additionally rewrites
`OrcaSlicer/src/libslic3r/PrintObject.cpp:4306-4436`, limited to the
break_bridge=false path called by Tree detection. It shares geometric
Polyline contracts, not the Normal SupportMaterialInternal filtering kernel.
The same OrcaSlicer repository AGPLv3 attribution and retained license apply.

## boostvoronoi 0.12.1

Ares links `boostvoronoi` 0.12.1 for its pure-Rust Boost.Polygon-compatible
integer segment Voronoi implementation. Copyright Andrii Sydorchuk 2010-2012
and Eadf 2020-2025. Licensed under the Boost Software License 1.0; see
`LICENSES/BSL-1.0.txt`. Its uncalled filesystem-reader utility is compiled by
the crate and cannot be feature-disabled; Ares neither calls nor re-exports it.

## getrandom 0.3.4 (`wasm32` feature qualification)

On `wasm32`, Ares names the already-transitive `getrandom` 0.3.4 dependency
directly with its `wasm_js` feature. This qualifies `boostvoronoi`'s `cpp_map`
dependency chain for browser builds; Ares does not call or re-export it.
`getrandom` is licensed under either Apache-2.0 or MIT, at the user's option.

## Clipper 6 closed-path Boolean, PolyTree, and closed-offset rewrite

The safe indexed implementation under
`crates/ares-core/src/geometry/clipper.rs` and
`crates/ares-core/src/geometry/clipper/`, except for `ordering.rs` described
separately below, rewrites the closed-path Boolean, PolyTree, and closed
ClipperOffset behavior of Clipper 6.4.2 as bundled by OrcaSlicer at commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, principally
`deps_src/clipper/clipper.hpp` and `deps_src/clipper/clipper.cpp`.

Copyright Angus Johnson 2010-2017. Licensed under the Boost Software License
1.0; see `LICENSES/BSL-1.0.txt`. Ares does not link or invoke the C++ library.

## MSVC STL compatibility rewrites

`crates/ares-core/src/geometry/clipper/ordering.rs` rewrites only the sort
control flow required to reproduce the separately audited MSVC STL 14.44.35207
equal-key ordering target. The relevant source boundaries are `algorithm` and
`__msvc_heap_algorithms.hpp`; their audited SHA-256 digests are
`e4cfb31da8ec07af89834d829ea72b20c7e3202476af3b0641cfe8d6ebb245d7` and
`56c6be67b7c0ff9b3ffb7d48943c1ec01728f41f0663dca2c49c296f492bf619`.

Microsoft C++ Standard Library Copyright (c) Microsoft Corporation. Licensed
under the Apache License v2.0 with LLVM Exception; see
`LICENSES/Apache-2.0-WITH-LLVM-exception.txt`. These are independently audited
compatibility targets, not a claim that OrcaSlicer's workflow pins that
toolset. Ares does not link or invoke the MSVC STL or runtime.

## libstdc++ bridge-direction traversal and Lightning stable ordering

`crates/ares-core/src/geometry/bridge_direction.rs` models the 64-bit double
hash and unique-key traversal of `BridgeDetector.hpp`'s
`unordered_map<double, ...>`. The supporting GNU ISO C++ Library source
boundaries are `libstdc++-v3/include/bits/{functional_hash,hashtable}.h`,
`libstdc++-v3/libsupc++/hash_bytes.cc`,
`libstdc++-v3/src/c++11/hashtable_c++0x.cc`, and
`libstdc++-v3/src/shared/hashtable-aux.cc`. The original Linux reference was
audited against GCC 14.2.0; the separately initialized Nix reference links
GCC 15.3.0 libstdc++ (recorded in ARD-0075). Rust does not select behavior
from the host standard library.

`crates/ares-core/src/geometry/clipper/ordering/stable.rs` rewrites the
normal-buffer `bits/stl_algo.h` stable-sort sequence (seven-element insertion
runs, paired merge passes and forward/backward half merge) used by
`Fill/Lightning/DistanceField.cpp`'s non-transitive comparator. This is a
fixed Rust compatibility implementation, not a host C++ library dependency.
`fill/lightning/layer.rs::SparseNodeGrid` also models the same GNU library's
no-hint equivalent-key insertion order from `bits/hashtable.h`, consumed by
`Fill/Lightning/Layer.cpp`'s strict grounding selection.

GNU ISO C++ Library Copyright Free Software Foundation, Inc. The reference
sources are GPL version 3 or later with GCC Runtime Library Exception 3.1;
see `LICENSES/GPL-3.0.txt` and `LICENSES/GCC-exception-3.1.txt`. Ares does not
link or invoke libstdc++ for this Rust rewrite.
