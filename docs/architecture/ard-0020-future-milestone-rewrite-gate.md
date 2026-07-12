# ARD-0020: Future milestone rewrite gate

## Status
Accepted

## Context
Ares must become a Rust rewrite of OrcaSlicer's slicing and visualization-neutral G-code libraries. Earlier milestones introduced Ares-owned pipeline scaffolding to make the first STL-to-G-code path testable, but continuing that style would create a new slicer instead of porting OrcaSlicer's `libslic3r` and `libvgcode` architecture.

## Decision
Every future milestone must be framed as a source-cited rewrite slice of `OrcaSlicer/src/libslic3r` or `OrcaSlicer/src/libvgcode`, not as a new Ares-designed pipeline feature. The milestone unit is the upstream library boundary being ported; an Ares pipeline stage is acceptable only as a temporary compatibility shell around a named upstream concept.

Each milestone spec and plan must include:
- The exact upstream OrcaSlicer file(s), class(es), function(s), or data structure(s) being ported or mapped.
- The Rust destination boundary (`ares-core`, `ares-vgcode`, `ares-cli`, or a milestone-approved future crate) and why that boundary matches upstream ownership.
- A clear list of upstream behavior included now and upstream behavior explicitly deferred.
- A statement that the milestone does not add a new Ares-owned pipeline abstraction unless it is a temporary compatibility shell around a named upstream concept. Any slicing, G-code, configuration, or viewer-data behavior must cite the owning `libslic3r`/`libvgcode` source before Rust design begins.
- A migration note when existing Ares scaffold code is reused, renamed, or replaced by an upstream-aligned Rust boundary.

## Consequences
- Future roadmap entries after M23 must use `libslic3r`/`libvgcode` names and source boundaries in their goals and exit criteria.
- Option milestones must cite `PrintConfig.*` definitions and validation paths, and should move toward full Orca option compatibility rather than inventing independent option groups.
- G-code work must cite `GCode.*`, `GCodeWriter.*`, `GCodeProcessor.*`, or related upstream files before changing command semantics.
- Viewer/G-code visualization data work must cite `libvgcode` files and remain rendering-neutral unless a later UI/runtime milestone creates an adapter.
- Independent review must reject milestone specs/plans that primarily describe Ares pipeline growth instead of upstream rewrite progress.

## Rejected
- Add more Ares pipeline stages as the main design unit | This preserves the wrong architecture and hides parity gaps.
- Use upstream file names only as loose inspiration | The rewrite needs traceable source boundaries and deferred-scope accounting.
