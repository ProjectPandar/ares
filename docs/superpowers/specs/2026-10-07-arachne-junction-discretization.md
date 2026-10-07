# Spec: Arachne junction-level parity of stored perimeter entities

Status: draft for coordinator review
Origin: the C1 curled-perimeter residual
(`docs/superpowers/specs/2026-10-07-curl-artificial-distance-residual.md`)
was localized outside the curl pipeline — the formula is proven
at parity by `artificial_distance_matches_the_sermoon_v1_oracle_records`
(commit `0786b86a9`, the C1 pre-evidence that rides this slice's
RED chain and does not merge separately). The root divergence is the
junction density of ares' stored `ExtrusionLoop`/`ExtrusionPath`
polylines. **This spec supersedes the implementation half of the C1
spec**: the C1 whole-file byte-compare on `case-6qvP0t` is one of
this slice's acceptance cases.

## Verified facts (probes reverted; none committed)

Case: stock matrix Sermoon V1 cube10
(`/tmp/ares-stock-matrix-028c758-seeded/case-6qvP0t/input.3mf`).

- Upstream's stored `erExternalPerimeter` entity for the layer-1
  outer wall carries **junction-level points spaced ~1.1612mm**
  (`/tmp/curldump.txt` CURL `seg=` records, sha256
  b7cad1f2f806e1c8a67c657aac4f110ff5fb58d20465bf87ba4f08e3e58d4977;
  entity A: `(4.597,4.645)→…→(-4.645,1.935)`, 11 pts; entity B:
  `(-4.645,1.935)→…→(4.645,4.645)`, 4 pts; entity C: `(4.645,4.645)
  →…→(-4.645,1.983)`, 11 pts).
- Ares' stored `ExtrusionLoop.paths` for the same wall = **1 path,
  5 points** (4 corners + close). Its pre-simplify `ExtrusionLine`
  has **9 junctions** — 4 corners + 4 edge midpoints, all uniform
  width 567079 — and `ExtrusionLine::simplify` legitimately removes
  the 4 collinear midpoints, matching the upstream algorithm
  (`Arachne/utils/ExtrusionLine.cpp:43-148`).
- So upstream's beading emits mid-edge junctions that ares' does
  not produce at all (not a simplify difference — a generation
  difference). The emitted G1 stream happens to coincide: ares'
  scarf `detail()` bisection (9.29mm edge / 8 = 1.1612) recreates
  the same vertex positions for the slope segments.

## Upstream anchors (source-cited boundary)

- `src/libslic3r/Arachne/SkeletalTrapezoidation.cpp`:
  `discretize` (:220-310, `discretization_step_size` = `scaled(0.8)`
  at `WallToolPaths.cpp:491,548`), `generateTransitioningRibs`,
  `generateExtraRibs`, `generateSegments` (`generateToolpaths`
  :496-545) — VD-edge discretization produces skeleton rib
  endpoints that become wall `ExtrusionJunction`s at ~rib-spacing
  intervals, not only at transition anchors.
- `src/libslic3r/Arachne/WallToolPaths.cpp:545-561`:
  `wall_maker.generateToolpaths(toolpaths)` → `stitchToolPaths` →
  `removeSmallLines` → `separateOutInnerContour` →
  `simplifyToolPaths`.
- `src/libslic3r/PerimeterGenerator.cpp:389-524`: entities are
  materialized (`clip_extrusion` / `extrusion_paths_append`) AFTER
  toolpath generation, preserving junction positions.
- Ares port: `crates/ares-core/src/arachne/trapezoidation/`
  (`generate_toolpaths` at `trapezoidation.rs:72-86`,
  `discretize.rs`, `transitions/`, `builder.rs`),
  `crates/ares-core/src/arachne/wall_toolpaths/` (`generate`,
  `postprocess::simplify_toolpaths`).

## The open delta

For a 9.29mm square edge, upstream emits wall junctions at
1.1612mm spacing while ares emits only the two mid-edge
transition(?) junctions and loses those to simplify. Hypotheses,
to be dispositioned in order:

1. `generate_extra_ribs` / `generate_transitioning_ribs` insert
   fewer ribs in ares' port (marking/transition propagation or the
   extra-rib heuristic differs for a uniform-bead region).
2. `generateSegments` walks upstream's ribs into per-rib wall
   junctions while ares' port collapses rib-to-rib runs.
3. `stitchToolPaths` or the skeleton discretization in ares
   produces fewer/differently-placed rib endpoints
   (`discretize_point_point`/`discretize_parabola` step rounding,
   `builder.rs:183-300`).

## Method

- Junction-level first-difference evidence: for the same
  input/layer/loop, dump ares' `ExtrusionLine.junctions`
  (pre-simplify) vs upstream's stored entity vertex sequence —
  the fixture holds upstream's minimal junction excerpt (from the
  curldump CURL records) with sha + provenance; the ares side is
  captured by a unit/integration test that builds the same
  polygon → `generate` → asserts the junction sequence, failing
  today.
- Temporary env-gated dumps are allowed during investigation only;
  no probe code in any commit.

## Fixture

`crates/ares-core/src/arachne/<fixture>.txt`: the upstream
junction sequences for the Sermoon-V1 cube outer-wall entities
A/B/C at layer 1 (vertices from the curldump `seg=` records
above), plus the input polygon (9.29mm square at ±4.645mm,
bead width 0.61/height 0.2, uniform) needed to drive
`generate`. Header records `/tmp/curldump.txt` sha256
`b7cad1f2…` and the nix oracle-probe provenance.

## RED tests (required order)

1. `ares-core` white-box RED: feed the fixture polygon through the
   ares Arachne wall generator → assert the pre-simplify AND
   post-simplify junction sequence equals upstream's (count +
   coordinates). Fails today (9 vs ~19 junctions; 5 vs ~15 stored
   points).
2. E2E whole-file byte compare on `case-6qvP0t` (Sermoon V1),
   normalize only `; generated by` — the C1 acceptance case —
   must go green with the fix, including the M73/E hunks (which
   follow from the vertex sequence).

## Blast radius / gate requirements

Junction density feeds every downstream consumer of the stored
perimeter polylines, not just curl:

- `estimate()` boundary distances per extended point (all layers,
  all profiles with `enable_overhang_speed`).
- `CurlTracker::collect` (all `detect_overhang_wall` Arachne
  profiles — i.e. effectively every stock Arachne case).
- Emitted G1 vertex sequences (wipe paths, seam placement vertex
  search, E values, emitted move splitting at ProcessedPoints).
- VG1/moves vertex data fed to libvgcode consumers.

Therefore the GREEN bar is: **full v2 control set (seeded +
official) on the submission hash must be name-for-name
non-regressing vs the `d789b0750` base control** — identical or
better per test name; every changed case must be a convergence
(was DIVERGENT → byte-equal or fewer diff lines), never a new
divergence. `cargo nextest run --workspace`, clippy, fmt,
wasm32 check as usual.

## Scope

In: junction generation/density inside `SkeletalTrapezoidation` /
`WallToolPaths` ports for the parity-failing case(s); the
junction-sequence fixture test; the C1 byte-compare flip.
Out: curl formulas (proven equal), emitted-move simplification,
scarf detail(), other DIVERGENT matrix cases unless they share the
boundary.
