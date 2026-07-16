# Task 20A.1: Typed Profile Composition Implementation Plan

> **Execution contract:** Follow the approved SDD workflow and this checklist
> in order. No production implementation may begin until this plan receives
> literal `VERDICT: APPROVE` from both a fresh independent Codex reviewer and
> the required default-model OpenCode reviewer. Use fresh bounded implementer
> and reviewer subagents for every implementation slice. Do not commit between
> slices. Commit and push only after whole-spec implementation approval,
> documentation approval, and a fresh release matrix.

**Approved specification:**
`docs/superpowers/specs/2026-07-14-ksr-fdmtest-v4-task20a1-typed-profile-composition.md`

**Approved specification SHA-256:**
`D9D8A8F9B559AE9239D987A5184FC948C16BBAEF86E21C16D652E6186BBD0A0F`

**Pinned Orca baseline:**
`8500fcdccaa10b5099ac20d252af3a7c560046f1`

**Ares baseline SHA:**
`656b32f987827b29d08010802ba03ef6ba822980`

## Goal and fixed source boundary

Replace the remaining dynamic profile fragment, same-kind inheritance, and
selected FFF profile composition code with concrete sparse typed option
owners. The source boundary is:

- `Preset.hpp:22-24,43-65` for kinds and profile metadata;
- `Preset.cpp:491-504,1476-1494,1622-1703,3112-3140` for kind ownership,
  loading, direct-parent resolution, and parent-first overlay;
- `PresetBundle.cpp:3884-4165::full_fff_config` for the
  `apply_extruder=false`, no-`filament_maps_new` composition subset;
- `PrintConfig.hpp:610-682,695-914,916-1666` for the upstream dynamic load
  shell and concrete FFF owners being represented by existing Ares types.

`PresetBundle.cpp:68-242::construct_full_config`, including its calibration
call at `CalibUtils.cpp:937`, is explicitly excluded. The implementation must
not connect profiles to `slice_project`, retained STL planning, geometry,
toolpaths, or G-code. A valid project continues to end with
`ProjectSlicingIncomplete` after the released Task 19C config writer.

The final implementation removes exactly the 29 approved profile findings
from `scripts/dynamic_value_baseline.txt`, leaving 683 rows and no allowlist
addition. It does not retain a dynamic bridge, compatibility map, JSON DOM,
serialization round-trip, runtime option registry, or fixture/reference
branch.

## Review and evidence discipline

Before Task 1, freeze and record:

```powershell
$spec = 'docs/superpowers/specs/2026-07-14-ksr-fdmtest-v4-task20a1-typed-profile-composition.md'
(Get-FileHash $spec -Algorithm SHA256).Hash
git rev-parse HEAD
git status --short
```

Require the exact spec and baseline SHAs above. Record commands, exit codes,
test counts, reviewer identities, and hashes in ignored
`.superpowers/sdd/task20a1-evidence.md`; never stage evidence files.

Task 1 freezes all public behavioral and structural REDs before production
changes. Task 2 then uses four fresh implementers for four bounded work
packages inside the one atomic fragment/composition migration unit required by
the spec. Each package receives a bounded independent review, but no partial
package is called behaviorally GREEN or released. After the fourth package,
the complete public suite must turn GREEN and every package is reviewed again
against the integrated bytes and test evidence until literal
`VERDICT: APPROVE`.

For every work package, give the implementer an exact path list plus approved
spec/plan hashes, inspect every tracked and untracked path, freeze an owned
path/hash manifest, and give a fresh reviewer the complete owned patch. Fix
every finding, rerun the applicable compile/integration gate, refreeze, and
re-review before the atomic unit can advance.

For untracked files, an ordinary `git diff` is insufficient. Review and check
them with `git diff --no-index -- /dev/null <path>` as well as full file reads.
Preserve unrelated user changes and never include them in a slice or final
manifest.

