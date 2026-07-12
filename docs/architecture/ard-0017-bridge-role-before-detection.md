# ARD-0017: Bridge role behavior before bridge detection

## Status
Superseded by `docs/architecture/ard-0018-libslic3r-libvgcode-rewrite-boundaries.md` for future milestone planning. The M18 bridge option/role behavior remains accepted, but future bridge geometry work must be planned as a Rust rewrite of `OrcaSlicer/src/libslic3r/BridgeDetector.*` and related `libslic3r` surface/support boundaries.

## Decision
Ares will add typed bridge options and downstream `PrintPathRole::Bridge` flow/speed behavior before porting bridge geometry detection.

## Context
OrcaSlicer bridge detection depends on unsupported-region geometry and direction optimization. M18 introduced the bridge role first so a later `BridgeDetector.*` port can reuse typed flow and speed semantics. Future work must not extend a custom downstream pipeline unless the boundary is justified against upstream `libslic3r` concepts.

## Consequences
- Future bridge detection must focus on porting `OrcaSlicer/src/libslic3r/BridgeDetector.*` and related upstream surface/support concepts without also changing option, extrusion, and speed plumbing.
- Current slicing output remains unchanged because M18 does not generate bridge paths.
- Internal bridge geometry, support interaction, and exact Orca bridge parity remain separate milestones.
