# Slice B SPEC/proof: nonpositive sparse-density guard

Status: proposed SPEC for direct deletion; no B production edit.
Audited base: `c22f5a81da41fce715b528268711393649daad66`.

## Effective-region proof, not just the global schema

Paths below are relative to `crates/ares-core/src/`.

1. `options/region_options/merge.rs:15-51` resolves the complete effective
   region: process/parent, object, volume, material and layer-range overrides.
   It calls `normalization::normalize` after all these overrides.
2. `options/region_options/normalization.rs:12-16` maps every density below
   `f64::from(0.00011_f32)` to exactly zero, and densities above 100 to 100.
   This includes negative values arriving through region metadata, independently
   of global loaded-config validation.
3. `project/effective_config/candidates.rs:110-124` uses this resolve function
   for model-part region candidates; `project/region_graph.rs:156-167` uses
   it for parameter modifiers. Base/parent field copying inside the resolver
   is followed by normalization, not an alternate unnormalized output path.
4. `project_slice/prepare_infill/bridge_over_infill/transaction/candidate_expansion/layers.rs:97-99`
   bypasses anchor validation when the resolved region density is zero.
5. The remaining check in `transaction/anchor_projection.rs` uses the
   resolved f64 density and returns unsupported only for `density <= 0.0`.
   All such normalized values equal zero and were handled in step 4.
   A NaN, even if introduced through a malformed region carrier, would not
   satisfy `<= 0` and cannot be evidence for this particular return site.

Thus the nonpositive-density unsupported return has no public active witness.
This proof includes local object/volume/range overrides; it does not assume
that global numeric validation also validates every region metadata carrier.

## Upstream source boundary

Source commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `OrcaSlicer/src/libslic3r/PrintObject.cpp:3682-3706`,
  `region_config_from_model_volume`: apply object, volume,
  material and layer-range config, then map density below 0.00011f to zero
  and otherwise clamp to 100. Ares' resolver uses the same source boundary.
- `Fill/Fill.cpp:858-902`, `group_fills`: derive each surface's pattern/density
  by role, and produce no sparse fill for nonpositive sparse density.
- `Fill/Fill.cpp:1394-1505`,
  `Layer::generate_sparse_infill_polylines_for_anchoring`: consume grouped
  `stInternal` fills. Zero sparse density does not suppress solid surface work
  or the complete region/bridge transaction.

The admission function is a temporary compatibility shell around these ports.
Its unreachable error does not own density semantics.

## Proposed minimal change

Delete only `if sparse_infill_density <= 0 { unsupported(...) }` in the anchor
validator. Preserve region normalization, the caller's existing zero-density
selection, real grouped-fill behavior, geometry and every other check.
Do not add a new unsupported gate, silently discard an entire zero-density
region, invent anchors, or move validation into another internal layer.
The now-adjacent gyroid support comment belongs to the old pattern context;
remove it only if it is orphaned by the earlier C1
whitelist removal; it carries no executable behavior.

Deferred: C2 top/solid role enum overrides, D anchor-length float projection,
all fill algorithms and density normalization changes. This proof is not a
public RED, nor by itself a decision to remove B.

## Verification exit

After RESTORED: existing zero-density, mixed-region and no-bridge
public cases plus existing bridge/anchor regressions official+seeded. Fresh
Native cache and artifact root per mode/run; unchanged
whole-output API/CLI comparisons. Strict clippy, fmt and wasm32 must pass.
Separate B implementation commit. Preserve every existing assertion.

## C2/D investigation status

Top/solid role schema allows eight values globally. Local region metadata uses
`RegionMetadataCodec` for the full `ProcessInfillPattern` enum, and its merge
normalization does not clamp pattern values to those eight. Therefore these
checks are not yet proven unreachable. A Native-admitted local override must
be established before removing either guard or claiming a public RED.

Likewise local anchor metadata is not range-checked in the region normalizer.
A finite f64 carrier can overflow when narrowed/projected to f32. Native
release acceptance/output must be measured after RESTORED; no D proof or
production change is proposed here.

## Deletion decision and B behavior gate

Decision `20229a3d6` accepts deletion of the unreachable density rejection in
its own commit. Remove the orphaned pattern-support comment with this block.
This does not complete B: upstream `PrintObject.cpp:2521` considers `stInternal`
unsupported for density zero, while only Fill produces no sparse polylines.

Trace the public path separately from the admission proof. On the audited
Ares base, `candidates.rs` uses the same `kind != Internal || density == 100`
lower-solid predicate. The zero-density continue in `candidate_expansion/layers.rs`
occurs inside the per-region validation loop; the actual sparse generator call
is after this loop and still runs. These observations suggest the check is
only a validation bypass, but do not establish whole-output parity.

Before further production changes, build public Native E2E cases with an
internal solid ceiling at density `0%` and `0.0001%` (below the normalization
threshold), plus a positive-density control. Assert the Native internal-bridge
extrusion witness, no sparse extrusions for zero/near-zero, exact applied
public input values, and whole-output generator-only parity with Ares.
Persist input, Native/Ares bytes, hashes and comparison artifacts externally.
Use official and seeded after RESTORED with per-run fresh cache/root.
Any mismatch is B's actual RED and must be preserved before porting the
missing upstream behavior; matching cases become regression controls.
The caller and candidate logic are unchanged by the deletion.

### Public ceiling regression results (before rebase)

On deletion head `b3ca141c4`, the new public tests ran against both real Native
modes with independently fresh cache/artifact roots.
After correcting the worktree's missing vendor resource link (first attempt
was setup failure, not RED), all 11 selected tests PASS in both modes:
3 ceiling cases and 8 existing bridge/anchor cases.

- `0%`: Native internal-bridge extrusion moves 36, sparse moves 0.
- `0.0001%`: Native internal-bridge extrusion moves 36, sparse moves 0.
- `20%` control: Native internal-bridge moves 54, sparse moves 419.

All three whole-output generator-only comparisons PASS in each mode; Ares
API and CLI raw bytes agree. These are regression controls, not RED. No
candidate/generation repair is warranted by these inputs: the zero-density
continue bypasses only admission validation, not candidate gathering or the
subsequent anchor-generator call. The existing port preserves the upstream
zero-density ceiling behavior verified here.

Records join exact public input SHA and Native output SHA to the live
producer. Official records use wrapper SHA
`1689a2ff46564cf723ad90c663051b58115920c3cce639bd2961340955cebbf3`
(`setarch -R`); no deterministic new mismatch appeared. External evidence
is retained outside Git. This is evidence for
actual `b3ca141c4` plus the recorded test-source SHA, not a run on future main.
