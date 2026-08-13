# Task 22O.44 — Implementation plan

## Outcome

Implement the exact crate-private `Fill::connect_infill` dependency described
by the matching spec. O44 will not advance the prepared-project lifecycle;
public slicing remains terminal after O43 until complete CrossHatch generation
and anchor-map integration are ported.

## Source boundary

This plan implements pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`FillBase.hpp:48-52,56,58-61,100,219-224` and active
`FillBase.cpp:323-398,420-842,995-1241,1243-1252,1263-1269,1432-1566,
1580-1588,1594-1614,1690-1818`. `FillCrossHatch.cpp:178-232` and
`FillBase.cpp:1820-1829` are dispatch evidence. The matching spec's exact
dependency citations and excluded subranges govern implementation; in
particular, compile-disabled, support connector, alternate wrapper,
`chain_polylines`, full CrossHatch/grouping, and project-lifecycle behavior are
deferred. The Rust destination is crate-private `fill::connect`; legacy
`infills.rs` remains an uncalled compatibility scaffold.

## Red-green sequence

1. Add the source-owned `fill::connect` module, source-shaped parameter record,
   and a compiling empty-output stub. Add one exact nonempty connector vector
   from the pinned Orca harness and retain the behavioral RED with
   `cargo nextest run -p ares-core task22o44`.
2. Add only the narrow EdgeGrid closest-point and line-visit operations
   required by the cited helper. Audit bbox inflation, resolution conversion,
   contour/segment insertion, the inclusive closest cell rectangle, and raster
   cell order; freeze stable edge/first-win order and checked error behavior
   before composing them.
3. Port boundary copying/splitting, Euclidean parametrization, stable-index
   T-junction links, collision interval math, and occupied-boundary trimming.
   Turn on focused tests one source behavior at a time.
4. Port active arc collection/fixed MSVC sorting, the source-shaped
   `merged_with` parent-root updates, path reversal/merge, limited hooks,
   remaining-endpoint processing, and survivor emission. Do not port the
   compile-disabled or support-only branches.
5. Add exact outer/hole/multi-vertex/overlap/threshold/multiline/dont-sort/
   dual-scale vectors, off-boundary original-endpoint and negative-cast cases,
   early/late errors, and separate greater-than-32 endpoint-hit and arc adapter
   literals. Freeze the complete pinned-Orca and audited-MSVC oracle record.
   Mutate each sort call site and the other high-risk source decisions, then
   restore every production file exactly.
6. Run focused predecessor and geometry regressions, full workspace/Tier-1
   gates, static audits, the normalized golden progress probe, and independent
   reviews. Repair and re-review until both tracks approve unconditionally or
   identify a concrete blocker.

## Oracle record

The disposable direct harness is `/tmp/task22o44-direct-oracle.cpp`, SHA-256
`666cbdeeceb64ec46a486ff44e70f1b79844c0e3e53d8ec9f5c2acb480d100d6`.
Temporarily list it in pinned Orca's `tests/libslic3r/CMakeLists.txt`, then run
the following once for `Debug` and once for `Release`:

```bash
podman run --rm \
  -v /tmp/task22o38-orca:/__w/OrcaSlicer/OrcaSlicer -v /tmp:/tmp \
  -w /__w/OrcaSlicer/OrcaSlicer -e HOME=/tmp/orca-home \
  localhost/orcaslicer-linux-builder:ubuntu-24.04--cmake-4.3.0--a292a8ae33f3 \
  cmake --build build --config Release --target libslic3r_tests -j2
podman run --rm \
  -v /tmp/task22o38-orca:/__w/OrcaSlicer/OrcaSlicer -v /tmp:/tmp \
  -w /__w/OrcaSlicer/OrcaSlicer -e HOME=/tmp/orca-home \
  localhost/orcaslicer-linux-builder:ubuntu-24.04--cmake-4.3.0--a292a8ae33f3 \
  build/tests/libslic3r/Release/libslic3r_tests '[task22o44]' -s \
  | awk '/^scale (normal|large)$|^distinct_|^polyline /' \
  | sha256sum
```

