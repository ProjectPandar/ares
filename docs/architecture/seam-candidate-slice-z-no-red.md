# Seam candidate slice-Z v2 closure: NO-RED

Status: **closed NO-RED by coordinator ruling**. Approved design remains
`892a850d9d3a0f5387b2a6aac83bb1071cf8801e`; observed physical-plane handoff is
not retracted, but these v2 inputs cannot isolate a qualifying public RED.
No retry, production correction or test merge is authorized.

## Frozen attempts

- `1b9c785999f8be345f73df86df22fb2ce520eadb`: official first positive only,
  Native exit0/complete, public API/CLI footer barrier (`inherits_group`). Owner
  ruled mixed public entries a harness error, not a product finding.
- `c7aab21f8c88ba43ff8effd48a77e3124e352409`: owner-approved entry amendment,
  project plate validated/default overrides/seed0 versus bare CLI 3MF, unchanged
  fixture/comparator/assertions. Fresh official positive+repeat+single consumed
  three Native commands exit0/complete. API/CLI raw equality3/3 and repeat equality
  passed. Single failed: first full difference M73byte1002/line40, lengths64683/
  69418. Positive boundary changed to byte1239/line47, lengths64920/69623.
  Seeded0; no further comparisons. Neither attempt is a qualifying public RED.

These unmerged test commits remain evidence only. This docs-only branch starts
from approved docs892a850d9, excluding both failed-test commits and old Classic
lower-slice discovery ancestry. No permanent diagnostic hook or production change.

## One authorized retained-output motion localization

No Native command/build/debugger: one external script inspected retained single
Native/API/CLI complete G-code. First motion text difference: motion34, layer0,
line99, AFTER identical skirt retract/wipe and TOWARD first Inner wall:

```
Native:  G1 X143.482 Y144.261 Z.72 F15000
API/CLI: G1 X142.113 Y142.113 Z.72 F15000
```

Native next target(143.819,144.75) begins a clockwise ring bounded143.819..156.181;
API target(142.113,142.113) begins a larger CCW ring bounded142.113..157.887.
The carry-forward TYPE is Skirt until the next Inner wall marker: do not label
this travel skirt geometry. Different ensuing loop identity precedes routing;
not merely two seam positions on one common loop.

Native effective settings are Classic/inner-outer/none/no_brim/reverse1;
API/CLI embedded-project settings are Arachne/inner-outer-inner/sacrificiallayer/
auto_brim/reverse0. Different configured producers cannot establish algorithmic
parity RED. Literal fuzzy vocabulary also differs, without an independent alias
behavior claim. First-layer sandwich special ordering is deferred upstream past
layer0, so do not attribute this to that special reorder.

Source owner: Orca8500fcdcc `LayerRegion.cpp::make_perimeters:86-142`, especially
138-141 config-driven generator dispatch; `PerimeterGenerator.cpp::traverse_loops:
237-279` / `process_classic:1450-1469`; `process_arachne:2269-2340,2471`.
`GCode.cpp::extrude_perimeters:6131-6146` emits entity order; `extrude_entity:
6087-6097` / `extrude_loop:5744,5769-5775` then consume loops/place seams.
Internal first-loop causality within a matched generator remains unproved.

## Disposition and evidence

Choose option(b): close this fixture's seam slice NO-RED and propose the separate
[config-to-perimeter dispatch boundary](perimeter-input-dispatch-spec.md), SPEC
first. No new fixture, RED budget or production authorization follows automatically.
Original append lower-slice discovery remains deferred, not resumed.

Durable external reports: `single-control-motion-localization.md`,
`single-control-first-motion.json`, `seam-project-entry-red-stop-y1PN.md` under
`/home/indexyz/.ares-evidence/worker4/`. Original barrier archive
`seam-plane-public-red-stop-zTYK.tar.gz` SHA256
`62e426a868fbdd2428bd5cc6388fbcf92f3e9098b547d985ff5df8da1a5bbd57`.
Corrected-entry archive receipt is published with the review request. Complete
streams/receipts stay intact; motion projection is diagnostic only, not comparator
normalization. Native binaries invariant; CPU15/jobs1/threads1, fmt/clippy passed.
