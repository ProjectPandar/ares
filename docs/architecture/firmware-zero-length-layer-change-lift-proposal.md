# Proposal: firmware zero-length layer-change lift parity

Status: **DOCS-ONLY PROPOSAL / REVIEW-REQUEST**. New worktree
`/home/indexyz/ares-worktrees/ares-worker1-travel-plate-discovery`, branch
`worker1/travel-plate-discovery-spec`, base exact main
`8f6ce1660ccfaf95a6888b60c258047183d19ff0`. No production/test changes or local
fix probe. Upstream source pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`, read
from the existing pinned Orca checkout, not copied into the new worktree.

Scope is travel / retract-before-layer-change ONLY. No brim/skirt/sequence,
perimeter/arachne/seam feature work. One gap found; discovery stops here.

## Frozen accepted witness and two whole-output PASS controls

Evidence `/tmp/ares-w1-fw-layer-discovery-8f6ce1660/discovery-report.json`;
complete input/member hashes in `inputs/manifest.json`, motion attribution in
`motion-attribution.json`. Base input-only committed fixture
`tests/parity/gcode-flavor-admission/marlin2-wipe0.3mf`, SHA256
`d0d89d6d9bab58cc0738cc558bbc5d92b766b4cb4980055defa4aca2afd42071`.
Geometry/placement/model members unchanged: one overhang model, ByLayer,
Marlin2, one filament, no brim/skirt/support/raft/tower. All cases fix
use_firmware_retraction=1, wipe=0, retract_when_changing_layer=1,
Normal Lift, shared minimum travel100mm. That activation suppresses ordinary
short-XY travel retractions so the mandatory LAYER-CHANGE retract/lift is
observed independently. No perimeter/seam settings changed.

| Case | configured retract length / hop | SHA256 |
| --- | --- | --- |
| fw-zero-hop TARGET | 0 / .4 | `453bebb895051c0be2511acc252c14ce1bcb63e54bec11d523800d34b4bc0a0b` |
| fw-positive-hop CONTROL | 4 / .4 | `157d05c6b9d764282077935410fdba5a1cf0b88459405124857c75baf5525bb4` |
| fw-zero-no-hop CONTROL | 0 / 0 | `8732d9cdb516841c8bad35878a1f63f7e9287ec733ac1c56738a0faaa95d43f8` |

Only project_settings.config differs among cases: each control changes exactly
ONE field from target. Input-only ZIPs contain no G-code. Frozen before Native
runs. All Native runs first, fresh serial official/seeded roots/caches, canonical
references explicitly passed to Native AND public launches, actual sampled
Native/API/CLI descendants/threads CPU18. Exact source clean8f6ce1660 throughout.

BOTH modes give Native CLI0, public API success, actual CLI0, raw API==CLI for
ALL cases. Two controls each PASS the COMPLETE unchanged ordered Native/API/CLI
comparison. Target FAILS the COMPLETE comparison in both modes identically:

- First byte1168 / line42,column9: Native `M73 P3 R20`, Ares `M73 P3 R19`.
- Normalized expected205487 / actual201116 bytes; no output line was filtered.
- Physical motion inventory: first different motion ordinal92. Native line155
  `G1 Z.8 F9000 ; normal lift Z`, then line156
  `G1 X114.025 Y114.325 Z.8 ; move to first perimeter point`, then line157
  `G1 Z.4 ; reset Z after contouring`. Ares line154 instead travels directly
  `G1 X114.025 Y114.325 Z.4 F9000 ; move to first perimeter point`.
- All75 model layers print in both outputs; G10 counts76 in both. Native has
  **75 normal lifts**, Ares **1**: first-layer lift is present, the later74
  layer-change lifts disappear. Positive model extrusion remains active.

Motion inventory is READ-ONLY attribution, NOT a filtered parity comparator or
passing surrogate. The original M73 failure remains authoritative. This is an
actual lift/path omission, not evidence for estimator repair. No separate timing
port or M73 normalization is proposed; future whole-output residuals still STOP.

## Owning upstream rewrite boundary and exact missing Rust consumer

- `GCode.cpp:5685-5718::change_layer` invokes `retract()` on
  retract_when_changing_layer && will_move_z, before silently advancing writer Z.
- `GCode.cpp:7652,7678-7704::retract`: `needs_lift` is toolchange OR configured
  length>0 OR **use_firmware_retraction**. In this non-toolchange slice, firmware
  mode therefore lifts even when configured length is0. It calls lazy_lift.
- `GCodeWriter.cpp:1026-1046::_retract` explicitly substitutes fake length1 for
  firmware retraction because configured length may be0; emits G10/G11.
  `:626-648::lazy_lift` owns the pending-lift state consumed by the next travel.
- Existing Rust `motion/travel.rs::effective_retraction_length` already returns1
  for firmware mode. `motion.rs::retract_before_layer` uses it, explaining the
  single correct first-layer lift. `motion/travel/lift.rs::schedule_at` already
  accepts firmware mode with configured length0.
- The exact omitted predicate is
  `crates/ares-core/src/project_slice/gcode_emit/layers/boundary.rs:189-197`:
  pre-Z `layer_retract_pending` capture additionally requires RAW
  `state.options.retraction_length > 0.0`, ignoring firmware mode. It prevents
  `advance_for_layer` from calling the EXISTING `motion::defer_layer_change_lift`
  (`boundary.rs:53-59`) for later layers, despite successful firmware retraction.

Requested slice: port upstream `needs_lift` into this existing layer-change
capture/deferral consumer for firmware zero length. Reuse existing pending-lift,
writer-Z timing, lift-band/enforcement and normal-travel consumers. Expected
production boundary is that predicate in boundary.rs; no new scheduler/state,
no Ares-owned pipeline, no fallback/API/parser widening, no geometry/seam,
estimator/exporter or general retraction-policy changes. Existing scaffolding
is only the source compatibility shell whose missing firmware condition is being
replaced. Preserve all non-firmware and first-layer behavior.

## Requested next barrier, not implementation authorization

Approve frozen family/source boundary for test-only Native-first public RED:
Native successful output,75 layers/positive model extrusion, G10/G11 activity
and75 normal lifts for target, controls active as configured, then success/raw
API==CLI and complete `golden::compare_ordered_bytes_generator_only` equality.
Target must genuinely FAIL on clean baseline; both controls must whole-output
PASS. Freeze input-only SHA/member manifests; never commit Native outputs.
Independent RED approval BEFORE production GREEN. Current discovery did NOT
modify code or use a temporary correction to claim clean GREEN.

GREEN must pass the complete three-case comparison BOTH modes, controls' raw
API/CLI RED→GREEN stable, related travel/lift/retract/layer-change/firmware
families BOTH modes, fmt/strict clippy/WASM, exact clean hash, CPU18/jobs2/threads2,
changed Rust files<=400 LOC. Any residual divergence means STOP/report rather
than adjacent work. Deferred: other lift modes/flavors, new templates, support,
raft, multi-filament/toolchange and broader firmware behavior. No claims about
brim/skirt/sequence/perimeter/arachne/seam parity from this witness. No full or
workspace test runs, rebase of the old A/B branch, merge or push.
