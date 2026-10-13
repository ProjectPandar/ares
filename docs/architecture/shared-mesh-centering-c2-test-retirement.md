# C2 obsolete guard assertions: retirement request

Status: coordinator approved the exact list on2026-10-14; applied in a
separate retirement test commit before GREEN.
Public RED commit: `4334347e441cfd60fbf26afd90590f7f1486d23d`.
Both modes23tests:7PASS/16FAIL; each25 Native outputs admitted0.
Evidence: `/tmp/worker3-centering-c2-red-matrix-izu0d3pw/red-evidence.json`.
First implicit3mm RED: `/tmp/worker3-centering-c2-red-first-6_cfjgdm/red-evidence.json`.

`bbs_3mf.cpp:4931-4974,5025-5052` admits explicit/default/-1 sharing and
restores the reused init shift. `Model.cpp:1295-1307` deliberately does not
recenter a shared mesh. A presence bool or rejection in the slicing stage is
the obsolete scaffold replaced by approved SPEC5d0f9f2d9.

Requested changes, only in a separate retirement test commit:

1. Delete `raw_preflights::task22b_mesh_shared_presence_and_repeated_resource_keys_are_rejected_request_wide`
   in full: its two Unsupported assertions, two presence-bool assertions,
   and their exclusive fixtures. The private bool is being replaced, so the
   two bool assertions describe that retired representation rather than the
   admitted loader semantics. Public Native RED covers explicit presence,
   repeated leaves and cross-file keys.
2. In `task22b_shared_mesh_gate_ignores_empty_occurrences_and_precedes_dense_or_coordinate_errors`,
   remove only the dense/huge conflict and unprinted-sharing rejection blocks
   (two Unsupported assertions) and their exclusive setup. Preserve the
   accepted empty-occurrence fixture and its retained_facts assertion exactly;
   rename the test to describe the surviving positive control.
3. In `raw_support::preflight_order_scenarios`, remove only `sharing_later`
   and the sharing precedence row; array3→2. Range/dense rows and the generic
   consuming assertion remain unchanged. Delete `unique_unprinted_shared_request`
   and its sole import after item2 makes them unused.
4. Remove the retired private bool setter and final bool argument from
   `tests/support::project_volume` and mechanically update its call arguments
   across the existing test fixtures. Remove no assertion outside items1–3;
   do not retain an ignored compatibility argument or an unused bool setter.

Every other assertion, including both C1 centering controls, empty-occurrence
facts, range/dense precedence and all strict Native public comparisons stays.
No replacement expected value, relaxed comparison, ignore or suppression.
Retirement and GREEN commits stay separate and are reviewed together.
