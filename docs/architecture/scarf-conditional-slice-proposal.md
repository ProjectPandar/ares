# PROPOSAL: conditional scarf smoothness eligibility

Status: Proposed.
No production/test/fixture/comparator edit.

## ONE observed gap, not a hypothetical feature

Select the scarf deferral: `seam_slope_conditional` currently unconditionally
suppresses active scarf in Ares, even when Native's smoothness predicate accepts
the loop. The existing nonconditional scarf implementation is already ported;
this proposal is NOT "implement scarf" or an Ares-designed pipeline.

Frozen valid one-object/part/filament10x10x0.8mm triangular prism,6 vertices,
8 in-bounds triangles, four0.2mm layers, Classic2walls, aligned seams;
support/brim/tower off. Active external scarf, inner0/entire0, start10%, length10,
steps10, angle threshold0 degrees (Native's declared0..180 domain), overhang
threshold0%. The ONLY control→positive change is seam_slope_conditional0→1.
All mesh/relationship/transform/other settings bytes and every other ZIP member
are identical; whole/ZIP-entry SHA and singleton decoded-setting proof retained.

| Frozen input | SHA256 | Fresh official2.4.2 Native FIRST → real bare Ares CLI |
| --- | --- | --- |
| conditional control0 | `0e2448b51db7003392c83e123aed4d1340d49eecbee2b725c4ca5bf12fd842b1` | Native0 complete; CLI0 complete; unchanged generator-only full strict PASS |
| conditional positive1 | `b89abfb6df717f88a513723f469c966577c7361d789f623dcd3e9feddca4849d` | Native0 complete; CLI0 complete; full strict FAIL |

Positive first complete strict difference: byte451, line18, column9;
Native38646 versus Ares33780 normalized bytes:

```
Native: M73 P0 R2
Ares:   M73 P0 R1
```

First differing G0/G1/G2/G3 movement, diagnostic ONLY, motion91:

```
Native raw181: G1 X109.964 Y114.425 Z.22 F9000 ; move to first perimeter point
Ares   raw183: G1 X110 Y114.497 F9000
```

Native control/positive complete motion sequences are identical. Positive Native
has48 actual Z+extrusion moves, versus0 in Ares, beginning:
`G1 X109.72 Y113.938 Z.23 E.0027 ; perimeter`. Both Native/Ares complete config
footers echo external/conditional1/angle0/overhang0%/start10%/length10/steps10.
This is an active eligibility discrepancy, not a bare-versus-loaded request
mismatch. M73 is an earlier duration symptom, NOT a proposed clock owner.
Complete comparator remains unchanged, including all M73/motion/arc/statistics/
configuration/footer bytes. No projection/filter/tolerance replaces that oracle.

## Source owner and exact Rust destination

Pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`, libslic3r:

- `GCode.cpp::GCode::extrude_loop:5744`, activation5777-5782 and conditional
  smoothness5783-5785: call loop.is_smooth(angle_threshold*PI/180,nozzle_diameter),
  not "conditional means off". Layer0 remains inactive as source specifies.
- Delegated `ExtrusionEntity.cpp::ExtrusionLoop::is_smooth:369-424`: source
  cyclic polygon/arm-distance/angle predicate. No substitute "rounded object"
  heuristic, angle-only fixture shortcut, generic new geometry API or pipeline.
- Adjacent optional overhang predicate is `GCode.cpp:5787-5791`; its source
  handoff is `SeamPlacer.cpp::place_seam:1563` returning unsupported_dist,
  distinct from the comparator's overhang penalty. It is NOT exercised by0%.

Rust destination, relative to `crates/ares-core/src/project_slice/gcode_emit/`:
`motion/scarf.rs::build` (current `|| options.scarf.conditional` bypass at56),
`motion/options.rs::ScarfOptions`, and `motion/options/build/scarf.rs::from_region`
for native region/default angle configuration. Existing `motion/loop_paths.rs::emit`
provides the source-owned pre-clipping loop and active-layer context;
MotionOptions already has nozzle_diameter. Reuse existing scarf construction/
quantized emission/speed/flow logic; they already pass the active control.
A future SPEC must source-bound any necessary private helper split (Rust≤400LOC).
No seam geometry, travel, front-clip, speed, clocks or schema redesign is proposed.

**Scope/dependency gate, not implementation permission:** the measured slice is
conditional smoothness eligibility, with optional overhang threshold disabled.
Do not silently absorb unsupported_dist production/propagation, or mistake its
penalty field for raw distance. Prior seam audits explicitly defer that handoff.
Before a SPEC can admit GREEN it must resolve how the adjacent nonzero-
overhang predicate remains source-faithful; independently qualify/source any
necessary dependency, or stop. Do not introduce a permanent threshold0
special-case/fallback, enable all conditional scarves without the predicate,
or claim this one input qualifies nonzero-overhang behavior. No such design or
new guard is part of this PROPOSAL. Scarf path generation, painted seam
rules and new random/nearest behavior remain outside the selected gap.

## Already-done / UB check

At discovery time (main `41f2c90f765d42c6620583121296bb61a2cb839c`) every
pending scarf source snapshot still contained
`|| options.scarf.conditional`, and none added the smoothness predicate.

Main/base scarf history already includes3a4324d95 active scarf,
016456549 default-speed cap,765ae4964 front-clip positive seam-gap gate,
48ea1d7d8 contour-Z kernel. All-branch `git log -G` on conditional/smoothness/
angle in scarf owner finds only the initial3a4324d95 blanket bypass. Existing
work is explicitly acknowledged, not mislabeled as missing scarf implementation.
Pending painted work is painted BRIM admission, not painted
seam selection. Scarf rounding/front-clip work is already in this base; no
second proposal for it. Prior option-coverage PASS is not parity proof for this
active setting combination; this concrete complete positive now diverges.

This is NOT a request to remove guards on degenerate native geometry, zero
steps, nonpositive length or undefined native data. Mesh is finite/closed,
loops have positive nondegenerate arms, nozzle0.4, length10, steps10 and
start ratio0.1; angle0 is explicitly valid (PrintConfig.cpp5546-5556).
Native0/complete is observed, not used alone as proof against UB. In particular,
SeamCandidate.unsupported_dist is not initialized by its constructor
(SeamPlacer.hpp59-70); the optional conditional overhang consumer is therefore
**not** used as authority here. Frozen0% short-circuits it at GCode.cpp5787;
first-layer scarf is separately inactive. No undefined distance/overhang value
is needed to establish this smoothness discrepancy, and no native-UB guard
retirement or initialized-oracle substitution is proposed.

Random was actually tested on the same accepted prism (aligned control versus
random positive): both Native0/CLI0/complete whole PASS. Not proposed on arithmetic
suspicion. Painted XML admission and other nearest residuals are not secondary
proposals; no missing Ares stream is called equality/divergence evidence.

## Evidence and next step

Discovery evidence contains frozen inputs, whole/ZIP SHAs, singleton delta,
exact commands/rc/
stdout/stderr/complete streams, activation+footer proof, unchanged comparator
source inclusion/build receipt, base/current binary/source hashes.
Four actual Native calls total (random pair+selected
conditional pair), per-input fresh cache/TMPDIR; no Native retries or
production edits. Existing exact9d981 CLI reused with its
frozen GREEN build/hash evidence; no claim that this discovery is formal API+
CLI/seeded RED. Proposal documents only, not permanent fixtures/tests or SPEC.

Next step: a bounded SPEC/dependency decision for this ONE source-cited gap;
public Native-first frozen RED and
separate bounded GREEN follow. If the optional dependency
cannot be safely bounded, stop rather than expanding the pipeline or inventing
a fallback.
