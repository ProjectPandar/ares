# Native artifact cleanup — slice B

Status: DRAFT for approval. Overall owner worker4; processor sub-slice worker6.
Prerequisite: accepted Native-required slice A. This specification records
migration boundaries and non-negotiable outcomes, not execution logs or a plan.

## Decision

Do not commit Native-derived G-code, stdout/stderr, result JSON, maps, CSV,
TSV/RLE projections, receipts, diagnostic logs or snapshots. Commit only source
inputs, preset references, command recipes and source-cited expected contracts.
Generate Native references during each test's execution via existing OrcaRunner,
cache under ARES_ORCA_CACHE_DIR with actual input/producer/command identities,
and preserve every current byte/channel/map/bit assertion.

This is test-boundary/reference-source migration, not an Ares slicing feature.
Existing OrcaSlicer libslic3r/libvgcode ownership remains unchanged. No new
pipeline, viewer runtime, normalizer, oracle fallback or second cache framework.
No core/WASM production filesystem, subprocess, terminal or Native dependencies.
No stored Native output renamed as a fixture input, copied into Rust constants,
compressed, embedded in JSON or moved to another tracked directory.

The only production API exception is worker6's approved explicit post_process
parameter on the existing gcode_processor::process_gcode thin delegate,
corresponding to GCodeProcessor.hpp:915 finalize(bool post_process) and
cpp:2715-2716 run_post_process. It delegates the same processor loop, retains
false-path behavior and threads typed loading delay only for true replay.
Other production changes require a separate reviewed exception, not this spec.

## Ownership and cited source boundaries

worker6 exclusively owns runner/cache/processor.rs (including same-process
GDB slice mode and explicit GDB identity), tools/chain-probe/role-filament-map.sh,
new runner/processor_migration*.rs, the12 processor cases and their listed
Native products, and process_gcode(post_process). worker4 must not edit these
files concurrently. Shared extension requirements are interface requests to
worker6/coordinator, not authorization for a second writer. Worker4 owns the
other reference recipes/consumers. The adjacent processed_color_change/native.rs
family and processed_motion_attributes.rs (including dynamics RLE migration)
are also worker6-owned, with their own spec-first review. worker4 does not edit
these groups or the shared core policy helper in parallel. If processor CSV/kernel
products need a shared adapter change, agree the writer and contract before
implementing.

Reference processor spec: docs/architecture/native-artifact-cleanup-processor-spec.md,
initial approved2e142621, with coordinator-required explicit-GDB correction.
Its12 active case mappings, whole replay streams, M73 C/P/R and role f64 bits
are authoritative. The two source-less ignored envelope diagnostics remain
explicitly deferred as approved there; they are not counted as migrated PASS.

Inventory source: worker6 /tmp/worker6-core-native-gcode-inventory.md, snapshot
238559840:15 frozen-output consumer modules/24 active+3 existing ignored tests,
plus adjacent live color-change/dynamics consumers. This is a handoff reference;
consumer classifications are recorded below so a temporary file is not required
for future implementation. Processor group is owned by worker6, not worker4.

| Group | Upstream concept and preserved Rust test seam | Owner |
| --- | --- | --- |
| role map/cache, seam/envelope, color-stop | GCodeProcessor cpp:1621-1649,4192-4230,981-1290; public process_gcode + processed slicing;12 source-mapped cases | worker6 |
| slicing validation/boundaries/jerk/SEMM/plate geometry | Print::validate, PrintObject transformed hull, GCode object/tool transitions and machine limits; existing public byte-in slicing/error APIs, moved core Native consumers to CLI | worker4 |
| KSR configuration + full stream | PrintConfig/GCode CONFIG_BLOCK writer, existing bounded full/runtime export and public slice seam; no runtime-value omission/duplicate-assignment loss | worker4 |
| CLI channels/failures | OrcaSlicer.cpp error dispatch/result.json; exact exit/channel/JSON/export artifact inventory | worker4 |
| color-change/raw-stable/dynamics | GCodeProcessor store_move_vertex/process_file/options correction; existing public processed slicing/API | worker6, adjacent spec-first sub-slice |
| marker time/kernel + web consumers | LibVGCodeWrapper::convert, ViewerImpl.cpp:1461-1465 ordered f32 source kernel, Utils::get_time_dhms; pure viewer/WASM APIs, runtime test oracle products | worker4, coordinate web owner before touching web |
| avoid-crossing/replay | Existing source-cited travel paths and processor replay; existing CLI full ordered comparator | worker4 |

## Existing cache is the single reference authority

Reuse runner/cache.rs and runner/cache/processor.rs with cache_identity producer-v2,
existing per-entry lock, staging directory and atomic publication. No generic
parallel cache engine, external prewarming service, fallback frozen references
or test-order dependency. Every test independently obtains its reference;
cache miss generates and validates, hit checks prerequisites, key and output SHAs.