The builder-presence primitive is accepted only through the shared public
parser/inheritance RED and final GREEN. This is deliberate: the approved spec
forbids tests from inspecting private builder state. No private builder field,
presence bitmap, or test-only production accessor is exposed.

## Planned file boundary

Exact private names may be simplified during implementation, but ownership
must remain equivalent and every Rust file must stay below 400 physical lines.

**Create:**

- `crates/ares-core/src/profiles/fragment/metadata.rs`
- `crates/ares-core/src/profiles/fragment/payload.rs`
- `crates/ares-core/src/profiles/inheritance.rs`
- `crates/ares-core/src/profiles/composition/filament.rs`
- `crates/ares-core/src/profiles/composition/metadata.rs`
- `crates/ares-core/src/profiles/tests/mod.rs`
- focused modules under `crates/ares-core/src/profiles/tests/`, expected as:
  - `fragment_parsing.rs`
  - `inheritance.rs`
  - `errors.rs`
  - `composition_single.rs`
  - `composition_multi.rs`
  - `composition_metadata.rs`
- `crates/ares-core/tests/profile_public_api.rs` for an external-crate public
  visibility/shape smoke test
- a crate-private typed append helper beside the option declaration macros,
  only if the minimum opt-in macro cannot remain in `option_group.rs`
- `crates/ares-core/tests/no_unapproved_dynamic_values/profile_shell.rs`
- `crates/ares-core/tests/no_unapproved_dynamic_values/profile_shell/identity.rs`
  solely for the guard's declaration/import/public-re-export identity index,
  resolver, and focused symbol-resolution mutations

**Modify narrowly:**

- `scripts/dynamic_value_baseline.txt`
- `crates/ares-core/tests/no_unapproved_dynamic_values.rs`
- `crates/ares-core/src/options/option_group.rs`
- `crates/ares-core/src/options.rs` for crate-private builder/module wiring
- `crates/ares-core/src/options/printer_options.rs`
- `crates/ares-core/src/options/process_options.rs`
- `crates/ares-core/src/options/filament_options.rs`
- the four fixed filament declaration modules:
  - `filament_options/gcode_source.rs`
  - `filament_options/print_source.rs`
  - `filament_options/region_source.rs`
  - `filament_options/retract_overrides.rs`
- `crates/ares-core/src/profiles/fragment.rs`
- `crates/ares-core/src/profiles/composition.rs`
- `crates/ares-core/src/profiles/mod.rs`
- `crates/ares-core/src/lib.rs`
- `crates/ares-core/src/tests/mod.rs` only to remove the two obsolete
  map-to-retained-STL profile tests
- approved architecture and roadmap documents only after whole implementation
  approval

Do not modify the project reader/resolver, `project_slice.rs`, retained STL
pipeline modules, G-code modules, CLI/WASM APIs, committed KSR fixture or
reference G-code, dynamic allowlist, pinned Orca checkout, or Task 19C config
writer. Do not add an executable source-pinning test.

---

## Task 1: Freeze the ratchet and establish the complete public RED suite

**Files:**

- Modify `scripts/dynamic_value_baseline.txt`.
- Create `tests/no_unapproved_dynamic_values/profile_shell.rs` plus its
  `profile_shell/identity.rs` test-only identity-resolver helper and wire the
  guard in `tests/no_unapproved_dynamic_values.rs`.
- Create `profiles/tests/mod.rs` and all focused test modules listed in the
  planned boundary.
- Create `crates/ares-core/tests/profile_public_api.rs`.
- Modify `profiles/mod.rs` only to add `#[cfg(test)] mod tests;`.

This is a test-only preparation slice. It must not change option or profile
production behavior.

### Step 1.1: Reproduce the exact owned baseline bytes and audit RED

Read current baseline lines 684 through 712 without culture-sensitive sorting.
Encode the existing Rust `BTreeSet` ordinal order as UTF-8 without BOM, LF
between rows, and one terminal LF. Require:

```text
29 rows
373e1a695854439c94e33220b1fdd47c74bad5842fef4489ccc03408ced0fe55
```

