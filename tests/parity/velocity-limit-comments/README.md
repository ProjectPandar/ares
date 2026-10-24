# Merged velocity-limit comments parity inputs

Source `GCodeWriter.cpp:311-350`, suffix341-346. SPEC
`docs/architecture/2026-10-24-velocity-limit-comments-spec.md`.
StockEnder3/toweroff, common Klipper/defaultaccel500/travel700/initial500.
Only `gcode_comments` differs (target1/control0), GUI Tab2753, PrintConfig3965.
No geometry/template/temperature/lift/jerk/PA edits.

- klipper-acceleration-comments.3mf SHA256
  ce5dc71dc0a8e39933614cf0d7f752071a5fd3728cdc88a666bfb673560b938b
- klipper-acceleration-control.3mf SHA256
  644dbe2a3e522c3f835ff854c5388ef546820a02cf31eeab442deb8fee3553e9

Two direct existing `native_stream_case::check` calls, no new helper. Fresh
Native double replay precedes callback: actual velocity commands must carry
ACCEL_TO_DECEL and ACCEL; verbose case must reach the exact upstream merged
suffix; control requires no semicolon in those commands. No numerical Native
reference constants/counts/coordinates. Observations do not filter acceptance:
complete API/CLI streams compared unmasked using existing generator-only seam.
No Native outputs/goldens committed. RED target expected missing suffix, control
full PASS. Frozen tests/inputs unchanged during GREEN; cache/clamp/tolerance and
empty-command admission remain source-review invariants. No FULL.
