# Ordinary scan-unretract comment inputs

Source: `GCode.cpp:4782-4789` calls the scan restore;
`GCodeWriter.cpp:1056-1081` passes `" ; unretract"` to emit_comment, whose prefix
in `GCodeWriter.hpp:266-269` produces the literal `" ;  ; unretract"` suffix.
SPEC: `docs/architecture/2026-10-24-scan-unretract-comment-spec.md`.
Base: `5aa5b3950`. This tests-only RED does not change production code.

The coordinator confirmed scan.rs is unchanged in all frozen worker tips, W1
is HOLD without reservation, and W4 may later change only this ordinary
non-firmware E-restore comment. GREEN is not authorized by this stage. No lift,
travel/origin, state, zero-E, firmware, fan, brim or configuration code changes.

All inputs derive from the existing GUI-generated BBL one-object project,
stock flavor and tower disabled. The target changes only gcode_comments=1;
the second control changes only scan_first_layer=0 relative to that target.
No power-loss override, geometry, template or retraction setting changes.

| Input | SHA256 |
|---|---|
| stock-control.3mf | 748222a3819ea8ad5c3a346bac61f65149dd753bfc12f468b51d8b9e7cf96c0c |
| scan-unretract-comments.3mf | f75d406432bc3bcb659f68705ab24faef2e345c1d7687141ef73bb256483a071 |
| scan-disabled-comments-control.3mf | 4725806a943799978387ac2f1809e9dbbab5bfa7cbf9224db654cb7ae97bec31 |

Three direct calls use the existing native_stream_case seam. Fresh Native replay
qualifies first. Callbacks read the saved BBL/scan/comment flags and confirm
ordinary relative extrusion. Enabled scans must contain the inspection marker,
wait, and subsequent positive nonzero E-only restore before acceleration resets.
That restore must preserve both source comment separators when comments are on
and contain no comment when off. The scan-disabled control must lack the marker.
No Native E/F values, coordinates or counts are hardcoded as output goldens.

Complete Native/API/CLI comparison follows with established generator-line
normalization only. M73, headers, configuration and footers remain included;
raw API==CLI is required. Callbacks are observations, not filters. Expected RED:
target fails at byte 71178 (`G1 E.8 F1800 ;  ; unretract` versus the same command
without the suffix), stock and scan-disabled controls pass. Discovery found
only this one comment line and equal M73 records; public runs qualify it afresh.
No Native output references are committed.

Failure modes were written before implementation: omitted comment, normalized
double separator, comment emitted when disabled, wrong restore context,
firmware/zero-E confusion or a lift/state refactor hidden in a text fix.

Freeze inputs/tests before any later GREEN. CPU17 taskset/PIN17, jobs=1 and
threads=1; no FULL. Submit docs SPEC plus tests-only RED for review and HOLD.
