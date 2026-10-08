# filament_map / filament_ramming_parameters legal coverage cases

Status: spec for coordinator review (case authoring per ARD-0025). No
production change. Base: main ec17814fd. The option-coverage sweep marked
these three keys INCOMPLETE ("required legal cases not all compared")
because their inventory plans authored zero cases; this slice authors the
missing legal-domain cases for two of them and records the third.

## filament_map (PrintConfig.cpp:2489-2494, coInts, default {1})

Legal domain: one extruder index per filament (1-based), sized to the
filament count; meaningful on multi-extruder machines with
`filament_map_mode = Manual`. Consumer: config export bookkeeping
(`GCode.cpp:5633` is_nil) plus extruder assignment during tool-change
generation.

Cases (fixture: the H2D dual-extruder Manual-map setup already proven by
`filament_variant_zero_fill.rs` — H2D 0.4 + 2 filaments + High Temp Plate):

1. identity map `["1","1"]` — baseline (covered today by zero-fill; keep).
2. cross-map `["2","1"]` — filament 1 prints on extruder 2: the tool-change
   and purge sequence move to the other extruder.
3. cross-map `["1","2"]` — the mirror assignment.
Boundary (record-only): out-of-range index (`["3","1"]`) — upstream
validation rejects; classify REJECTED per ARD-0025.

## filament_ramming_parameters (PrintConfig.cpp:2833-2840, coStrings)

Legal domain: one ramming-profile string per filament; the profile is the
RammingDialog two-row `|`-separated numeric string (default in
`:2836-2840`); consumed by tool-change ramming generation.

Cases (fixture: H2D dual-extruder, two distinct filaments so each carries
its own profile; tool change between them exercises the ramming sequence):

1. default profile on both filaments (baseline export).
2. per-filament distinct shortened profiles (e.g. `"100 100 1 1|0.05 1 1"`)
   — the exported config lines and the tool-change ramming block differ per
   filament.
Boundary (record-only): empty string (upstream keeps the string; the
ramming sequence degenerates) — classify per observed native behavior.

## filament_self_index (PrintConfig.cpp:5432, record only)

`static_owner: unowned`, `effective_projections: []`: the option is config
export bookkeeping with no consumption projection — no behavioral case to
author. Recorded as INCOMPLETE-by-design (authored none) unless a consumer
appears.

## Case form: dedicated runner tests, not plan cases

The sweep's plan schema is a single-filament/single-extruder fixture
(Ender-3 default filament); neither key's full legal domain fits it:

- `filament_map` cross-map cases need a dual-extruder machine
  (`filament_map` assigns per-filament extruders).
- `filament_ramming_parameters` is consumed only through the SEMM ramming
  path — `WipeTower2.cpp:1378-1390` (`if (m_semm)` parses the profile into
  `ramming_speed` per 0.25 s piece; the BBL `WipeTower.cpp:1593` copy is
  `#if 0`-disabled), which fires at multi-filament tool changes with the
  wipe tower — a flow the plan schema cannot express.

Both cases are therefore implemented as dedicated runner E2E tests
(the `filament_variant_zero_fill.rs` pattern):

1. `filament_map` cross-map on the H2D Manual dual-extruder fixture
   (identity case already covered there): `['2','1']` and `['1','2']` move
   the tool-change/purge to the other extruder; out-of-range `['3','1']`
   recorded as: the native reference SIGSEGVs (`ToolOrdering.cpp:990`
   indexes `extruder_count` past its length) — recorded as the upstream
   fact, never used as a comparison reference — while Ares rejects the
   same project through the typed option boundary
   (`resolve_filament_indices` bounds-checks the mapped slot →
   `InvalidInput("invalid Orca option filament_map")`).
2. `filament_ramming_parameters` on a single-extruder multi-filament SEMM
   fixture: **未达成，待续**. The fixture reached the native export after
   two fixes (below), but never reached a tool change: the export case
   inputs (`CaseInputs`, `runner.rs:175`; `OrcaRunner::export_case` →
   native `--export-3mf`) carry no object→filament assignment, and the
   native export composes every object onto filament 1 — a single-object
   fixture prints exactly one tool. On a 1-extruder machine the
   `filament_map` slot lever cannot help either (a mapped slot beyond the
   single extruder is rejected by the same boundary). Unblock: add an
   object→filament assignment to the export case inputs (or a multi-part
   model convention), then restore the test (two objects on filaments
   1/2, `enable_prime_tower`, `single_extruder_multi_material` from the
   Ender-3 profile) and assert the ramming block changes with the
   per-filament profile.
   Fixture learnings to reuse: (a) the native export rejects a project
   whose `filament_is_support` list does not match the filament count
   ("filament_is_support's count 1 not equal to filament_colour's size
   2"); (b) a machine profile without a tower position needs one inside
   the bed (the Ender-3 default lands outside the 220×220 bed and the
   export rejects with an empty `-5`).

If the ramming case is DIVERGENT/ARES_ERROR: record the first difference
and report for assignment (the wipe tower region belongs to worker2) —
not fixed in this slice.

## Acceptance

Per ARD-0025: every selected domain PASS, complete-output byte comparison
per case (generator-only normalization), REJECTED classifications preserved
with their native stderr, no comparator relaxation.