If a local recomputation sorts values, use
`[Array]::Sort($rows, [StringComparer]::Ordinal)`. Do not use PowerShell
`Sort-Object` or `Set-Content` to establish canonical bytes.

Delete exactly those 29 rows. Require 683 retained rows, every other byte and
row in its original order, and an unchanged `dynamic_value_allowlist.toml`.

Run:

```powershell
cargo +1.91.0 nextest run -p ares-core `
  --test no_unapproved_dynamic_values `
  -E 'test(=no_unapproved_dynamic_values)' --no-capture
```

Capture `$LASTEXITCODE` immediately. Require nonzero exit and exactly the
frozen 29 `new dynamic value:` rows, with no missing, extra, renamed, or moved
fingerprint.

### Step 1.2: Add the syntax-aware profile-shell guard and prove RED

Add a focused `syn`-based test over production modules rooted at
`crates/ares-core/src/profiles/`, reusing the audit's production-file and
`#[cfg(test)]` exclusion rules. It must reject exact production uses of:

- `SliceOptions`, `serde_json::{Value, Map}`, `RawValue`, and an erased `Any`
  owner;
- `json!`, `from_value`, `to_value`, and equivalent JSON round-trips;
- a profile option-map `.values()` contract.

It must not reject comments, tests, `IgnoredAny`, or unrelated method names.
The `.values()` rule is driven by a resolved `SliceOptions` type/alias or the
fixed public profile-shell owner, never by a receiver variable's spelling.
Keep production scanning/orchestration in `profile_shell.rs`; the sibling
`profile_shell/identity.rs` may own only the compact symbol identity index and
resolver needed to keep both Rust files below 400 physical lines. It must
resolve local declarations before imports, follow public re-export chains for
the fixed API owners, and resolve impl self types through `crate`, `self`,
repeated `super`, named/renamed imports, and glob imports. It must not use a
last-segment type-name fallback. A public item inside a private module is not a
fixed public owner unless it is reachable through the approved public
re-export chain.

Synthetic mutation tests must cover a glob-imported `SliceOptions` alias; a
root fixed-owner declaration with an impl in a child module; `crate`, `self`,
repeated-`super`, named/renamed-import, glob-import, and public-re-export owner
paths; a private nested non-public same-name owner; a `pub` same-name owner in
a private non-re-exported module; an unrelated receiver named `options`; and
`IgnoredAny`.
This is an AST guard, not an `rg`-only assertion. Run it and require RED against
the old fragment/composition pair.

### Step 1.3: Wire deterministic final test names

Wire all files through `profiles/mod.rs -> profiles/tests/mod.rs`. Their final
fully-qualified prefixes are fixed as:

```text
profiles::tests::fragment_parsing::
profiles::tests::inheritance::
profiles::tests::errors::
profiles::tests::composition_single::
profiles::tests::composition_multi::
profiles::tests::composition_metadata::
```

The fragment expression is:

```text
test(/^profiles::tests::(fragment_parsing|inheritance|errors)::/)
```

The composition expression is:

```text
test(/^profiles::tests::composition_(single|multi|metadata)::/)
```

The complete expression is `test(/^profiles::tests::/)`. Do not use
`test(/profile/)`, which currently selects unrelated tests.

The external integration test is rooted at exact test name
`public_profile_api_is_externally_usable` in the `profile_public_api` test
binary. It imports only through `ares_core::{...}` and proves that
`ProfileFragment`, `ProfileKind`, `MergedProfile`, merged metadata and concrete
options, `ProfileSelection`, `ComposedProfile`, `ProfileGroupMetadata`, and the
merge/compose functions are externally public. Crate-internal unit tests do
not substitute for this visibility proof.

Freeze it with exact commands:

```powershell
cargo +1.91.0 nextest list -p ares-core `
  --test profile_public_api `
  -E 'test(=public_profile_api_is_externally_usable)'
cargo +1.91.0 nextest run -p ares-core `
  --test profile_public_api `
  -E 'test(=public_profile_api_is_externally_usable)'
```

