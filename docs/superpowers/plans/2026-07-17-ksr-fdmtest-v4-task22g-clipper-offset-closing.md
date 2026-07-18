# Task 22G Implementation Plan: Clipper 6 Closed Offset and Project Slice Closing

## Status, fixed points, and success condition

This plan is a draft. No production or test implementation is authorized until
the exact specification and plan bytes receive all pre-implementation review
approvals.

The fixed Ares baseline is commit
`ca667a8a3b595cfd2bdde5ced357010830051360`, tree
`56343294a9195f53f63c6d3295272186c7ca64cd`; exact-SHA Tier-1 run
`29642639170` is green on all five jobs. The fixed OrcaSlicer source is commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`, tree
`b62d6017ba1ac7cb986f70fd6844353c7a776549` and the blobs listed in the Task
22G specification.

Success means:

- a source-cited pure-Rust ClosedPolygon ClipperOffset implements all three
  closed join branches and exact positive/zero/negative cleanup;
- the directly used ExPolygon `offset_ex` and `offset2_ex` wrappers preserve
  fixed single-pass union and ownership semantics;
- project slicing consumes each resolved object's 3MF-derived
  `slice_closing_radius` through the exact f32 narrowing and coordinate-scale
  chain, with source-fixed `extra_offset=0`;
- Ares matches the independently approved complete `ARES22G` KSR oracle;
- every required native, WASM, browser, structural, provenance, and review
  gate passes while the public API still reports `ProjectSlicingIncomplete`;
- the exact reviewed bytes are committed, pushed normally, and green in
  exact-SHA Tier-1 before Task 22H begins.

Task 22G does not claim complete normalized G-code parity.

## Immutable behavior ledger

The implementation must preserve these non-substitutable facts:

1. The KSR consumer is
   `offset2_ex(union_ex(...), offset_out, offset_in)`, not generic
   `closing_ex`.
2. Task 22F already owns `union_ex`; Task 22G consumes its ordered ExPolygons
   and does not repeat fill-rule selection.
3. Offset wrapper cleanup uses one Clipper Paths or PolyTree execution. It must
   not call Task 22F's two-pass Paths-then-fresh-PolyTree `union_ex`.
4. ClipperOffset-internal positive cleanup uses Positive fill, but the outer
   positive wrapper union uses NonZero. They are separate executions.
5. Single-ExPolygon Paths ownership recovery uses EvenOdd, multi-ExPolygon
   PolyTree ownership recovery uses NonZero, and negative contour/hole
   subtraction uses one NonZero Difference.
6. Raw wrapper offset processes each path separately and corrects original CW
   orientation with delta sign and output reversal.
7. First-stage ExPolygon contour/hole offset does not call that raw wrapper:
   it directly executes contour with delta and each hole with `-delta`, then
   follows the source positive/negative ownership branch.
8. Shortest-edge filtering is strict `<`, with factor 0.005.
9. Generated coordinates use Clipper `fixed_round`, not Rust `round()`.
10. Near-zero generation copies accepted paths, then cleanup still uses exact
    `delta > 0`: `+0.0`, `-0.0`, and negative sub-tolerance values take the
    Negative branch; a positive sub-tolerance value takes Positive cleanup.
11. Negative cleanup uses normalized bounds, the exact 10-unit outer rectangle,
   reverse solution, Negative fill, and outer-node removal.
12. Option scaling preserves `f64 -> f32 -> scaled f64 -> f32 -> f64`; it is
   not integer scaling.
13. `extra_offset=0.0f32` is fixed upstream consumer behavior, not an Option.
14. Object, volume, layer, mode, empty slot, and point order are observable.
15. Largest-contour selection and simplification remain Task 22H/I.

The invalid initial oracle SHA
`13a017cef00ac91c07a5e62556ed4be30b901ad2260f66b7659c5af06279aed6`
is prohibited. It came from transposed input coordinates and an incorrect
hole-offset sign. Only a corrected oracle approved by two independent
read-only source reviewers may enter tests.

## Working protocol

Work proceeds in serial TDD packages. For every package:

1. freeze the exact allowed paths and source boundary;
2. add only package-owned tests in separate modules;
3. run the focused command and record the real expected RED;
4. implement the smallest source-cited behavior that makes the RED green;
5. run focused regressions, rustfmt, relevant Clippy, and LOC/macro checks;
6. freeze a per-file hash manifest;
7. obtain independent specification and quality approval before the next
   package begins.

The complete native KSR and browser oracles are registered once in Package 0,
before any offset/closing behavior. They remain real expected REDs throughout
Packages A-C. Package D promotes those same unchanged assertions to final GREEN;
it does not register them after production behavior already exists.

The ignored evidence ledger is `.superpowers/sdd/task22g-evidence.md`. It
records commands, exit codes, nextest run IDs, hashes, oracle reviews,
fix/re-review rounds, release identity, and Tier-1. Evidence artifacts are
never build or test dependencies.

Use `apply_patch` for source and document edits. Do not modify committed
fixtures. Do not amend, squash, force-push, or rewrite released Task 22A-F
history.

## Pre-implementation exact-byte gate

Before Package 0:

1. obtain two independent read-only approvals of the corrected oracle probe,
   input, wrapper vector, output hashes, counts, and representative layers;
2. freeze the tracked specification and plan SHA-256 values;
3. dispatch an independent fixed-source/spec reviewer;
4. dispatch an independent current-Ares/plan reviewer;
5. run the direct default-model OpenCode review with task/edit tools denied;
6. require literal approval from all reviewers on the same exact bytes.

Any spec or plan edit invalidates all approvals. Any unresolved P0-P3 finding
blocks implementation.

## Exact planned tracked manifest

No tracked path outside this list may change without a plan amendment and
fresh approvals.

### Specification, architecture, provenance, and roadmap

- `docs/superpowers/specs/2026-07-17-ksr-fdmtest-v4-task22g-clipper-offset-closing.md`
- `docs/superpowers/plans/2026-07-17-ksr-fdmtest-v4-task22g-clipper-offset-closing.md`
- `docs/architecture/option-parity-v4.md`
- `docs/roadmap.md`
- `THIRD_PARTY_NOTICES.md`

### Production geometry

- `crates/ares-core/src/geometry.rs`
- `crates/ares-core/src/geometry/clipper.rs`
- `crates/ares-core/src/geometry/clipper/bounds.rs`
- `crates/ares-core/src/geometry/clipper/polytree.rs`
- `crates/ares-core/src/geometry/clipper/offset.rs`
- `crates/ares-core/src/geometry/clipper/offset/input.rs`
- `crates/ares-core/src/geometry/clipper/offset/generate.rs`
- `crates/ares-core/src/geometry/clipper/offset/execute.rs`
- `crates/ares-core/src/geometry/clipper/offset/expolygon.rs`

### Production project stage

- `crates/ares-core/Cargo.toml`
- `crates/ares-core/src/lib.rs`
- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/closing.rs`
- `crates/ares-core/src/project_slice/task22g_oracle.rs`

