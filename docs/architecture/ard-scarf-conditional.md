# ARD: source-owned conditional scarf eligibility

Status: proposed; docs-only [SPEC](scarf-conditional-spec.md), based on approved
proposala30db87 and approved spRear9d981. Exact SPEC approval precedes RED;
separate RED review precedes any production GREEN.

## Source owner and decision

Pin8500fcdcc, libslic3r GCode.cpp::GCode::extrude_loop5777-5791 delegates smoothness
to ExtrusionEntity.cpp::ExtrusionLoop::is_smooth369-424. Replace the existing Rust
motion/scarf.rs conditional-returnNone scaffold with this upstream decision;
reuse existing scarf construction/emission, not a new Ares scarf pipeline.

Adjacent nonzero-overhang predicate is **IN SCOPE**, not a zero-only deferred
runtime feature. It requires SeamPlacer.cpp1083-1092 raw unsupported_dist and
place_seam1550-1563 chosen-index handoff through the existing plan/ordered loop/
nearest emission context. The clamped comparator penalty cannot supply this
value. Deferring this predicate while enabling conditional scarves would invent
wrong behavior; introducing a positive-threshold bypass is equally unacceptable.

## Non-negotiable decisions

- Source activation/short-circuit order, cyclic arm walks, f32 lengths and strict
  signed-angle/unsupported-distance comparisons are the contract. Angle0 is a
  legitimate Native input, not a production shortcut. No epsilon/threshold
  invention or blanket conditional-enable/disable fallback.
- AABBTreeLines.hpp300-352 and ExPolygon.hpp175-193 own lower-layer signed f64
  distance and empty-tree +infinity. Reuse existing f64 tree/outside APIs, build
  once per lower-layer batch. This is a distinct raw field, NOT permission to
  change old overhang/embedding scores, candidate identity or chosen positions.
- Optional per-object-layer vector/selected-loop scalar express raw-field
  lifetime. Frozen no-raft data-index0 has print ID0 and cannot read Native's
  uninitialized field. Do not conflate these IDs generally (GCode.hpp671,
  PrintObjectSlice.cpp57 raft offset); any admitted reachable unassigned-field
  consumer means STOP/report, NOT zero/default/new guard or scarf disable.
  Keep chosen candidate index BEFORE fitted/inner projection; no final-point
  nearest reconstruction. Do not lose metadata through a shortcut or invent
  another planner to recreate it.
- Configured role `get_abs_value(nozzle)` width, including source auto0 value,
  is distinct from calculated emission-flow width and candidate external width.
  Keep region/default precedence and mixed f32/f64 arithmetic source-faithful.
- The original conditional0/1 inputs are frozen discovery, not relabeled formal
  API/seeded RED. Additional nonzero dependency needs Native-ACTIVE versus
  Native-INACTIVE single-threshold pair qualification, fixed budget/no retries.
  Failure means STOP/report, not input search or a zero-only feature.
- Permanent RED freezes all five cases and completes BOTH API/CLI calls and
  whole comparisons before final assertions in BOTH modes. No missing output
  as equality, no filter/tolerance/clock change, no conditional expected-error
  PASS. Native first-layer/role/activity witnesses are hard prerequisites.
- Geometry/scoring prerequisite outside the read-only source data handoff or
  residual whole mismatch means STOP. Scope does not absorb painting, new
  random/nearest arithmetic, scarf construction, travel, M73 or processor work.
- Platform-neutral byte API, browser WASM and all desktop Tier1; private helpers
  split before400LOC. CPU15 targeted only; coordinator owns full/workspace/
  integration. RED integrates only with separately reviewed GREEN.
