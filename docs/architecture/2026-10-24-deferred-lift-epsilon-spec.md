# SPEC: `GCodeWriter::travel_to_xyz` deferred-lift EPSILON gate

Status: discovery / SPEC-RR only; no RED or implementation authorization.
Base: `9cfaf0d16f0f5c4bf5fb6bf6c62d1c54e0d94ecf`.

## Owning upstream boundary

This is a source-cited Rust rewrite correction, not a new Ares travel policy.

- `OrcaSlicer/src/libslic3r/libslic3r.h:52`: `EPSILON = 1e-4`.
- `OrcaSlicer/src/libslic3r/GCodeWriter.cpp:625-648`, `lazy_lift`:
  a positive configured `target_lift` can be retained in `m_to_lift` even if
  it is at or below EPSILON. Scheduling and consumption have different gates.
- `GCodeWriter.cpp:685-710`, `GCodeWriter::travel_to_xyz`: the deferred-lift
  arm starts only when `std::abs(m_to_lift) > EPSILON`. Only inside that arm
  does the writer compute a raised destination and clear `m_to_lift`.
- `GCodeWriter.cpp:730-777`: slope/normal lift and clear/unclear XY/XYZ
  emission belong to that gated arm.
- `GCodeWriter.cpp:778-808`: an unconsumed subthreshold lift takes the ordinary
  `will_move_z` / forced-Z travel path. This is not equivalent to consuming it
  and emitting an apparently zero-height hop after coordinate rounding.

Ares destinations (relative to `crates/ares-core/src/project_slice/gcode_emit/`):

1. `motion/travel/lift.rs:146-153`, `emit_pending`: currently calls
   `state.pending_lift.take()?` without the upstream numeric EPSILON gate.
2. `motion/path/start_travel/approach.rs:196-236`: pending-lift classification
   and the separate first-position consumption bypass `emit_pending` and need
   the same upstream distinction between scheduled and consumable lift.
3. `motion/path/start_travel/approach/move_emit.rs:181-219`: first-position
   dispatch and layer-change split classification currently treat `Some` as
   sufficient evidence that a deferred lift will be consumed.

Existing pending-lift/first-travel scaffolding is only a temporary compatibility
shell for the cited writer state machine; it must not define an independent
Ares epsilon or hop policy. No change to `motion/state.rs` is proposed.

## Observable difference

Stock `tests/parity/orca_cli_ender3.3mf`, prime tower disabled in every input,
otherwise change only the real printer `z_hop` setting. Same geometry, templates,
materials and travel/retraction policy. GUI owner:
`OrcaSlicer/src/slic3r/GUI/Tab.cpp:5150-5155`, particularly `z_hop` at5152.
`PrintConfig.cpp:5255-5264` defines a floating value with min0/max5, default0.4;
there is no positive lower floor or minimum increment in that option definition.
The two targets deliberately exercise a numerical boundary, not a recommended
physical printer hop height. No claim is made that the UI displays five decimal
places; these are schema-valid saved printer values, accepted by Native CLI.

Fresh official Native double replay and actual Ares CLI discovery:

| Case | `z_hop` | Input SHA256 | Whole generator-normalized comparison |
|---|---|---|---|
| stock-control | 0.4 | `a360b2d4d71a140a197726b8ab4c8c8b370ad2e17957fa742da2a2346f9336ae` | PASS |
| zero-control | 0 | `6ec170cd2be5d9598f978c3363587e571814ab118b704a81b636c53c71244f8d` | PASS |
| below-epsilon | 0.00005 | `7614d97c841ae5300be6f4aa219e05241ef6f876c199a084a38afc323fc682c5` | FAIL |
| at-epsilon | 0.0001 | `efec866bdcb60c0e523c064fa73bb298eb11298c469dad7610e1e6d01e1892ba` | FAIL |
| above-epsilon | 0.00011 | `a5320b5b36a1596031dc0d6bb25c74447ccfbb948169ebfdf1f0b6192aa9c2fa` | PASS |
| one-micron | 0.001 | `0a80395c48bea47ebb7558f7653a9f523a52950fff9c290fe16c0e1e07e82710` | PASS |

All12 fresh Native slices replay-stable; all probe commands exit0. The first
whole-stream target difference is byte1041: `M73 P6 R7` versus Ares `M73 P6 R8`.
Do not mislabel this as a primary progress-estimator bug or remove that line.
The motion witness immediately after the first retraction/layer fan marker is:

