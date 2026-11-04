# C1 obsolete guard assertion retirement

Only the three listed blocks and their newly orphaned fixtures are retired in a
separate test commit. SPEC49cf8a57a, public Native RED508383c69.
Both modes: Native13/13 admitted; public API/CLI2 strict controls pass and11
centering admission failures. This is the separate retirement required by the SPEC.

In `project_slice/tests/raw_preflights.rs`, only the three `assert_eq!` blocks
inside `task22b_print_object_centering_gate_accepts_collapsed_xy_and_rejects_distinct_or_mismatched_groups`
expecting `unsupported("print_object_centering")` are deleted:

1. The constructed resolved object with two transforms. Native accepts distinct
   rotation/scale groups in RED; PrintApply.cpp138-170 groups all printable
   transforms and PrintObject.cpp93-130 supplies their coordinate frames.
2. The constructed one-group X-scaled transform inconsistent with the source
   instance list. Public resolution cannot construct this mismatched test-helper
   request: it groups actual source instances. Source has no such centering guard.
3. Nonprintable scaled first instance followed by printable identity. Native
   `c1_nonprintable_first` admits this, using original first raw bbox
   (Model.cpp1586-1599) and printable-group members (PrintApply.cpp145-165).

Delete the third block's solely used `nonprintable_first` fixture construction.
Delete `xy_scaled` only if it becomes unused after that removal. Retain both
positive assertions in this test, its source fixture and Z-only-compensation
comment/control, and every assertion in every other test. Rename the test only
to describe its retained collapsed-XY/Z-compensation controls. No broad test
retirement is made. The retirement goes in its own test commit,
separate from C1 GREEN, citing the upstream ranges and RED508383c69.

## Precedence-row retirement

Code audit found one further old guard expectation in
`raw_support.rs::preflight_order_scenarios`: its `centering` row supplies a
shared-mesh first object and a later two-transform resolved object, expecting
`print_object_centering` to outrank `shared_mesh_centering`. With the C1 guard
removed, that ordering is no longer part of the upstream contract. C2's shared
mesh guard still exists until C2, so this row must not redefine C1 admission.

Delete only this obsolete `centering` table row and its solely used
`sharing_first`/`centering_later` fixture construction; change the return array
length4 to3. Keep range/shared/dense rows, `distinct` (still used by range), and
the consuming test's generic assertions entirely untouched. Same upstream
PrintApply.cpp138-170/PrintObject.cpp93-130 and public RED508383c69 apply.
All generic assertion macros and the other three rows remain unchanged. It is committed independently
from production.
