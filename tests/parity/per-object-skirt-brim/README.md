# Per-object skirt + explicit outer-brim inputs

SPEC: `docs/architecture/per-object-skirt-outer-brim-spec.md` (approved
`7c89fef57`). These four 3MFs are pure inputs; no Native output is checked in.
Exact input hashes, translations and Native-observed topology are in
`manifest.json`. Shared mesh and distinct source object identities are retained
from `../per-object-skirt/disjoint.3mf`.

Both pairs keep single instances at X80 and X140 (far) or X110 (near). Only
`brim_type`, `brim_width` and explicit `combine_brims=0` differ within a pair.
Native has 2 skirt groups for both controls and far-brim, but 1 for near-brim.
Each brim input has 2 independent brim emissions with real positive-E XY
motion. The test first retains Native admission/output/topology artifacts,
then compares public Ares API and CLI using the existing strict full-output
comparator and raw API/CLI equality. Success, not rejection, is the assertion:
baseline rejection is retained as RED evidence before any production edits.

Native-only discovery: `${TMP_ROOT}/ares-skirt-brim-native-ANfbTd/`, 8/8 accepted,
actual sampled descendant/thread affinity CPU18. Discovery is not public RED.
Repeat public verification with `ARES_REQUIRE_ORCA=1` and the established
Native environment, isolated target, fresh caches/artifacts; select
`test(orca_parity_per_object_skirt_brim_)`. Artifacts include exact input,
Native full G-code/result/launch, motion witness, Ares errors/output and strict
comparison. Do not change these inputs/assertions to conceal GREEN differences.