Three bounded recipes are needed inside that cache boundary:

1. Existing export/slice reference: extend plate selection/explicit overrides
   only where required by the retained fixture; selected options must reach
   actual Native input, not just Ares. Include actual input bytes and recipe.
2. Native CLI command result: cache the Native exit status, raw stdout, raw
   stderr, complete result.json when present, retained G-code and exact product
   inventory. Expected Native nonzero exit is a validated reference outcome,
   not a launcher success or setup failure. Existing stages::run rejects nonzero,
   so use an explicitly bounded command-result path rather than weakening that
   stage globally or treating every exit as usable. Persist cwd/path-sensitive
   inputs/arguments and normalize NOTHING not already permitted by the test.
3. Processor reference: typed process versus slice mode in the EXISTING
   processor cache. slice+gdb returns plate G-code and map/footer from the SAME
   inferior run; process mode returns map/CSV/metadata for its actual input.
   Ordinary Native slicing followed by a second map replay cannot stand in
   for worker6's same-process footer proof. Named modes have distinct product
   sets and distinct recipes, not optional-product guessing.

Processor extension key and receipt include actual input SHA, mode, selected
plate, command recipe, pinned reader hash, launcher identity if any, actual
pinned Native ELF hash and actual GDB binary hash, plus existing chosen
producer/resources/replay policy. GDB must be explicit ARES_GDB_BIN; no nix shell
package-environment resolution, PATH substitution or implicit alternative.
If a launcher is necessary it must be explicitly pinned and hashed into key
and receipt. Current role-filament-map.sh's nix-shell wrapper must be replaced,
not copied to a new untracked fallback. Reader ELF/SHA/map offsets, node count,
ordered unique roles/reset/exit checks remain unchanged. Chosen slice producer
and pinned GDB producer are separately named, never represented as the same ELF.

Raw/stable/dynamics/marker-kernel producer identities remain separate. Additional
processor output types stay recipes/products under the same cache; no copying
raw results into stable expectations. CSV/JSON parsing and deterministic output
projections preserve complete identities/counts/bits, not a summarized pass count.
Cache failures or missing products do not publish a success entry. Tampered
outputs, wrong mode, changed input/tool/resource/recipe must fail or produce a
fresh keyed miss; never silently repair by accepting stale reference bytes.

## Availability and platform boundary

Fully reuse tests/support/native_environment.rs and runner availability from A.
Required1 missing any case's bin/resources/artifact/GDB/kernel/tool configuration
fails with its named env even on a cache hit. Optional absence prints caller-
qualified SKIPPED and states Native comparison not executed. Configured invalid
paths, Native launch/setup errors, parse failures and divergence never become skip.

B cancels ARES_ROLE_ORACLE_LIVE as a secondary opt-in for migrated role cases:
selecting the case means execute its comparison subject only to required/optional
availability. This is not permission to force seed-only cases in official mode.

All Native execution belongs to CLI/test-only host support, never core production.
Core tests dependent on frozen Native streams are relocated to CLI E2E. Pure
handwritten core unit tests remain. Viewer/WASM Rust host integration tests and
web E2Es may consume runtime external artifact paths obtained through the same
cache/test-support boundary; remove compile-time include_str of Native products.
They must not require production viewer/WASM code to spawn Native or read files.
Browser consumers use runtime paths/fixtures produced during their test setup,
not checked-in transformed Native tables or fake JS-calculated Native expectations.

## Artifact/consumer migrations

The companion inventory enumerates individual tracked product paths and owners.
Delete each product only with its consumer rewired (or approved deferred ignored
case disposition). Input .3mf/.stl, user overrides, original source excerpts,
source extraction manifests, manual G-code and pinned recipe code remain inputs.
Native command receipts containing historical input/output hashes become runtime
receipts; retain command arguments as a source recipe, not a timestamped receipt.
Historical RED/capture logs stay outside Git; preserve conclusion-only README/ARD.

Worker4 core Native families moving to CLI include overhang_wall_jerk,
e5max_klipper_jerk, semm_m104, o6_plate_origin/o6_print_volume, bbl_object_boundary,
bbl_toolchange_boundary, exclude_outline and arachne_default_prisms. Keep existing
public API/control branches, including -102 retained rejected bytes and exact
plate-origin/hull coordinates; do not reduce to success/substring/footer checks.
Mixed pure-local cases remain in core only if independent of Native references.

The core config_export fixture includes private full/runtime collector assertions.
Keep pure full/runtime sentinel/nullable/duplicate semantics in core with source-
owning manual inputs, and put Native comparison at the public emitted CONFIG_BLOCK
CLI seam. Do not delete internal view-difference assertions merely because the
public seam cannot inspect the private collector. If exact equivalent coverage
cannot be split without a second production API, stop for review rather than
inventing that API. Existing fixed-length/digest/assignment contracts remain;
fresh reference mismatch is evidence, not reason to regenerate constants.

