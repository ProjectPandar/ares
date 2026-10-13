# C1 checkpoint before W6 exclusive full

Approved SPEC49cf8a57a. Public Native RED508383c69 is complete:
`/tmp/worker3-centering-c1-red-77qm3ho2/red-evidence.json`.
Each mode Native13/13 admitted; single/XY controls strict PASS, eleven cases
fail public API with print_object_centering and CLI exit1. Production was
unchanged when the RED commit ran and its evidence was reported before edits.

Retirements are independent approved test commits3c5c0e105 and6df499622.
The first removes exactly three old refusal assertions and orphaned fixtures;
the second removes only the obsolete centering-vs-sharing precedence row.
Both positive controls and every other assertion macro remain untouched.

C1 production is WIP, not GREEN/review/full-ready. It ports group center offsets
through raw slicing, support projections, visibility/seam choices, emitted
instance shifts, skirt/footprint/brim and auto-lift consumers. The internal group
identity remains the existing `(source_object_index, transform_index)`.
No public load/selection API contract changes; C2/assemble is untouched.
Rotation extraction follows Geometry.cpp420-426/471-474/754-777 using the fixed
real 3x3 Eigen JacobiSVD and signed axis-angle operation; its MPL attribution is
retained in rotation.rs. No atan2-column-only shortcut or new feature guard.

Before later WIP edits, workspace/all-target cargo check exited0 on CPU12:
`/tmp/worker3-centering-c1-green-check-v2.log`.
Strict clippy failed with two nesting and one range-loop lint in rotation.rs:
`/tmp/worker3-centering-c1-green-strict.log`. Those are corrected in the WIP
without suppression, but the correction has not been verified. Initial fmt
exited0; subsequent edits need fmt again. WASM and public GREEN are not run.
All own cargo/nextest/Native sessions ended before20:30; IDLE-FOR-FULL recorded.

Touched pre-existing layers.rs exceeded400LOC. Mechanical refactor bfa38231f
moves the existing285-line drive loop verbatim to an included lexical macro;
this retains append's bindings, command order, returns and branches. It is
separate from the production change to the brim collection. Parent/child are
both under400LOC. Exact reconstruction SHA evidence:
`/tmp/worker3-centering-c1-layers-mechanical-split.json`.
Macro expansion/format/lint/runtime checks are pending; no acceptance claim.

After explicit W6 END: CPU12 fmt + strict + WASM, C1 thirteen-case official
and seeded strict GREEN in fresh Native cache/artifact roots, affected existing
controls and retirement unit controls. Any deterministic mismatch must be fixed
from its owning source boundary without comparator/fixture/assertion relaxation.
Then commit actual GREEN, code review, explicit FULL-GO, exclusive canonical
full, exact-hash review and coordinator merge. C2 starts only after C1 closes.
