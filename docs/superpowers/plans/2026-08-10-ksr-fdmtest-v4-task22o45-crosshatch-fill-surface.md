# Task 22O.45 — Implementation plan

## Outcome

Implement the exact crate-private public CrossHatch fill transaction described
by the matching spec. O45 consumes O44 but does not advance the prepared-project
lifecycle; public slicing remains terminal after O43 until complete
`group_fills`, parameter projection, and lower-layer anchor-map integration are
ported.

## Source boundary

This plan implements pinned OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`FillBase.cpp:105-119`, `FillCrossHatch.hpp:12-25`, complete
`FillCrossHatch.cpp:28-232`, the active connector dispatch at
`FillBase.cpp:1820-1823,1827-1829`, and the multiline-one return at
`FillBase.cpp:2712-2715`. The matching spec's exact dependency citations and
deferred ranges govern implementation. The Rust destination is crate-private
`fill::cross_hatch`; legacy `infills.rs` remains an uncalled compatibility
scaffold.

## Red-green sequence

1. Finish and independently review the disposable pinned-Orca public
   `FillCrossHatch::fill_surface` harness. Freeze exact comparator-distinct
   Normal/LargeBed ordered vectors and restore the Orca tree.
2. Add a compiling narrow open-polyline Intersection stub over the existing
   checked Clipper open-path worker. Use one integrated literal
   open-subject/closed-clip order discriminator for RED, then make the minimal
   source-shaped one-line delegation GREEN. Add hole, no-recombination, and
   range-error cases afterward as characterization/regression coverage; do not
   contrive partial production behavior merely to manufacture more REDs.
3. Add `fill::cross_hatch`, its source-shaped parameter record, and a compiling
   empty-output stub. Add the first exact public Orca vector and retain the
   behavioral RED with focused Nextest, then implement the minimum complete
   end-to-end repeat-path behavior needed to make that vector GREEN before
   adding another CrossHatch test.
4. Through only the crate-private `fill_surface` seam, add the remaining
   pattern branch, threshold, cast, and scale variants not exercised by the
   first repeat vector. For each missing variant, add one literal test, retain
   its RED against the current implementation, port only the missing source
   branch, and return the complete focused suite to GREEN before continuing.
   Do not claim or rewrite base behavior already driven by step 3.
5. Still through only `fill_surface`, apply the same vertical sequence to the
   remaining public-wrapper and geometry edge variants: offset cast order,
   empty inset/filter results, strict remnant equality, any component-order
   case not already covered, dual scale, and reachable early/later errors with
   atomic ownership. Existing happy-path offset, clipping, O44 connection, and
   rotate-back from step 3 are characterization targets, not pretexts for
   contrived new implementations.
6. Once all specified behavior is GREEN, run reversible mutations over every
   acceptance-critical arithmetic/order/geometry seam. If a mutant survives,
   first add a literal test that is RED under the mutant, restore production,
   and confirm GREEN. Restore and record exact production hashes after every
   mutation window.
7. Run focused dependency/predecessor bands, full workspace and Tier-1 gates,
   static audits, the normalized golden progress probe, and independent
   source/spec and standards reviews. Repair and re-review until both approve
   unconditionally or identify a concrete blocker.

## Oracle record

The public harness is `/tmp/task22o45-crosshatch-oracle.cpp`, SHA-256
`dc41ed54fba644b589c41d4208847347b2c5e7626367660b1fd547d843ce542f`.
It directly calls inherited `FillCrossHatch::fill_surface` on a raw
`stInternal` Surface and therefore covers the public half-spacing offset,
ordered two-component inset, one retained hole, complete CrossHatch generation,
open clipping, O44 connection, and rotate-back. It uses a negative-coordinate
asymmetric raw ExPolygon with a sub-full-spacing neck narrower than twice the
inset, so the public offset splits it into exactly two components.

Reproduce its existing Debug and Release build commands, execute both scales,
normalize, compare, and hash with:

```bash
nix-shell -p clang lld ninja --run /tmp/task22o45-build-run.sh
```

The script SHA-256 is
`dc3633fea2c2485c451308baa5cc17bfe0ae1e688244d90d1c8370e7fa8b8560`.
It filters only `scale`, `tuple`, `case`, and `polyline` records. Debug and
Release are byte-identical with normalized SHA-256
`17b755322c8d1e586e29145836f04ea728f4fdd846cce965430f8af1fea8691f`.
Five additional runs of each configuration produce the same digest.

The four cases use actual KSR lower layers and the exact KSR tuple:

- layer 44, z bits `0x4022000000000000`: negative-direction repeat,
  3 paths on each scale;
- layer 31, z bits `0x401999999999999d`: positive-direction repeat,
  2 paths on each scale;