When the test compiles, the list command must select exactly one test. At
initial RED, listing may fail only because the approved public typed symbols
are not yet exported.

Before each later GREEN run, execute `cargo +1.91.0 nextest list` with the same
expression and compare the nonempty exact test-name set with the frozen Task 1
test manifest. During initial compile RED, `nextest list` itself may fail only
for the new typed API symbols or type expectations; record those exact errors.
If listing succeeds, require the selected set to be nonempty before running it.

### Step 1.4: Establish complete fragment/inheritance RED

All tests use JSON bytes, public fragment accessors, exhaustive
`MergedProfile` matching, and public resolved option groups. They never name,
read, or assert a private builder or presence field. Cover:

- metadata-first, options-first, and `type`-last member order;
- all `ProfilePresetMetadata` fields: required `type`/nonempty `name`, optional
  `from`, `version`, `setting_id`, `instantiation`, `description`, `url`,
  `renamed_from`, and filament-only `filament_id`;
- exact loader strings for `version`/`instantiation`, while `renamed_from` is
  parsed but not used as an alias;
- malformed/non-object/trailing JSON;
- missing, duplicate, wrong-type, unsupported, or empty required metadata;
- duplicate optional/local metadata and invalid optional metadata types;
- direct machine, process, and filament typed decode;
- unknown, misplaced, duplicate, and malformed concrete option rejection;
- process/filament compatibility ownership and machine rejection of all four
  compatibility keys;
- missing and explicit empty `inherits` as equivalent roots and positional
  empty composition slots;
- grandparent/parent/child parent-first whole-field overlay;
- omission retaining parent data, explicit fixed default overriding parent,
  and present child vector replacing the complete parent vector while
  preserving typed nullable elements;
- compatibility omission, override, and explicit-empty clear;
- selected loader-local identity not inheriting, except a child filament uses
  its resolved parent's ID while a root retains its own ID;
- duplicate fragment identity, missing target, missing/cross-kind parent,
  self-parent, and longer cycle;
- input-order independence;
- exhaustive tagged result with the kind-correct resolved concrete owner;
- every error returning `InvalidInput` with stable profile/option category
  context but without freezing incidental serde wording, no partial result,
  and input fragments remaining byte-for-byte/equality unchanged.

Require compile or behavioral RED against the old dynamic API.

Run the external visibility test separately and require compile RED because
the new public tagged/metadata/settings contract is not exported yet. It must
not use `crate::`, private modules, or a test-only re-export.

### Step 1.5: Establish complete composition RED

Cover through public `compose_profile_fragments`:

- valid single-filament `ProjectSettings` and selected-name accessors;
- two-filament append order for numeric, bool, string, enum, nullable,
  explicit empty, `VariantStride`, and another opaque string-vector owner;
- compile-time coverage of the fixed 122 vector-like filament fields;
- `print_settings_id`, `printer_settings_id`, `filament_settings_id`,
  `filament_map`, and positional `filament_ids`, including empty/all-empty IDs;
- `inherits_group` in process, filaments, machine order;
- compatible-machine condition group in process then filaments order;
- compatible-process condition group in filament order;
- absence only when every group slot is empty, without compacting interior
  empty slots;
- `print_compatible_printers` absent/all-empty and present positional behavior;
- single/multi `filament_self_index` from fully resolved variant cardinalities;
- inherited and explicitly cleared process compatibility flowing through to
  composed `print_compatible_printers`;
- inherited filament IDs and inherited variant cardinality flowing through to
  composed `filament_ids` and `filament_self_index`;
- missing/empty selection and every missing selected-profile error;
- `settings()`/`into_settings()` returning `ProjectSettings`, never
  `SliceOptions`;
- atomic error behavior and unchanged input fragments/selection.

Freeze the test paths/names and independently approve the complete RED suite
before Task 2.

---

## Task 2: Implement the fragment/composition pair as one atomic review unit

