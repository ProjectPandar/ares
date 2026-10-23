# ARD: GUI fuzzy painting is a consumed upstream region rewrite

Status: proposed; independent SPEC approval required before public RED.
Base/frozen beam tip:92ce9b02c. Upstream:8500fcdccaa10b5099ac20d252af3a7c560046f1.

## Non-negotiable decisions

1. Owner is bbs_3mf/ModelVolume::fuzzy_skin_facets → PrintApply's actual
   FuzzySkinPaintedRegion → fuzzy wrapper of segmentation_by_painting →
   PrintObjectSlice.cpp1033–1141/1227–1241 real slice replacement → classic
   FuzzySkin.cpp561–684 masked perimeter application. No Ares-owned pipeline.
2. Canonical paint_fuzzy_skin is not paint_color, global fuzzy, or a permissive
   unknown-XML extension. None permits paint activation; Disabled_fuzzy stays
   disabled. Preserve codec/tree/shared mesh identity and real parent-region IDs.
3. Existing staged print_apply marker/config shells cannot stand in for actual
   project registry or LayerRegion surfaces. Add source-owned records inside the
   existing registry, never a shadow registry or independent public phase API.
4. Reuse existing geometry only with the fuzzy wrapper's actual parameters:
   2states, max real external-perimeter flow width,0depth,false beam,no caps.
   MMU's filament-count/caps/extruder-remapping assumptions are not reusable as-is.
5. Real masks must reach classic compatible/merged-region and line-splitting
   application. A config-only whole-polygon perturbation or XML-admission-only
   patch does not satisfy this complete consumed slice.
6. Keep literal MMU→fuzzy→single existing interlocking→compensation order.
   Preserve stock EFC0.1 and zero XY fixture values; don't infer nonzero XY
   behavior from warning text. Deferred compositions remain fail-closed.
7. Discovery has8actual Native and2CLI, not public RED. Replays match full bytes
   with only established generator-line allowance;3/4pairs have timestamp-only
   raw differences and1/4raw equality. No stochastic waiver or broader raw-byte
   stability claim. No resource/replay/control failure gets semantic gate credit.
8. Public RED must require real API/CLI success and complete per-input ordered
   Native/API/CLI equality plus raw API==CLI. Current rejection is real FAIL,
   never expected-error PASS. No mocks, constants, tolerance or output filtering.
9. First runtime domain is the exact stock classic/Ripple single-face pair.
   MMU+fuzzy, beam+fuzzy, Arachne-painted, nonzero XY and broader domains remain
   deferred. Existing global-fuzzy support is preserved, not rewritten arbitrarily.
10. SPEC approval→public RED RR→separate GREEN instruction. Core stays portable,
    memory/bytes only; no fallback/backend/API redesign. Files≤400, no include
    tricks. No FULL/rebase/merge/push here or unfreezing92ce; integration is separate.

## Rejected shortcuts

Silently ignore the XML attribute; only relax admission; rename MMU carrier as
fuzzy; pass MMU states/caps; stage config markers without consumption; perturb
the whole polygon; duplicate interlocking; add a second registry; hardcode stock
face coordinates/Native output; accept stochastic official output as strict PASS.
