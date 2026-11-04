# SPEC: BBL jerk Z/E tail after flavor selection

Docs SPEC and public RED; no GREEN or FULL in this stage.

## Source boundary and non-negotiable decisions

`OrcaSlicer/src/libslic3r/GCodeWriter.cpp:263-305::set_jerk_xy` first selects the
Klipper, Repetier or ordinary M205 prefix. The BBL check at lines 298-299 is
outside all those flavor branches. It appends the raw Z/E cache values using
`std::setprecision(2)`. The optional ` ; adjust jerk` comment follows that tail,
then a newline. The preceding upstream TODO acknowledges the gate placement;
this slice ports the literal current structure, not a speculative TODO fix.

Rust destination: `gcode_emit/motion/jerk.rs::set`. Its BBL tail is currently
inside only the ordinary M205 arm. After RED, move the existing tail
outside the flavor match, with flavor prefix first, optional BBL Z/E tail next,
then the upstream comment and newline. Reuse the existing
`format::ostream_with_precision(value, 2)` calls. No new formatting algorithm,
command-builder abstraction, cache mutation, flavor/model predicate, or global
emission redesign. Preserve the existing prefix formatting and clamp behavior.

The options and emitter scaffolding are compatibility shells for this upstream
writer operation. The rewrite goal is source parity, not an Ares pipeline.
Included: the placement of the existing BBL suffix and source-ordered final
comment/newline. Deferred: other comment policies, PA precedence, raw headers,
model/tag inference, firmware validation, acceleration, travel and time code.
Do not touch auto_lift/M73, tokenizer/fan/zero-E or value.rs/coInts files.

This is independent of the accepted Repetier XY-clamp bounded null. Repetier
X/Y caches remain zero under the flavor gate, while upstream initializes the
raw Z/E caches unconditionally. No cache changes are included here.

## Admitted saved configuration

The target combines a saved BBL printer-model identity with Repetier flavor.
Native accepts and slices this archive. Native-admitted archives are in
scope, even for this cross-family case.
There is no claim that factory Bambu hardware runs Repetier, or that this is a
recommended printer profile. Do not manufacture writer state or override model
admission to create the witness.

BBL cases use the existing GUI-generated
`tests/parity/bbl-layer-object-labels/one-object.3mf`; the genuine non-BBL control
uses `tests/parity/orca_cli_ender3.3mf`. All selected cases disable the tower and
set `default_jerk=9` and `travel_acceleration=6000`. No geometry or template edits.
The BBL target changes only `gcode_flavor` from marlin to repetier relative to
the BBL control. The genuine Ender3 control uses Repetier and must not get a tail.

| Case | SHA256 |
|---|---|
| Stock BBL control | 6cb522350420360807f04e3e920a71c71286a92583f2d513a8def2eec5b2c975 |
| BBL/Repetier target | 92dbe0333f559c08a88f125018bf719dfcf2d2588b1cff98e7b059a634f58204 |
| Ender3/Repetier control | bf34d2e7ccfc2e33b1311cab464d1f57f7883da5165a4f643da309d98df38254 |

The target's first normalized difference is byte 67234: Native emits
`M207 X9 Z3 E2.5`, Ares emits `M207 X9`. Its complete diff contains exactly
299 M207 commands missing the Z/E suffix. Line counts and all M73 records match.
Stock BBL and Ender3/Repetier controls match over complete streams. All six fresh
Native slices for these three inputs are replay-stable and all CLI exits are
zero. Discovery is not public API, seeded, RED or FULL qualification.

Earlier travel=10000 probes exposed additional M202 differences. They remain
external evidence and are explicitly deferred. The common saved travel=6000
request isolates the tail without filtering output or changing travel code.
No travel=10000 archive belongs in this RED. A renamed-model BBL archive was
not a clean control; use the genuine Ender3 archive instead.

## RED seam and failure modes, before production work

Freeze only the three discovery archives. Add direct calls to the existing
`native_stream_case` Native/API/CLI seam. Fresh Native replay qualifies first.
Callbacks read the input printer identity, flavor and Z/E values, then inspect
Native body jerk commands for the matching prefix and BBL tail. Expectations
must come from saved values, not hardcoded Native coordinates, counts or output
goldens. The selected Z/E inputs already have at most two significant digits.
The non-BBL callback must establish that M207 commands lack Z/E words.

Then compare complete Native/API/CLI streams in official and seeded modes using
the unchanged generator-line normalization only. Keep all M73, headers,
configuration and footers; require raw API==CLI. Observations are not filters.
No Native reference outputs may be committed. Expected RED: BBL/Repetier target
fails at the omitted tail; stock BBL and Ender3/Repetier controls pass; Native
replay and callbacks qualify. RED precedes production changes.

Failure modes: a suffix gated only by M205; accidental tails on non-BBL commands;
changes to the existing two-digit formatter, prefix/clamps or cache state;
comments appearing before the tail; and masking unrelated travel/time output.
Only the three clean cases are added; Klipper-specific public witnesses
and additional comment variants are separate slices.

After RED: minimal source-placement GREEN, tests and inputs frozen; fmt, workspace all-target
clippy and the same three cases in official then seeded mode. Preserve normal
group separators, root registries and the 400-line Rust file limit.
