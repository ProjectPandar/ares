# ARD: source-owned spRear comparison and alignment

Status: proposed, docs only. [SPEC](seam-rear-spec.md) requires exact-hash
approval before RED. Base approved GREEN4dbba4fc09b4b84033797ab83f2941e5d8484b95;
not permission to change production, tests or fixtures yet.

## Owner and decision

Orca8500fcdcc `GCode/SeamPlacer.cpp::SeamComparator` owns spRear semantics:
Y preference777-779, rear similarity839-841, selection917-928. Rear alignment
uses accumulated length*0.3 at1364-1366 and is explicitly routed at1486-1492.
Port these branches into existing Rust project seam runtime/preparation/
alignment, preserving the object's Back mode as the upstream setup identity.
The current project Back bypass is temporary Ares scaffolding to replace,
not an alternative algorithm to retain. This is a libslic3r rewrite slice.

## Non-negotiable decisions

- Public value back, Rust ProcessSeamPosition::Back and upstream spRear refer
  to the same placement. AlignedBack is a distinct existing mode and is not
  renamed, reused or changed to approximate Rear.
- Native rear comparison is not a standalone max-Y vertex rotation: earlier
  priorities, equal-Y scoring, similarity and fitted seam positions remain
  meaningful. No basic-perimeter first_max_y helper or new shortcut/fallback.
- Reuse the approved physical-layer plane handoff, existing candidates and
  spline. No second planner, scoring pipeline, core I/O, public diagnostic
  API, geometry/travel/processor changes or new defensive guard.
- Type/overhang/embedding ordering remains source-owned. No invented paint
  state or newly implemented paint/scarf/random/nearest behavior is included.
  Preserve existing unrelated modes when shared alignment code becomes mode-aware.
- The6-vertex/8-face prism is already admitted by both engines. Its aligned
  control whole-PASS and back positive whole-FAIL on4351 are discovery, not
  relabeled API/seeded/on4dbb evidence. Formal RED verifies the actual new base.
- Whole-byte oracle retains M73/arcs/config/statistics/footer; first M73
  difference is not an authorization to modify clocks. First Native motion
  witness and complete public receipts are required; missing is NOT EVALUATED.
- SPEC approval→frozen public RED REVIEW→separate bounded GREEN REVIEW. Actual
  Native control PASS, positive target FAIL, and both public calls/results
  before assertions; after GREEN, residual whole mismatch means STOP/report.
- Keep API logic platform-neutral for browser WASM and all desktop Tier1.
  CPU15 targeted only; no full/workspace/merge/push. RED integrates only with
  its GREEN, under coordinator control. Approved4dbb ancestry stays intact.
