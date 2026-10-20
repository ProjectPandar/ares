# ARD: staggered seam interpolation f32 semantics

Status: approved source boundary (original revised SPEC c3c30003d);
pre-window preparation separately authorized on main e81afae89. The
historical diagnosis below used base a5eb0f035 and superseded the
random-position hypothesis in 4cbbb68d0 / 26d895d98. No FULL is authorized.

## Witness and falsified hypothesis

`tests/parity/random-seam-precision/random-staggered.3mf` has
`seam_position: "random"`, `staggered_inner_seams: "1"`.
RED `5b99cbf0c` fails against both official and seeded Native at line
9865: `G1 X100.675 Y118.143 E.0392` versus Ares `E.03921`.
API and bare CLI agree. Receipts:
`/tmp/w5-rs-red1-{off,seed}-art/orca-runner-*/random-staggered/public/public-result.json`.

The original attribution to random.rs's ratio-form random selection was
falsified: replacing it with `edge.vector.normalized() * picked` did
NOT change the Ares output SHA or the failed comparison. That uncommitted
trial was discarded. Identical printed XY does not establish a seam ULP
difference; the previous SPEC's inference was unjustified.

## Read-only GDB evidence (CPU 7-8)

Native: seed-replay `/nix/store/1k0p5nlbs3zsfdcy08g3y5fb7bcm1qrv-orca-slicer-seed-replay-2.4.2`,
matching debug output `bms1pq7n8cmzjc2zpgnjybzs1vrfz6rq-…-debug`.
Ares: RED binary `/tmp/w5-targets/rs-red1/debug/ares`.
No inferior function calls, memory/register writes or behavior patches.
Scripts and JSONL captures are under `/tmp/w5-x/random-*-trace.*`,
`random-*-split.*`, and `random-*-stagger.*`.

100 random selections matched by seed bits + start index, not thread
arrival order. For the failing layer (slice_z 12.5, output Z12.6),
Native and Ares agree BIT-FOR-BIT at random selection:

| Value | Both implementations |
|---|---|
| selected index | 1 |
| seed | (9.77500057220459, 9.77500057220459, 12.5) |
| selected position | (-9.774999618530273, 9.77500057220459, 12.5) |
| edge | (0, -19.549999237060547, 0) |
| edge length | 19.549999237060547 |
| picked_len | 1.2218742370605469 (bits 1067214432) |
| final_seam_position | (-9.774999618530273, 8.553126335144043, 12.5) |
| final position bits | (3239863910, 1091099035, 1095237632) |

Nine OTHER random final positions differ in low bits across the 100
selections; none is this layer. They do not establish the cause of this
witness and remain deferred.

Downstream, immediately before stagger interpolation:

- Projected scaled point: (-9324999, 8553126), equal.
- Next scaled point: (-9324999, -9324999) on the inner loop.
- depth: 0.449999988, equal.
- dist: 17.8781242, equal.

The first observed unequal stage output is the staggered point passed
to loop splitting:

| Scaled coordinate | Native | Ares |
|---|---|---|
| X | -9324998 | -9324999 |
| Y | 8103126 | 8103125 |

The outer seam remains equal at (-9774999, 8553126). Native and Ares
both have 200 split calls; the target inner split is ordinal 124
(zero-based), confirmed by its coordinates and layer ordering.
Read-only debug runs still reproduce the original E difference.

## Upstream rewrite boundary / Rust destination

`OrcaSlicer/src/libslic3r/GCode/SeamPlacer.cpp`,
`SeamPlacer::place_seam`, staggered-inner-seam loop at 1604-1619:

    Vec2f a = unscale(projected_point.foot_pt).cast<float>();
    Vec2f b = unscale(next_point.foot_pt).cast<float>();
    float dist = (a - b).norm();
    if (dist > depth) {
        Vec2f final_pos = a + (b - a) * depth / dist;
        next_point.foot_pt = Point::new_scale(final_pos.x(), final_pos.y());
    }

Destination:
`crates/ares-core/src/project_slice/seam_placement/placement.rs::staggered_seam`.
The current scaffold interpolates on integer coordinates using a f32
ratio promoted to f64, then casts to i64. It omits the upstream
unscale-to-f32 and rescale boundaries and changes arithmetic order.

## Approved included behavior

Port the cited interpolation exactly: unscale endpoints, cast each
coordinate to f32, compute f32 vector distance and `(b-a)*depth/dist`
in upstream operation order, add a in f32, then reuse existing
`scale_position` for `Point::new_scale` conversion. Replace only the
stagger interpolation scaffold; do not modify E formatting or random
selection. Reuse existing helpers; no new normalization helper.

## Deferred / non-negotiable limits

- `alignment/random.rs` remains unchanged, including its current ratio
  guard and terminal-edge condition. The earlier terminal-edge
  equivalence argument is not needed for this revised boundary.
- Other low-bit random-position differences require their own witness;
  they cannot be justified by this E difference.
- No changes to random constants, seed generation, viable selection,
  nearest/aligned seam algorithms, E rounding or unrelated path handling.
- The first unequal stage is established, but whether porting it alone
  eliminates the E witness remains to be tested, not asserted.

## Verification gate

Reuse the existing dual-Native RED test. Before declaring GREEN, show
that the corrected staggered point matches Native and that the SAME
whole-output API/CLI test passes in both modes. If not, stop and trace
the next downstream boundary without speculative changes. Final merge
requires current-main exact-hash official/seeded FULL, fmt, clippy and
wasm32 checks. This remains a source-cited upstream rewrite slice, not
an Ares-owned seam or G-code design.
