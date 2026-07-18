# Task 22H: Post-Closing Largest-Contour Selection

## Status and objective

This specification is a draft. Production or test implementation may begin
only after the exact specification and implementation-plan bytes receive
independent fixed-source/spec, independent Ares/plan, and direct default-model
approval.

Task 22H is the next bounded source-rewrite package in the persistent
`ksr_fdmtest_v4` project-to-G-code parity program. Released Ares commit
`b53a0a7432b5c71d4a1f3b15139fbb873674f09e` produces the exact ordered Task
22G post-closing `ExPolygon` stream. Task 22H ports Orca's
`keep_largest_contour_only` operation and wires it after closing and before
Task 22I simplification.

The committed KSR project has `spiral_mode=0`, so its 460 layers retain
`Regular` mode and this stage is a geometry no-op. That baseline is necessary
for complete regression and platform conformance but cannot prove that the
selector runs. A second full-path test mutates only three Options inside the
3MF (`spiral_mode`, `bottom_shell_layers`, and
`bottom_shell_thickness`) and freezes the resulting non-vacuous selection in
both native Rust and a real WASM browser. A second, independently frozen
threshold-21 3MF mutation proves the bottom-shell threshold is not fixed to the
first mutation's layer boundary.

Task 22H introduces no new external Option and no fixture-specific branch. It
consumes only the retained per-layer `SlicingMode` already derived by Task 22E
from resolved 3MF Options. It does not read a filename, fixture digest,
reference G-code, metadata, or a global default.

Task 22H stops immediately after largest-contour selection. It does not
simplify, combine volumes, generate regions, surfaces, perimeters, infill,
supports, extrusion paths, or G-code. The public project API traverses the
owned intermediate and continues to return
`SliceError::ProjectSlicingIncomplete`.

## Fixed Ares and upstream identity

The fixed Ares baseline is commit
`b53a0a7432b5c71d4a1f3b15139fbb873674f09e`, tree
`5931e386545fe919fb420323017a6a3a497acf45`. Exact-SHA Tier-1 run
`29653761751`, attempt 4, passed format, Ubuntu/Linux, macOS, Windows, and WASM
including the real-project browser checkpoint.

