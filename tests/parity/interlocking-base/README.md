# Base generator Native inputs

Only input-only projects for the current producer slice are included.
The test source freezes each archive SHA. Native is mandatory and runs twice
before the public byte API and CLI; API/CLI raw equality and both complete
ordered generator-only comparisons are unchanged.

On current main `8acb19cf7`, both modes show six PASS controls/admission checks
and two genuine producer REDs: default two-extruder beam1 and boundary-zero
beam1. Same-extruder beam1 already passes on main and is only a no-op control.
Evidence: `/home/indexyz/ares-evidence/w6-clean-generator-red-{official,seeded}/run.log`.

The unresolved WIP inputs/tests for three-region beam1, odd-kernel, orthogonal
wipe, auto-width gap and percent-width gap are deliberately not in this chain;
they remain on `worker6/interlocking-wip`. This is an explicit deferred scope,
not a skip, ignore, altered comparator or claim that those cases are fixed.
No Native output or production WIP implementation/history is copied here.