### Geometry tests

- `crates/ares-core/src/geometry/tests/clipper.rs`
- `crates/ares-core/src/geometry/tests/clipper/offset.rs`
- `crates/ares-core/src/geometry/tests/clipper/offset/helpers.rs`
- `crates/ares-core/src/geometry/tests/clipper/offset/input.rs`
- `crates/ares-core/src/geometry/tests/clipper/offset/joins.rs`
- `crates/ares-core/src/geometry/tests/clipper/offset/execute.rs`
- `crates/ares-core/src/geometry/tests/clipper/offset/expolygon.rs`

### Project tests

- `crates/ares-core/src/project_slice/tests.rs`
- `crates/ares-core/src/project_slice/tests/closing.rs`
- `crates/ares-core/src/project_slice/tests/closing_fixture.rs`

### WASM browser conformance surface

- `crates/ares-wasm/Cargo.toml`
- `crates/ares-wasm/src/lib.rs`
- `crates/ares-wasm/tests/browser/index.html`
- `crates/ares-wasm/tests/browser/project-slice.spec.mjs`
- `.github/workflows/tier1.yml`

The planned maximum is 34 tracked paths. Deleting a planned unused new path is
allowed before freeze; adding a new path is not.

## Module ownership and line budgets

All Rust files must remain below 400 physical LOC. Start split before the
limit, not after it.

