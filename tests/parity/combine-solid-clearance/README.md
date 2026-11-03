# Combined-solid clearance: frozen Native-first RED inputs

Source rewrite boundary:
`docs/architecture/2026-10-20-combine-solid-clearance-spec.md`.
Orca8500fcd `PrintObject.cpp:4252-4264 combine_infill` clearance;
`Flow.hpp:61-62 scaled_width` truncates to coord_t before float arithmetic.
No new Ares pipeline, kernel/phase/graph or emission behavior is specified.

Single-region75-layer overhang, same placement/profile/geometry,0.2 layers,
0.4 nozzle. Source is the already-frozen
`../combine-infill-region-view/combined-single.3mf`; reuse that exact file
for the sparse15-on control instead of duplicating it here.

| Input | Density | Combine | Solid pattern | Maximum | Original Native result |
|---|---|---|---|---|---|
|existing combined-single|15%|1|monotonic|100%|PASS control|
|solid100-off|100%|0|monotonic|100%|PASS control|
|solid100-on|100%|1|monotonic|100%|DIVERGENT target|
|solid100-rect|100%|1|rectilinear|100%|DIVERGENT target|
|solid100-concentric|100%|1|concentric|100%|DIVERGENT target|
|solid100-one-layer|100%|1|monotonic|0.2|PASS control|

Five ZIPs copied byte-for-byte from accepted read-only discovery, never
regenerated for RED:
`${EVIDENCE_ROOT}/w3-combine-solid-readonly-231bd11cc/official/<case>/input.3mf`.
Every non-settings ZIP member equals source; model_settings.config also
unchanged (one ModelPart/no modifier). Only GUI project settings density,
combination flag, internal-solid pattern and max combination height differ.

SHA-256:

```text
76fbbcdf669a9ba39c368c9a27724c865ced36761a315cb00274ad160ac4c6ad  ../combine-infill-region-view/combined-single.3mf
c7a885fc01d6d0e28921a377aae9e3d4310cb258e6a32d8e5f1216cad4e9f3fd  solid100-off.3mf
4c4403fcd748643b7831be80ec8ea5f0856bcc069d21a70ff28a2a64e8861163  solid100-on.3mf
e5cf9a53a47f9e3a4413a4b2f3f81895fc9bf48b08a5993151e17c4a409458b9  solid100-rect.3mf
3bbbe5853630542b8671521e35c1dde38f763ffef4f133f904eb25ee7aa76a9e  solid100-concentric.3mf
80910d21602bb78ca36df0f5771064b041d768016294deccfdbdf4c60c251791  solid100-one-layer.3mf
```

Tests reuse unchanged W3 case_runner/complete: frozen hash/project admission,
two fresh canonical Native references plus public API and CLI per case,
repeat stability and API/CLI raw equality, effective Native GUI footers,
positive Internal solid activity, sparse presence/absence and0.4 combined
height activation controls. Per-run witness JSON is diagnostic; strict
complete-output PASS remains unconditional for all targets and controls.
No Native G-code golden, tolerance, output filter, new ignore/allow/expect,
unsupported guard, fallback or inverted failing-target assertion. No
production change belongs in RED. Preserve all existing44 selected cases.

Run via cargo nextest, both canonical official and seeded0 modes, CPU12,
ARES_REQUIRE_ORCA=1, fresh caches/artifacts, selector `test(anchoring_zaa)`.
Expected RED:3 strict FAIL/47 PASS per mode after adding these six cases.
Full M73/time/feedrate/E/wipe/retraction/footer bytes remain significant;
normalize only existing generator/date identity. No FULL without COORD's
explicit after-W2 integration window. Freeze these inputs/tests/helpers during
future approved GREEN; any strict residual after the bounded arithmetic port
means STOP/report, never adjacent kernel or G-code repair.