```gcode
; Native
G1 X103.206 Y103.296 F9000
G1 Z.2
G1 Z.2

; Ares
G1 Z.2 F9000
G1 X103.206 Y103.296
G1 Z.2
G1 Z.2
```

Later same-layer travels likewise gain extra Z-only / XYZ / unlift moves in
Ares. Their timing changes also move M73 records. The strict full stream remains
the acceptance surface; diagnostic motion excerpts are not a comparison mask.
The >EPSILON control0.00011 passes despite also rounding the tiny height at
normal output precision, separating the writer branch gate from formatting.

## Included / deferred behavior

Included, after separate approval:

- Port the exact `abs(m_to_lift) > 1e-4` consumption boundary, including equality.
- Retain a scheduled subthreshold amount/pending state as upstream does when
  the consumption arm is skipped; do not simply disable small positive hop
  scheduling or clear it eagerly.
- Keep first-position and ordinary travel classification consistent with
  `travel_to_xyz`'s gated branch, using existing private state where possible.
- Preserve normal/spiral/slope emission and ordinary Z-travel behavior outside
  this narrow gate. Preserve strict stream comparison and resulting M73 output.

Deferred: eager lift, spiral-vase special lift, general `will_move_z` redesign,
new lift storage/API, AutoLift geometry, clipping/routing, retract/wipe policy,
processor parsing/time algorithms, templates, bed, startup fan, fuzzy, tower,
skirt/brim/support speed, and all other workers' reserved slices. No changes to
`motion/state.rs` or W4's frozen temperature scanner. If the bounded rewrite
requires changing these deferred owners, stop and report before expanding.

No epsilon is to be invented from formatter precision or Rust machine epsilon.
The existing enum/boolean shell must be evaluated against the upstream numeric
state semantics, not strengthened into an Ares-owned interpretation.

## Proposed RED and exit criteria (not executed)

After SPEC approval, freeze SHA-pinned inputs and Native-first
`native_stream_case::check` cases: below/equal targets plus zero/above/default
controls. Qualify fresh Native replay and Native branch-shape observations,
then compare complete public API and CLI streams with the unchanged comparator.
Official then seeded on CPU17/PIN17, jobs1/threads1. No Native constants/goldens,
input-specific branches, order masks or altered generator normalization.

Failures to pin before implementation: `<` instead of `<=` boundary; clearing
unconsumed pending state; first-position bypass; ordinary layer-change split
using scheduled rather than consumable state; regressions for zero and above
threshold; treating M73 consequences as an independent processor fix.

Freeze RED-RR before requesting GREEN. Only after separate GREEN authorization:
all directed cases pass both modes, fmt and workspace all-target clippy pass,
and coordinator-authorized exact integration-tip dual FULL passes. This document
makes no public API, seeded, RED, GREEN or FULL claim for this new slice.

## Evidence / ownership audit

`/home/indexyz/ares-evidence/w4-deferred-lift-epsilon/`: `probe.py`, `probe.log`,
`results.json`, six input archives, per-case two Native logs/streams, actual Ares
CLI stream/log, full `ordered.diff`, `overlap-audit.json`, `binary-provenance.json`.
Probes were serial on CPU17, `ARES_PARITY_THREADS=1`, `ARES_PIN_OFFSET=17`,
`OMP_NUM_THREADS=1`, standard official inherited-affinity wrapper with setarch.

Discovery binary `/tmp/w4-temperature-red-target/debug/ares`, built at frozen
`a5cc569c05857121af80533679c801f6258b9cb5`; its entire motion subtree is identical
to SPEC base main9cfaf0d16. This is disclosed CLI discovery, not an exact-main
build qualification. The frozen temperature slice does not affect these spaced
stock templates. No build or production change was needed for discovery.

Preproposal audit: candidate paths have no overlap with W5 Type1 work/proposal,
W1 fuzzy qualified/startup-fan green, W6 frozen six-chain, W2 skirt/support state
and seam scope, or W2 frozen515e95975. A327-ref worker1/2/3/6 + W5 architecture
scan found no `m_to_lift`/EPSILON or subepsilon deferred-lift SPEC reservation.
W3 processor-word work is excluded explicitly; this slice owns writer branch
selection only, not processor input parsing. Exact audited tips and changed
paths are retained in `overlap-audit.json`; historical unrelated worker branches
are not permission to touch their currently reserved areas.

Stop after SPEC-RR. No production or test/fixture edits accompany this proposal.
