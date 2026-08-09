# Third-party notices

This file identifies source and license provenance for source-cited Rust
rewrites and linked dependencies in Ares. It does not set the license of the
repository or of unrelated code.

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

`crates/ares-core/src/geometry/bridge_direction.rs` rewrites only the
`unordered_map<double, ...>` hash, unique-emplace, bucket-list, growth, and
rehash iteration control flow required by the same deterministic compatibility
target. The separately audited official `microsoft/STL` tag is
`vs-2022-17.14`; the relevant `xhash` and `type_traits` SHA-256 digests are
`b5b183c4fb05fa5c1079a6eb79b7de6b395bd5cb405c09832820e89e82423435` and
`357e102b4e6ab85a864980a01bba28440791311df288b8987b580e577c928d5c`.

Microsoft C++ Standard Library Copyright (c) Microsoft Corporation. Licensed
under the Apache License v2.0 with LLVM Exception; see
`LICENSES/Apache-2.0-WITH-LLVM-exception.txt`. These are independently audited
compatibility targets, not a claim that OrcaSlicer's workflow pins that
toolset. Ares does not link or invoke the MSVC STL or runtime.
