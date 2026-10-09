# Modifier wall-loop PrintRegion verification

Status: REVIEW; sole-coordinator approval required before push. Spec `bfe079a01`
was approved before runtime tests or production changes.

## Source-owned implementation

PrintObject.cpp:3664–3710, PrintApply.cpp:958–1058 and Layer.cpp:139–211
remain owned by existing RegionOptions resolution, region_graph and
VolumeRegionRange. Modifier overrides follow the already range-resolved parent;
existing region identity, reverse-parent ordering and incompatible-perimeter
handling require no parallel registry or new pipeline.

Production delta against main bf0dd12b8:

- effective_config/candidates.rs removes only the modifier wall_loops gate.
  All other option-family gates remain. No silent skip replaces rejection.
- arachne/wall_toolpaths.rs retains the original outline as the inner contour
  when inset_count is zero. PerimeterGenerator.cpp:2169–2175 consumes this
  behavior through Arachne/WallToolPaths.cpp:778–789::getInnerContour.
  That source file SHA256 is
  `0a40472ff3621ecbc5dd9bf0008686d71fb990f63bf54871909d405d014dc29c`.
  No new geometry algorithm or ordering policy is introduced.

The obsolete unit-test wall_loops rejection row and wall-first combined
assertion block were deleted with coordinator approval. The remaining eight
key assertions and their order are unchanged; no replacement Unsupported
expectation was introduced. Positive wall_loops coverage is provided by the
modifier_wall_loops Native complete-output equality E2Es.

## Independent runtime RED and unchanged GREEN

Only the entire generated-by line is normalized. Native streams, inputs as
executed, commands, receipts, failures and diagnostics remain under /tmp.
Seeded Native is the existing fail-closed runner with seed0, 16 threads/build8;
no new official-binary run is claimed.

| Exact tested commit | Nextest run | Result |
|---|---|---|
| 9fbbbe51a0232b363a557d7da15b37db0bb3b88f, test-only | aae22d30-3360-41d6-b70e-0839b9402211 | independent classic/Arachne controls complete-equal; both wall4 Native exit0, Core Unsupported causes success assertion RED; 2 FAIL |
| 2b5258eea980cb39c30a983cf6d750b6f892b71f | 1cc0b32b-062b-4ac6-b043-b2f42b193ac8 | unchanged primary tests, 2 PASS |
| 0531d48f701aad2c505064e0109651ea10c10fcc, test-only overlay on 702 | 2edefea9-6b0a-4507-b0bd-671e80d24ca6 | 12 RED, all Native exit0; production delta zero; overlay removed before clean baseline |
| c468e8807533e0d40565899d50966034c31afae1 | c4aa259f-eb8f-4917-8093-1a27c84548a1 | 11 PASS / 1 FAIL: Arachne zero walls loses infill; first complete-output difference byte74, line3, 197709 versus159446 normalized bytes |
| e682f8152aea366318ea0f5c3bb48e3fad9b9eb7 | ee9080b6-0294-47f4-b959-2035a9dba86b | unchanged matrix 12 PASS after source-owned zero-wall contour port |
| cec36898b987eba4e57566747fa9ca02f45c4b47 | 6a34577a-664c-44e9-bdc6-fcdb019bed3b | directed 13 PASS: existing remaining-key guard plus12 Native E2Es |

Each backend independently covers inherited control, half-cube wall4/parent2,
explicit-height range parent3/modifier4, equal parent2/range3/modifier2,
Native-admitted zero, outside overlap and ordered nested wall4→wall5.
Original assertions and inputs remain unchanged after RED. Tests compare the
public byte API with independently executed complete Native output, not an
expected rejection or committed golden stream.

## Fresh ordered seeded pair and final gates

Fresh clean BASE `70243e353eb1f214512e5c12c35a731a1570f73d`, run
`8c04a67b-ab29-4212-a835-a67b4c3f679e`: 758 PASS, zero FAIL/skip.
Then CAND `e682f8152aea366318ea0f5c3bb48e3fad9b9eb7`, run
`24a46318-5ddc-4d7b-904b-2ebc5351d113`: 770 PASS, zero FAIL/skip.
Full common-byte audit: 624 input-identity groups, **643 actual common Ares
streams unchanged**, 14 new complete Native-equal streams, zero changed,
base-only or ambiguous groups. ZIP framing is ignored only for input identity,
never for output comparison. Receipts preserve actual input hashes.

These seeded executions remain bound to 702/e682, not to rebased HEAD.
Coordinator authorized carry-forward over two lint attributes (977), docs-only
1e6, tests/docs/product deletions c5 and bf, and the production seam visibility
radius double-promotion change895. The latter is not production-zero:
SeamPlacer.cpp:685 ownership affects first-layer-hole cases and is covered by
worker1's new E2E. A new full workspace was required after that rebase.

Final required-Native workspace actually executed at
`3d6fcc0c7027dab6c4848f2894e49e85c56bf3b5`, based895, run
`211c74e3-a47e-490f-a516-eb7915765480`: **8148 PASS, zero FAIL, six existing
skips**, exit0. This includes unchanged epsilon, Stage1/Stage2 and zero-height
controls and all modifier E2Es.

After workspace completion, git-only rebase to bf produced
`5bb3c7423f0d28cda7aa663e25e88d0ef09ce89a`; inspected production src delta
from3d6 to5bb is empty. Coordinator explicitly authorized this carry-forward.
At exact5bb: workspace/all-target/all-feature clippy `-D warnings`, fmt check,
and ares-core+ares-wasm/all-features wasm32 check all exit0. Compilation is not
Native execution. This conclusion document is the only subsequent change.

## Retained non-PASS and scope

- Initial module-path compilation failure101 before Native, preserved as
  `/tmp/worker3-modifier-wall-module-error-v1.*`; not runtime RED.
- Supplemental module-path rustfmt failure before runtime, tool3467 log
  `/tmp/pi-unified-exec-3467-6fae2ff3.log`; not Native evidence.
- Primary GREEN v1 interrupted by coordinator quiet-window SIGTERM, preserved
  `/tmp/worker3-modifier-wall-green-v1-INTERRUPTED.*`; no verdict.
- c468 supplemental 11/1 failure preserved, not relabeled GREEN.
- e682 workspace run3f8d03cf-42b5-4b4e-927e-2edf8311d927:
  8146 PASS / 1 FAIL / six skips, exit100, obsolete wall-rejection assertion.
  Original `/tmp/worker3-modifier-wall-workspace.*` retained.
- c603 workspace interrupted by coordinator load-saving SIGTERM:
  `/tmp/worker3-modifier-wall-workspace-c603-INTERRUPTED.*`; not review evidence.

Evidence prefixes: `/tmp/worker3-modifier-wall-{red,green-v2,supplement-red,
supplement-green,zero-contour-green,base702,cande682,common-byte-record,
common-byte-summary,assertion-directed,workspace-final,final-clippy,final-fmt,
final-wasm}`. Independent commands/status/streams are inside the associated
artifact roots. Native binary SHA256 remains
`59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff`.

Other keys/material expansion, MM/fuzzy, multifilament, spiral ranges,
adaptive/painted profiles and active ZAA remain DEFERRED. Worker6's object-height
adapter files were not edited. No Native product is added by this slice.
