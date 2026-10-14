# Positive minimum skirt filament length + owned outer brim

Approved SPEC: `docs/architecture/per-object-skirt-minimum-outer-brim-spec.md`
(`f46da38bd`). These four 3MFs are pure inputs; no Native output is checked in.
Hashes/source hashes and geometry/budget expectations are in `manifest.json`.

Both pairs preserve the existing far/near singleton outer-brim inputs and all
settings except min_skirt_length 0 -> 20. Brim remains outer_only/width8/gap0,
combine_brims0; one filament, by-layer, skirt_loops2, no other deferred feature.
Native discovery `/tmp/ares-skirt-min-brim-spec-oBAAYD/` accepted 8/8 two-mode
exports on sampled CPU18, with real additional skirt motion independently for
both far groups or once for the shared near group; two brim owners persist.
The current-main CLI rejects positive inputs and admits the zero controls.

Do NOT assert brim bytes/segmentation are identical between minimum0 and20:
Native far brim changes positive XY segments 920 -> 918 per owner due to the
upstream current-position split interaction. Strict comparisons are between
Native/API/CLI outputs for the SAME exact input. Raw zero-control bytes are
retained for later RED -> GREEN stability checks.

Public selection: `test(orca_parity_per_object_skirt_min_brim_)`, required
Native environment, fresh per-mode cache/artifacts, explicit owned target,
CPU18. Tests first preserve Native admission/full stream/real-motion witness
and exact input/producer hashes, then require public API and CLI success,
existing strict ordered full comparisons and raw API/CLI equality. Rejection
is never asserted as success. No fixtures/assertions are weakened in GREEN.
Selected-input >20mm emitted skirt E is a Native-observed fact, not a general
minimum algorithm rule: source generation counts full loop length and export
may clip the seam. Budget must not include brim extrusion.
