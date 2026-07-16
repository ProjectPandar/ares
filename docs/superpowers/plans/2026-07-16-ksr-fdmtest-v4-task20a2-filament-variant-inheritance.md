# Task 20A.2: Typed Filament Variant-Aware Inheritance Implementation Plan

> **Execution contract:** Follow the approved SDD workflow and this checklist
> in order. No production or test implementation may begin until these exact
> plan bytes receive literal `VERDICT: APPROVE` from both a fresh independent
> Codex reviewer and the required default-model OpenCode reviewer. Execute the
> six bounded packages with fresh implementer subagents and no package commits.
> Every package needs fresh spec-compliance and code-quality approval; commit
> and push only after whole-implementation and documentation approvals plus a
> fresh release matrix.

**Approved specification:**
`docs/superpowers/specs/2026-07-16-ksr-fdmtest-v4-task20a2-filament-variant-inheritance.md`

**Approved specification SHA-256:**
`5C2C39394BD9362477CA11A1E3FDEB1BF0B7BDBE88EF6C0365AFB7400874668D`

**Pinned OrcaSlicer SHA:**
`8500fcdccaa10b5099ac20d252af3a7c560046f1`

**Ares baseline SHA:**
`e0c50564283744b3dd3388eeaa10f624a492ff1f`

## Goal and immutable behavior ledger

Port only Orca's filament, no-extruder-ID, stride-one variant inheritance
slice into the existing typed profile resolver. The root is resolved against
typed defaults and normalized once; descendants remain sparse, preserve
ordinary non-variant whole-field replacement, and map the exact 36 data
vectors through `filament_extruder_variant`. Machine/process inheritance,
profile-to-project wiring, geometry, and G-code remain unchanged. Valid project
slicing must still return `ProjectSlicingIncomplete` after producing the
released 49,004-byte config block internally.

The fixed source boundary is `PrintConfig.cpp:63-84,8375-8415,10209-10297`,
`Preset.cpp:231-278,922-945,1679-1697`, `Config.hpp:558-580,624-665,
812-837,921-931,1008-1016,1203-1218,1872-1879`, and
`libslic3r.h:52,306-310` at the pinned SHA. Executable tests must not pin those
paths, symbols, line ranges, or source bytes.

### Exact 1 + 36 inventory and equality partition

The implementers must use compile-time field identifiers/concrete owner
methods, never a runtime key registry. The local source-versus-child equality
partition is fixed and must not change any option type's global `PartialEq`.

**Approximate nullable vectors, 19 total (`abs(delta) < 1e-4`):**

- GCode (4): `filament_flow_ratio`, `retraction_distances_when_ec`,
  `filament_flush_volumetric_speed`, `filament_cooling_before_tower`.
- Region (4): `filament_ironing_flow`, `filament_ironing_spacing`,
  `filament_ironing_inset`, `filament_ironing_speed`.
- Retract (11): `filament_retraction_length`, `filament_z_hop`,
  `filament_retract_lift_above`, `filament_retract_lift_below`,
  `filament_retract_restart_extra`, `filament_retraction_speed`,
  `filament_deretraction_speed`, `filament_retraction_minimum_travel`,
  `filament_wipe_distance`, `filament_retract_before_wipe`, and
  `filament_retraction_distances_when_cut`.

**Exact vectors, 17 total:**

- GCode (5): `filament_max_volumetric_speed`,
  `long_retractions_when_ec`, `filament_flush_temp`,
  `volumetric_speed_coefficients`, and
  `filament_adaptive_volumetric_speed`.
- Print (7): both nozzle-temperature vectors, all three air-filtration bool
  vectors, and both exhaust-fan-speed integer vectors.
- Retract (5): `filament_z_hop_types`,
  `filament_retract_lift_enforce`,
  `filament_retract_when_changing_layer`, `filament_wipe`, and
  `filament_long_retractions_when_cut`.