- layer 40, z bits `0x4020666666666668`: forward/first-half transform,
  4 paths on each scale; and
- layer 14, z bits `0x4008000000000001`: backward/second-half transform,
  2 paths on each scale.

Normal produces 11 paths / 172 points and LargeBed 11 paths / 173 points; the
one-point difference is source scale rounding and is identical in Debug and
Release. Full literal coordinates will be copied from
`/tmp/task22o45-debug-normalized.txt` into Rust tests during the first RED;
tests never read that file. The first Normal repeat path is
`[(8834954,5166463),(8895784,3539281),(6563806,5871259),
(2620980,5669063),(9056756,-766712),(8995926,860470)]`.

Comparator equivalence is independently checked with:

```bash
/tmp/task22o45-comparator-run.sh
```

Endpoint counts for the eight Normal connector calls are
`[10,8,10,8,12,6,10,8]`; arc counts are `[10,8,9,8,11,4,9,7]`.
LargeBed repeats those counts. Every endpoint and arc equivalent-key-pair count
is zero in Debug and Release, so these vectors do not depend on the host C++
sort's order for equivalent keys. Comparator patch/runner/check hashes are
`a0ab4b909818f2d1a477c8bfb60d17b0c9876f45cb91bb0c54698b6eb2326359`,
`a385bb9f42aacef43172b81ad4645b487d51908ef0bb171c3ef09cb899e46863`,
and `2d2c4676c7824a593f72c02749c6bb224ab085b32791aea0a228da3367ba44bd`.

The supplemental public f32-repeat-ratio harness is
`/tmp/o45-exp-public-oracle.cpp`, SHA-256
`5cf7c7847b079ff8d71b9240856ffd21f6ce3d1701ad5f1c12d8566a71ba7d84`.
It directly calls inherited `FillCrossHatch::fill_surface` with the z=41 mm
Normal-scale literal and is reproduced with:

```bash
nix-shell -p clang lld ninja --run /tmp/o45-exp-public-build-run.sh
```

The build/run script SHA-256 is
`14f91c8d39768df80559e86c1554c41f426a49d616c2a00f4f2a774d60a5dab9`.
It compiles Debug and Release, normalizes only `count` and `polyline` records,
and runs each configuration at least three times plus five additional times.
Every run has normalized SHA-256
`e9b62afdc6fe0f7b03e4baf86d9c0e13e4692398f5ac89b6d8850bc82bd01aa2`.

The full evidence record is `/tmp/task22o45-oracle-evidence.md`, SHA-256
`831221270b56383b5a5cf1a1d25da94e937be07462a751f1941e60ec193cbe93`.
The pinned checkout is clean at the required commit. Temporary comparator
instrumentation was removed and disposable `FillBase.cpp` restored to
`30c898b2a0f9ae99a000d80f8c0e3f67b721d36f8017f360fcb58c0bda58ca38`.
A protected `_fill_surface_single` harness remains subordinate evidence only.

## Implementation shape

The crate-private seam is:

```rust
pub(crate) fn fill_surface(
    surface: &ExPolygon,
    params: CrossHatchFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError>;
```

- `crates/ares-core/src/fill.rs`: declare the crate-private CrossHatch module;
- `fill/cross_hatch.rs`: source-shaped parameters, public-wrapper offset,
  ordered component orchestration, clipping/filtering/O44 composition, and
  atomic owned result;
- `fill/cross_hatch/pattern.rs`: one-cycle, repeat, transform, and Z-phase
  generation with exact source arithmetic and order;
- `fill/cross_hatch/transform.rs`: checked halves-away conversion, fixed-point
  rotation/translation, contour bbox/grid alignment, and polyline length;
- `fill/cross_hatch/tests.rs` plus focused shards: literal unit/oracle/error
  vectors kept below 400 LOC each;
- `geometry/clipper/polyline.rs`, `geometry/clipper.rs`, and `geometry.rs` plus
  their existing tests: add and reexport only open-polyline Intersection beside
  the existing open Difference wrapper; and
- `fill/connect.rs`: remove only the now-fulfilled temporary dead-code
  expectation; do not change O44 behavior.

Keep pattern and transform helpers private and do not test them as independent
behavioral seams. All CrossHatch behavior assertions enter through
crate-private `fill_surface`; only the separately reusable open-intersection
geometry seam has direct geometry tests. Do not add generic geometry
transforms, a public API, a Rust Fill hierarchy, a prepared lifecycle wrapper,
or a caller in the old infill pipeline. Consume owned offset components and
working paths; borrow the source surface without a defensive clone.

## Verification