Replace both `Release` occurrences with `Debug` for the second run. Their
normalized exact output is identical, SHA-256
`ba259b3e0a2b14aa4880585a759bc30ee9d028297f081822f27edcdd9d13a89d`.
The initial RED uses the comparator-distinct Normal hook literal
`[(0,3628318),(0,2000000),(12000000,3000000),(12000000,4628318)]`; the first
whole-merge literal is
`[(12000000,11628318),(12000000,10000000),(0,6000000),(0,2000000),
(12000000,3000000),(12000000,1371681)]`. The full Normal/LargeBed record is
frozen into Rust tests during step 5, not read from `/tmp` at test time.
Every endpoint tuple in the disposable harness is distinct. Its nominal arc
key multisets are: single `{17,23}`, merge `{4,7,17,26}`, threshold
`{20,23,30,37}` (the below-threshold variant replaces `20` with `19.999999`),
multiline `{0.5,0.7,26.1,30.7}`, hole `{4,5,7,8,11,12,26,29}`, and
unconnected `{18,22}`. Thus these Linux Debug/Release literals do not depend
on either host `std::sort` equivalent-key permutation.

The two fixed-MSVC adapter records are independently reproducible with:

```bash
rustc --edition=2024 --cfg test /tmp/o44_callsite_vectors.rs \
  -o /tmp/o44_callsite_vectors
/tmp/o44_callsite_vectors
```

The output SHA-256 is
`9be6075c2b41146d8b4238515501dde8e170aecfbbf9dbe7be41be35e5e1e545`.
Both 35-record adapters produce the exact identity permutation in the spec and
trace `insertion=2, median3=1, ninther=0, partition=1, heap=0`, with partition
`[35,35,1,33,1]`.

Review-added full-vector provenance is independently reproducible from
`/tmp/task22o44-decisions-oracle.cpp`, SHA-256
`6ecdb2899d0db6f2b5d6de3de9aa916b9c574887db78872d7fbd3cae7e763869`.
Temporarily list it in the same pinned Orca test CMake file, build and run both
Debug and Release with filter `[task22o44-decisions] -s`, and normalize all
four cases with:

```bash
awk '
  /^scale (normal|large)$/ { print; next }
  /^(remaining_threshold_equal|distinct_dont_sort_multiline_2|equal_complete_and_hook_sides|multivertex_interior_top) count=/ { print; next }
  /^polyline / { print }
' | sha256sum
```

Both configurations produce SHA-256
`08e846d991b104443e8c96009206ca050e774e358328e3ea7b3005f86ab3fe2c`.
The equal-side case has deliberate arc keys `{5,5,90}` mm but disables sorting;
the multi-vertex case has distinct sorted keys `{17,23}` mm. The remaining
equality and `dont_sort`/multiline cases also disable sorting and have nominal
keys `{20,23,30,37}` and `{0.5,0.7,26.1,30.7}` mm. All endpoint tuples are
distinct. The pinned CMake list was restored to SHA-256
`c5087ca8a66be47ddefeeaaa6787b50485fec4a3f025fdbb0a67797c9834f224`,
and no disposable oracle remains registered.

## Implementation shape

The crate-private seam is exactly:

```rust
pub(crate) fn connect_infill(
    infill_ordered: Vec<Polyline>,
    boundary: &ExPolygon,
    spacing: f64,
    params: FillConnectionParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError>;
```

`FillConnectionParams` contains source-typed `f32` anchor and anchor maximum,
`i32` multiline, and `bool dont_sort`; it does not contain density or already-
scaled integer distances. Its fields are crate-private so the next source-
cited sibling filler can construct the record without a public API.

- `crates/ares-core/src/fill.rs`: crate-private source-owned Fill module root;
- `crates/ares-core/src/lib.rs`: declare the crate-private Fill module;
- `crates/ares-core/src/fill/connect.rs`: source-shaped connection parameters,
  interface, and orchestration;
