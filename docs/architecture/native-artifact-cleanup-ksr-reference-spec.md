# KSR Native references and CONFIG_BLOCK consumer migration

The post-preflight authority ruling in
[KSR runtime Native authority](ard-ksr-initialized-runtime-authority.md)
supersedes preservation of original frozen-file self-checks below. Ares output
comparisons retain their strictness against runtime initialized bytes; original
capture self-checks retire with that capture, without rebinding their hashes.

## Source classification

Upstream is OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Read-only audit uses `git -C OrcaSlicer ls-files -z`, compares every tracked
upstream `.gcode` and `.3mf` SHA, and checks same-name entries. Neither KSR
G-code nor classic has a same-name or identical upstream G-code. Upstream's
only tracked G-code resources are `ams_load.gcode` and `ams_unload.gcode`;
these are not KSR references.

| File | SHA-256 | Classification |
| --- | --- | --- |
| `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode` | `3f2d028f82b6a50f5244f0bfdb193370975b21c55661575edbf98d78d8aa25e4` | Historical Native output, not upstream input |
| `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.initialized-orca.gcode` | `7e52b76af65c5438f49b73efcad63327cf3f73623022c96d79efca12e1fd98b7` | Historical initialized Native output |
| `tests/parity/replay/ender3-classic.orca.gcode` | `3ed255be8081c66c1a512a838040752cdb076d3aeae9be9a632d0f2e43fabb97` | Historical Native output |
| `OrcaSlicer/resources/handy_models/ksr_fdmtest_v4.drc` | `cd3a418e521ccca9e021b9d345f9de9c3650c2cb905de54bcd6089f8670c4081` | Upstream model resource; retain |
| `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf` | `698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9` | Existing external project input; retain unchanged |

The KSR project is already tracked, first visible in squashed commit
`5977b66b6`, with 15 ZIP members. Its Application metadata is
`BambuStudio-02.06.00.51`; model/source_file metadata names
`ksr_fdmtest_v4.drc`. No reproducible authoring script is available. This does
not establish mesh equivalence to the upstream DRC, and this slice neither
requires that proof nor recreates the project. Preserve the classic 3MF input
and KSR options source data as well. The initialized JSON is a generated
historical receipt, not project source; delete it only after its consumers move.

## Upstream and Rust boundary

- `libslic3r/Print.cpp:2618-2638`, `Print::export_gcode`, delegates to
  `GCode::do_export`; complete output remains the acceptance boundary.
- `libslic3r/GCode.cpp:5591-5643`, `GCode::append_full_config`, emits ordered
  full configuration, bans nine keys, omits nil values, handles flush scaling,
  plate-selected wipe-tower coordinates and filament/extruder colour.
- `libslic3r/GCode.cpp:2640-2658,3549-3565` wraps that serialization in
  CONFIG_BLOCK markers; `libslic3r/Config.cpp:543-548`,
  `ConfigBase::opt_serialize`, delegates to option serialization.
- `GCodeWriter` movement/control emission and GCodeProcessor post-processing
  retain the ordered comparator's existing source boundary (ARD-0081).

Production code and APIs are unchanged. Runtime Native work belongs to CLI
E2E using existing `OrcaRunner::slice_case`, fixture adapter, lock, producer
identity, cache integrity and external artifact receipts. No second cache,
prewarming protocol, Native build or core filesystem/oracle dependency.

## Included consumers and destination

Migrate Native-dependent KSR tests into dedicated
`orca_parity/runner/native_ksr_reference_migration.rs` and same-prefix helpers,
registered in runner.rs. Keep pure golden/semantic tests active in the
standalone KSR binary; preserve diagnostic parsers as diagnostics only.

- `fixture_contract_is_stable`: preserve project SHA, package count, complete
  Native line/layer counts, validated generator normalization and reference
  integrity checks; producer-bound fixed-hash handling requires the preflight
  below, rather than silently updating literals.
- `project_matches_initialized_orca_242_ordered_bytes`: public CLI output,
  complete ordered generator-only byte comparison, retained full artifacts,
  Ares generator identity, and object-ID mutation rejection remain intact.
- `gcode_writer_and_do_export_real_records_reject_33_mutations`: generate
  classic from its retained 3MF; keep all eleven categories and change/delete/
  reorder checks. Locate the actual validated generator line at runtime instead
  of requiring a historical timestamp. Do not loosen body anchors on mismatch.