The approved spec identifies `fragment.rs` and `composition.rs` as the smallest
closed ownership set. Changing the merge return before composition changes
would make the crate uncompilable; keeping it compiling would require the
forbidden dynamic bridge. Therefore Work Packages 2A-2D are bounded subagent
contributions to one atomic production migration, not separately releasable
partial implementations.

Use a fresh implementer for each work package and freeze its owned diff. Two
fresh independent reviewers perform bounded spec-compliance and code-quality
reviews after each contribution.
Where the pair is intentionally between signatures, review is static and the
expected integration compile error is recorded rather than hidden. After 2D,
all four packages receive fresh integrated reviews against the public GREEN
evidence and must each end in literal `VERDICT: APPROVE`. No package may add a
temporary adapter, old/new dual public path, or dynamic fallback.

### Work Package 2A: Sparse typed builder overlay primitives

**Files:** `options/option_group.rs`, `options.rs`, and the printer, process,
and filament aggregate builder modules.

In `declare_option_group!`, make sparse builders clonable and generate a
crate-private overlay that replaces a field only when the child contains
`Some(value)`. Leave default injection exclusively in `resolve`.

Add aggregate overlay delegation and handle only direct
`ironing_expansion`/`pellet_flow_coefficient` presence explicitly. Re-export
builders only at crate visibility. Add no presence bitmap, runtime field-name
lookup, default comparison, serde conversion, test accessor, or second option
table.

The public inheritance RED from Task 1 is the acceptance test; do not add a
private-builder white-box test. `cargo check -p ares-core --lib` and rustfmt
must remain green after 2A. Independently review the bounded option diff.

### Work Package 2B: Direct typed fragment parsing and inheritance

**Files:** replace/split `profiles/fragment.rs`; create
`fragment/metadata.rs`, `fragment/payload.rs`, and `inheritance.rs`; update only
the fragment-side profile wiring needed by these modules.

Delete the old inline fragment map-contract tests as part of replacing
`fragment.rs`; their complete public typed replacements already exist in the
Task 1 RED suite. Do not defer those now-invalid callers to 2D.

Implement two independent serde passes over borrowed input:

1. A metadata visitor reads explicit concrete local/config metadata and skips
   payloads using `IgnoredAny`.
2. A kind-carrying seed re-reads the bytes, skips local/config metadata, and
   dispatches every option directly into the selected sparse builder.

Call `Deserializer::end()` after each pass. Store no input bytes, generic tree,
raw value, unknown side map, or serializer output. Enforce exact kind ownership
using compile-time builder predicates only.

Represent loader-local metadata and the sparse config patch separately.
Process accepts printer compatibility list/condition; filament accepts those
plus print compatibility list/condition; machine accepts none. Missing and
empty `inherits` are roots. Compatibility values presence-overlay; selected
loader identity does not. Keep `version` exact and defer alias/Semver/UI logic.

Index unique `(ProfileKind, name)` values, detect every parent error/cycle,
overlay oldest parent through child, and resolve once. Return public tagged
`MergedProfile`. A child filament recursively inherits the resolved parent's
ID; a root uses its own.

Remove the dynamic fragment code and `values()` accessor without adding an
adapter. At this exact intermediate point, `cargo check --lib` is expected to
fail only where the still-old composition module consumes the changed merge
result. Record and review that narrow expected dependency; any other compiler
error is a defect. Rustfmt must pass. Do not run or claim partial profile GREEN.

### Work Package 2C: Concrete filament append and typed composition

**Files:** the opt-in append helper/macro, four filament declaration modules,
`filament_options.rs`, replaced/split `profiles/composition.rs`, and new
`composition/filament.rs`/`composition/metadata.rs`.

Delete the old inline composition map-contract tests as part of replacing
`composition.rs`; their complete public typed replacements already exist in
the Task 1 RED suite. Do not retain `options()`/`into_options()` assertions for
2D.

