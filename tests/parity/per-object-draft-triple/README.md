# Triple shield/minimum/brim: twelve pure inputs

SPEC5a7206e96 conditionally approved with RED coverage expanded to12 inputs:
4 original both-brim triples plus8 mixed left-only/right-only ownership cases.
Stacked on82d11ee6f; no changes to frozen parent or previous controls.

Exact original inputs: /tmp/ares-draft-triple-native-G4zq1A/inputs (Native8/8).
Exact mixed inputs: /tmp/ares-draft-triple-mixed-jhXvqN/inputs (Native16/16).
SHA/source hashes and expected groups/brim-owner counts are in manifest.json.
Only pure3MF inputs are committed, no Native outputs.

All use draft enabled/minimum20, global OuterOnly width8/gap0/combine0,
positive2skirt loops/height1/distance6, singletons and by-layer/one filament.
Mixed cases override exactly one object to no_brim/width0; Native honored it.
Far2 separate skirts/near1 shared skirt; physical skirt lifetime20Z0.2..4.0.
Full/single first-layer motion is identical; single later innermost emits less
than20mm (no per-layer top-up). Only first layer has2/1 independent brim owners.
No cross-input equality assertion for brim segmentation/bytes.

Public selection: test(orca_parity_per_object_draft_triple_).
Native-first real per-Z/group/positive-budget/first-layer-owned-brim gates,
then API+CLI success and existing strict ordered Native comparisons, raw
API/CLI equality. Every baseline case must FAIL the existing draft guard in
both modes; rejection is never accepted as success. Keep all inputs/assertions
and previous control raw bytes unchanged; independent RED review before GREEN.