| Module | Ownership | Budget |
| --- | --- | ---: |
| `clipper/bounds.rs` | fixed normalized Clipper bounds only | 100 |
| `clipper/polytree.rs` | existing ownership plus narrow outer promotion | 280 |
| `clipper/offset.rs` | join/options/state vocabulary and re-exports | 160 |
| `offset/input.rs` | AddPath, shortest-edge filter, lowest point, orientation | 200 |
| `offset/generate.rs` | normals, preparation, closed DoOffset loop | 300 |
| `offset/execute.rs` | raw wrapper and positive/negative single-pass cleanup | 240 |
| `offset/expolygon.rs` | contour/hole ownership, offset_ex, offset2_ex | 300 |
| `project_slice/closing.rs` | per-object Option scaling and owned stage | 260 |
| each geometry test file | one behavior family | 320 |
| `project_slice/tests/closing.rs` | synthetic Option/ownership behavior | 320 |
| `closing_fixture.rs` | KSR lifecycle and oracle assertions | 320 |
| `project_slice/task22g_oracle.rs` | gated canonical ARES22G encoder/hook | 240 |
| `ares-wasm/src/lib.rs` | existing adapter plus gated byte-only hook | 180 |
| each browser test source | real fixture, parser, digest, repeatability | 220 |

Do not add code to the existing 399-LOC Clipper intersection or ordering
modules, or to the 384-LOC project test support module. Source splitting uses
real Rust `mod` files only. `include!` and `include_bytes!` are forbidden for
this split.

## Error contract

The closed offset API returns `ClipperError::CoordinateOutOfRange` when
generated output cannot be accepted by the released fixed-range Boolean
kernel. It does not clamp, wrap, panic, or fall back to raw paths.

At the external project boundary:

- nonfinite or negative `slice_closing_radius` returns
  `SliceError::InvalidInput` naming the Option;
- a nonfinite scaled f32 offset, including one produced from a finite external
  radius, returns `InvalidInput` naming the Option;
- Clipper range failure maps once to deterministic project-closing
  `InvalidInput` text;
- resolved-object association and duplicate internal ownership remain
  assertions because callers are trusted internal stages.

No error includes a fixture name, expected digest, reference G-code, Orca
source path, or fallback suggestion.

## Oracle protocol

The corrected fixed-source probe must remain ignored and below 400 LOC. It
must:

1. compile with MSVC `/fp:precise` default behavior against fixed Clipper and
   Int128 blobs;
2. first execute the upstream exact square-with-hole
   `offset2_ex(+5,-2)` coordinate vector;
3. consume corrected Task 22F input 1,645,481 bytes / SHA-256
   `209c6149c93994cc3ae6fa8e2f8f43dc9875b1b07b2320da9e67d8a2c43ab6e2`;
4. apply only `offset2_ex(+49000.f,-49000.f,Miter,3.0)`;
5. preserve the `ARES22G` ownership encoding;
6. run five times with byte-identical output;
7. receive two independent fixed-source approvals.

Two independent read-only fixed-source reviewers approved the corrected probe
with no remaining P0-P3 findings. The normative result is 1,644,681 bytes /
SHA-256
`29ffb501c54190dd4336cc1371fc5e480c5b87ac6a8184366bd072bf5cb90919`,
with 2,890 contours, 395 holes, and 99,212 points. Representative layers are
0 (`28fbbcc6...`), maximum-loop 46 (`8dba7c5e...`), and 459
(`c8822b67...`).

Rust tests encode Ares output independently from the fixed constants. They do
not read ignored oracle files, invoke the probe, inspect Orca source, or read
the reference G-code.

The canonical encoder is compiled only under `cfg(test)` or the single
non-default `task22g-browser-oracle` conformance feature. Native unit tests call
it directly. A feature-gated `ares-core` byte function and one-line
`ares-wasm` adapter call the same private post-closing pipeline and encoder;
they expose no geometry types and are absent from normal builds. This feature
does not select algorithm behavior, expected output, or a fallback.

## Planned test inventory

### Closed offset input and joins

- empty, one-point, two-point, flat, duplicate terminal, consecutive duplicate;
- shortest-edge squared distance 60020 (`244^2 + 22^2`) removed and 60025
  (`245^2`) retained;