`filament_extruder_variant` is the one exact string mapping identity and is
never a data assignment. Nullable `Nil` equals only `Nil`. Use zero versus
`0.0001` for the strict epsilon boundary test so floating subtraction and the
upstream constant are the same representable value. Non-nullable
`OrcaFloats` remains exact even below epsilon.

### Root/child nil and resize partition

For the concrete root, derive `N` from the resolved identity before touching
data fields. Apply exactly these upstream `is_nil` classes before resize:

- all-`Nil` reset to typed default (11): the seven nullable GCode fields
  (`filament_flow_ratio`, `long_retractions_when_ec`,
  `retraction_distances_when_ec`, `filament_flush_volumetric_speed`,
  `filament_flush_temp`, `filament_cooling_before_tower`,
  `filament_adaptive_volumetric_speed`) plus all four Region fields;
- empty concrete vector reset to typed default (8):
  `filament_max_volumetric_speed` plus all seven Print fields;
- never reset (17): string tuple `volumetric_speed_coefficients` plus all 16
  Retract override fields. Identity determines `N` and is not reset.

Then normalize identity and all 36 data vectors: zero clears, excess truncates,
and a nonempty short vector grows by cloning its first value. At `N > 0`, any
vector still empty after the permitted reset returns `InvalidInput` naming its
literal option key.

For each descendant, `M` is its explicit identity length or one when omitted.
Normalize only present family fields, never reset nil/defaults, and reject a
present empty vector when `M > 0`. An explicit empty identity gives `M == 0`.
Build its identity mapping once before visiting data fields. Then, for every
present data field, the application order is invariant:

1. normalize the child vector;
2. compare the normalized child directly with accumulated source using the
   fixed equality class; an equal field is a no-op;
3. using the retained-root `Vec<Option<usize>>` mapping, when source length
   differs from mapping length, whole-copy the child before any slot read;
4. otherwise keep unmatched slots, skip mapped child `Nil`, and copy mapped
   concrete slots. A missing mapped child slot is `InvalidInput` with the key.

With an empty accumulated identity, the mapping is still `[Some(0)]`. Thus an
`N == 0` first child with an implicit one-slot identity reaches the
source-length fallback, while a later one-slot source applies nil/value slot
semantics. Identity always remains the normalized root identity. Omitted or
explicitly empty child identity maps only source slot zero; otherwise exact
string matching uses the first child match, ignores child-only variants, and
keeps unmatched sources. There is no ID tie-breaker.

## Review and workspace discipline

Before Package T0, record the spec hash, `git rev-parse HEAD`, branch, status,
baseline/allowlist hashes, command exit codes, test manifests, package hashes,
and reviewer identities in ignored `.superpowers/sdd/task20a2-evidence.md`.
Never stage that file. Preserve unrelated changes. Inspect untracked files by
full read and `git diff --no-index -- /dev/null <path>` because ordinary
`git diff` omits them.

For each package below:

1. dispatch one fresh implementer with only its owned paths, the approved spec
   and plan hashes, dependencies, RED evidence, and acceptance commands;
2. inspect the complete owned patch and freeze path/SHA-256 hashes;
3. dispatch a fresh spec-compliance reviewer and a different fresh
   code-quality reviewer; both must return literal `VERDICT: APPROVE`;
4. on any revision, use a bounded fixer, rerun applicable checks, refreeze,
   and rerun both reviews. Do not commit between packages.

O1 and O2 may run in parallel after H because their paths are disjoint. No
other packages overlap. Rust files must remain below 400 physical lines, with
working targets `option_group.rs < 365`, `gcode_source.rs < 381`, and the new
focused test `< 390` to retain review headroom.

## Exact tracked manifest

**Create:**

- `crates/ares-core/src/profiles/tests/filament_variant_inheritance.rs`
- the approved specification file as an already-frozen artifact
- this plan file

**Modify:**

