# Medial-axis endpoint zero-direction normalization

Status: docs-only SPEC; public Native-first RED is next. GREEN requires separate
RED approval. Base b9cc91ff0d1f4909bf23e87286be88c34ec5cd74.
Work branch worker6/medial-endpoint-zero-spec. CPU15 only; no FULL window.

## Upstream ownership, including Eigen semantics

This is a Rust rewrite slice of libslic3r ExPolygon endpoint postprocessing,
not an Interlocking/Voronoi redesign or Ares cooling/timing policy.

- `OrcaSlicer/src/libslic3r/ExPolygon.cpp:295–303::ExPolygon::medial_axis`
  extends an initial endpoint with `(p2-p1).normalized() * max_width` before
  contour intersection. The final endpoint branch306–313 uses the same operation.
- This `normalized()` is Eigen `MatrixBase<Derived>::normalized`, NOT unconditional
  division by norm. Source: `Eigen/src/Core/Dot.h`, function body:
  `RealScalar z = n.squaredNorm(); if (z > RealScalar(0)) return n /
  numext::sqrt(z); else return n;`. Its documentation explicitly says norm-zero
  vectors return a copy of the input. Thus zero direction MUST remain zero.
- Exact tagged upstream references, both inspected:
  [Eigen3.4.0 Dot.h](https://gitlab.com/libeigen/eigen/-/blob/3.4.0/Eigen/src/Core/Dot.h)
  and [Eigen5.0.1 Dot.h](https://gitlab.com/libeigen/eigen/-/blob/5.0.1/Eigen/src/Core/Dot.h).
  Both have that same branch. Orca's checked source dependency declaration
  `deps/Eigen/Eigen.cmake:6–9` identifies5.0.1. These references establish the
  semantic contract independently of the Native observations below; no claim is
  made that the local Native DWARF capture contained the Eigen header body.
- `Polygon.cpp:158–167::intersection` and `Line.hpp:123–152::line_alg::intersection`
  are downstream consumers. Degenerate P→P fails intersection and retains the
  original initialized endpoint; they do not need correction.

## Exact Rust boundary and architecture constraints

Destination: `ares-core/src/geometry/medial_axis.rs` PRIVATE `normalize` helper
and its two callers inside `extend_endpoints`. The helper currently computes
length=sqrt(x*x+y*y) then always divides. For zero input this returns NaN/NaN;
subsequent float→integer conversion makes an origin point instead of a local
endpoint. This is an incomplete compatibility shell around the Eigen concept.

Included future correction: reproduce `squaredNorm > 0` guarded normalization,
returning the original direction otherwise; preserve nonzero arithmetic/order
and both initial/final endpoint callers. A zero-vector branch ports the cited
upstream algorithm, not an independently designed fallback or defensive policy.

Do NOT deduplicate points or drop zero-length medial segments; they are present
in Native too and belong to downstream source algorithms. No new epsilon,
threshold, endpoint search policy, public API or helper/geometry abstraction.
Do not modify global vector normalizers, slicer transforms, Polygon intersection,
Voronoi construction/validation/chaining, medial retention, VariableWidth,
InterlockingGenerator/VoxelUtils, seam selection/cache, fill, cooling, time/M73,
brim, painted, spiral or ironing. COORD confirmed no W1 painted ownership conflict.
Core remains platform-neutral for WASM/Windows/macOS/Linux; no I/O enters core.

## First producer evidence

Read-only source-cited evidence root:
`/home/indexyz/ares-evidence/w6-b9cc-interlock-params-j7321r01/`.
Authoritative `downstream-proof.json` enumerates valid captures and checks.

- All300 generator pre/post ordered layer/region polygon records are exact.
- All150 delivered Classic PerimeterGenerator region/layer input polygons exact.
- All86 post-Voronoi/chaining, PRE-endpoint-postprocess records exact (contour,
  holes, max_width, thick points, widths and endpoint flags). This excludes a raw
  Voronoi producer difference, not merely a similar-looking final polygon.
- Shared raw thick polyline starts P,P,Q with
  P=[-10081435,3147594], Q=[-10311000,1993495], true/true endpoint flags.
- Native contour query is P→P, returns false, output remains P.
- Rust local normalize(0,0) normal returns NaN/NaN in two reached cases. The
  resulting contour query is [0,0]→P, returning [-9824031,3067228]. This is the
  first demonstrated geometric producer divergence and matches Eigen's missing
  zero-direction branch exactly.
- VariableWidth then receives2 changed prefixes among44 polylines, with the
  corresponding width arrays unchanged. Do not compensate in conversion.
- At actual printZ9.4/layer46, all14 postseam loop paths match exactly, but gap
  paths are Native30/Rust31: the false extension adds a width0.5800495 segment
  and emitted `G1 X105.919 Y113.148 E.0118` plus different travel start.
- The apparent Z9.8 is TRAVEL LIFT, not a different slicing layer. Raw precooling
  outer feed7059.559 matches; final2427/2429 occurs after cooling consumes the
  different layer geometry. First final M73 placement is likewise not the root.
  Complete output parity, not only this segment/feed, is the future GREEN gate.

All18 valid GDB runs use may-call-functionsOFF, field/memory reads and normal
return breakpoints, no inferior calls or data/register writes. Normal exits and
complete output equality against non-GDB runs are verified. Early env/optimized
or partial-lambda probes are explicitly excluded; no invalid before-state claim.

## GUI-reachable Native-first public RED matrix

Use exactly the two ALREADY exported Native input-only archives:

| Role | boundary_avoidance1 input | SHA256 |
|---|---|---|
| Target | beam1 | `a9ee974eec98a58a499dea4b508dc186cc32708d8364faff4098e0257eec8775` |
| Sole-key PASS control | beam0 | `42309a59caea9f564dd566e2c0aa8914d5d22bc21fa127accbe4574408b7c523` |

GUI `Tab.cpp:2696–2699` exposes these interlocking controls. Only settings JSON
interlocking_beam1/0 differs; geometry and other payloads match. Do not regenerate,
change settings to avoid residuals, commit Native output or duplicate existing
fixture hashes. Existing nondefault-layer/orientation passing inputs remain
external diagnostic neighbors, not additional frozen targets.

Discovery already observed fresh Native2 stable BEFORE public byte API/real CLI
in both modes: target strict FAIL, control complete PASS, API success/CLIexit0,
raw API/CLI equal. Canonical first target difference byte95402/line4865 (M73 before
versus after travel). No mask/filter of M73, motion, timing or metadata.

Formal tests must reuse existing `runner::native_stream_case::check` and its
unchanged canonical `native_api_cli.rs::NativeCase` engine. No copied runner,
normalizer, internal-only substitute or error-as-success assertion. Check runner
LOC before registration. All changed text files must be <400 lines.

Sequence: this docs-only SPEC -> test/input-only Native-first RED at exact commit
hash, official/seeded2case runs -> RED REVIEW-REQUEST and STOP for COORD approval
-> production-only private helper correction if separately authorized. Preserve
complete ordered Native/API and Native/CLI comparison plus raw API/CLI equality.
On RED canonical comparison panics before its final raw equality assertion;
audit actual emitted raw files independently, do not claim that assertion ran.
Any strict residual on GREEN means STOP. FULL/rebase require a later window.
No push/self-merge. User's loop_paths/tests.rs keep-deleted decision is unchanged.