- `fill/connect/types.rs`: T-junction state, stable links, and working graph;
- `fill/connect/contour.rs`: contour distances, interpolation, full/limited
  path taking, and local path mutation;
- `fill/connect/scale.rs`: continuous scaled distances, source-specific rounded
  and truncating checked coordinate conversions, and checked rounded bbox
  inflation;
- `fill/connect/collision.rs`: source interval clipping and rounded-thick-
  segment collision math using local cast-before-subtract `f64` vectors rather
  than the integer-subtracting Ares `Line` helper;
- `fill/connect/touching.rs`: ordered raster visits and occupied-boundary
  interval updates;
- `fill/connect/graph.rs`: endpoint association, source-required boundary
  copy/splitting, and parametrization;
- `fill/connect/apply.rs`: arc ordering, union-find merging, hooks, and exact
  survivor emission;
- `fill/connect/tests.rs` plus focused shards: exact in-process vectors;
- `geometry/edge_grid/query.rs` and tests: the audited closest-point and line-
  cell queries, split from the existing module to remain below 400 LOC;
- `geometry/clipper/ordering.rs` plus its geometry re-export: widen the already
  audited fixed MSVC-sort helper only to crate-private visibility.

Keep the interface crate-private and consume owned paths. Represent Orca raw
pointer links with stable indices and the source-shaped `merged_with` parent-
root table. Borrow the source ExPolygon but create exactly the required
contour-then-hole working copy for T-junction insertion; make no additional
defensive ExPolygon clone. Do not expose the graph, add public options, reuse
`infills::anchored_segment`, or create a prepared lifecycle wrapper. Until the
CrossHatch caller lands, use one narrow reasoned
`#[cfg_attr(not(test), expect(dead_code, reason = "..."))]` on the inactive
source seam and remove it at integration rather than fabricating a caller.

## Verification

```bash
cargo nextest run -p ares-core task22o44 --no-fail-fast
cargo nextest run -p ares-core \
  -E 'test(/(task22o44|edge_grid|polyline|bounding_box|task22f_fixed_sort)/)'
cargo nextest run -p ares-core -E 'test(/task22o(2[4-6]|4[0-4])/)' \
  --no-fail-fast
cargo nextest run --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown
cargo nextest run -p ares-cli --test ksr_fdmtest_v4 \
  -E 'test(project_matches_orca_242_except_generator_line)' \
  --run-ignored ignored-only --no-fail-fast
```

Also require `git diff --check`, every Rust file below 400 physical lines, no
production `include!`/`include_bytes!` source splitting, no production
fixture/hash/reference-G-code reads, exact pinned-Orca source restoration, and
fresh independent review evidence.

## Completion record

- Stub RED: Nextest `61fe52e1-dd5e-4128-8333-3b6dd160cc54` failed because the
  empty connector returned `[]` instead of the exact four-point hook.
- Final focused: Nextest `17d49a39-7781-473c-bc8c-b25dd2b5ab19`, 41/41.
- Geometry/fixed-sort band: `255de89d-c07f-4eac-ac7a-ac0a11119f67`, 76/76.
- O24-O26/O40-O44 band: `0395cc2e-fc9b-434e-b3a8-8879956a9267`, 194/194.
- Workspace: `e7097ac0-0f71-4eea-8da9-8d0935382928`, 6,201/6,201,
  27 slow, two skipped.
- Workspace warning-denying Clippy, rustfmt, wasm32 core/adapter checks, diff,
  LOC, include, fixture-read, and restored-source audits pass.
- The ignored normalized golden probe `f48f6f75-5660-4bcd-890c-7b3db2ce4b08`
  remains the expected RED because the CLI still requires `--options`.
- Independent implementation and repaired specification/source reviews approve
  unconditionally; the final standards review approves after this evidence and
  roadmap refresh.