- `crates/ares-core/src/options.rs`
- `crates/ares-core/src/options/option_group.rs`
- `crates/ares-core/src/options/filament_options.rs`
- `crates/ares-core/src/options/filament_options/gcode_source.rs`
- `crates/ares-core/src/options/filament_options/print_source.rs`
- `crates/ares-core/src/options/filament_options/region_source.rs`
- `crates/ares-core/src/options/filament_options/retract_overrides.rs`
- `crates/ares-core/src/profiles/inheritance.rs`
- `crates/ares-core/src/profiles/tests/mod.rs`
- `crates/ares-core/src/profiles/tests/inheritance.rs`
- `scripts/dynamic_value_baseline.txt`
- after whole approval only: `docs/architecture/option-parity-v4.md` and
  `docs/roadmap.md`

**Delete exactly:**

- `crates/ares-core/src/options/update_diff_values_to_child_config.rs`
- `crates/ares-core/src/options/update_diff_values_to_child_config/tests.rs`
- `crates/ares-core/src/options/update_diff_values_to_child_config/tests/full_update.rs`

No other tracked path may change. In particular,
`profiles/tests/composition_multi.rs` already has symmetric typed composition
normalization and must remain byte-identical. Any indispensable extra path
requires a spec revision and fresh dual spec approval before it is touched.

---

## Package T0: Freeze the complete RED contract

**Owned paths:**

- `crates/ares-core/src/profiles/tests/mod.rs`
- `crates/ares-core/src/profiles/tests/inheritance.rs`
- `crates/ares-core/src/profiles/tests/filament_variant_inheritance.rs`

This package is test-only. Do not modify production or debt files.

### T0.1: Rewrite only the two superseded Task 20A.1 regressions

Rename or narrowly rewrite
`present_nullable_vector_replaces_the_whole_parent_vector` and
`omitted_nullable_vector_retains_the_parent_value`. Both final expectations
are exactly:

```rust
vec![Nullable::Value(OrcaFloat(0.9))]
```

The present-child case proves the three-element root truncates to the implicit
one-element root identity and child `Nil` inherits slot zero. The omitted-child
case proves the two-element root truncates to the same one-element identity
before omission retains it. Change no other test in `inheritance.rs` and no
composition test.

### T0.2: Add the focused public behavioral matrix first

Wire `mod filament_variant_inheritance;` and use small synthetic JSON profile
bytes through `merge_profile_fragments`. Group cases compactly under the final
prefix `profiles::tests::filament_variant_inheritance::` and cover:

- test-side exact owner inventory: one identity plus GCode 9, Print 7, Region
  4, Retract 16 unique data fields, each checked against the existing concrete
  owner's `DECLARATION_ORDER`; integrated structural review later proves the
  owner-local operation lists match this frozen test contract exactly;
- root default expansion across a two-variant identity and reordered child;
- all-nil non-override reset versus all-nil Retract preservation;
- root and child grow-first, truncate, explicit zero cardinality, and every
  positive-cardinality empty class (GCode nullable/non-nullable float, Print
  int/bool, SpaceTuple, Retract), including child empties never resetting;
- public `N == 0` root/child/grandchild flow: identity stays empty,
  `[] -> [Value(1.2)]` by fallback, later `Nil` retains it, and an omitted
  family field remains accumulated; include explicit-empty-child identity
  cases that prove equality avoids a read and that equal source/map length
  with missing slot zero errors; the direct helper test below proves fallback
  avoids that read;
- omitted data, reordered identity, first duplicate-child match, unmatched
  source, child-only variant, and a later descendant still mapping against the
  retained root identity;
- non-nullable GCode and Print mapping plus nullable Region/Retract nil
  inheritance and concrete override;
- reversed-identity equality short-circuit for exact equality, nullable float
  and percent deltas below `1e-4`, a zero-to-`0.0001` strict unequal boundary,
  and a non-nullable `filament_max_volumetric_speed` sub-epsilon delta that is
  still exact and therefore maps;
- present non-variant `filament_type` whole replacement and unchanged input
  fragments/error atomicity;
- representative malformed/wrong-shape family JSON remaining keyed
  `InvalidInput`, without freezing incidental serde wording.

Before adding the direct private-helper test, run and record a behavioral RED:

```powershell
cargo +1.91.0 nextest list -p ares-core `
  -E 'test(/filament_variant_inheritance/)'
cargo +1.91.0 nextest run -p ares-core `
  -E 'test(/filament_variant_inheritance/)' --no-capture
cargo +1.91.0 nextest run -p ares-core `
  -E 'test(/^profiles::tests::inheritance::(present|omitted)_nullable_vector/)'
```

Require a nonempty fixed test-name set and nonzero behavioral failures against
Task 20A.1 for the new semantics. Freeze the names and file hash.

### T0.3: Add the direct helper compile RED last

Add exact test
`variant_source_length_mismatch_whole_replaces_child` against the planned
crate-private typed helper. It must prove equality is tested first, then a
source/mapping length mismatch whole-copies the normalized child without a
slot read. `nextest list` may now fail only because that helper does not yet
exist. Record the exact missing-symbol/compiler diagnostic; any other compile
failure is a test defect.

Tests must not call OrcaSlicer, read the KSR fixture/reference, include source
pins, or inspect a runtime registry. Freeze and independently approve the
complete T0 bytes before D.

---

## Package D: Close the exact eight-row dynamic debt

**Owned paths:**

- `scripts/dynamic_value_baseline.txt`
- `crates/ares-core/src/options.rs`
- the three exact obsolete paths to delete

### D.1: Prove the baseline RED while source still exists

At baseline, require 683 rows, baseline SHA-256
`c904133ca48ac46620046f14dcfbf4a8bbd6597841d9eb0e21760ca8188cd166`,
and allowlist SHA-256
`6b9c3ba6a1c52118a14d66f607cf85a9d13c27185b1fa22d670983e9371a94b6`.
Select the eight rows whose ordinal prefix is
`crates/ares-core/src/options/update_diff_values_to_child_config.rs#`, sort
with `[StringComparer]::Ordinal`, encode UTF-8 without BOM with LF and one
terminal LF, and require eight rows plus SHA-256
`93ee0515d6afb622094a9d7ca4b24753f63e15e822e01d3c6c6222ecb3a87fb0`.

Remove only those eight rows with `apply_patch`. Before deleting source, run:

```powershell
cargo +1.91.0 nextest run -p ares-core `
  --test no_unapproved_dynamic_values `
  -E 'test(=no_unapproved_dynamic_values)' --no-capture
```

Require nonzero exit and exactly those eight `new dynamic value:` findings,
with no ninth finding and no missing/moved baseline row.

### D.2: Delete the scaffold and turn the ratchet GREEN

Confirm `rg` finds no non-test production caller. Remove the module declaration
from `options.rs`, delete exactly the module and its two test files, and do not
adapt any function. The retained baseline must be exactly 675 rows, 76,581
UTF-8/LF bytes, SHA-256
`0dcea4c112ef10f0d6e8c8ee7f63cfef1831d7c2ae2e399016f1e38372543be7`,
with unchanged empty allowlist.

Run the exact audit above and the full audit binary; both must pass. Run
`cargo +1.91.0 check -p ares-core --lib` and rustfmt check. Review D until both
package reviewers approve.

---

## Package H: Add only zero-cost typed helpers

**Owned path:** `crates/ares-core/src/options/option_group.rs`

Extend `declare_option_group!` with a crate-private concrete-group operation
that applies only `Some` fields from a builder. Variant owner methods will
`take()` their family fields before invoking it, so no variant field or
identity can fall through ordinary assignment.

Add the minimum typed/vector helpers needed by the fixed family:

- compile-time access to `Vec<T>`, `OrcaFloats`, `OrcaInts`, `OrcaBools`,
  `SpaceTuple`, and `VariantStride` storage;
- clear/truncate/grow-by-first normalization with a literal-key
  `InvalidInput` for positive-target empty vectors;
- local exact and nullable-float/percent approximate comparisons without
  changing global `PartialEq`;
- typed `Vec<Option<usize>>` slot application whose order is
  equality short-circuit, source-length fallback, then checked slot read; nil
  behavior is supplied statically by the concrete call site.

