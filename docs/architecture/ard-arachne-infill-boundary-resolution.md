# ARD — distinguish Arachne wall-input and infill-boundary resolution

Status: proposed; [SPEC](arachne-infill-boundary-resolution-spec.md).
Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Source-owned contract

`PerimeterGenerator.hpp:135` initializes member `m_scaled_resolution` from the
configured resolution with EPSILON floor and active scale.
`PerimeterGenerator.cpp::process_arachne:2125,2148` uses a separately reduced
wall-input resolution with arc fitting and no fuzzy-skin contour.
The same function at `:2506` simplifies generated infill contours with the
**unreduced member value**.

Paired read-only GDB proves raw inner contours agree and the infill simplifier
receives identical geometry but different tolerances: Native12000 versus Rust2400.
The difference reaches Bottom-surface monotonic fill before connection, then
changes the point sequence and ultimately fitted G2 I/J. Do not repair the
fitter or connector to compensate for this earlier contract violation.

## Decisions

- Preserve two policies at their exact source call sites; do not globally change
  `surface_simplify_resolution`, wall input, fuzzy masks, or geometry helpers.
- Approved adjustment: retain the already computed `scaled_resolution` in
  `ArachneRecordConfig` (`arachne/config.rs`), and pass `config.scaled_resolution`
  from `arachne/materialize.rs`. These are the only production files allowed.
  Do not duplicate the EPSILON floor calculation or invert the reduced scalar;
  no new pipeline stage/API or moving modules is justified.
- Core logic remains portable and filesystem/terminal-free. GDB temporary probes
  are diagnostic artifacts, never committed production tooling.
- Keep the existing public None RED and strict full-stream comparator, enabled
  arcs, geometry, presets and placement. Numeric contract equality supplements
  public E2E; it does not replace it.
- ArcFitter/Circle, Fill/grid, Arachne backend, classification/interface-shell and
  centering are read-only. If one scalar correction does not produce complete
  parity, stop and seek a separate cited owner/scope instead of expanding.
- Counterbore activation and its ten rejection assertions remain separate. No
  Unsupported additions, region[0], legacy fallback or Native substitution.

## Consequence

The earlier fitter-only prerequisite is superseded, not implemented. This is a
bounded Rust rewrite alignment of an upstream argument handoff, not a pipeline
feature or tolerance tuning designed for one mesh. Complete-output GREEN and
control invariance remain unverified until the approved change is executed.
