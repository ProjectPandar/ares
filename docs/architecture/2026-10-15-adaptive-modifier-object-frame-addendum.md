# AdaptiveCubic modifier witness: PrintObject coordinate-frame amendment

Docs only. This supersedes the Adaptive exclusion in the
[original spec](2026-10-15-adaptive-noop-modifier-fill-loss-spec.md). The public RED `c488128e8` and
[STOP evidence](2026-10-15-modifier-sparse-loss-red-localization.md) stay frozen.
Builds on main `3ee8f272c6d8e2db22dde9003a01afeb7201e36a`.
Upstream revision: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Native measurements, before proposing the fix

Existing official Native binary, SHA256
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`,
was executed once each on the frozen A-C0 and A-C1 inputs under read-only GDB.
No rebuild, inferior call, memory/register assignment, source probe,
input change, or retry was used. The debugger disables ASLR, matching the
canonical wrapper. Both Native processes exit 0, and their observed CPU
masks are 11.

| Measured value | A-C0, no modifier | A-C1, same-config modifier |
| --- | --- | --- |
| `PrintObject::m_center_offset`, scaled integer XY | `(0,0)` | `(118000000,138000000)` |
| Same offset in mm | `(0,0)` | `(118,138)` |
| `m_trafo` XY translation | `(0,0)` | `(0,0)` |
| Root center before inverse rotation | `(2.886723518371582, 2.8867650032043457, 2.886765480041504)` | exactly equal |
| Root center after inverse rotation | `(-1.1605717435613449e-7, 3.223302673660555e-7, 4.999999979200413)` | exactly equal |
| Adaptive line spacing | `4.499999865889554` | exactly equal |

These are actual Native values. The offset read is anchored to the official
`prepare_adaptive_infill_data` disassembly: hidden result pointer in RDI,
`this` in RSI at entry, `m_center_offset` at object+0xb40; instructions
+512..567 convert these two signed 64-bit coordinates, multiply by
`SCALING_FACTOR`, and subtract them from the matrix XY translation.
`transform_center` entry/finish observes the same root pointer before and
after the rotation, without evaluating an inferior function.

Each observed Native output is compared to its prior ordinary Native output.
Exactly one Orca generated-by timestamp line is normalized on each side;
all remaining bytes compare exactly equal. Raw files differ on that timestamp
line. Both normalized streams have SHA256
`dbb296c77583bb99b777148b92a5f63b607fdc5a7aea13819db6f6d410289ede`.
This is an observer-neutrality check, separate from the unchanged Ares/Native
comparator in the public RED.

All Native products remain outside Git.. The earlier symbol-only type lookup returned
no type information; that failed lookup is retained and is not measurement.

## Owning upstream contract

`PrintObject.cpp:91-110` computes `m_center_offset` from the object bounding
box without instance XY translation, with the print-group rotation adjustment.
`Print.hpp:327-328`, `PrintObject::trafo_centered`, subtracts the scaled center
from the object's transform. `PrintObject::prepare_adaptive_infill_data`,
`PrintObject.cpp:981-984`, applies `to_octree * trafo_centered()` to the raw
ModelPart mesh before building its tree at :1005-1007.

`FillAdaptive.cpp:1495-1497` computes the float bbox/size of that already
centered mesh. At :1524-1526 it rotates root/child centers and origin back to
the fill coordinate frame. The actual `transform_center` expression at
:1472-1480 is rotation only, with no addition of instance or raw-mesh XY
center. Native A-C1 has a nonzero center offset but the same final local root
as A-C0: the offset is consumed by centering, not reintroduced afterward.

Ares already implements this PrintObject frame for slicing:
`raw_intersections.rs:163-172` calls `centering::center` for the actual source
object and resolved print group, removes instance XY translation, and
pretranslates by the negative scaled center. `centering.rs::center` implements
the source bounding-box/print-group rotation contract. These existing helpers
are authoritative; a modifier bbox, region-zero proxy, selected fill polygon
bbox, or scene translation must not stand in for the owning PrintObject.

## Exact Rust boundary and proposed behavior

The production destination is
`project_slice/fill_entities/adaptive.rs::build_object_octree` and only its
private frame plumbing if required to reuse the checked centering helper.
For `OctreeKind::Adaptive`, construct the ModelPart mesh in the same centered
PrintObject frame used by raw slicing, then rotate it into octree coordinates.
The mesh-to-tree/output conversion must preserve that frame. An offset already
consumed by PrintObject centering must not be added to the returned cube
centers. Retain the existing float-bound and spacing precision contracts.

Current code instead constructs vertices with the full resolved transform,
subtracts a transformed bbox center for the rotation, and passes that bbox
center back into the builder. The generic `octree.rs:225-244` helper adds it to
the rotated cube centers. In A-C1/A-C2 this produces root XY
`(117.99999988394282,138.00000032233027)` while all 43 sparse append inputs
remain local and equal to A-C0. The difference is observed before line
extraction, not inferred only from output counts.

Do not globally delete translation in `octree.rs`. Prefer selecting the correct
Adaptive input frame and resulting offset in the caller, keeping the shared
builder unchanged. If implementation proves a shared-builder change necessary,
stop and bring the specific evidence before that edit. Existing
adapters remain temporary compatibility shells for the cited PrintObject
transform and FillAdaptive tree; no new pipeline or public API is introduced.

## SupportCubic isolation and deferred behavior

`OctreeKind::Support` must retain its current mesh transform, centering offset,
overhang triangle coordinates, f32 mesh narrowing, and builder arguments.
Its preparation caller is `adaptive/preparation.rs::prepare_support`, which
uses the Support-kind cache key and original bridge candidates. The branch is
separate from `OctreeKind::Adaptive`; no cache-key, spacing average, preparation
order, bridge candidate, or sharing-policy change is included.

This gives a structural isolation argument, not a substitute for execution:
the GREEN must demonstrate that existing SupportCubic controls remain raw-byte
unchanged. `adaptive/hooks.rs`, touching-grid S-C0, and adaptive
object-region spacing are separate slices. No connector, hook, collision-index, support spacing,
`touching.rs`, surface grouping, bridge-angle algorithm, or emission change is
included. A residual after fixing this frame is reported with the frozen
strict RED intact, not absorbed through assertion changes.

## TDD and exact-head checks

Keep RED `c488128e8` separate; implement GREEN in its own production commit.
In canonical official and seeded modes, with fresh cache/root
for every run, verify:

- Frozen public modifier tests: A-C0/A-C1/A-C2 must all match complete Native
  bytes, line counts and layer vectors, with raw public API==CLI. C0 is a
  control; no tolerance or new normalization is permitted.
- Existing `runner::adaptive_cubic_precision` family: A45/A50/A125/AAUTO,
  S45/S50/S125/SAUTO, Rectilinear and Aligned controls, both modes. Existing
  non-RED controls retain their raw Ares outputs compared with the base.
- The frozen `support_touching_grid_s_c0_public_api_cli` witness
  (input SHA `9580fd4f17fad55829f561c13ee0ae336b447016c7c9682ce301f79afc9d0bc1`)
  is run unchanged on both base and candidate. It is currently an acknowledged
  independent public RED in `c15b29c60`; a continuing known panic
  is reported as FAIL, never PASS. If its fix has merged before these runs,
  use the merged test and require PASS. Do not edit its assertions.
  An external validation-only test tree can carry its exact unchanged test
  while leaving this candidate's production and test commits untouched.
- Entire option_coverage group in both modes, including non-Native inventory
  tests; exact clean-head strict clippy/fmt/wasm32.

Any changed SupportCubic behavior or unexplained non-RED byte change means STOP.
Report all outcomes, including the separate S-C0 baseline failure if still
present. No Native products, ignored tests, new guards, assertion retirement or
global frame reset is part of this docs-only amendment.