- `options/tests/config_export/fixture.rs` Native checks migrate to a separate
  KSR CONFIG_BLOCK CLI helper/test module, not web's `config_block_roundtrip.rs`.
  Preserve marker extraction, exact block bytes, fixed size/hash, assignment
  order/count/keyset, nil omissions, retained empties, duplicate wipe-coordinate
  records, temperature tail and full/runtime sentinels. Pure Rust view/collector/
  writer consistency checks remain in core. CLI uses the existing public project
  slicing output's CONFIG_BLOCK; no new public/test-only production API.

The audit found exactly two Native-dependent config_export tests and one
reference_block caller. Re-audit all callers before removal; preserve every
Native-dependent assertion and retain pure core coverage separately. Do not
modify web's Ender3 input or config roundtrip module.

## Browser chain

P3 owns `ares-wasm/tests/browser/project-slice.spec.mjs` and its existing
`browser_output_matches_initialized_orca_242_ordered_bytes` consumer. Remove
that test's old ignore attribute and move it to the runtime Native boundary.
Update its browser selector to select exactly the new test, without
`--run-ignored all`. Browser supplies its actual complete WASM output file,
not CLI output standing in for browser output.

Use the `web/e2e/presetSlice.spec.ts:88-99` nativeEnv pattern: an external
artifact root, explicit Native executable/resources, `ARES_REQUIRE_ORCA=1`
for the browser comparison, and the existing wrapper/cache environment.
Initialized producer selection remains explicit and verified; do not silently
substitute official for initialized. Native reference is generated from the
unchanged KSR 3MF by OrcaRunner, then compared with browser bytes using the
unchanged complete ordered comparator. Retain paired input/Native/browser bytes
and receipt outside Git. Missing Native fails in required mode; optional mode
reports SKIPPED via tests/support/native_environment. No new ignore attributes.

Browser parity evidence requires a real browser capture from the candidate.
Ordinary seeded/official controls without `ARES_BROWSER_GCODE` report visible
SKIPPED: "browser capture absent; run via project-slice.spec.mjs", including
required mode, and return before starting Native. They cannot claim browser
parity PASS. With capture present, Native dependencies retain the shared
optional/required policy. Default workspace has no ARES_* and visibly skips
Native comparisons; its capture-absence policy test actively executes. Pure
comparator tests continue to run. The unrelated ignored Elegoo diagnostic does
not read these products and is not newly ignored here.

## Producer preflight and RED

Before implementing/deleting references, fresh-capture KSR with the historical
initialized producer recorded in initialized-reference.json, validate wrapper
and ELF SHA, and compare against the retained initialized body and CONFIG_BLOCK.
The recorded wrapper exists and its current SHA matches the historical receipt.
Use the same input SHA and seed zero. Keep original and fresh bytes outside Git.
Separately audit the original capture's normalized hash/block contract and the
classic runtime anchors. Different same-version binaries are not equivalent.

Never rebind fixed hash expectations to a new build. Raw output SHA includes
its timestamp and is distinct from body equivalence. Historical raw SHA values
authenticate the old capture; retain their classification here rather than
claiming a fresh timestamp must have the same raw SHA. Proposed runtime raw
integrity checks compare the saved product SHA to its producer-bound cache
receipt, retaining the original integrity assertion count. Normalized-body and
CONFIG_BLOCK fixed hashes are not rebound: preflight must demonstrate their
exact equality before migrating those assertions. Report each fixed raw,
normalized and CONFIG_BLOCK hash's actual reproducibility. A different timestamp
alone is not body divergence. Any producer/body/fixed-contract disagreement
blocks that consumer's deletion pending a separate decision; preserve its
original assertions and evidence. No tolerance increase,
object-ID masking, or producer relabeling.

The real migration RED is a controlled optional child with missing Native:
current frozen consumers still consume products/write comparison artifacts;
new runtime consumers must visibly skip before doing so. Required missing
Native must fail before a warm cache hit. Policy tests use tempdir when no
external root is set. RED outputs are not committed.

## Exit and deferred scope

After verified producer/contract equivalence, migrate consumers, prove required
cold/hot behavior and corruption/missing-tool policy, then delete all three
Native G-code products and the initialized receipt. Re-audit entire-tree
consumers before each deletion. Preserve assertion inventories, exact comparison
policy and full artifact retention. Run real browser comparison, fresh seeded/
official base/candidate controls, default-environment workspace, clippy -D
warnings, fmt and WASM checks. Pre-existing Native nondeterminism or browser
byte divergence must be reported, not normalized away.

Deferred: all production slicing/configuration changes, historical mesh-authoring
reconstruction, web's Ender3 encode-roundtrip group, the CLI exit/retained/
O6 -102/marker groups, shared cache recipes and new Native builds. This is
reference lifetime cleanup around upstream exports, not a new Ares pipeline.