Literal keys are diagnostic arguments only, never dispatch input. Do not add
`serde_json::Value`, `SliceOptions`, registry lookup, serialization, a dynamic
adapter, a runtime key loop, or broad public API.

Run:

```powershell
cargo +1.91.0 check -p ares-core --lib
cargo +1.91.0 nextest run -p ares-core `
  -E 'test(=profiles::tests::filament_variant_inheritance::variant_source_length_mismatch_whole_replaces_child)'
cargo +1.91.0 fmt --all -- --check
```

The direct helper test must turn GREEN; the public focused matrix must remain
behaviorally RED. Keep `option_group.rs < 365` lines and review H to dual
approval.

---

## Packages O1 and O2: Implement concrete owners in parallel

Dispatch these only after H approval. Agents must not touch facade, profile,
tests, debt, or each other's files. Owner-local compile-time field lists may
generate repetitive operations and a `#[cfg(test)]` inventory view; production
must be statically unrolled and must not expose a runtime registry.

### Package O1: GCode identity and 9 data fields

**Owned path:**
`crates/ares-core/src/options/filament_options/gcode_source.rs`

Implement root normalization for identity plus the nine GCode data fields,
child identity extraction/pre-normalization, and child application for the
four approximate and five exact fields in the ledger. Root reset is seven
all-nil nullable fields plus empty `filament_max_volumetric_speed`;
`volumetric_speed_coefficients` never resets. Identity is mapping input only,
is exact, and never ordinary-assigned. All other GCode fields remain ordinary
present-field replacements.

Expose only the crate/super-private concrete methods needed by F. Keep the
existing declaration order/defaults/serde unchanged and
`gcode_source.rs < 381` lines.

### Package O2: Print, Region, and Retract data fields

**Owned paths:**

- `crates/ares-core/src/options/filament_options/print_source.rs`
- `crates/ares-core/src/options/filament_options/region_source.rs`
- `crates/ares-core/src/options/filament_options/retract_overrides.rs`

Implement the same concrete root/child operations for:

- Print: seven exact non-nullable vectors; every empty root resets, every
  positive-target empty child errors;
- Region: four approximate nullable vectors; all-nil roots reset, child nils
  remain sparse inheritance markers;
- Retract: 11 approximate and five exact nullable vectors; none of the 16
  roots reset, and mapped child nils preserve source.

All other owner fields retain ordinary present-field replacement. Do not
change defaults, serde, declaration order, or append behavior.

For each package, run rustfmt and `cargo +1.91.0 check -p ares-core --lib`.
Unused crate-private-method warnings are acceptable at this intermediate seam
but must not be hidden with `allow`/`expect`; F must eliminate them. Review O1
and O2 separately to dual approval before F.

---

## Package F: Wire the filament facade and profile resolver

**Owned paths:**

- `crates/ares-core/src/options/filament_options.rs`
- `crates/ares-core/src/profiles/inheritance.rs`

### F.1: Coordinate typed normalization/mapping in the facade

Replace the filament aggregate's old builder-overlay inheritance seam with two
private operations:

1. resolve the root builder once, derive `N` from its concrete identity, and
   normalize all four owners against typed defaults using the root ledger;
2. apply a sparse descendant by extracting/normalizing identity, deriving
   `M`, building the first-match `Vec<Option<usize>>` mapping from the retained
   root identity, stripping all 37 family fields from ordinary present-field
   application, and delegating the 36 data fields to their typed owners.

`pellet_flow_coefficient` and every other present non-variant field retain
whole replacement. Remove the obsolete `FilamentOptionsBuilder::overlay`
entry point/import if no longer used; do not leave a parallel whole-vector
filament inheritance fallback.

### F.2: Change only the filament resolver arm

Keep chain collection, metadata merge, and machine/process arms byte-for-byte
semantically unchanged. In the filament arm:

- resolve and normalize `chain[0]` as the concrete root;
- apply each later sparse builder in chain order without resolving it;
- return the final concrete `MergedProfile::Filament` or the typed keyed
  `InvalidInput`.

