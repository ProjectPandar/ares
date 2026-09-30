# Locked Zag width activation

Follow ARD-0044's upstream flow boundary and Rust destination. The existing
option sweep injects width values but leaves the owning Locked Zag feature
disabled. Repair that test-input dependency without changing the width
domain or treating sampled interior values as complete range coverage.

Acceptance:

- Both skin and skeleton width probes activate Locked Zag at 40% density.
- Exported projects carry the requested width, pattern and density.
- The 0.3 mm and 150% probes produce positive sparse extrusion at 0.3 and
  0.6 mm respectively; the resulting extrusion commands differ.
- Both complete outputs match original Orca for each key/value pair.
- Sensitive live RED/GREEN, targeted sweep, workspace Nextest, fmt/Clippy,
  Tier 1 compilation and independent six-axis review precede commit/push.
- Preserve INCOMPLETE range status and all unrelated pending work.

No new Ares-owned pipeline or source-layout pinning tests are introduced.
