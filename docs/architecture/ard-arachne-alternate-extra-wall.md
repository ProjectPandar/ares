# ARD: Arachne alternate extra wall count ownership

Status: accepted.

The owning algorithm is OrcaSlicer `PerimeterGenerator.cpp::process_arachne`,
2131–2144 and 2163–2175, with regeneration consumers at 2224–2244.
Rust owns it inside `project_slice/perimeters/arachne/{config,walls}.rs`.

Non-negotiable decisions:

- Preserve the upstream per-surface count and integer sparse-density predicate.
  Apply its increment before bottom/top single-wall overrides and regeneration.
- Preserve effective per-record spiral dispatch; do not invent a project-wide
  Arachne fallback or select the first region for all records.
- Replace only the alternate-extra-wall rejection with its upstream behavior.
  Fuzzy and overhang changes remain separate slices.
- Reuse the existing Arachne generator and typed wall/infill consumers.
  The separate public wall-count helper is compatibility scaffolding outside
  this slice and does not define the project count contract.
- Core stays platform-neutral for WASM, Windows, macOS and Linux. Native and
  filesystem evidence gathering live only in E2E adapters/external artifacts.
- Establish public Native official/seeded admission and RED before production;
  keep strict output equality, fixtures and assertions unchanged through GREEN.

See [SPEC](arachne-alternate-extra-wall-spec.md) for scope, failure inventory,
controls, deferred behavior and resource/evidence gates.