The local accumulator may be mutated before an error because no partial result
escapes, but fragments must remain unchanged. Do not change public signatures,
composition, project slicing, or metadata behavior.

Run:

```powershell
cargo +1.91.0 nextest list -p ares-core `
  -E 'test(/filament_variant_inheritance/)'
cargo +1.91.0 nextest run -p ares-core `
  -E 'test(/filament_variant_inheritance/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/profile/)'
cargo +1.91.0 check -p ares-core --lib
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets --all-features -- -D warnings
```

Require the focused name set to equal frozen T0, both rewritten one-element
regressions GREEN, the direct fallback test GREEN, the complete profile suite
GREEN, and no warning suppression. Review F to dual approval.

---

## Integrated package gate

Freeze the six package submanifests and complete implementation manifest.
Re-run fresh spec-compliance and code-quality review for T0, D, H, O1, O2,
and F against the integrated bytes and GREEN evidence. Each must again end in
literal `VERDICT: APPROVE`; a correction invalidates affected hashes and both
reviews.

Run the focused/regression gates exactly:

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/filament_variant_inheritance/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/profile/)'
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
cargo +1.91.0 nextest run -p ares-core config_export
cargo +1.91.0 nextest run -p ares-core project
```

Require 675 baseline rows, unchanged allowlist, no obsolete module, exact
49,004-byte KSR config block with SHA-256
`b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`,
and valid project slicing still ending at `ProjectSlicingIncomplete`.

## Freeze, structural audits, and whole implementation reviews

Build ignored SHA-256 manifests for every tracked/untracked path and the
complete patch. Require `git diff --check` plus no-index whitespace checks for
untracked paths. Reject anything outside the exact manifest and explicitly
prove `composition_multi.rs` and the dynamic allowlist are byte-identical to
the Ares baseline.

Audit added production/test lines and full new files for:

- no `ksr_fdmtest_v4`, fixture/reference hash, 49,004 constant, timestamp, or
  `generated by` behavioral branch;
- no new `serde_json::Value`, `SliceOptions`, registry dispatch, string-key
  loop, serialization round-trip, dynamic adapter, source pin, filesystem I/O,
  or legacy fallback in the typed path;
- no equality-to-default presence inference and no global `PartialEq` change;
- exact 1+36 field ownership, 19 approximate/17 exact equality, 11/8/17 root
  reset partition, no-ID stride one, and equality-before-fallback-before-slot;
- no added `#[allow(...)]` or `#[expect(...)]` warning suppression;
- every changed Rust file below 400 physical lines.

Run the fresh release matrix:

```powershell
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets --all-features -- -D warnings
cargo +1.91.0 check --workspace --all-targets --all-features
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
cargo +1.91.0 build -p ares-wasm --release --target wasm32-unknown-unknown
cargo +1.91.0 nextest run -p ares-cli
cargo +1.91.0 install --locked wasm-bindgen-cli --version 0.2.121
wasm-bindgen --version
wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm `
  --target web --out-dir target/wasm-browser