- global-lowest orientation repair and original CW/CCW raw wrapper behavior;
- `+0.0`, `-0.0`, positive/negative sub-tolerance cleanup, and default
  miter/arc/shortest values;
- convex miter, sharp miter-to-square fallback, direct square, direct round;
- concave three-point corner and collinear one-point corner;
- complete ordered coordinates and repeatability.

### Cleanup and ExPolygon wrappers

- positive Positive-fill cleanup;
- negative bounds, outer rectangle, reverse solution, Paths/tree outer removal;
- empty shrink and complete erosion;
- fixed upstream square-with-hole `offset_ex(+5)` and
  `offset2_ex(+5,-2)` coordinates;
- positive hole shrink/reversal, negative hole expansion/difference, consumed
  contour;
- single-ExPolygon EvenOdd recovery, multi-ExPolygon NonZero recovery, and
  negative NonZero Difference with distinguishing overlap/winding vectors;
- multi-ExPolygon positive union only when collected count exceeds one;
- negative no-cross-union and single-pass ordering oracle;
- generated range error.

### Project and KSR

- two resolved objects with different effective radii;
- process-base and object override precedence sourced only from 3MF-derived
  typed options;
- zero radius keeps every Task 22F owned record and point while canonical output
  still uses the `ARES22G\0` marker;
- KSR 0.049 normal-scale exact f32 chain to +/-49000;
- large-bed and non-integer scaled mutation vectors;
- negative/nonfinite radius boundary error and a finite-radius vector whose
  scaled f32 delta overflows to nonfinite;
- object/volume/layer/mode/ordinal/empty-slot retention;
- full 1,644,681-byte candidate oracle, counts, representative layers, and
  byte repeatability;
- unchanged project/config/reference fixture hashes;
- public `slice_project` remains `ProjectSlicingIncomplete`.

The browser test fetches the real 3MF through the existing server route, calls
the gated WASM hook twice, parses the complete `ARES22G` framing, and checks
magic, 1 object, 1 volume, 460 layers, 2,890 contours, 395 holes, 99,212 points,
1,644,681 bytes, SHA-256
`29ffb501c54190dd4336cc1371fc5e480c5b87ac6a8184366bd072bf5cb90919`, exact
EOF, and byte repeatability.
It uses Web Crypto for SHA-256 and never fetches or reads the reference G-code.

## TDD package sequence

### Package 0: complete native/browser oracle registration

Allowed paths are the project test roots, gated oracle module, core/WASM
feature declarations and exports, existing browser index/spec, project pipeline
seam, and Tier-1 WASM build command.

1. Add the native complete-oracle and browser assertions against the planned
   but absent gated hook, then record their compile/binding RED before adding
   the seam.
2. Extract one private `prepare_post_closing` pipeline used by both the normal
   incomplete lifecycle and the conformance hook; preserve all existing public
   behavior under the already-green Task 22F regressions.
3. Add the canonical encoder, independently checked minimal encoder/parser
   vectors, Web Crypto known-answer check, and gated byte-only adapter. The
   minimum closing signature needed to compile returns the existing typed
   incomplete condition; it implements no offset or closing behavior.
4. Run native and real Playwright oracles and record both real behavior REDs.
   They must
   fail because Task 22G output is unavailable, not because of encoder/parser,
   fixture routing, binding, or digest tooling drift.
5. Freeze the tests and expected constants. Do not change them during Packages
   A-C unless independent fixed-source review proves the oracle wrong.
6. Run existing Task 22F lifecycle regressions, default no-feature checks, fmt,
   Clippy, LOC, and forbidden-macro checks.
7. Obtain fresh Package 0 specification and quality approval.

### Package A: closed offset domain, input, and joins

Allowed paths are the geometry roots, `clipper/bounds.rs`, `offset.rs`,
`offset/{input,generate}.rs`, and input/join/helper tests.

1. Register the upstream square/miter, orientation, strict-shortest-edge,
   concave, Square, and Round tests.
2. Run the focused nextest filter and record compile/behavior RED.
3. Implement only JoinType, closed AddPath, orientation, numeric preparation,
   normals, and closed corner generation.