All upstream citations refer to OrcaSlicer commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549`. The ignored Orca checkout has a
different HEAD; source evidence is read from the fixed Git objects.

Primary fixed blobs are:

- `src/libslic3r/TriangleMeshSlicer.cpp`,
  `2c1c0da23fe569c93b5d243a14494792956533d0`;
- `src/libslic3r/ExPolygon.cpp`,
  `185e92508449a425064b26690e3d74d06a16fda8`;
- `src/libslic3r/ExPolygon.hpp`,
  `ce7ebe892f64b3d4e2e9fb0c85bd77b99e889d54`;
- `src/libslic3r/Polygon.cpp`,
  `32b4d062f1b8f898866a0e0e55672dcd5f54ac89`;
- `src/libslic3r/Polygon.hpp`,
  `7d996055e5d9403f871071ef82baa140c03492b5`;
- `src/libslic3r/TriangleMeshSlicer.hpp`,
  `1f7bba9d273f930785279ef82ef3258f191acd3e`;
- `src/libslic3r/PrintObjectSlice.cpp`,
  `07eb885eda83a495001467c22c0452dfc36e55c2`;
- `src/libslic3r/Point.hpp`,
  `039f361eaa18db9c6e7d2c35d1c61af78bcad51b`;
- `src/libslic3r/libslic3r.h`,
  `f4291d36df8175c700fa9374c5b5c07e6880e706`.

Normalized fixed excerpts were independently frozen as:

- `TriangleMeshSlicer.cpp:2025-2037`:
  `10ff1a7b5f501f05ce6e985b9dd9f23ebe3b22b0aed17ee25d4deb62771c4190`;
- `ExPolygon.cpp:532-549`:
  `0cda9f17bf0c5b50de4af0be8b51b7eb3d10ac2552b650f238eac890d9fd83b8`;
- `ExPolygon.hpp:493-497`:
  `ca5d1a6fab7fd78b4b5c079036b25dd73a541aee5e14b3d04e4f8067758b2c8b`;
- `Polygon.cpp:52-69`:
  `c335dfeaae3193db923aef2d4629d1b10d3c61c88766d56469c83ca2ef585667`.

No new production third-party implementation or license is introduced. The
browser test package adds exact `fflate=0.8.3` only as a Node-side dev dependency to
rebuild a mutated 3MF ZIP before passing those complete bytes into WASM. It is
not linked into `ares-core`, `ares-wasm`, or shipped browser bindings.

## Exact upstream rewrite boundary

The direct Task 22H boundary is:

- `TriangleMeshSlicer.cpp:2025-2037` for per-layer mode recovery, the
  post-`make_expolygons` call site, and selection before simplification;
- `ExPolygon.cpp:532-549` for `keep_largest_contour_only`;
- `ExPolygon.hpp:35-36,493-497` for contour/hole ownership and the helper
  declaration;
- `Polygon.cpp:52-69` and `Polygon.hpp:56-57` for signed `double` polygon area.

`TriangleMeshSlicer.hpp:11-47` and `PrintObjectSlice.cpp:166-209` are consumer
context already implemented by Task 22E: `PositiveLargestContour` is retained
only for spiral ModelPart layers at or above the bottom-shell threshold. They
do not authorize new Task 22H Option parsing.

Task 22I begins at `TriangleMeshSlicer.cpp:2038`. Resolution mapping,
Douglas-Peucker simplification, `ExPolygon::simplify`, Boolean repair, and
Clipper `StrictlySimple` are explicitly outside this boundary.

## Normative selection semantics

Selection is independent for each post-closing object, volume, and layer.
Only a layer whose retained mode is `PositiveLargestContour` is changed.
`Regular`, `EvenOdd`, and `Positive` layers are exact identities.

For a selected layer:

1. Zero or one `ExPolygon` is returned byte-for-byte unchanged. A single CW or
   degenerate contour is not reoriented, rejected, or normalized.
2. For more than one `ExPolygon`, initialize the maximum area to `0.0` and no
   selection.
3. Visit candidates in input order and compute only
   `candidate.contour.area()`.
4. Replace the selection only when `candidate_area > maximum_area`.
5. Move the entire selected `ExPolygon`, clear the layer vector, and insert
   that value as its sole element.

Consequences are observable:

- area is signed, not absolute;
- holes never participate in ranking;
- equal positive areas keep the first candidate because the comparison is
  strict;
- the chosen contour's start point, point order, and orientation are unchanged;
- every chosen hole, including hole order and point order, is retained;
- all unchosen sibling ExPolygons and their holes are discarded;
- object, volume, layer, mode, source index, ordinal, and empty-slot order stay
  unchanged.

For a multiple-candidate layer with no positive contour, fixed debug source
asserts because its selected pointer remains null. This is a trusted internal
orientation invariant, not an external validation point. Safe Rust must use a
private assertion or `expect`; it must not invent an absolute-area fallback,
choose the first item, return empty output, or add a new `SliceError`.

## Numeric contract

Coordinates are signed 64-bit integers. `Polygon::area` performs the fixed
serial `double` shoelace calculation:

1. fewer than three points returns `0.0`;
2. cast the final point's X and Y separately to `f64`;
3. visit every point in order, cast both coordinates to `f64`, add
   `previous_x * current_y - previous_y * current_x`, then advance previous;
4. return `0.5 * accumulated`.

The operation order is part of the contract. A widened integer shoelace,
Clipper's algebraically transformed area formula, reordered reduction,
parallel sum, `total_cmp`, tolerance, or net ExPolygon area is not a valid
substitute.

## Ares destination boundary

Task 22H remains private and platform-neutral:

- `geometry/polygon.rs` owns exact signed area;
- `geometry/expolygon.rs` owns the pure in-place largest-contour helper;
- `project_slice/largest_contours.rs` traverses mutable Task 22G owned records
  and gates the helper by retained mode;
- `project_slice.rs` adds a post-largest preparation seam used by the public
  incomplete lifecycle and the Task 22H conformance checkpoint;
- Task 22G's checkpoint remains available only in native tests as a released
  pre-stage regression;
- one non-default `task22h-browser-oracle` feature exposes byte-only pre-stage
  `ARES22G` and post-stage `ARES22H` checkpoints through `ares-core` and
  `ares-wasm` for browser conformance.

The project traversal mutates existing `PostClosing*` ownership records in
place. It does not duplicate stage structs or select across layers, volumes,
or objects. Narrow mutable accessors are allowed only for the stage traversal.

The Task 22G encoder may expose one internal marker-parameterized helper so
Task 22H reuses the same complete ownership format. The Task 22H output wrapper
changes only the eight-byte magic to `ARES22H\0`. Under the H conformance
feature, a separately named `task22h_browser_input_oracle` exposes the exact
post-closing `ARES22G` input so Playwright can prove that WASM executes the
selector rather than merely compiling it. The old non-default Task 22G browser
feature and `task22gBrowserOracle` WASM export are removed, not retained as
compatibility aliases; the native Task 22G test checkpoint remains under
`cfg(test)`.

## 3MF Option ownership

No Task 22H code parses or synthesizes an Option. The retained mode comes from
the already released Task 22E chain:

- resolved `slicing_mode` supplies the base mode;
- resolved process `spiral_mode` enables the special mode only for ModelPart;
- the matching resolved region's `bottom_shell_layers` and
  `bottom_shell_thickness` determine the strict layer-index threshold;
- NegativeVolume and ParameterModifier retain their base mode.

The committed KSR values are:

- `slicing_mode=regular`;
- `spiral_mode=0`;
- `bottom_shell_layers=3`;
- `bottom_shell_thickness=0`;
- `slice_closing_radius=0.049` from Task 22G;
- `resolution=0.012`, deferred to Task 22I.

The non-vacuous oracle changes only:

- `spiral_mode: 0 -> 1`;
- `bottom_shell_layers: 3 -> 0`;
- `bottom_shell_thickness: 0 -> 0.5001`.

No test changes a production default or provides an out-of-band Option.

## Fixed-source oracle protocol

The ignored Task 22H C++20 probe consumes the complete released `ARES22G`
ownership stream, validates exact EOF, applies only the source-fixed selector
to mode code 3, and emits the same ownership stream with `ARES22H\0` magic.
It is 334 LOC with SHA-256
`8eaee1cc464bf9ee7fe729c9c8b6d61716158c598b484ddea1c9e614533b265c`.
MSVC 19.44 compiled it with `/std:c++20 /EHsc /O2 /fp:precise /W4 /WX`;
the executable SHA-256 is
`425c510f3be5162ae8bedbd0e163306cf656323619a84fc2c67b8b809c2c767c`.

Its built-in source vectors distinguish signed from absolute area, distinct
equal-area first-tie selection, selected two-hole ownership, and single-CW
identity. Five exact runs are required for the committed project and for each
of the two 3MF Option mutations. The probe, generated archives, and outputs are
ignored evidence only and are never build, test, or runtime dependencies.

Two independent fixed-source reviewers approved the committed-project probe,
protocol, five-run output, EOF, counts, and representatives. Both Option
mutations use the same approved executable and require two independent
read-only approvals of their provenance and results before their constants may
enter RED tests.

Tracked tests encode and parse Ares output independently. They never invoke the
probe, inspect Orca source, open ignored evidence, or read the reference G-code.

## KSR acceptance at this boundary

### Committed project

The exact `ARES22H` stream is 1,644,681 bytes with SHA-256
`e15967c36c0aa47a9a1a3fc31053587777359bedef796053022eaeb36ad49163`.
It contains 1 object, 1 model-part volume, 460 Regular layers, 2,890 contours,
395 holes, and 99,212 points. Compared with released Task 22G, only byte offset
6 changes from ASCII `G` to `H`.

Representative records remain:

- layer 0: 14,913 bytes, SHA-256
  `28fbbcc66d73c037a5dbb3c60363d83bfaeaaf1d9d8a49451594f227ea0d4fcf`;
- maximum-loop layer 46: 46,233 bytes, SHA-256
  `8dba7c5e51c74e803903b513c5165dffb9d1c55be108e39fbccca4309a603e69`;
- layer 459: 737 bytes, SHA-256
  `c8822b67958531cb4b043d338b53f7329e0b00cb4f08108306763e763cd52f80`.

### 3MF Option mutation

Two independently packaged archives containing the same three Option changes
produce the same released Task 22G checkpoint: 907,601 bytes, SHA-256
`0ca404fa4a5a6fb0a97899fe6ff8fd45815a9439378708bbe594614587e38034`.
It contains 1 object, 1 volume, 460 layers, mode histogram
Regular/EvenOdd/Positive/PositiveLargestContour = `2/0/0/458`, 2,622 contours,
14 holes, and 53,603 points. Exactly 337 PLC layers contain multiple
ExPolygons and therefore exercise selection.

The fixed probe produces 427,465 bytes, SHA-256
`a0df3397e498306bfcade84b03721fe345d2f4b501e578a5b54df39faff44353`,
with the same object, volume, layer, and mode counts, 470 contours, 13 holes,
and 25,747 points. It removes 2,152 sibling contours. Five runs are
byte-identical and end at exact EOF.

The ordered ASCII comma-list of the 337 input layer slots requiring selection
has SHA-256
`24dad9513353d3cf165101199c4514830b5cbcbfe08ce2a100c469bc0eade813`;
the first is slot 20 and the last is slot 459. Native and browser tests derive
that list from the complete G stream, check this fixed digest, and then prove
the corresponding H records contain exactly one retained ExPolygon.

### Independent threshold mutation

The second mutation changes `spiral_mode: 0 -> 1` and
`bottom_shell_layers: 3 -> 21`; it keeps committed
`bottom_shell_thickness=0`. Therefore slots 0-20 are Regular and slots 21-459
are PLC. This boundary crosses slot 20, the first multi-ExPolygon layer in the
three-Option oracle, so an implementation hardcoded to start selecting at slot
2 or 3 cannot satisfy both streams.

Its exact Task 22G checkpoint is 1,154,017 bytes with SHA-256
`f19e168ee3ad5d6a6c882f20bda26d8f0aedeca793fe38be7258b19abd7f4f8c`.
It contains 1 object, 1 volume, 460 layers, mode histogram `21/0/0/439`, 2,717
contours, 128 holes, and 68,852 points. Exactly 336 PLC layers require
selection; their ordered ASCII comma-list begins at slot 21, ends at slot 459,
and has SHA-256
`39a5798f846adf8d41e76c8d6888c6afa6fc9f0d81e3b463989ecc2bb2cd5bc3`.

The fixed Task 22H result is 674,201 bytes with SHA-256
`4b64a4e70bfceabf414572f6dbe13903245612908cbaf2d12985b6c1ed440214`,
569 contours, 127 holes, and 41,012 points. Five runs are byte-identical and
reach exact EOF. Regular slot 20 remains an exact 16,689-byte multi-ExPolygon
record with SHA-256
`e408ee218b9fa4a2dd09da1254bc4a6e74c1d5190ca54ba5156558a5f9292730`.
Both this mutation and the three-Option mutation must fail against a
marker-only post-closing pass-through before selection is wired.

The committed project and reference G-code fixture hashes remain respectively
`698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9`
and `10aec9a156849f59929b578429a764a61453996a5834056f600c0adbb5d6a1b3`.
The reference G-code hash is an integrity fact only; Task 22H code and tests do
not open that file. Older unrelated CLI and config-export tests may continue to
consume it; the Task 22H prohibition is audited over this slice's changed
manifest and diff.

## Planned test inventory

### Geometry

- fewer-than-three, CCW, CW, and large-coordinate serial-f64 area vectors;
- empty and single-CW/degenerate identity;
- strict greatest positive signed contour area;
- a larger-absolute CW decoy that must lose to a small positive contour;
- distinct equal-area candidates proving the first candidate wins;
- a larger contour with a large hole beating a smaller no-hole contour,
  proving contour-only rather than net area;
- exact selected contour start/order/orientation and two-hole ownership;
- multiple all-nonpositive candidates triggering the internal invariant.

### Project stage and real 3MF

- mixed Regular, EvenOdd, Positive, and PLC layers proving mode-only gating;
- per-layer independence across multiple objects and volumes;
- source object index, transform index, planned layers, source volume index,
  ordinal, volume type, empty slots, mode, contour, and hole retention;
- post-closing stage order using Task 22G-owned ExPolygons;
- committed KSR complete H oracle, Task 22G body identity, EOF, counts,
  representatives, repeatability, and unchanged fixture hashes;
- full 3MF Option mutation exact H oracle, `2/0/0/458` modes, 337 selected
  layers, counts, EOF, and repeatability;
- the independently frozen threshold-21 mutation with exact Task 22G/H hashes,
  `21/0/0/439` modes, 336-slot set digest, unchanged multi-ExPolygon slot 20,
  counts, EOF, and repeatability, proving the threshold is read from 3MF
  bottom-shell Options rather than fixed to layer 2 or 3;
- public `slice_project` remains `ProjectSlicingIncomplete`.

### WASM browser

The browser first runs the real committed 3MF checkpoint. It then reads that
same committed archive in the Playwright host, uses test-only `fflate=0.8.3` to make
exactly the three approved Option replacements inside
`Metadata/project_settings.config`, and passes the complete mutated 3MF bytes
to WASM. There is no out-of-band Option or mutation API.

For the mutation, Playwright calls the gated H input/output hooks twice, parses
both complete streams, and verifies the Task 22G input digest, `2/0/0/458`
modes, fixed 337-slot digest, Task 22H output digest/counts, exact EOF, and byte
repeatability. It also checks the committed no-op checkpoint and a Web Crypto
SHA-256 known-answer vector. Thus WASM executes `Polygon::area` and the PLC
selector branch rather than merely compiling them.

## Included behavior

- Exact serial signed polygon area required by the selector.
- Exact post-closing `keep_largest_contour_only` semantics.
- Per-layer retained-mode gating over Task 22G ownership records.
- Complete committed and both mutated 3MF checkpoints.
- Native, WASM, Windows, macOS, and Linux deterministic behavior.

## Explicitly deferred behavior

- Resolution mapping and the `resolution > 0.001 -> 0.0025 mm` tolerance.
- Douglas-Peucker, `ExPolygon::simplify`, `simplify_polygons`, Boolean repair,
  and Clipper `StrictlySimple` from Task 22I.
- Cross-volume negative/modifier combination, regions, and surfaces.
- Perimeters, fill, supports, extrusion paths, G-code assembly, metadata,
  post-processing, and complete normalized reference-G-code equality.

## Structural, hardcoding, and platform constraints

- Every Rust production and test file remains below 400 physical LOC; split
  before reaching the limit.
- Tests live in separate real `mod` files. `include!` and `include_bytes!` may
  not split Rust source or test modules. Existing fixture byte embedding is not
  a source split and remains test-only.
- No unsafe, FFI, filesystem, process, thread, UI, terminal, OpenGL, native
  dependency, platform branch, or second geometry engine enters `ares-core`.
- No production fixture name/hash, reference-G-code read, expected count or
  coordinate table, KSR-specific branch, literal spiral threshold, or mode
  override is allowed.
- Existing obsolete executable Orca source-pinning tests remain deleted; no
  source-path/line/hash test is added.
- The browser feature changes only checkpoint visibility. Node-side test
  preparation changes Options only inside a complete 3MF archive. Neither can
  select an algorithm, expected output, out-of-band Option, or fallback.
- Tier-1 remains WASM browser, Windows, macOS, and Linux.

## Verification and review exit criteria

Implementation follows strict RED-GREEN-REFACTOR packages. The complete
committed and both mutated KSR assertions are registered before selector
behavior. The baseline may become green after checkpoint plumbing because it
is a source-proven no-op; both mutated oracles must remain real behavior REDs
until the post-closing selector is wired. Expected constants do not change
unless an independent fixed-source review proves the oracle wrong.

After implementation, one independent read-only reviewer must assess the same
candidate across requirement completeness, logical correctness, edge cases,
code quality, test coverage, and actual execution results. It returns a
prioritized P0-P3 fix list. Only the main thread changes code; the same reviewer
rechecks after each repair until all six axes pass or a concrete external
blocker is reproduced.

Then fresh whole-candidate specification, quality, and direct default-model
reviews must approve unchanged bytes. Documentation review, the complete
native/WASM/browser matrix, exact manifest audit, Conventional Commit, normal
push, remote identity, and exact-SHA Tier-1 success are required before Task
22H is released and Task 22I begins.

Task 22H release does not complete the persistent user goal. Work continues
through later source-cited slices until normalized KSR G-code parity and a
final six-dimensional result review are present.

**Status: DRAFT — implementation is forbidden until the exact specification
and plan receive all pre-implementation approvals.**