The fixed filament owner has 122 vector-like fields: 53 G-code, 48 print, four
region, 16 retract, and direct `pellet_flow_coefficient`. Generate append only
for the four opted-in filament groups beside their declarations. Support
concrete `Vec<T: Clone>` and the current concrete vector newtypes. A future
scalar must fail compilation until it receives an explicit first-filament
rule. Never inspect runtime shape, key names, tokens, registries, or serde.

Append fully inherited, once-resolved filaments in selection order. Record
each filament's resolved variant length before append.

Construct `ProjectSettings` directly from resolved printer/process/filament,
default `ProjectRuntimeOptions`, and default project `PresetMetadata`. Set only:

- `print_settings_id` and `printer_settings_id` from selected names;
- `filament_settings_id` from selected filament names;
- `filament_map` as one concrete `1` per selected filament;
- `filament_ids` with every positional slot preserved;
- `print_compatible_printers` only when any resolved process slot is nonempty;
- `filament_self_index` from recorded one-based variant cardinalities.

Build the three optional `ProfileGroupMetadata` vectors in the exact Task 1
orders and omit only all-empty groups. Expose typed settings/name/metadata
accessors and remove composition map helpers. Do not add project config,
extruder remapping, clamps, AMS count, forced technology, or `slice_project`
wiring.

After 2C, `cargo check -p ares-core --lib`, rustfmt, and warning-denying lib
Clippy must be green. Unit tests may still fail to compile only at the two
obsolete retained-STL tests assigned to 2D. Independently review the bounded
append/composition diff.

### Work Package 2D: Obsolete shell/API cleanup and exact closure

**Files:** profile/lib exports, `src/tests/mod.rs`, Task 1 test wiring, dynamic
baseline/audit guard, and only stale profile-shell paths revealed by the exact
RED evidence.

Delete the two old `slice_accepts_*_profile_options` tests; do not rewire them
to retained STL slicing. Remove old `options`, `into_options`, `values`,
`SliceOptions` returns/imports, and compatibility aliases. The owning 2B/2C
packages have already removed their superseded inline tests. Keep only the
typed public contract and Task 1 public tests.

Do not delete unrelated tests or change project/CLI/WASM slicing signatures.
Drive the AST guard and 29-row ratchet to GREEN with exactly 683 retained rows
and no allowlist change. Independently review this bounded cleanup diff.

---

## Task 3: Prove atomic integration GREEN and approve every package

### Step 3.1: Freeze and verify exact test selection

Run `nextest list` for the fragment, composition, and complete expressions
from Task 1. Require each set to be nonempty and exactly equal its frozen test
manifest. Then run all three expressions. No old inline/map-contract profile
test may remain.

List and run exact `public_profile_api_is_externally_usable`; require one
selected external integration test and GREEN through only `ares_core` exports.

```powershell
cargo +1.91.0 nextest list -p ares-core `
  --test profile_public_api `
  -E 'test(=public_profile_api_is_externally_usable)'
cargo +1.91.0 nextest run -p ares-core `
  --test profile_public_api `
  -E 'test(=public_profile_api_is_externally_usable)'
```

List the obsolete retained-STL expression and require zero matches:

```powershell
cargo +1.91.0 nextest list -p ares-core `
  -E 'test(/^tests::slice_accepts_(merged|composed)_profile_options$/)'
```

### Step 3.2: Prove structural and regression GREEN

```powershell
cargo +1.91.0 nextest run -p ares-core `
  -E 'test(/^profiles::tests::(fragment_parsing|inheritance|errors)::/)'
cargo +1.91.0 nextest run -p ares-core `
  -E 'test(/^profiles::tests::composition_(single|multi|metadata)::/)'
cargo +1.91.0 nextest run -p ares-core -E 'test(/^profiles::tests::/)'
cargo +1.91.0 nextest run -p ares-core `
  --test profile_public_api `
  -E 'test(=public_profile_api_is_externally_usable)'
cargo +1.91.0 nextest run -p ares-core `
  --test no_unapproved_dynamic_values `
  -E 'test(=profile_modules_use_only_typed_shells)'
