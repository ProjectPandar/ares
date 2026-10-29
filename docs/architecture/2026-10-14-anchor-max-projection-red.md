# D-M: admitted local maximum-projection public RED

Approved SPEC 90c43e984, on main 5a9687ac0. Production is unchanged.
Source owner: Fill.cpp::group_fills:978-989 narrows/project lengths and takes
min(anchor, maximum); Layer::generate_sparse_infill_polylines_for_anchoring
:1477-1482 passes them to FillParams. PrintObject.cpp::
region_config_from_model_volume:3664-3709 merges local options. The selected
Ares transaction rejection is anchor_projection.rs:28-30; D-A is deferred.

## Native admission before writing RED

Driver: /tmp/worker2-dm-preflight.py. Existing solid-ceiling exported3MF,
ordinary global anchor=400%/maximum=20, sparse rectilinear20%, local object
or model-part maximum=20/1e39/1e39%. Each invocation gets fresh cache/root;
non-metadata ZIP members are checked unchanged. Inputs are finite f64;
both extreme carriers narrow to non-finite f32, as upstream projects them.

Both official/seeded each accepted all6 inputs, exit0 and complete footer,
with52 positive Internal Bridge and378 Sparse infill extrusion moves each.
Fresh repeat6/6 per mode used identical inputs and produced identical full
streams except the canonical generator timestamp; no other line was removed.
Thus24/24 Native invocations completed before public RED execution.

| Mode | First root | Fresh repeat root |
|---|---|---|
| official | /tmp/worker2-dm-preflight-official.rd_qrkgo | /tmp/worker2-dm-preflight-official.y_t_lq2k |
| seeded | /tmp/worker2-dm-preflight-seeded.8z45iqs7 | /tmp/worker2-dm-preflight-seeded.e9mtqlui |

Additional already-supported local max=0 controls completed in both modes
and scopes; sparse paths differ from max=20, confirming local-key consumption.
Roots: /tmp/worker2-dm-preflight-zero-official.nv26ctmy and
/tmp/worker2-dm-preflight-zero-seeded.mw11py_p.
These controls do not add a new input policy or production scope.

## Separate public RED

New runner/compatible_regions/bridge_anchor_max_projection.rs reuses the existing solid-ceiling
fixture helper with local solid=monotonic equal to the global pattern.
Only local maximum varies. Four tests cover object/part × absolute/percent;
each first compares max=20, then its extreme. Each case saves existing
Native/API comparison artifacts, exact local XML, member hashes, role counts
and CLI output before the final parity assertions. No Native output is in Git.

Driver: /tmp/worker2-dm-red-final-leg.sh; outer taskset CPU13/14 respectively,
jobs2/threads2 and independent fresh root/cache/target. Filter:
test(runner::compatible_regions::solid_pattern::anchor_max_projection::).

| Mode | Root | Public nextest result | Case evidence |
|---|---|---|---|
| official | /tmp/worker2-dm-red-final-official.XfnQ0V | 4expectedFAIL, exit100 | 4controls PASS;4exact max-guard errors |
| seeded | /tmp/worker2-dm-red-final-seeded.PtCKq3 | 4expectedFAIL, exit100 | 4controls PASS;4exact max-guard errors |

All16 public Native cases completed with52 bridge/378 sparse moves and footer.
Each ordinary control passes whole Native/API/CLI comparisons and raw API/CLI
equality. Each extreme fails at line83 status==PASS: actual ARES_ERROR,
"unsupported project feature: infill_anchor_max". CLI exits1 with the same key.
No setup/native/role failure is counted as RED. Offline reread verified exact
input hashes, unchanged members and ordinary global anchors in all16 cases.
External aggregate: /tmp/worker2-dm-red-final-evidence.json; original failures kept.

No guard deletion or Fill/schema/comparator change is included. RED assertions
remain the intended GREEN contract. Await RED review/repair authorization.


## Engineering check and preserved initial attempt

Final fmt and strict test-target clippy exit0, CPU13/14 jobs2:
- cargo fmt --all -- --check
- cargo clippy -p ares-cli --test orca_parity -- -D warnings

Logs: /tmp/worker2-dm-red-final-gates.Gtmvzk. All touched files <=400 lines.
Initial public roots /tmp/worker2-dm-red-official.aDxjGr and
/tmp/worker2-dm-red-seeded.6VzAdG already showed the same four guard failures,
but clippy caught duplicate_mod from including the existing fixture twice.
The new tests were moved below the original fixture module and import it.
All four tests/helpers/assertions are byte-identical from the first test
attribute onward. Final runtime legs above were rebuilt with fresh target,
cache and root; the original strict failure and original runtime artifacts
remain external. No lint suppression was added. No full/workspace or push.
