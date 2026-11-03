# Scarf default-speed threshold — independent reachable RED

The approved spec is `gcode-scarf-default-speed-threshold.md`; upstream pin
is8500fcdccaa10b5099ac20d252af3a7c560046f1. The source owner is
`GCode.cpp:6515–6526`: default sloped Inner/Outer speed only resolves the
role-relative scarf setting and caps it at role speed. Small-loop selection
already belongs to `GCode.cpp:5808–5813`; it must not be repeated in mm here.

## Reachability after loop-only GREEN

On clean Rust maindbcd52d5a, a real public CLI slice of frozen byte input
1646ba992cc58685074abd95e009d17f85f88e190379912201d9d9f88399f4e5 was inspected
with nix-provided GDB17.2 (CPU0). No production edits, panic instrumentation,
private emitter API or Orca rebuild were used.

At logical layer1, GDB captured one ExternalPerimeter path containing the
closed axis-aligned square with coordinates±5000137 and z0. Actual complete
scaled length is40001096; caller limit is40001095.99999999, so its actual
Option is None. Actual per-member XY-mm length is40.001096 and local limit
is40.001096. `scarf_speed` returned36; `features::speed` received requested
None with Outer wall/Slope.speed36 and returned36. This proves consumption,
not just a computed value masked by positive caller speed. Ordinary small
loops remain masked, as the spec anticipated.

## Full-byte E2E RED and controls

Before changing production, the Rust public slicing/Native parity E2E ran
with CPU0/seed0, runs1, fresh per-case caches, a shared300s deadline per case,
and descendant cleanup. Compilation happened before that deadline with
CARGO_INCREMENTAL=0, not inside a single-CPU cold build.

| Case | Complete comparison | Non-first outer feedrates Native/Ares |
|---|---|---|
| scaled/mm threshold disagreement | DIVERGENT (intended RED) | 3600/7200 vs2160/7200 |
| ordinary non-small, scarf50% | PASS | 3600/7200 both |
| ordinary non-small, absolute60 | PASS | 3600/7200 both |
| ordinary non-small, scarf150% | PASS | 7200 both |
| existing positive small-loop scarf/ramp | PASS | existing1392/1584 witness unchanged |

The boundary input is exact: outer120, small30%→36, scarf50%→60,
threshold6.3663721574934415; wall/layer0.4/0.2, ramp/cooling/volumetric masking
off. Controls were frozen before their Native comparisons: only the settings
ZIP member changes threshold to6 and scarf to50%,60 or150%. Their model and
other member payloads are unchanged. Fixture hashes/provenance are in
`tests/parity/scarf-default-threshold/README.md`.

The Rust runner first difference is byte1074/line42: Native `M73 P57 R0`, Ares
`M73 P56 R0`. Earlier standalone CLI comparison reports byte1059 on the same
line because its generated-by replacement marker has a different fixed
length. Both compare all remaining bytes; no M73/statistics/footer removal
or expected-value relaxation occurs. The feedrate witness independently
identifies the scarf symptom; any unrelated behavior remains visible in the
complete assertion.

Five/five Slice receipts bind exact input and Native output hashes with
Native SHA59fb88a2…, resources286fab66…, requested and pinned CPU0/seed0;
all five Slice outcomes are fresh misses, zero timeouts. The positive-small
control additionally has a fresh Export receipt. Raw outcomes remain one
RED plus four PASS, not an overall passing test run. A preliminary standalone
single-call Native RED is also retained; it was not retried until success.

## Boundary for subsequent GREEN

Only `motion/scarf.rs`: remove its local threshold early return and unused
`loop_length` parameter from `scarf_speed`; retain build's length for slope
geometry. Resolve the role-relative scarf value and `base.min(configured)`.
Do not alter loop caller/role eligibility, ramps, geometry, flow, cooling,
precision, M73/time processing or comparisons. Keep positive caller override
behavior and all pre-existing expectations.

The machine record `gcode-scarf-default-speed-threshold-red-record.json`
binds manifests/receipts and observed live geometry. Retained local artifacts:
`${TMP_ROOT}/ares-scarf-threshold/post-loop-3d65/{reachability-observation.json,
fresh-native-first,rust-red}`; original GDB log
`/tmp/pi-unified-exec-2195-b2005a89.log`. The unused panic proposal remains
external only and did not enter any build or commit.
