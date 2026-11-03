# ARD: source-owned numeric Option-domain metadata coverage

## Boundary and non-negotiable decisions

This is a test-inventory rewrite slice, not an Ares slicing feature. The owning
upstream structures are `libslic3r/Config.hpp::ConfigOptionDef` (2475–2479:
float min/max default to `-FLT_MAX`/`FLT_MAX`) and individual definitions in
`libslic3r/PrintConfig.cpp` at upstream8500fcdcc. These implicit defaults mean
that upstream declares no boundary on that side; they are not sliceable endpoint
samples. Record each absent side as `unbounded`, never generate implicit
±FLT_MAX, and never substitute INT_MIN/INT_MAX or a vector nil sentinel as an
invented boundary. Generated nullable overrides
copy base min/max/enum metadata at7288–7302. `max_literal` (Config.hpp2481–2482)
is a GUI warning threshold, not a replacement legal maximum.

Rust destination: `crates/ares-cli/tests/orca_parity/option_coverage/{bounds,
numeric_domains,domains}.rs`. No changes to `ares-core`, slicing, validation,
G-code, or viewer behavior. Existing local numerical probes are temporary
sampling scaffolds; replace only their missing source-defined bound generation.
Keep bool/enum sets, width/context admission and complete-byte comparison intact.
Numerical domain completion is scoped to the endpoints of declared boundaries
plus existing interior samples, not exhaustive legal-range coverage. Unbounded
sides are counted and listed separately in the report: neither PASS nor an
INCOMPLETE coverage gap. Successful comparisons may establish completion only
of the declared-boundary/sample scope; they never certify an unbounded side.

## Retained inventory classification

The exact650-domain single-attempt matrix at5cde90c23 had224 INCOMPLETE rows,
not224 Ares failures. The immutable row source and hash-bound classification are
`${TMP_ROOT}/ares-option-numeric-224/classification.json`; execution base is integrated
M205 `d0e8b64f9`.

-196 historical numerical-omission rows:150 implicit upper sides;39 both sides;
  four implicit lower sides; resolution's implicit FLT_MAX upper side;
  min_bead_width's reported expression; and the named coInt retraction samples.
  This is retained historical accounting, not196 missing finite-boundary gaps.
  Reclassify absent source bounds as unbounded sides and list them per key;
  do not generate their extrema or credit them as PASS/INCOMPLETE. The revised
  side counts must come from the corrected source-definition inventory.
-12 baseline/non-injected gaps: filament_map, filament_ramming_parameters,
  filament_self_index, master_extruder_id, nozzle_flush_dataset,
  other_layers_print_sequence_nums, physical_extruder_map, post_process,
  print_compatible_printers, print_extruder_id, printer_extruder_id,
  wrapping_exclude_area. Record-only, not silently injected here.
-16 context gaps:13 authored interior/shape probes whose raw schema endpoints
  were not admitted by existing validation, two single-nozzle width contexts,
  and the rejected timelapse_type case. Record-only, no fixture/context repair.

Classification reflects the retained omission evidence; it does not guarantee
that every newly generated schema endpoint is a feasible slicing input. Native
rejection, crash, timeout, application mismatch or unsafe/UB-adjacent behavior
must stay separately recorded, never become numerical parity PASS.

## Included and deferred behavior

Include only declared min/max endpoints that are representable by the upstream
storage type, along with existing interior samples. Storage limits and implicit
finite defaults are not source-declared slicing boundaries and cannot supply
missing endpoint samples. Named coInt probes must not be invented into an enum:
the definition still owns the type. Generated nullable options must read their
actual base definition, preserving which sides are declared versus unbounded.
Unknown bound expressions remain unresolved, not silently unbounded. Replace the
old full-range completion contract with declared endpoints + existing interiors;
retain complete-byte comparisons and require every case in that scope to finish
and compare successfully. Rejection, missing cases or TIMEOUT cannot complete it.

`resolution` has declared min0 and no declared maximum (PrintConfig.cpp5172);
its implicit FLT_MAX upper side is unbounded. Keep existing interior samples,
exercise the declared minimum, and do not generate FLT_MAX or a storage ceiling.

`min_bead_width` is `coPercent`, min0, implicit maxFLT_MAX atPrintConfig.cpp7275.
The old definition-block scanner leaks the following dynamic `add_nullable`
loop into this option and sees that loop's `def->max = it_opt->second.max` as
an unsupported bound expression. Fix only this source-definition boundary,
not the option or production parser. Its corrected upper side is unbounded;
only its declared min0 and existing interior samples are comparison obligations.

Defer context/baseline coverage, SLA-to-FFF application mismatch, Native crashes,
Top Only reference sensitivity and worker3's Sermoon C1. Never relax comparators,
ignore M73/footer lines, add skips, pad malformed vectors, emulate upstream UB,
or fix new runtime divergences in this task.

## Pre-implementation failure inventory and verification

Metadata generation can fail by confusing an absent assignment with an unknown
expression; generating implicit FLT_MAX or invented integer storage endpoints;
swallowing the next dynamic definition; reading a generated nullable
key as unrelated neighboring source; mistaking max_literal for max; emitting
unrepresentable int/nil values; treating named integer samples as an enum;
omitting percent or literal branches; crediting an unbounded side as PASS or
INCOMPLETE; or marking context/baseline probes complete. Timeout handling can fail
by leaving owned children running, losing partial evidence, retrying a timed-out
case, or counting unfinished output as a comparison.
First write isolated inventory/definition tests for these failure modes before
changing generation. These tests verify the upstream planning boundary without
running a slicer; existing E2E byte comparisons remain behavioral authority.

Then run one directed scan of the196 historical numerical keys under the revised
sample obligations, with explicit identified seeded Native, `ARES_ORCA_RUNS=1`
and fresh cache. Preserve complete original input/config/output/M73/footer and
stage/producer receipts for completed cases. The bounded scan driver is external
test orchestration, not a production timeout API. Set a300-second wall-clock
deadline per case, covering input
preparation/export, Native slicing and Ares processing/comparison, rather than
only the outer scan. On expiry terminate that case's owned process group and
children, retain commands, inputs, partial stdout/stderr/output, elapsed time and
stage/producer receipts, and record the case as `TIMEOUT`. No automatic retry,
PASS credit, comparison of unfinished output, or silent omission; continue to the
next case only after the owned processes are stopped. No retry-until-success.
Record/cluster newly exposed differences and first failures without production
repairs. Separate generated cases, comparisons, declared-scope domain completion,
unbounded side counts/per-key listings, and TIMEOUT outcomes. Preserve nonzero
scan exit/non-GREEN evidence when required cases do not compare successfully.
