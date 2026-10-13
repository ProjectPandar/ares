# C1 obsolete guard assertion retirement request

Status: explicitly approved by coordinator; only the three listed blocks and
their newly orphaned fixtures are retired in a separate test commit.
Approved SPEC49cf8a57a, public Native RED508383c69.
Evidence: `/tmp/worker3-centering-c1-red-77qm3ho2/red-evidence.json`.
Both modes: Native13/13 admitted; public API/CLI2 strict controls pass and11
centering admission failures. This is the separate request required by the SPEC.

In `project_slice/tests/raw_preflights.rs`, only the three `assert_eq!` blocks
inside `task22b_print_object_centering_gate_accepts_collapsed_xy_and_rejects_distinct_or_mismatched_groups`
expecting `unsupported("print_object_centering")` are proposed for deletion:

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
retirement is requested. An approved retirement goes in its own test commit,
separate from C1 GREEN, citing the upstream ranges and RED508383c69.
