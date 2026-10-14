# Requested exact retirement: modifier top-shell rejection row

Status: approval requested; no assertion or test changed.
SPEC 9b7ef7b71, public RED ad004150139475b852dbcc53cc84938cfd9ccd87.
Native official/seeded each17 admissions, public matrix each3 strict controls
and14 expected top-shell Unsupported failures:
/tmp/worker3-modifier-top-red-knep0e3j/evidence.json.

Upstream PrintObject.cpp:3664-3710 applies modifier config to the parent;
PrintApply.cpp:1032-1088 creates child modifier regions;
PrintObject.cpp:2223-2251/3973-3995 consumes region top counts.
Therefore the old rejection is incompatible with the approved rewrite.

Request: remove only the top_shell_layers tuple from the cases array in
crates/ares-core/src/project/tests/effective_config/candidates.rs,
usage_affecting_modifier_keys_are_rejected_in_fixed_order (currently line205).
Array entries8 ->7. Keep bottom and all six filament selector tuples exactly,
keep test name, shared assert_eq, loop, helper and imports unchanged. No unused
import cleanup is needed. No other assertion, fixture or production change.

If approved, put this exact one-row deletion in its own test commit citing
these upstream lines and RED hash; keep it separate from GREEN.
