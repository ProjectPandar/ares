# ARD: Match configured perimeter owners before claiming algorithm parity

Status: proposed, docs-only [SPEC](perimeter-input-dispatch-spec.md) review required.

## Source and destination

Orca8500fcdcc PrintConfig.hpp:1020/1217 types object generator and region sequence;
PrintConfig.cpp::PrintConfigDef::init_fff_params:2158-2182/7155-7167 defines them.
LayerRegion.cpp::make_perimeters:86-142 passes configs into PerimeterGenerator and
selects its implementation at138-141. Rust destination is perimeters/context.rs::
prepare_object_contexts:59-144, especially107-111, using existing retained typed
options/record. This is an upstream handoff rewrite slice, not a new Ares pipeline.

## Non-negotiable decisions

- Different effective wall generators cannot establish same-input geometry/order
  parity. Input-byte identity alone is insufficient when one entry loads profiles
  and overrides while another uses embedded project settings.
- The first motion travel toward a different loop identifies upstream perimeter
  producer/first-entity ownership; it does not establish a route or seam bug.
- A qualifying byte-equal main control and matched effective config/provenance
  must precede RED. v2 failed control is excluded from positive evidence, not
  deleted/substituted inside a comparison. No retry or new budget is implied.
- Keep complete ordered G-code and raw API/CLI equality. Motion projection is
  diagnosis only: retain M73, arcs, extrusion, statistics and footer in acceptance.
- Existing config/dispatch scaffold remains a compatibility shell. Do not add
  an Ares-specific dispatch policy, new public API, guard, fallback or diagnostic I/O.
- Deferred downstream algorithms cannot be repaired under this proposal. Evidence
  crossing this source boundary requires a new SPEC before implementation.

## Consequence

Seam v2 closes NO-RED; two unmerged failing harness test commits remain external
history, not merge candidates. The proposed next source boundary still needs
SPEC, fixture/control and budget rulings before new tests or execution.
