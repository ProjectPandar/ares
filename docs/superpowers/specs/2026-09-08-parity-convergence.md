# Spec: full-output Orca parity convergence

## Strict oracle follow-up

The [strict-output oracle slice](../plans/2026-09-08-strict-output-oracle.md)
now replaces partial semantic acceptance with shared ordered byte comparison,
normalizing only independently shaped generator identity/timestamp lines.
The bounded wave/evidence below describes the preceding fail-closed replay wave,
not the current comparator. Unchanged classic is now truthfully rejected with
paired evidence; no prior partial PASS becomes a strict producer success.
Inventory/default/domain/effective-config/all-plate coverage and legacy producer
provenance remain unverified. ARD-0023 is unchanged.

## Unchanged user goal

All 1,001 supported printer presets, with their real default process/filament
profiles, must produce the full OrcaSlicer artifacts through the public project
bytes/CLI seam. Every legal Boolean and Enum value and range min/max/seeded
interior must be applied to its upstream owner and verified in effective
exported configuration. Baseline-only plans, rejected cases, substitutions,
missing evidence and skipped offline tests do not satisfy coverage.
Full output includes deposited and travel/motion commands in order, timing,
M73, config, statistics and every generated plate/artifact. ARD-0023's allowed
generator identity/timestamp normalization is not permission to discard other
output. No source pinning, comparator weakening, new tolerance, or legacy
fallback is permitted.

## Source-owned boundaries

The accepted ARD-0023 and four-crate architecture remain authoritative.
`libslic3r/Format/bbs_3mf.cpp::_BBS_3MF_Importer` owns project interpretation;
`PrintConfig.*`, `Preset.cpp` and `PrintApply.cpp` own domains/configuration;
`Print::export_gcode`, `GCode::do_export`, `GCodeWriter` and
`GCodeProcessor::{process_file,run_post_process}` own emitted output.
Their Rust runtime destination remains `ares-core`, with no filesystem or
terminal behavior; `ares-cli` tests own external reference execution, replay and
artifact I/O. Viewer data stays rendering-neutral in `ares-vgcode`; browser
WASM calls the same core API. This wave validates these rewrites, not a new
Ares pipeline. Existing semantic comparison is only a temporary diagnostic
shell and cannot certify whole-output equality.

## Bounded wave: truthful replay and complete paired bytes

The existing `orca_parity_replay_sweep` continues to replay cached `.3mf` and
`<stem>/plate_1.gcode` inputs without invoking Orca. Explicit requests must fail
on empty/unreadable inventory, missing/unreadable bytes, slicing error,
divergence, or artifact/report write failure. With the replay environment unset
it may return an explicitly labelled offline skip, never an executed parity run.

Artifacts go only under caller-selected external `ARES_PARITY_ARTIFACT_ROOT`;
no tracked report or fixed temporary diagnostic paths. Each invocation/case
must preserve complete available project, reference and Ares output before
comparison, with SHA-256 content identities, case label, comparator mode,
executable identity/provenance metadata and useful errors when no Ares output
exists. Legacy reference producer provenance is unknown; hashes attest saved
bytes, not their origin. Supplied provenance must not be silently promoted to
verified provenance. Build revision claims must be distinguished from observed
checkout state; executable hashes identify the actual running build.

The current verdict must say **partial semantic evidence**, never full-output
parity. Successful harness regression tests may assert a real divergent replay
exits nonzero; that does not repair the underlying slicing difference.

## Current non-green facts and exclusions

Scout measurements at 431fb361: 1,001 printer labels, historical 702 PASS /
276 DIVERGENT / 23 ARES_ERROR; these are not fresh successful executions.
The comparator ignores XY travel, timing in this entrypoint, M73, config and
some statistics/order. Option generation contains 302 baseline-only cases;
legal-domain completeness and effective exports remain unverified. Live runner,
vendor enumeration, default-profile substitutions, cache provenance, plate
coverage and many runtime debug hooks remain outside this bounded correction.
No historical pass count or focused semantic success closes any of those gaps.

## Acceptance and review

Current-source red regressions must demonstrate the reachable false greens.
After correction, fail-closed cases must fail and a successful partial comparison
must retain verifiable paired bytes. Fresh independent Orca execution, command
exit codes, output paths, build identities, Nextest, fmt, Clippy and diff/LOC
checks accompany the commit. Docs land only with tested harness code.
Medium read-only scouts and high isolated writers operate under coordinator
scope authority. Final acceptance requires independent six-axis review of
source fidelity, domain/default coverage, complete-output oracle sensitivity,
artifact/build reproducibility, regression correctness, and architecture/
portability/maintainability. Findings require bounded fixes, original+changed
case re-verification and independent review again until all axes are green.
This wave is not overall completion or publication approval.

## LOC debt final state (#156)