```bash
cargo nextest run -p ares-core task22o45 --no-fail-fast
cargo nextest run -p ares-core \
  -E 'test(/(task22o45|task22o44|open_polyline|clipper)/)' --no-fail-fast
cargo nextest run -p ares-core \
  -E 'test(/task22o(2[4-6]|4[0-5])/)' --no-fail-fast
cargo nextest run --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown
cargo nextest run -p ares-cli --test ksr_fdmtest_v4 \
  -E 'test(project_matches_orca_242_except_generator_line)' \
  --run-ignored ignored-only --no-fail-fast
```

Also require `git diff --check`, every Rust source file below 400 physical
lines, no production `include!`/`include_bytes!` source splitting, no
production fixture/hash/reference-G-code reads, exact pinned-Orca restoration,
and fresh independent review evidence.

## Completion record

- The compiling empty-output and transform stubs produced genuine exact-vector
  REDs. Final focused O45 Nextest `083dc9db-5ad2-48a2-9612-ed1b2e39af68`
  passes 34/34.
- The Clipper/open-intersection/O44 dependency band
  `8cfd30f0-bd9f-4402-b2ec-e1fa6339ab57` passes 305/305; the O24-O26/O40-O45
  band `daf0e79f-dc26-4943-a7d2-a2e80b4691e8` passes 228/228; and workspace
  run `818f7790-9db4-41d2-9206-ebb4f969f8a4` passes 6,235/6,235 with 30 slow
  and two skipped.
- The public Orca harness/output/evidence hashes are respectively
  `dc41ed54fba644b589c41d4208847347b2c5e7626367660b1fd547d843ce542f`,
  `17b755322c8d1e586e29145836f04ea728f4fdd846cce965430f8af1fea8691f`,
  and `831221270b56383b5a5cf1a1d25da94e937be07462a751f1941e60ec193cbe93`.
  The supplemental pattern-order harness/output hashes are
  `e07a31cedb92637b35750e6ac2b287a5dbddc644b6924ea14085502a5e92411e`
  and `bda674683e3990477401aeba3dcb3deec1a817f98d1fae049bc9b73744071f84`.
  The current arithmetic harness/stdout hashes are
  `24040248e57f2dadb2aae060e1c32ecd357ffe57f57127453ea28d5ab4362200`
  and `42434f1fad069e70c09e5538da1e173e2ce8919fe225c4b5fff8897608b10ea7`.
  The supplemental public f32-repeat-ratio harness/output hashes are
  `5cf7c7847b079ff8d71b9240856ffd21f6ce3d1701ad5f1c12d8566a71ba7d84`
  and `e9b62afdc6fe0f7b03e4baf86d9c0e13e4692398f5ac89b6d8850bc82bd01aa2`.
  Correcting LargeBed inputs to source `scaled<coord_t>` truncation toward zero
  leaves all four LargeBed cases GREEN without changing production.
- Reversible arithmetic mutants were RED at `450edfd2`, `ad7d22e6`,
  `5e5c4daf`, `ca0f7ff5`, `5f8ebdca`, `24e1fe07`, `a9a5e4f7`, and
  `fdbc73aa`. Composition mutants were RED at
  `392a2e7c-bfbc-4486-aca0-d58333d30749`,
  `11372d71-86bd-488e-bf86-e7c8906b3418`,
  `039b09c9-0bb3-4f31-9916-0de7d1604a0e`,
  `07f8a4bd-4687-4e36-a41b-8d109da26268`,
  `bc2558dd-0384-431f-b64f-2fb62bf9e532`, and
  `ab792c8b-97ed-4a67-9317-d200e97e11b8`.
  Public f32-repeat-ratio mutants were RED at
  `f2038fcc-86eb-4e2d-9988-b9bd477c0186` and
  `c3ecc38a-67c9-42de-9f43-ab6d35d28385`.
- Production restoration hashes are
  `369d5c44a09822b05c6ef16770bc1431c61d2160a4cf28166bd58a1d5e7f46c4`
  (`cross_hatch.rs`),
  `e1cd61932b98e248152c75e862f736b5b0b32c755ed3e08363059856c426cb3a`
  (`pattern.rs`),
  `7a26f837fb94aed660e92354aa9338c3fb686d2d9fcaed56f64d3f82bff9b54a`
  (`transform.rs`), and
  `b8b385224223a2702b63e40732012e2d2f74abfb584bbd87419cdb3a1c816201`
  (`geometry/clipper/polyline.rs`).
- Rustfmt, workspace all-target/all-feature warning-denying Clippy, wasm32 core
  and adapter checks, and static audits pass. Ignored golden run `9f4804f9`
  remains the expected RED at the unchanged missing `--options` contract.
- O45 is an implemented, verified dependency-first crate-private CrossHatch
  fill transaction only. Public prepared slicing still consumes and disposes
  O43, returns `ProjectSlicingIncomplete`, and has no O45-driven G-code or
  public activation. Final independent source/specification and standards
  reviews unconditionally approve this completed state.
