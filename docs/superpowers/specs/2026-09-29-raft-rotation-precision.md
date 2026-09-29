# Raft support fill rotation precision

## Upstream boundary

Port the output rotation of `FillSupportBase::fill_surface`
(`libslic3r/Fill/FillRectilinear.cpp:3627-3633`) into
`ares-core/src/project_slice/raft/fills.rs`. The upstream rotation angle
is a float. Its unqualified `cos(float)` and `sin(float)` calls return
float values which are then promoted to double for `Point::rotate`.

## Observed failure

The real Orca `raft_layers=max` output differs on a wipe segment:
`E-.20595` versus Ares `E-.20594`. A temporary round-call probe found
Orca's endpoint Y102.712134 where Ares has Y102.712133.
The upstream unrounded local Y is -7287866.473413825; Ares's direct
double trig gives -7287866.600352854. Float trig reproduces the oracle
value exactly.

## Included and deferred

Match the float trig result and the existing double coordinate arithmetic
at this rotation boundary. Preserve upstream point rounding. Add a
regression on output geometry and re-run complete G-code comparisons.
Do not change global geometry rotation, comparator tolerance, E formatting,
or wipe arithmetic. Other fillers and missing support generators remain
separate upstream rewrite slices; existing scaffolding is temporary where
those upstream behaviors are absent.

## Acceptance

- Output geometry regression fails before the fix.
- `raft_layers` min/max/seeded complete G-code matches real Orca.
- Recheck the other raft option domains against real Orca.
- Remove all temporary instrumentation.
- Rustfmt, Clippy, Nextest, and independent six-aspect review pass.
