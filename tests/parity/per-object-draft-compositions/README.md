# Divergent draft compositions: pure inputs

SPEC: `docs/architecture/per-object-draft-compositions-spec.md`.
Twelve exact input hashes/source hashes/settings are in `manifest.json`.
No Native G-code or other expected output is committed.

Far/near geometry × control/minimum/brim × all/single-loop modes:
- control: enabled draft, no brim, minimum0 (four already admitted controls);
- minimum: enabled draft, no brim, minimum20 (four divergent inputs);
- brim: enabled draft, singleton OuterOnly width8/gap0/combine0, minimum0
  (four divergent inputs).
Triple draft+positive minimum+brim is intentionally absent and stays deferred.

Native discovery accepted24/24 two-mode exports with20 skirt Zs0.2..4.0. Minimum generation emits
extra first-layer skirt loops per group; later single-loop layers emit less
than20mm filament, not a per-layer top-up. Brim-aware near grouping changes2
skirt groups to1 while two independent brims print only on the first layer.
No equal-brim-bytes assertion is made across inputs/modes.

Public selection: `test(orca_parity_per_object_draft_composition_)`, required
Native environment, fresh caches/artifacts and explicit target.
Tests first retain Native full G-code/result/launch/per-Z real-motion witness,
then require public API+CLI success, existing strict ordered same-input Native
comparisons and raw API/CLI equality. Rejection is never asserted as success.
Baseline should retain8guardFAIL+4controlPASS per mode.
Keep every input/assertion fixed and preserve control raw bytes for the later
RED->GREEN stability audit. No stream normalization or new guard policy.
