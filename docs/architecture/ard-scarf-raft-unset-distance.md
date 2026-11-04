# ARD: raft conditional scarf without a previous-layer distance

Status: accepted deterministic policy, 2026-10-29. Supersedes only the
first-object-over-raft guard amendment in `scarf-conditional-spec.md`; it does
not redefine the remaining conditional scarf port.

## Source boundary

Pinned OrcaSlicer: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `src/libslic3r/GCode.cpp:5777-5791`: `layer_id() > 0` admits the first
  object layer over a raft; conditional scarf compares `seam_overhang` with
  `scarf_overhang_threshold * 0.01f * line_width`.
- `src/libslic3r/GCode/SeamPlacer.hpp:59-70`: the `SeamCandidate` constructor
  does not initialize `unsupported_dist`.
- `src/libslic3r/GCode/SeamPlacer.cpp:1083-1092,1563`: a previous-layer
  distancer assigns the field; seam selection later reads it.

The first object layer has no previous object-layer distancer. Reading that
field is undefined in C++; the source does not specify a numerical value.
Repeated Native admission at 5% and 40% does **not** prove the field is zero.
It observes the comparison outcome for the pinned binary and fixtures only.

## Decision and Rust boundary

The absent selected support distance is defined as
`0.0` at
`crates/ares-core/src/project_slice/gcode_emit/motion/scarf/eligibility.rs`.
Delete the corresponding raft + active conditional scarf rejection in
`crates/ares-core/src/project_slice/capabilities.rs`.

This is a deliberate deterministic policy at an undefined upstream boundary,
not a literal rewrite of defined C++ behavior and not a claim about its bits.
A positive threshold and positive line width admit an absent-distance seam.
An available distance continues through the unchanged upstream comparison.
The old spec's prohibition on fabricating this scalar is superseded **only**
for this absent-distance policy. No legacy fallback or alternate pipeline is
introduced; the existing upstream-owned seam and scarf paths remain owners.

## Included and deferred behavior

Included: first-object-over-raft scarf admission, the existing conditional
smoothness/width/threshold checks, computed later-layer distances, shared API
and CLI behavior, and strict Native byte comparison on the frozen witness.

Deferred: defining upstream C++ UB, claiming all binaries/allocators/platforms
produce the same Native result, or changing seam candidate selection, raft
geometry, unrelated validation guards, and scarf path construction.

Residual risk: Native builds or memory layouts may disagree with this policy.
The evidence qualifies the selected binary and fixture, not universal parity.
Rust introduces no uninitialized read and the policy is platform-independent.

## Verification plan

1. Keep the Native-backed RED at the guard rejection, not archive/bootstrap
   failure. The fixture is derived from `scarf-conditional/overhang-high.3mf`
   with `raft_layers=2` and the overhang threshold changed from 100% to 40%;
   both hashes are in its manifest.
2. Re-run pinned Native three times and retain normalized byte digests and
   complete layer activity, not partial grep counts. The 40% witness has 64
   slope moves: 16 on each object layer 3–6, none on raft layers 1–2.
3. Compare complete Native/API and Native/CLI streams using only the existing
   generator normalization, then run the scarf/seam directed families.
4. Run workspace non-parity tests, fmt and clippy. Report inherited base
   failures separately; do not claim an all-green gate without their removal
   or a clean integration base. Retain repeatable artifacts.
