# SPEC: BBL pressure-advance command precedence

Base: `a567f40f43831019fb4646a481e311c1017fdb0f`.
The coordinator authorized docs SPEC and public RED with one combined review
request. Hold before GREEN; no FULL run is authorized.

## Source owner and architectural constraints

`OrcaSlicer/src/libslic3r/GCodeWriter.cpp:370-392::set_pressure_advance` first
suppresses negative PA, then checks `m_is_bbl_printers` before any flavor.
BBL emits `M900 K... L1000 M10 ; Override pressure advance value`, formatted
with four significant digits. Only non-BBL writers enter the Klipper, RRF,
Repetier and generic command-selection chain. The fixed-PA caller is in
`GCode.cpp:8048-8053`.

Rust destination: `gcode_emit/pressure_advance.rs::command`. The current
negative guard is correct, but the BBL guard follows the Klipper, RRF and
Repetier arms. After RED approval, move ONLY that existing BBL arm before the
flavor-specific arms. Preserve every command string, `format_pa`, negative
handling, append/enable logic and reset/adaptive state. No new formatter,
helper, tag predicate, command builder or public configuration behavior.

The current emitter is a compatibility shell for this cited upstream writer
operation; the rewrite is not an independently designed PA pipeline. Included:
branch precedence only. Deferred: other public flavor witnesses, numeric
precision, signed zero, adaptive PA, reset scheduling, bool/int admission and
firmware-policy validation. No W1 auto_lift/M73, W3 tokenizer/fan/zero-E or
W6 value.rs/coInts edits. This is a directed BBL-only slice, not W6's non-BBL
stock-preset survey. No masks or processor compensation.

## Admitted BBL-only saved-project witness

All cases use the existing GUI-generated
`tests/parity/bbl-layer-object-labels/one-object.3mf`, retain BBL printer identity,
disable the tower and set `default_jerk=9`, `travel_acceleration=6000`, and
`pressure_advance=0.02`. Only the saved flavor or PA enable flag varies; no
geometry, template, temperature or fan changes are made.

The BBL/Repetier target is an admitted cross-family saved configuration,
explicitly accepted as in scope. This is not a claim that factory Bambu hardware
runs Repetier or a recommended firmware setup. Do not manufacture writer state.

| Case | Flavor / PA | SHA256 |
|---|---|---|
| Stock BBL control | marlin / enabled | 1d56a96dfa6bb2d86ef8a2d29be12ee0ea395e25d96172ee9a07a87f70456b14 |
| BBL/Repetier target | repetier / enabled | f90235bc55fec8361f0ebe8ecf7b78239229e6dca766d7a647d794c55a295693 |
| Disabled BBL/Repetier control | repetier / disabled | 92dbe0333f559c08a88f125018bf719dfcf2d2588b1cff98e7b059a634f58204 |

The target first differs at normalized byte 66908, line 1302:
Native `M900 K0.02 L1000 M10 ; Override pressure advance value` versus
Ares `M233 X0.02 Y0.02 ; Override pressure advance value`.
The complete target diff contains ONLY that PA line; line counts and all M73
records match. Both controls match complete streams. All six fresh Native
slices replay identically and all Native/CLI exits are zero. This discovery
is not public API, seeded, RED or FULL qualification.

Evidence: `${EVIDENCE_ROOT}/w4-bbl-pressure-advance-precedence/`,
including `target-diff-audit.json`. No non-BBL stock presets were explored.

## RED seam and failure modes, before implementation

Freeze only the three approved archives. Add direct calls to the existing
`native_stream_case` Native/API/CLI seam. Fresh Native replay qualifies first.
Callbacks read BBL identity, PA enable flag and PA value from saved settings,
then inspect Native override-command records. Enabled BBL cases must use the
input-derived M900 K value and BBL L1000/M10 suffix regardless of flavor. The
disabled case must contain no generated override record. Match the distinctive
override comment, not unrelated custom-template M900 commands. No hardcoded
Native coordinates, counts or output goldens; the selected PA value has fewer
than four significant digits and ordinary input text is sufficient.

Then compare complete Native/API/CLI streams in official and seeded modes
using the existing generator-line normalization only. Include timing, headers,
configuration and footers; require raw API==CLI. Callback observations are not
filters. Commit no Native outputs. Expected RED: BBL/Repetier target fails on
the selected command; stock BBL and disabled BBL/Repetier controls pass; replay
and input-derived callbacks qualify. Submit a combined SPEC/RED review request
and hold before production changes.

Failure modes: flavor-first instead of BBL-first selection; accidentally
enabling disabled PA; changing the existing formatter or negative guard;
mutating reset/adaptive state; altering non-BBL commands; confusing custom PA
commands with generated overrides; or hiding downstream output. Do not add
unapproved flavor, signed-zero, rounding or adaptive variants to this RED.

After RED approval: only move the existing BBL match arm, with tests and inputs
unchanged. CPU17/PIN17, one build job and one parity thread; fmt, workspace
all-target clippy and the same three directed cases in official then seeded
mode. Preserve normal registration separators, root registries and the 400-line
Rust file limit. Submit GREEN review request and hold. No FULL, merge, push or
stash.