cargo +1.91.0 nextest run -p ares-core `
  --test no_unapproved_dynamic_values `
  -E 'test(=no_unapproved_dynamic_values)'
cargo +1.91.0 nextest run -p ares-core `
  --test no_unapproved_dynamic_values
cargo +1.91.0 nextest run -p ares-core config_export
cargo +1.91.0 nextest run -p ares-core project
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy -p ares-core --all-targets --all-features -- -D warnings
```

Require 683 baseline rows, no allowlist addition or moved finding, and the
complete audit's one intentional print-only ignore. `config_export` retains the
exact 49,004-byte Task 19C fixture. `project` retains
`ProjectSlicingIncomplete` with no profile call.

### Step 3.3: Integrated per-package reviews

Freeze one complete atomic implementation manifest plus owned submanifests for
2A, 2B, 2C, and 2D. Dispatch a fresh spec-compliance reviewer and a different
fresh code-quality reviewer for each package against the integrated GREEN
bytes and its public acceptance tests. Both reviews must end in literal
`VERDICT: APPROVE`. Fix, rerun, refreeze, and re-review before Task 4.

---

## Task 4: Freeze the implementation and run whole reviews

### Step 4.1: Build exact path and hash manifests

Create ignored manifests containing every Task 20A.1 path and the production
subset. Include untracked files. Record SHA-256 for every file and the complete
tracked-plus-untracked patch. Run `git diff --check` and no-index whitespace
checks for each untracked path. Reject unrelated changes.

### Step 4.2: Run hardcoding, dynamic, pinning, and LOC audits

At minimum, prove:

- no production path contains `ksr_fdmtest_v4`, reference G-code hashes,
  golden byte counts, fixture timestamps, or `generated by` literals;
- production profiles contain no dynamic JSON/erased value owner, map shell,
  `SliceOptions`, registry lookup, serialization round-trip, raw profile byte
  storage, filesystem access, or test-only compatibility bridge;
- the AST profile-shell guard and complete dynamic audit pass;
- `scripts/dynamic_value_allowlist.toml` is byte-identical to baseline;
- no executable test pins Orca source lines, symbols, or source bytes;
- no changed production path adds `#[allow(...)]` or `#[expect(...)]` to hide
  a warning;
- every changed Rust file is below 400 physical lines.

`rg` may support manual audits, but an exit code of one for no match is not a
replacement for the syntax-aware guard. Review every positive search match;
do not evade an audit by renaming or moving forbidden code.

### Step 4.3: Run the fresh native/WASM/browser matrix

```powershell
cargo +1.91.0 nextest run -p ares-core -E 'test(/^profiles::tests::/)'
cargo +1.91.0 nextest run -p ares-core `
  --test no_unapproved_dynamic_values
cargo +1.91.0 nextest run -p ares-core config_export
cargo +1.91.0 nextest run -p ares-core project
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

Require `wasm-bindgen 0.2.121`, zero npm vulnerabilities, the real browser test
green, and the complete KSR golden test still intentionally ignored because
full G-code parity is not yet implemented. On Windows, capture native command
exit codes immediately and do not use Playwright `--with-deps`.

### Step 4.4: Required whole implementation reviews

Dispatch fresh reviewers against the same frozen manifests, complete patch,
and verification evidence:

1. Whole spec compliance: literal `VERDICT: APPROVE`.
2. Whole code quality: literal `VERDICT: APPROVE`.
3. Required default-model OpenCode whole implementation review: literal
   `VERDICT: APPROVE`.

Any revision invalidates the frozen hashes. Fix with a focused regression or
mutation test where applicable, rerun affected verification, rebuild all
manifests, and rerun all three whole reviews until approved. Do not update
tracked architecture/roadmap documents before these approvals.

---

## Task 5: Documentation review after implementation approval

**Files:**

- `docs/architecture/option-parity-v4.md`
- `docs/roadmap.md`

Document only approved shipped behavior:

