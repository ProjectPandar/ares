# ARD0085 — Support output prerequisite checkpoint

Status: implemented dependency slice; not ordinary-support completion.

## Boundary and non-negotiable decisions

This checkpoints completed dependencies of the still-open ARD0084 normal
support rewrite. It does not replace or shrink that task or the global
all-printer/default/all-option output requirement. ARD0084's enabled
ordinary-support test remains RED and its exit criteria remain unchanged.

Upstream owners and Rust destinations:

- `Flow.cpp:232-274` support/base/interface/first-layer flow selection into
  `project_slice/perimeters/flow/support.rs`, replacing raft's approximate
  explicit-width chain with these upstream role contracts.
- `Slicing.cpp:86-190` support-nozzle height limits, asymmetric zero-gap
  decisions and synchronized gap rounding into `project_slice/parameters.rs`,
  replacing ordinary-gap zero placeholders and machine-epsilon rounding.
- `SupportParameters.hpp:134-144` and `SupportCommon.cpp:1569-1572`
  contact-pattern/filler selection into `raft/fill_params.rs` and the
  project raft bridge; native `FillConcentric::_fill_surface_single` and
  `FillGrid::fill_surface` into `raft/fills/patterns/`.
- `FillBase.cpp:2464-2466` and its selection/T-joint/long-arch consumers
  into task-local `SlicerContext` costs and `fill/base_support/`, with one
  explicit mutable job context through actual raft/skirt callers.

The raft pipeline is a temporary compatibility caller of these native
support contracts, not a new Ares-owned support generator. Freeze only
the three source first-call costs; preserve current-call geometry metrics.
Task-local lifetime means the accepted fresh-native-process-per-input
contract, not arbitrary native long-lived or parallel process equivalence.
Keep strict complete ordered bytes excluding only the generator line,
effective-value checks, active support XY extrusion and full artifacts.

## Included evidence and limits

The witnessed first-layer-width, zero-gap and automatic-width REDs, native
memory/counterfactual diagnostics and repair sequence are retained in
ARD0084. Final native probe has eleven full PASSs, including all five
contact-pattern Enum values, no-skirt initialization and identical-input
reuse after other jobs. Each case has full input/native/Ares streams and
manifest. The cache producer remains explicitly unknown; fresh direct
native replay of the automatic-width input reproduces its canonical stream.
Modified-process diagnostics are never canonical reference evidence.

Final production workspace:7462 PASS/9 skip,264.955s; fmt/diff checks and
all-target Clippy succeed. Linux/Windows/macOS core builds and release
browser-WASM adapter build succeed. Real Chromium has six full native
output PASSs, including before/after job isolation, no skirt and
concentric/grid contact patterns. See ARD0084 for exact artifact/log paths.
These target builds are not foreign-OS runtime test claims.

Deferred, still required work includes ordinary top/bottom contacts,
projections/heights/base/interface layers, generated paths and object/support
merging. Preserve separate support height maxima and zero-gap interface
flags before those consumers are ported. Multi-object first-call ordering,
arbitrary native parallel scheduling, ironing and no-interface variants,
broader numeric/material domains and untouched printer defaults are not
certified by this checkpoint. No completed option-domain inventory is
claimed from these sampled active contexts.

## Checkpoint exit

This checkpoint covers only this dependency. Keep the pending
ordinary-support regression, spec/plan and failure evidence beside it;
do not mark ARD0084 or the overall goal complete.