24 files split under the 400-line rule over turns #138–#155 (fan_mover,
seam_placement, gyroid, adaptive hooks/octree, avoid-crossing boundary,
motion, brim, pressure_equalizer emitter/line, shortest_path chain,
layers, gcode, fill_entities, processor motion, options build, cooling,
spiral vase, overhang, start_travel ×3). One file remains over:
`gcode_emit/motion/path/start_travel.rs` at 437 (37 over) after the
restate (110) and retract (44) extractions. The third extraction
(the travel emission if-chain) was attempted and REVERTED — the chain's
brace structure is entangled with the enclosing `if needs_travel`
block (the extracted region nets depth −1; the if-chain's first branch
starts before `if state.spiral_vase`), and a byte-identical golden
takes precedence over forcing the split. A future split must carry the
needs_travel guard into the extracted function or extract from a
balanced sub-boundary.

## start_travel.rs body-extraction attempt #2 also reverted (#157)

A second, differently-bounded extraction (the whole balanced
`if needs_travel {}` block into `start_travel/body.rs` with the guard
converted to a conditional call) was built and wired, but the block's
internal `} else if layer_change_travel` chain continuation does not
survive the wrapper-brace removal — the routed-travel section's final
`}` closes the wrong level and the parser rejects the `else`. Both the
guard-included and guard-stripped variants hit the same wall, which
pins the real structure: the if-chain's branches and the surrounding
`needs_travel` body interleave at TWO brace levels (the chain is not a
contiguous region of the block; routed-travel statements live between
branch closes). Reverted to the committed 437-line file (golden green,
tree clean). The remaining 37 lines stay as documented debt.

## CRITICAL FINDING (#158): the pipeline reorder diverges the ksr semantic contract

The first full-workspace test audit (post-LOC-debt) surfaced a REAL
regression hidden since the #113 reorder: `ksr_fdmtest_v4 ::
project_matches_orca_242_semantically` — the byte contract against the
LIVE Orca 2.4.2 output — now fails at the timing lines:
`; model printing time: 1h 43m 49s` → `1h 43m 52s` (+3s) and the file
shrank 6339122 → 6321961 bytes (−17 KB of F lines). The byte-snapshot
golden test passes only because its snapshot was regenerated during the
reorder work; the semantic test pins the ORACLE bytes and exposes the
divergence.

Interpretation: under the OLD order (equalizer after cooling) ares was
byte-identical to Orca on this fixture — empirical proof that Orca's
EFFECTIVE pipeline computes the cooling estimate on RAW text (or the
M73/timing estimate runs on the pre-equalizer stream). The source
reading (GCode.cpp:3752 equalizer→cooling) must be one of several
pipeline assemblies; the ksr fixture exercises the assembly where the
estimate does not see equalized F values.

ACTION (next session, priority 1): revert the finish_layer order to
cooling→equalizer (restoring the ksr semantic contract), re-run the
full sweep, and re-baseline. The printer sweep improved 871→875 under
the reorder — both effects must be reconciled: likely the correct model
is estimate-on-raw + equalizer-last, and the +4 sweep printers should
be re-checked for whether their gains survive (if they regress, the
timing estimator input needs the raw-text path specifically).
Also: 8 stale tests flagged by the audit (volumetric_rate_smoothing ×2,
slope_lowers, timelapse filament_map, replay rejection evidence, 2×
slice_stl writes, arachne prisms) — triage each after the order revert.

## CORRECTION to the #158 finding (#159): NOT a reorder regression

Forensics: the `; printing object <name> id:<N> copy <M>` emission is a
HARDCODED STUB since the July squash commit (5977b66b):
`gcode_object_labels.rs` emits `ares-object-0 id:0` — the file has zero
commits after the squash. The ksr semantic test (added 2026-08-26,
0aa6662a) has therefore NEVER passed against the fixture reference
(which carries Orca's `id:13965068898260364096`); it was not part of
the default test set and its failure predates the equalizer work. The
#113 reorder is correct per BOTH upstream pipeline assemblies
(GCode.cpp:3752 AND :3850-3857 — identical order in the two-path and
layer-loop variants). The order-revert is CANCELLED.

Real buckets in the 9 workspace failures:
1. Object-label identity stub (semantic test + arachne prisms test):
   the `id:N` is `PrintObject::get_id()` — a global sequential ObjectID
   counter (ObjectID.hpp:86, `++s_last_id`) assigned at model load;
   the huge value for ksr implies deterministic id derivation through
   the 3mf load path (needs the loader's object-creation order ported).
   The `copy M` matches (0). The object NAME is already correct.
2. Removed VolumetricRateSmoothing unit tests (×2) + slope_lowers: the
   smoothing was deleted in #104 (#d07553fa); the slope test's SPEED
   markers reflect pre-equalizer values by design under the new order
   — these three pin removed behavior.
3. slice_stl writes ×2 / replay rejection evidence / timelapse
   filament_map: triage pending.
