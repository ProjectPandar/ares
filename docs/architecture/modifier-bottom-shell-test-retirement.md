# Requested exact retirement: modifier bottom-shell rejection row

Status: approval requested; no existing assertion or test changed.
SPEC 7f0717e07, public RED 190a6a5e145b7213b2b1832022b37e6d0e08eb69.
Base main73e8e712e after top merge; bottom SPEC rebase range-diff1/1 '='.
Native official/seeded each17 admissions, public matrix each3 strict controls
and14 expected bottom-shell Unsupported failures:
/tmp/worker3-modifier-bottom-red-3_4fdaxu/evidence.json.
Fresh Native local-effect evidence:
/tmp/worker3-modifier-bottom-native-e_ef9axn/native-local-effect.json.

PrintObject.cpp:3664-3710 applies modifier config to the parent;
PrintApply.cpp:1032-1088 creates child modifier regions;
PrintObject.cpp:2252-2280/3973-3995 consumes region bottom counts.
The obsolete rejection conflicts with this approved bottom-only slice.

Request: remove only the bottom_shell_layers tuple from the cases array in
crates/ares-core/src/project/tests/effective_config/candidates.rs,
usage_affecting_modifier_keys_are_rejected_in_fixed_order.
Array entries7 ->6. Keep all six filament selector tuples exactly,
keep test name, shared assert_eq, loop, helper and imports unchanged. No unused
import cleanup is needed. No other assertion, fixture or production change.

If approved, put this exact one-row deletion in its own test commit citing
these upstream lines and RED hash; keep it separate from GREEN. Before REVIEW,
run the retained fixed-order core test plus the approved directed/gates matrix.
