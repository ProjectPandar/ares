# Fuzzy perimeter support-area restriction

Status: public RED approved; Arachne supported-area rewrite under verification.

## Ledger selection and source boundary

Pinned beta ownership/walls.jsonl SHA256
`3d5e0b949676ca12cd9481cbe033efd67cb44fe9630a57bc648848aa60e62349`
contains 62 official and 74 seeded failed observations, not closure counts.
Grouping the actual first observations by source function family, after excluding
config/admission and harness guards/static-reference failures, gives the largest
walls cluster: fuzzy wall output, 8 official + 17 seeded observations, 17 distinct
test names. All eight official rows use deterministic Ripple. Nine seeded-only
stochastic rows are historical context, not official parity witnesses. The
seeded small-padding first observation is a layer-count header difference and
belongs to a separate cluster, not fuzzy output.

The next eligible families are Arachne open-wall geometry/nearest (6 official),
seam starts/layer height (4), and already-ported scarf endpoint snap (4). The
external ranking artifact records every original row/name, first command pair,
exclusion, producer count and exact-pair counts; it does not rename ledger IDs.
The earlier verbal estimate of 19 fuzzy test names was corrected to the
machine-counted 17 before any runtime qualification.

Target `345efec64c7b95a05ce5cf96df0dde66a158835c`, upstream
`src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp`:

- `group_region_by_fuzzify:480-488` resets `fuzzy_supported_area`; when eligible
  fuzzy walls/holes and lower slices exist, offset lower slices by
  `float(ext_perimeter_flow.scaled_width() + max_thickness)`.
- `restrict_to_supported:587-593` replaces an unrestricted empty region with the
  support area, or intersects a nonempty region. An empty result now means NO
  fuzzy coverage, not the old unrestricted-mask sentinel.
- Arachne `apply_fuzzy_skin:722-857`, particularly `:727-730,742-744,792-806`,
  suppresses whole-line fast paths when support is present and restricts eligible
  merged masks before ordered line splitting and fuzzy application.
- `PerimeterGenerator.cpp:424` invokes this perimeter rewrite before downstream
  wall materialization; it is not a G-code sampler feature.

Rust destination is
`crates/ares-core/src/project_slice/perimeters/arachne/fuzzy.rs::Regions::{prepare,apply}`
and its mask application. Current Regions has only effect masks and no lower-layer
support restriction; its single-region fast path always applies to the whole
eligible extrusion. It is a partial upstream rewrite/temporary compatibility
shell, not authority for an independent Ares fuzzy-wall pipeline.

## Why this discriminating input instead of a feed-only ledger comparator

Old F1 Ripple-phase first observation is F1851/F1854, with later ordered geometry
also differing. Old F2 rows include ordered XY/E point differences. Those saved
captures may co-observe the queued printing-state sampler `eb9672f1b`. They are
NOT new REDs, nor proof that support restriction alone causes every cluster row.
No printing-state sampler/caller code will be duplicated here.

Input1 derives from the accepted Arachne stepped-cap exterior fixture
`31909cf497033d59292683c0224ccc6c6806a368f766a1bac04747e8c64ea64e`.
Only seven existing settings change: deterministic external Ripple, first-layer
on, four ripples, 25% phase offset/two-layer groups, and overhang bridge fan off.
Thickness0.2/point distance0.3/displacement mode remain unchanged. Overhang speed
was already off. All mesh, transforms, XML, other settings and archive payloads
remain byte-identical. This disables both overhang speed and fan sampling to
isolate the upstream perimeter support mask, rather than copying caller behavior.

New input `tests/parity/fuzzy_skin_supported_area/unsupported_cap_ripple.3mf`,
SHA256 `14dca2ee591608cdde18f8801ed3b49dc597746d1267bb0afb6970d6114fcfdf`.
The cap's exterior is fully overhanging far beyond the lower support-area offset.
Native should keep that exterior smooth even though deterministic Ripple remains
configured and operates on supported lower walls. Rust's unrestricted path is the
candidate discrepancy. Source gap is definite; public causal qualification is
still required. The separate Arachne collinearity dependency is not silently
applied to this 1fa-based branch or claimed closed by this witness.

## Strict public RED contract and stop rules

Budget at most two distinct inputs. Start with this single input/public test:
ordinary official beta Native stable DOUBLE export followed by public
`ares_core::slice_project_plate_validated`, deterministic metadata/default
overrides/context0. Ripple uses no stochastic noise or seeded oracle producer.

Compare the ENTIRE first cap object at Z1 using the existing extractor. Include
all XYZ/E, feeds, widths/types, M73, retract/wipe and terminal records. No command
filter, contour rotation, sorting, normalization, tolerance or corner forcing.
Native effective fuzzy/Ripple settings and Overhang-wall role must be admitted.
Freeze only a discriminating final whole-object failure; an unstable Native,
failed premise/admission, inert case or unrelated first difference is a failed
qualification, not production RED. Keep every failed receipt. Do not sweep inputs
or add one beyond the two-input bound. No private planner tests/diagnostics,
producer flags, source change, seeded/FULL or GREEN before RR review.

## Included and deferred upstream behavior

RED contains only this test, registration, source-cited ARD and derived fixture.
ROOT entrypoint, production, existing cases and producer scripts stay frozen.
A later GREEN may only be requested after Native/API qualification and an RR.

The approved Arachne concept includes source support-area construction,
restriction and its required distinction between unrestricted empty and
restricted-empty masks. The existing whole-lower-layer slices reach prepare
through the Arachne materializer; no cached input geometry is mutated. Reuse the
exact C2 geometry contract; no new kernel/rounding policy.

The Classic polygon overload in the same upstream file (`apply_fuzzy_skin`,
595-719, notably 601/662) also consumes `fuzzy_supported_area`. That consumer is
explicitly DEFERRED in this slice. Its existing Rust shared grouping constructor
receives no support-area input, preserving Classic behavior; Classic apply and
all shared effect-merging/region-exclusion logic are unchanged. This is an
Arachne-only rewrite boundary, not a claim of Classic fuzzy-support parity.
Defer closed/open endpoint transfer, open-wall mask expansion, new extrusion
width floor, random kernels, Ripple arithmetic, generic clipping/order changes,
wall chain/seam scoring, emission/cooling and caller sampling. Do not silently
include these if the support slice is insufficient. Original ledger rows remain
OPEN pending COOD disposition; this slice cannot claim the entire fuzzy cluster,
complete-file parity, historical seeded equality or global beta closure.