- the pinned Orca profile load/inheritance and
  `full_fff_config(false, std::nullopt)` subset;
- order-independent two-pass typed decode and strict wrong-kind/unknown
  boundary behavior;
- presence-preserving whole-field inheritance and explicit nil/variant defer;
- public tagged `MergedProfile` and typed `ComposedProfile`/`ProjectSettings`;
- compile-time typed multi-filament append and positional group metadata;
- exact removal of 29 fingerprints with 683 retained;
- no `slice_project` integration and all profile management, Task 20A-E,
  geometry, toolpath, G-code, and complete KSR parity deferrals.

Do not claim Task 20A.1 released or full G-code parity before exact-SHA Tier 1.
Dispatch a fresh documentation reviewer and require:

```text
ROLE: DOCUMENTATION
VERDICT: APPROVE
```

Revise and re-review until approved. Add approved docs to the final manifest,
recompute every hash, and rerun the complete Task 4.3 release matrix from the
approved documentation bytes. An implementation change invalidates whole
implementation and documentation approvals and restarts Tasks 4.4-5.

---

## Task 6: Commit, push, and exact-SHA Tier 1 release

Apply the Conventional Commits skill only after Tasks 4-5 are approved and the
fresh post-documentation matrix is green.

### Step 6.1: Stage only the frozen reviewed manifest

```powershell
git status --short
git diff --check
git add -- <exact reviewed manifest paths>
git diff --cached --name-status
git diff --cached --check
```

Do not use `git add -A`. Confirm no ignored evidence, generated wasm/npm
output, fixture/reference change, Orca checkout change, or unrelated user file
is staged.

### Step 6.2: Commit and push normally

Use the reviewed Conventional Commit subject:

```text
feat(profiles): compose typed preset options
```

Then push the current branch without force:

```powershell
git push origin codex/ksr-fdmtest-v4-parity
```

If the remote advances, fetch and rebase the reviewed commit without dropping
user changes, rerun relevant verification, and push normally.

### Step 6.3: Verify remote identity and exact-SHA Tier 1

```powershell
$branch = 'codex/ksr-fdmtest-v4-parity'
$local = git rev-parse HEAD
$tracking = git rev-parse "origin/$branch"
$direct = ((git ls-remote origin "refs/heads/$branch") -split '\s+')[0]
git status --short
```

Require local, tracking, and direct SHAs equal and the worktree clean. Locate
the Tier 1 push run whose `headSha` equals `$local`, watch it to completion,
and verify these five required jobs are green:

```powershell
gh run list --workflow tier1.yml --branch $branch --commit $local --event push `
  --json databaseId,headSha,status,conclusion,createdAt --limit 10
gh run watch <exact-run-id> --exit-status
gh run view <exact-run-id> --json headSha,conclusion,jobs
```

- `format`;
- `ubuntu-latest`;
- `wasm`;
- `macos-latest`;
- `windows-latest`.

Only then record Task 20A.1 as released in ignored progress evidence. The
persistent goal remains active for the other 683 compatibility findings,
remaining Task 20A consumers, Tasks 20B-20E, geometry, toolpaths, G-code,
metadata/post-processing, adapter dispatch, and complete normalized KSR G-code
parity.

## Plan exit criteria

This plan is complete only when:

- exact spec and plan bytes were approved before production work;
- the 29-row RED was reproduced exactly and final baseline has 683 rows;
- all production profile data and outputs are concrete typed owners;
- two-pass parsing, inheritance, typed append, composition metadata, and every
  required error case pass public behavioral tests;
- the AST shell guard and full dynamic audit are green with no allowlist
  addition or moved finding;
- every implementation slice, whole spec, whole quality, default OpenCode, and
  documentation review literally approves;
- the fresh native/WASM/browser release matrix is green;
- only the reviewed manifest is committed and pushed normally;
- local/tracking/direct remote SHAs match;
- the exact pushed SHA's five Tier 1 jobs are green;
- full KSR G-code parity remains explicitly open.