4. Keep cleanup behind an unimplemented seam until Package B.
5. Run Package A tests, geometry regressions, fmt, Clippy, LOC, and forbidden
   macro checks.
6. Obtain fresh Package A spec and quality approval.

### Package B: single-pass cleanup and ExPolygon wrappers

Allowed paths add `offset/{execute,expolygon}.rs`, PolyTree visibility/outer
removal, and execute/expolygon tests.

1. Register positive, negative, signed-zero, both sub-tolerance,
   outer-removal, complete-erosion, square-with-hole, fill-matrix,
   multi-ExPolygon, and single-pass-order REDs.
2. Add exact normalized bounds and narrow single-pass Paths/PolyTree helpers.
3. Implement raw per-path wrapper behavior separately from direct first-stage
   contour/hole behavior.
4. Implement `offset_ex` and exact two-stage `offset2_ex`.
5. Run all Task 22F+22G geometry tests, fmt, Clippy, LOC, provenance, and range
   checks.
6. Obtain fresh Package B spec and quality approval.

### Package C: per-object 3MF-derived project closing

Allowed paths add `project_slice/closing.rs`, production wiring, and synthetic
project tests.

1. Register REDs for object association, effective Option precedence, exact
   f32 scale chain, zero, large-bed/non-integer values, finite-radius scaled-f32
   overflow, invalid input, and owned slot retention.
2. Implement the owned post-closing types and per-object stage.
3. Wire it immediately after Task 22F pre-closing output.
4. Traverse the result in the existing incomplete lifecycle; emit no G-code.
5. Run focused project, Task 22F regression, config, fmt, Clippy, and WASM
   checks.
6. Obtain fresh Package C spec and quality approval.

### Package D: complete KSR oracle

The complete fixture, encoder, native oracle, browser parser, and expected
constants remain byte-for-byte those registered in Package 0. Paths outside a
source-proven implementation defect are closed.

1. Run the already-registered full native and browser oracles and preserve the
   Package 0 RED evidence trail.
2. Correct only source-proven implementation defects until both unchanged
   oracles become green.
3. Prove 3MF radius mutation changes output; for zero radius, prove every owned
   record and point equals Task 22F while the canonical marker remains
   `ARES22G\0`.
4. Run Task 22A-G focused regressions and the public incomplete lifecycle.
5. Obtain fresh Package D spec and quality approval.

### Package E: closure, review, docs, and release

1. Freeze the exact implementation manifest and normalized patch digest.
2. Run all structural, hardcoding, provenance, fixture, and platform audits.
3. Run the full local verification matrix.
4. Execute the mandatory same-thread six-dimensional review/fix/re-review
   loop.
5. Obtain fresh whole-spec, whole-quality, and default-model approvals.
6. Update notices, architecture, and roadmap; obtain documentation approval.
7. Rerun the docs-inclusive full matrix and refreeze the final manifest.
8. Stage exactly the approved paths, create a Conventional Commit, push
   normally, verify refs, and monitor exact-SHA Tier-1 through all five jobs.
9. Begin Task 22H immediately; do not mark the persistent goal complete.

## Focused and full verification matrix

Use `cargo nextest run`, never `cargo test`, as the default Rust runner.

Focused filters must cover all `task22g_` tests, then all Task 22F geometry and
project tests. Final verification includes:

```text
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace --all-targets
cargo nextest run -p ares-core
cargo nextest run --workspace
cargo check -p ares-core --target wasm32-unknown-unknown
cargo check -p ares-wasm --target wasm32-unknown-unknown
cargo build -p ares-wasm --target wasm32-unknown-unknown --release --features task22g-browser-oracle
fresh wasm-bindgen 0.2.121 web generation
committed real-3MF Playwright browser exact-oracle test
```

Record nextest run IDs, pass/skip counts, command exit codes, and the exact
tested manifest. Any platform digest disagreement blocks release for numeric
source tracing; platform-specific expected output is forbidden.

The default no-feature checks prove the normal API surface remains unchanged.
The feature build exists only for exact browser conformance and must compute the
same magic, ownership counts, length, digest, EOF, and repeated bytes as native
tests. Tier-1's WASM job must use this exact feature build before Playwright.

## Structural, provenance, and hardcoding audits

The final candidate must prove:

- every Rust production/test file is below 400 physical LOC;
- no Task 22G `include!` or `include_bytes!` source split;
- no unsafe, FFI, native dependency, platform branch, filesystem/process,
  thread, alternate geometry engine, or host-specific oracle;
- no fixture filename/hash/G-code read, no expected coordinate table in
  production, and no literal production 0.049/49000 branch;
- wrapper cleanup never calls two-pass `union_ex`;
- project delta is derived through the exact f32 chain;
- only approved 3MF-derived Option ownership reaches production;
- obsolete executable Orca source-pinning tests remain absent;
- both committed fixtures and all released Task 22F production blobs remain
  unchanged except the explicitly planned narrow integration seams;
- `git diff --check`, exact path manifest, per-file hashes, and normalized
  composite digest pass.

## Mandatory independent review loop

After implementation and the full local matrix, start one independent
read-only reviewer thread. Give it the exact manifest/digest, specification,
plan, source boundaries, test inventory, and execution evidence. It must issue
separate verdicts for:

1. requirement completeness;
2. logical correctness;
3. edge cases;
4. code quality;
5. test coverage;
6. actual execution results.

The reviewer returns P0-P3 findings and a concrete fix list. It may not edit
files. The main thread applies only source-cited fixes, reruns affected and
full verification, freezes a new digest, and sends the candidate back to the
same reviewer. Continue until all six axes pass or a reproduced external
blocker is documented.

After the six-axis loop passes, obtain three fresh reviews on unchanged bytes:
whole specification compliance, whole code quality, and direct default-model
implementation review with task/edit denied. Any code or test edit invalidates
all three.

## Documentation and release

Only after implementation approval:

1. update `THIRD_PARTY_NOTICES.md` so the Clipper component scope explicitly
   includes the closed offset rewrite without changing repository licensing;
2. update `option-parity-v4.md` with actual ownership, f32 scaling, exact KSR
   facts, and deferrals;
3. update `roadmap.md` to mark Task 22G implemented while full G-code parity
   remains incomplete and to keep Task 22H/I exact boundaries;
4. correct the prior roadmap's `closing*` context so it is not claimed as an
   implemented generic overload;
5. obtain independent documentation approval;
6. rerun the complete docs-inclusive matrix and exact manifest audits;
7. commit with Conventional Commits, expected subject
   `feat(geometry): port Clipper closed offset`;
8. push normally, verify local/tracking/direct remote identity, and monitor the
   exact-SHA Tier-1 run until all five jobs pass.

## Stop conditions

- Any fixed-source, f32, libm, wrapper-order, or oracle ambiguity stops
  implementation for source audit.
- Any corrected oracle disagreement is traced to source semantics before
  production or expected values change.
- Any path outside the 34-path manifest stops work for plan amendment and fresh
  review.
- Any file reaching 400 LOC is split with a real module before continuing.
- A Tier-1 platform mismatch is not normalized with a platform oracle.
- Never read the reference G-code to implement or repair this slice.
- Never claim full KSR parity or complete the persistent goal after Task 22G.

## Gate checklist

- [ ] Corrected oracle receives two independent approvals
- [ ] Exact spec/plan hashes frozen
- [ ] Fixed-source/spec approval
- [ ] Ares/plan approval
- [ ] Direct default-model spec/plan approval
- [ ] Package 0 native/browser oracle RED and two reviews
- [ ] Package A RED/GREEN and two reviews
- [ ] Package B RED/GREEN and two reviews
- [ ] Package C RED/GREEN and two reviews
- [ ] Package D unchanged complete KSR native/browser GREEN and two reviews
- [ ] Full native/WASM/browser matrix green
- [ ] Structural/provenance/hardcoding audits green
- [ ] Same-thread six-dimensional fix/re-review loop passes
- [ ] Whole spec, quality, and default-model approvals
- [ ] Provenance/architecture/roadmap docs reviewed
- [ ] Final docs-inclusive matrix and exact manifest green
- [ ] Conventional commit pushed normally
- [ ] Exact-SHA Tier-1 green on all five jobs
- [ ] Task 22H started; persistent goal remains active

**Status: DRAFT — implementation is forbidden until the exact specification
and plan receive all pre-implementation approvals.**
