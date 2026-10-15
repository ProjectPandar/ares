# SPEC request: loaded bridge-flow strict-positive admission

Status: **SPEC-REVIEW-REQUEST, docs only; no Native RED yet**.
Stacked approved00dc33bd85695f565b1356ccbe9db37ce8ba93e1 queuedstack17;
merged base3ee8f272c6d8e2db22dde9003a01afeb7201e36a.
Rebase only after coordinator broadcasts merged main, without stash.

## Upstream / Rust destination

PrintConfig.cpp:10426-10434, Slic3r::validate: bridge_flow<=0 and
internal_bridge_flow<=0 each insert their own key into error_message, with
'invalid value '+std::to_string(value). These explicit strict-positive tests
are distinct from the generic numeric min/max loop10519-10559 and option
schema declarations1327/1354. A schema-accepted zero does not imply runtime
admission. Error-map ordering/first-emplace semantics are retained.

Exact Rust destination options/validation/loaded.rs loaded normalized validation
chain and, only if necessary, the existing flow.rs::
SliceOptions::validate_skirt_and_bridge_flow_options immediate adapter. The
latter already implements both <=0 predicates and float error formatting;
aggregate.rs consumes it, but loaded.rs currently merges flavor/infill/width
and numeric ranges, not this helper. This static missing call is only a
candidate: other public entry validation may already reject equivalently.
Do not activate the entire aggregate or duplicate predicates/allowlists.
Existing scaffold is reused as a compatibility shell for the source validator,
not an independent Ares configuration policy.

## Bounded Native-first discovery after approval

Use the exact admitted Marlin2/wipe0 input from gcode-flavor-admission fixtures,
unchanged mesh/non-config bytes, PAoff, Classic/no scarf/tower. Exactly four
settings combinations, (bridge_flow,internal_bridge_flow):
(1,1) control, (0,1), (1,0), (0,0) candidate admission positives.
Both keys explicitly present, no negative/NaN/range sweep or synthetic shape.

Run fresh official+seeded Native first under current canonical ALL oracle env,
CPU18/jobs2/threads2. Record exact input SHA, Native exit/result/stdout/stderr,
matching error key(s), formatting and -18/shell238. For (0,0), require both keys
and sorted source map diagnostics, not acceptance of any one rejection.
Public validated API and realCLI follow for the same bytes even when Native
rejects: characterize admitted G-code versus exact InvalidValuesInProject map,
CLI exit/error and absence of G-code. A Native rejection plus Ares acceptance
is genuine admission divergence, not a G-code comparator result. A different
Ares error/exit must be reported as such, not relabeled equivalent success.

For admitted control require Native actual effective flow1/1 and positiveXY/E,
publicAPI/CLI admission/raw equality and existing unchanged generator-only
whole-output comparator. For rejected inputs no borrowed reference, no stream
comparison or skipped public request. If equivalent rejection already exists
for all candidates, **NO-RED STOP** with matrix; no passing-test/accounting
submission or production edit. If a live mismatch localizes to the two source
predicates/error-map consumption, report discovery before public RED.

## Included / deferred / acceptance

Included only both strict-positive bridge-flow predicates in loaded admission,
existing error message/key/map aggregation and shared API/CLI invalid-values
protocol. Reuse existing helper; if its bundled skirt_height check would expand
behavior beyond proven source parity, explicitly isolate/review the adapter
before code rather than silently authorize unrelated validation. No new guards,
fallback flow1, schema bound change or pipeline division-by-zero workaround.

Deferred skirt-height behavior, all other basic/clearance/filament validators,
bridging geometry/flow generation, option propagation/region overrides,
legacy/numeric parser policy, negative/NaN values, multi-error inventory outside
these two keys, flavor validation changes, firmware retraction, cooling/PA.
If another source stage causes mismatch, stop/request amendment, not broaden.

After discovery review: tests first, frozen pureinput SHA, required Native
OrcaRunner::from_env/stages exact rejection/error/noGcode contract for positives;
public validated API Err and CLI238/diagnostic/noGcode required. Control strict
whole-stream comparison and API=CLI unchanged. Independent public RED approval
before minimal loaded-validation wiring. GREEN same inputs/assertions, exact
Native/API/CLI admission parity, raw same-input control bytes unchanged plus
existing loaded validation controls; fmt/strict <=400LOC exact clean fullhash.
Canonical env archived/hashed at launch, fresh serial mode caches/artifacts,
actual Native/thread CPU18. One batch START then divergence/STOP only. Respect
exclusive freezes; no full/workspace/push/integration except coordinator. No Nix
GC/store deletion/rebuild. This commit is docs only, no new runtime/tests/code.