KSR official and initialized-seed reference producers must retain their separate
roles/identities and existing generator-only full-stream comparisons. Expected
hash literals already present are not replaced with Ares-produced values; new
Native output data must not be embedded as constants. Replay negative controls
still mutate independently generated runtime Native bytes and retain complete
input/reference/Ares/error artifacts, provenance limits and divergent outcomes.

CLI -3/-18/-19/-50/-51/-100/-102/-63 tests retain exact Native exit, raw stdout,
raw stderr, complete byte-equal JSON (including selected-plate metadata), absence
or retention of final exports. Legacy STL/custom-parameter local-only contracts
stay local; runtime Native fixture provision is never confused with those errors.

Motion attributes replaces15 Native RLE files with same-run dynamics CSV→RLE
projection outside Git, preserving vertex identities, original exact bits,
counts and all three channels. Color-change retains raw-vs-stable difference-set
proof, model color events/cardinality/time/volume checks; manual nine G-code
inputs stay. Processor-wipe-tower native.json is runtime diagnostic output,
not a hand-authored expected-value input.

Marker-time files retain handwritten path.gcode, tagged-overrides and source
extraction inputs. Runtime kernel generation obtains Native converted vertices,
synthetic kernel numeric cases and native formatting values; TSV projections
are runtime products. Preserve all ordered f32 bits, phantom/repeated ids,
reverse/filter/reload properties and approved raw/stable bounded difference
sets. Add explicit ARES_MARKER_TIME_BIN / ARES_MARKER_TIME_STABLE_BIN to the
required gate environment before deleting products; absent tools are blockers,
not a reason to derive Native expected strings from Rust/JS.

## Explicit prerequisite decisions

The historical raw marker-kernel path recorded in marker-time/oracle.json,
/nix/store/1fhccjjpi187yg45bchvg8935iwxywim-orca-slicer-marker-time-2.4.2/bin/orca-slicer,
is not available in the current environment. The stable marker executable
/nix/store/d9n5bs7mp9d3i7wlkcik81k47jpay8ir-orca-slicer-marker-time-stable-options-2.4.2/bin/orca-slicer
exists. Raw is not substitutable by stable. Before this group is implemented,
coordinator must authorize/provision the existing marker-time.nix source-kernel
recipe (pinned upstream8500fcdcc and ORCA_PROCGCODE_NIXPKGS) or supply a verified
raw binary. No diagnostic Native rebuild is launched by this specification.
Kernel source excerpt extraction remains read-only checked, not a production fix.

KSR initialized-reference.json records a historical seeded wrapper/ELF identity
separate from the current official/seed-only producer. Existing fixed output/
CONFIG_BLOCK hash contracts cannot silently be rebound to a different Native
build. Attempt a fresh runtime capture with the retained exact input and explicit
producer, then report any hash/stream disagreement for a producer-boundary ruling;
do not rewrite pinned expectations or claim all same-version binaries equivalent.
These prerequisites are named blockers for their groups, not optional exemptions
or a reason to start deletion ahead of runtime assertion proof.

## Verification and acceptance

Per old test name record its new counterpart, recipe and unchanged assertions;
counts alone do not establish equivalence. Migration is infrastructure/reference
change, not a fabricated production algorithm RED. Before implementation write
real E2E controls exposing missing fixture/runtime reference, wrong mode/input,
cache-hit missing tool, corrupt product and expected nonzero Native capture.
The worker6 API true-path separately needs real Native replay RED before code.

Verify fresh and warm cache with independent-test execution, actual output SHA
receipts and unchanged verdicts; no prior test/prewarm dependency. Run required
missing tool demos and optional visible SKIPPED demos. All related Native cases
must actually compare when full env supplied; no secondary opt-in escapes.

Fresh latest-main seeded/official base→candidate serial16threads/jobs2 with
independent caches/artifacts, raw Ares streams unchanged, per-test name/outcome
audit; disclose inherited seed-only setup/ND separately with same-label raw and
Native generator-only hashes. workspace nextest, strict workspace/all-target
clippy, fmt, actualCLIwasm checks all pass. Browser/kernel consumer changes also
require their owner's existing full assertions; do not shrink browser coverage.

Final Git audit finds no Native-derived output files or compile-time products
in any consumer, not merely no generated-by G-code headers. No missing consumers,
no outputs copied into constants, no newly ignored tests, no production drift
outside worker6's single approved API exception. Delete obsolete reproduction
output-copy paths only after references are audited. Preserve manual input list
and source-derived semantics. Conclude with separate exact-hash B REVIEW;
processor sub-slice is reviewed/owned by worker6 with shared cache integration
serialized under one writer per worktree.