npm --prefix crates/ares-wasm/tests/browser ci
npm --prefix crates/ares-wasm/tests/browser audit --audit-level=low
npx --prefix crates/ares-wasm/tests/browser playwright install chromium
npm --prefix crates/ares-wasm/tests/browser test
```

Require `wasm-bindgen 0.2.121`, zero npm vulnerabilities, and the real-project
headless Chromium test GREEN. On Windows, capture `$LASTEXITCODE` immediately;
do not use Playwright `--with-deps`.

Dispatch three fresh reviewers against the identical frozen manifest, patch,
and evidence:

1. whole-spec implementation reviewer: literal `VERDICT: APPROVE`;
2. whole-code-quality reviewer: literal `VERDICT: APPROVE`;
3. default-model OpenCode implementation reviewer, invoked without `-m`:
   literal `VERDICT: APPROVE`.

Any revision requires a focused regression where applicable, rerunning
affected checks, rebuilding all hashes, and rerunning all three whole reviews.
Do not update tracked architecture/roadmap docs before all three approve.

## Documentation gate

Only after whole approval, modify:

- `docs/architecture/option-parity-v4.md`
- `docs/roadmap.md`

First correct the stale Task 20A.1 status to released commit
`e0c50564283744b3dd3388eeaa10f624a492ff1f` and exact-SHA Tier 1 run
`29488449752`. Then document only approved Task 20A.2 behavior: the pinned
37-key filament/no-ID/stride-one boundary; concrete root default/is-nil
normalization; sparse descendant identity mapping and retained root identity;
19 approximate versus 17 exact equality; nullable nil, `N == 0`, and
source-length fallback semantics; exact deletion of eight findings with 675
retained; and unchanged `ProjectSlicingIncomplete` boundary.

Keep process/printer variants, stride two, profile-to-project wiring, remaining
Task 20A and Tasks 20B-20E, geometry, G-code, metadata byte parity, and full KSR
parity explicitly deferred. Do not call Task 20A.2 released before exact-SHA
Tier 1.

Require a fresh reviewer to return:

```text
ROLE: DOCUMENTATION
VERDICT: APPROVE
```

Revise/re-review until approved. Add docs to the frozen final manifest and
rerun the complete focused gates and release matrix from approved doc bytes.
Any implementation change invalidates whole and documentation approvals.

## Conventional commit, push, and exact-SHA Tier 1

Apply the Conventional Commits skill only after all approvals and the fresh
post-documentation matrix are green.

Stage only the frozen manifest; never use `git add -A`:

```powershell
git status --short
git diff --check
git add -- <exact reviewed manifest paths>
git diff --cached --name-status
git diff --cached --check
```

Confirm ignored evidence, generated WASM/npm output, fixture/reference files,
pinned Orca checkout, allowlist, `composition_multi.rs`, and unrelated user
changes are not staged. Use reviewed subject:

```text
feat(profiles): inherit filament variants by slot
```

Push normally without force:

```powershell
git push origin codex/ksr-fdmtest-v4-parity
```

If remote advanced, fetch/rebase without dropping user changes, rerun relevant
verification, and push normally. Then require local/tracking/direct remote SHA
identity and a clean worktree:

```powershell
$branch = 'codex/ksr-fdmtest-v4-parity'
$local = git rev-parse HEAD
$tracking = git rev-parse "origin/$branch"
$direct = ((git ls-remote origin "refs/heads/$branch") -split '\s+')[0]
git status --short
```

Locate only the Tier 1 push run whose `headSha` equals `$local`, watch it to
completion, and require all five jobs GREEN:

```powershell
gh run list --workflow tier1.yml --branch $branch --commit $local --event push `
  --json databaseId,headSha,status,conclusion,createdAt --limit 10
gh run watch <exact-run-id> --exit-status
gh run view <exact-run-id> --json headSha,conclusion,jobs
```

Required jobs are `format`, `ubuntu-latest`, `wasm`, `macos-latest`, and
`windows-latest`. Only then record Task 20A.2 as released in ignored evidence.
The persistent full-G-code-parity goal remains active.

## Plan exit criteria

This plan is complete only when exact spec and plan bytes were dual-approved
before implementation; all six packages and their integrated bytes received
dual approval; the 1+36 inventory, 19/17 equality partition, 11/8/17 root nil
partition, root/child resizing, `N == 0`, mapping, nil, fallback, and both
one-element regressions are GREEN; exactly eight dynamic findings were removed
with 675 retained; whole spec/quality/default OpenCode and documentation
reviews approved; the fresh native/WASM/browser matrix passed; only the frozen
manifest was conventionally committed and pushed normally; local/tracking/
direct SHAs match; and all five exact-pushed-SHA Tier 1 jobs are GREEN.

**Status: DRAFT — production and test implementation is forbidden until a
fresh independent Codex plan reviewer and the required default-model OpenCode
plan reviewer both return literal `VERDICT: APPROVE` for these exact bytes.**
