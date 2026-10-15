# SPEC request: loaded bridge-flow strict-positive admission

Status: **amended SPEC-REVIEW-REQUEST, docs only; four reproduced admission divergences**.
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

Exact Rust destination includes the earlier typed bridge-option loading boundary:
bridges.rs::parse_bridge_options (:158-178), flow_multiplier (:237 onward),
and its use of options::parsing::parse_positive_number_or_string. The discovered
bridge_flow=0 hits this positive-only parser before loaded validation, producing
InvalidInput/CLI1. Internal flow0 currently reaches acceptance on the same
public entry. Inspect actual typed/resolved loading provenance rather than
assuming both keys follow identical paths. Included is the minimum bridge-key
adapter/constraint correction so both zero values survive loading; do NOT change
the shared positive parser globally or admit unrelated options' zero values.

PrintConfig.cpp:1327-1337 and :1354-1365 declare coFloat with min=0,max=2
for both keys. Zero is a valid representable loaded configuration value. The
strict-positive runtime invariant is enforced later at :10426-10434, before
slicing proceeds. Thus accepting zero *into configuration state* is not accepting
zero into slicing and not a fallback. Preserve the exact rejected value until
validation can return all applicable keys in one map. Native evidence below
proves this stage distinction; a typed InvalidInput is not equivalent rejection.

The second Rust destination remains options/validation/loaded.rs normalized
validation chain and the existing flow.rs::
SliceOptions::validate_skirt_and_bridge_flow_options immediate adapter. The
latter already implements both <=0 predicates and float error formatting.
Reuse or minimally split the bridge-only part; **skirt_height is excluded** and
must not be activated as a side effect. No independent allowlist/pipeline rule.

## Completed discovery and unified four-case public RED

Runtime evidence /tmp/ares-bridge-flow-discovery-XfjEaw/discovery-report.json,
exact approved SPEC HEAD66b05815b on approved00dc33bd8 implementation. Fresh
serial official/seeded canonical ALL env, CPU18 actual Native/public threads,
clean HEAD and input SHA receipts. No production edits during discovery.

| bridge/internal | Native both modes | Ares validated API / CLI both modes |
|---|---|---|
| 1/1 | slices successfully | full comparator PASS, raw API=CLI |
| 0/1 | -18, bridge_flow: invalid value 0.000000 | InvalidInput invalid Orca option bridge_flow / exit1 |
| 1/0 | -18, internal_bridge_flow: invalid value 0.000000 | succeeds, CLI0, raw API=CLI,216028-byte output |
| 0/0 | -18, both exact lines sorted bridge then internal | same early InvalidInput / exit1, loses aggregate map |

All three non-control cases are genuine admission-protocol divergences, including
the two earlier typed rejections. No reduction to internal-only RED is allowed.
Native rejects before G-code emission. No borrowed G-code comparison applies to
rejected inputs. Freeze all four existing discovery pure3MFs and manifest SHAs;
no new inputs/geometry or additional discovery required for this public RED.

One public RED suite: OrcaRunner::from_env/stages requiredNative, explicitly
assert actual Native -18/shell238 and exact parameter-error key map for each of
the three positives (both sorted keys for0/0). Then require public validatedAPI
InvalidValuesInProject containing the **exact complete map**, realCLI238 and
matching complete map diagnostics with no output G-code. Baseline Ares acceptance
or InvalidInput/CLI1 must FAIL, never count as equivalent rejection. The1/1
control requires positive Native XY/E, effective flow1/1, publicAPI/CLI success,
raw equality and unchanged generator-only whole-output comparator. Expected
**three admission FAIL + one full-stream PASS per mode**. Do not relax errors,
ignore a key, substitute any rejection, or change comparator/classification.
Independent exact public RED review before any loading/validation production edit.

## Original bounded discovery protocol (retained provenance)

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

Included only both bridge keys' schema-compatible typed loading (zero retained)
and strict-positive loaded validation, existing error message/key/map aggregation
and shared API/CLI invalid-values protocol. No entire aggregate activation;
extract/reuse bridge-only validator if the existing combined helper requires it.
Skirt-height behavior is expressly excluded, with no separate Native evidence.
No global parser weakening, positive-number helper change, fallback flow1,
schema bound change or pipeline division-by-zero workaround. Existing bridging
execution must not see invalid zero after validated entry's rejection.

Deferred skirt-height behavior, all other basic/clearance/filament validators,
bridging geometry/flow generation, option propagation/region overrides,
legacy/numeric parser policy beyond the two bridge keys' zero-loading constraint,
negative/NaN values, multi-error inventory outside
these two keys, flavor validation changes, firmware retraction, cooling/PA.
The original discovery's earlier-stage STOP is now resolved by this amendment:
the bridge-key typed-loading boundary is explicitly included, not an excuse to
broaden to unrelated parsing or pipeline behavior.

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
