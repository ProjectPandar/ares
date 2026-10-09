# Vendor10 public profiles runtime verification

Approved spec388cbde1f: compare the libslic3r profiles boundary, not different
Native/Ares CLI argument surfaces. Runtime tests live in
`ares-cli/tests/orca_parity/runner/vendor_profile_compatibility.rs` and call only
public PresetCatalogHandle::add_vendor/filter_compatible on the Ares side.

## Source and exact comparison

Preset.cpp:809-850 owns process/printer compatibility; PresetBundle vendor
loading and inherited config composition supply that predicate's inputs. The
existing compatible.rs/catalog.rs implementation required no production change.

Three RatRig V-Core4 300/400/5000.8 and six TiQ2/TiQ8 0.25/0.6/0.8 selectors
retain their exact existing stock default processes. Both complete parent chains
and the original lists are unchanged. Native's same runtime call returns239,
result -17 and log `2652: process not compatible with printer.` Its conclusion
determines the reference compatibility decision, compared with exact default-
process membership in the public Ares filter output; all nine are incompatible.

K1 SE0.8 retains its absent default process name. Native's same runtime call
returns253/-3 and names the missing setting file. The public catalog loads the
real complete bundle and its selected-process lookup returns typed InvalidInput
naming that same missing default. This is not a missing-parent/file-at-catalog-
load case. Core's name-level payload is not compared with filesystem paths or
process exit channels. No expected Native output/decision is frozen in source.

The harness reuses profile_entry::vendor for exact manifest/files envelopes,
VendorProfiles/write_preset/stages::run/cache_identity for Native inputs,
execution captures and producer provenance. Native output stays in isolated
external work directories; failed exports do not publish success cache entries
or produce3MF. Default Native absence is visible SKIPPED before artifact access;
required absence fails closed. No worker4 cache/channel file was edited.

## Actual gates

All acceptance executions are bound to test-only
`a99ecf9b806ef43301606710cdc9b31128e9c179` on production main5c9e69226.
One job at a time,16test threads/build8; required runs use the current full
control script and declared producer/resources. No later rebase is relabeled
as execution on a new commit.

| Gate | Actual result |
|---|---|
| required seeded |11fa3f1f-7146-4def-9fec-129df03448aa;10PASS;exit0|
| required official |e22e9b23-5f04-43fb-8024-827fe7d1e341;10PASS;exit0|
| clippy --workspace --all-targets --all-features -- -D warnings |exit0|
| fmt --all -- --check |exit0|
| default workspace, all ARES_* removed |6d5aefce-c414-4d02-aa26-a3d5afa26963;8214PASS/6existing skips;exit0|

Default output contains exactly ten visible Native-comparison SKIPPED notices
for the new tests, with no panic. These default early-return successes are not
Native evidence. Both required producers independently supply all ten fresh
conclusions. All20 records were independently reread: actual statuses, Native
result/log/missing diagnostics, catalog hashes and public filtered/error output
agree with the assertions and source boundary.

The first attempt at8baae99ac (f43dab3a-fe98-4669-b661-47bd19b1f7fd,exit100)
passed9 and failed K1 before Core comparison because the test unconditionally
read a nonexistent early-error log. The fix only reads2652's log in the -17
branch; the approved K1 -3/diagnostic checks and API assertions are unchanged.
That attempt remains external and is not relabeled GREEN. Its harness correction
is amended into the single final test commit; no production RED was inferred.

## Conclusion / deferred work

All ten catalog/compatibility conclusions match: tests/docs-only closure, no
Core production RED or fix. Raw stock VENDOR_INCOMPLETE rows remain incomplete
selection inventory, not successful slicing/G-code PASS. GUI fallback state,
full slicing, general expression/filament compatibility coverage and the Native
CLI argv surface remain deferred. The same-argv observations remain non-product-
RED records under the coordinator's boundary ruling. No CLI interface was ported.

#30's four -50 and four -19 cases are separate and use worker4's merged public
EarlyExit recipe. This verification does not claim those cases were executed.

Machine review receipt: `/tmp/worker3-vendor10-review.json`; all actual command,
input/catalog/producer/exit/output bindings remain under
`/tmp/worker3-vendor10-runtime-v{1,2}-*`, `worker3-vendor10-official-*` and
`worker3-vendor10-gates-*`. The verification document is added after execution.
