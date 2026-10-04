# Timelapse state and smooth Type2 selective rewrite

Base: main `56e4da53`, after approved Adaptive PA integration. Worktree remains
`/tmp/ares-pi-tower`; the next feature branch is
`pi/smooth-type2-timelapse-tower` to preserve the already published PA review
history without force-pushing it.

Owning sources and destination decisions are in ARD-0105. Included work:

1. Reproduce stock BBL H2S / P1P / P1S first-travel Z-restatement differences
   at the public complete-output seam; port the applicable `_extrude` /
   `GCodeWriter` state transition into existing Rust motion emission.
2. Port Type2 smooth-timelapse planning/writer/geometry and integration from
   the historical branch selectively: current layer/tool ownership, occupancy,
   placeholders, first tower, final unload, statistics and Adaptive PA reset.
3. Verify source activation and full output, including tower-disabled controls,
   no-skirt bounds, thin layers, PA, offsets and unloading configurations.

Deferred/unclaimed: arbitrary Type1 and multi-material tower parity, exhaustive
printer/option coverage, hardware operation and native Windows/macOS runtime
evidence. These exclusions do not permit guards or silent omission of active
source behavior. Broaden the upstream port where an exercised dependency needs
it rather than weakening the fixture or comparison.

The BBL state repair may be reviewed as an independently green prerequisite;
it must not be represented as completion of the Type2 tower port.
