# Third-party notices

This file identifies the source and license provenance of two source-cited
Rust rewrites in Ares. It does not set the license of the repository or of
unrelated code.

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

## MSVC STL sort-control-flow compatibility rewrite

`crates/ares-core/src/geometry/clipper/ordering.rs` rewrites only the sort
control flow required to reproduce the separately audited MSVC STL 14.44.35207
equal-key ordering target. The relevant source boundaries are `algorithm` and
`__msvc_heap_algorithms.hpp`; their audited SHA-256 digests are
`e4cfb31da8ec07af89834d829ea72b20c7e3202476af3b0641cfe8d6ebb245d7` and
`56c6be67b7c0ff9b3ffb7d48943c1ec01728f41f0663dca2c49c296f492bf619`.

Microsoft C++ Standard Library Copyright (c) Microsoft Corporation. Licensed
under the Apache License v2.0 with LLVM Exception; see
`LICENSES/Apache-2.0-WITH-LLVM-exception.txt`. This is an independently audited
compatibility target, not a claim that OrcaSlicer's workflow pins that toolset.
Ares does not link or invoke the MSVC STL or runtime.
