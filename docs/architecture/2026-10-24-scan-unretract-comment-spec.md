# SPEC: ordinary scan-unretract source comment

Base: `5aa5b395011777910114562386995bf531d62954`.
The coordinator authorized docs SPEC plus tests-only RED, followed by a review
request and HOLD. GREEN is NOT authorized by this stage. No FULL run.

## Ownership and source boundary

The coordinator checked all frozen W1 (`3b85a2e84`), W5, W6 and W4 tips: none
changed scan.rs. W1 is HOLD with no reservation. W4 is explicitly assigned ONLY
the ordinary non-firmware E-restore comment in that file. This does not authorize
changing adjacent lift/travel/origin, E gates, state or firmware behavior.

`OrcaSlicer/src/libslic3r/GCode.cpp:4782-4789` runs retract, M976/M400 and
unretract during BBL first-layer inspection.
`GCodeWriter.cpp:1056-1081::unretract`, in its non-firmware nonzero-delta branch,
calls `w.emit_comment(full_gcode_comment, " ; unretract")`.
`GCodeWriter.hpp:266-269::GCodeFormatter::emit_comment` prepends `" ; "`.
The resulting literal suffix is `" ;  ; unretract"`. Preserve both separators
and spaces; do not normalize an apparently redundant upstream comment.

Rust destination: `gcode_emit/motion/scan.rs::scan_first_layer_block`, only the
existing non-firmware E-restore format block near lines 126-134. It currently
emits E/F and a newline without this comment. The existing local reconstruction
is a compatibility shell around the cited writer operation, not an independently
designed scan pipeline.

If a later GREEN is authorized, append the literal suffix under the existing
gcode_comments option in that block only. Preserve the E/F strings, current
delta guard, firmware branch, feedrate assignment and all state transitions.
No shared helper, generic unretract refactor or travel-call replacement.

Included: that writer comment's admission and exact bytes. Deferred: all lift,
travel/origin, pending hop, absolute-E/reset and zero-E behavior, firmware
retraction, other unretract callers, power-loss lifecycle, fan, tokenizer, brim,
value.rs and time estimates. No masks or processor compensation.

## Inventory and minimal real project witness

The preceding inventory covers 47 writer functions and one formatter helper,
with approved-test evidence separated from repository presence and discovery.
The power-loss Disable probe itself had matching power-loss commands. Its
commented BBL case revealed the unrelated scan-unretract omission; this slice
is NOT a power-loss fix.

All selected inputs derive from the existing GUI-generated
`tests/parity/bbl-layer-object-labels/one-object.3mf`, stock flavor, tower disabled.
Only the saved comment or scan flag changes; no geometry, templates, power-loss
overrides, retraction settings or firmware changes are needed.

| Case | Change | SHA256 |
|---|---|---|
| Stock control | none | 748222a3819ea8ad5c3a346bac61f65149dd753bfc12f468b51d8b9e7cf96c0c |
| Scan comment target | gcode_comments=1 | f75d406432bc3bcb659f68705ab24faef2e345c1d7687141ef73bb256483a071 |
| Scan-disabled control | target plus scan_first_layer=0 | 4725806a943799978387ac2f1809e9dbbab5bfa7cbf9224db654cb7ae97bec31 |

Each parent/variant edge changes one real saved key. The target first differs
at normalized byte 71178, line 1435, immediately after M976 S1 P1 / M400 P100:
Native `G1 E.8 F1800 ;  ; unretract`, Ares `G1 E.8 F1800`.
The COMPLETE target diff is this one nonzero-E comment. Line counts and every
M73 record match. Both controls match complete streams, all six fresh Native
slices replay identically and all CLI exits are zero. Discovery does not qualify
public API, seeded, RED or FULL tests.

Evidence and complete source inventory:
`/home/indexyz/ares-evidence/w4-writer-function-inventory/`, especially
scan-comment-witness/ and its witness-audit.json.

## Tests-only RED contract, before production work

Freeze only the three approved archives and add direct calls to existing
native_stream_case. Fresh Native replay qualifies first. Callbacks read the
saved BBL, scan and comment settings, then locate the actual inspection block.
With scan enabled, qualify M976 followed by M400 and the subsequent ordinary
nonzero E-only restore, checking the source comment suffix when comments are
on and its absence when off. With scan disabled, qualify absence of the scan
marker rather than pretending to observe the missing block. Do not hardcode
Native E/F values, coordinates or output counts as reference goldens.

Compare complete Native/API/CLI streams in official and seeded modes, retaining
timing, headers, configuration and footers with only established generator-line
normalization. Require raw API==CLI. Callbacks are observations, not masks;
commit no Native outputs. Expected RED: target fails at the missing comment;
stock and scan-disabled controls pass; all replay and Native callback
observations qualify. Production must remain unchanged versus the base.

Failure modes: dropping the comment, cleaning up its doubled separator, adding
it when comments are disabled, observing an unrelated unretract outside the
scan block, confusing firmware/zero-E with the nonzero ordinary restore, or
refactoring lift/state while fixing text. These failure modes are recorded
before any production implementation.

CPU17 taskset/PIN17, one build job and one parity thread. Run fmt/diff checks,
commit tests-only RED with required trailer, execute the three cases in both
modes and submit SPEC/RED review request. Preserve normal group separators,
root registries and the 400-line Rust file limit. HOLD before GREEN; no FULL,
merge, push, stash, new consumer abstraction or owner-scope expansion.
