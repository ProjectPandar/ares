# O8-X2D-SCALAR: unresolved original flush-context divergence

## Classification

**UB_ADJACENT.** The handling contract below is verified without production
changes.
This resolves the safe-input disposition, not Native numerical parity. Native
success is an observed result of the UB invocation, never a parity PASS.
Preserve the admitted test-only RED. No ignored test or numeric fallback is added.

## Exact input and fresh observation

Repository-cube10 X2D input (SHA `b980548a…`).
This is a **test-constructed input, not a real GUI-saved 3MF**: the test exports
repository `tests/parity/cube10.stl` with the stock X2D machine/process/filament
profiles through Native CLI, binds two logical filaments, then rewrites
`Metadata/project_settings.config` (including the scalar nozzle-volume type and
four-entry matrix) and adds explicit end-G-code tool switches. It is not a
cropped customer GUI project. O8-X2D-SCALAR is scheduled after FM; genuine GUI
provenance, if subsequently established for another reproducer, raises priority.

Public `slice_project` on the exact original bytes returns
`Flush volumes matrix do not match to the correct size!` after the dependency
coverage rule preserves the original matrix: this two-colour input needs
non-diagonal type/dataset calculations. It does **not** succeed.

Two prescribed seeded Native diagnostic invocations both succeed. Their logs
show `unsupported NozzleVolumeType=30898` and `=6` respectively, between
`flush_volumes_matrix before computing: 0,140,140,0` and the reconstructed
`0,339,621,0,0,339,621,0`. Complete output differs only at the generator
clock line. This establishes repeatable observed output for these two runs,
**not deterministic or defined C++ behavior**: the invalid enum value changes.

Ares error versus Native success remains an explicit
unresolved behavioral difference.

## Owning upstream boundary and Rust destination

Revision `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `OrcaSlicer/src/OrcaSlicer.cpp:1708–1718,3022–3047`: current raw type vector
  versus new CLI types; new-vector filling does not resize the current vector.
- `:3340–3351,3457–3462`: reconstruction predicate and unchecked per-nozzle
  volume-type/dataset selection. The original type vector has one entry for
  two nozzles; Native reads entry1 out of bounds.
- `libslic3r/PrintConfig.cpp:9004–9060`: variant lookup used by dataset selection;
  no inferred first-id or zero-dataset replacement may be introduced.
- `libslic3r/GCode.cpp:5596–5614`: unchanged scale/size guard and exact error.

Rust destination is the existing raw project/CLI preparation context and
`ares-core/src/options/flush_matrix.rs`, not emitter geometry or a new pipeline.
This follow-up must first define a source-owned handling contract for
this undersized input. Preserving cells with unresolved dataset dependencies
alone does not solve the original parity difference.
Do not derive that contract from observed numeric values, emulate UB, relax the
matrix guard, or silently replace the original input with a complete vector.

## Undefined-read boundary and proposed handling contract

This contract is not permission to emulate UB
or a declaration that the original Native-output parity difference is fixed.

### What is undefined, and what is not

The preserved input SHA is
`b980548a6170e454a0b0701c115aecbae3441487c35b92fcd64e8c50b50595b2`:
`nozzle_diameter=[0.4,0.4]`, `nozzle_volume_type=[Standard]`,
`extruder_type=[Direct Drive,Bowden]`, ids[1,1,2,2], four variants,
two ordinary non-support colours, and a four-entry supplied flush matrix.
The generated-id map is complete; the newly integrated generic selector does
not repair the one-entry nozzle-volume vector or define an out-of-range read.

`OrcaSlicer.cpp:1708–1718` copies exactly the stored type count into the current
vector. At3022–3047 the new CLI vector has two Standard entries when no type
override exists; only an **empty** current vector is filled. A nonempty
one-entry current vector is not resized. Their inequality activates3340–3351's
flush reconstruction predicate even though a matrix is present.

The critical raw selector copy at3450–3458 is separate from that new CLI vector.
It retains the one serialized type (unless explicitly overridden). In3460–3462,
`eidx=1` calls `volume_types[1]` on that one-element std::vector: an unchecked
out-of-range read with C++ undefined behavior. `get_at` elsewhere cannot supply
a guarantee here. If the invalid enum makes9004–9060's lookup return -1,3463's
`nozzle_flush_dataset_full[index]` is another unchecked read; unlike the later
Print::apply invocation, this CLI call does **not** map -1 to0 first.
Do not import PrintConfig.cpp9677–9681's call-specific handling into this path.
Adjacent `extruders[eidx]` is also unchecked, but this exact original input
contains both extruder entries; it is not the missing prerequisite here.

A one-element vector is not globally illegal: O3 loading validation, option
serialization and other source `get_at` consumers remain valid and authoritative.
Only a calculator invocation needing the unavailable per-nozzle selector is
outside the defined upstream domain. Native's changing invalid enums30898/6,
coincidentally equal reconstructed values and seed pinning do not turn UB
into defined arithmetic or a reproducible contract. Return status0 is likewise
not evidence of a valid index. No new global scalar-option rejection is implied.

### Proposed disposition: preserve, never synthesize

Keep the existing dependency rule at the shared byte/options boundary:

1. Retain the original raw option shape, old/new preparation context and supplied
   matrix. No automatic Standard extension, `get_at` substitution at the
   unchecked selector, palette heuristic, retry or UB/allocator emulation.
2. Inspect actual calculator dependencies, not printer names or palette count.
   If an off-diagonal non-support cell needs a missing selector, decline
   reconstruction before selecting any dataset. Do not invent dataset0 or reuse
   the previous nozzle's selector. Independent source multiplier resizing stays
   independent. Dataset-free diagonals/support constants remain computable under
   the FM contract; this proposal must not regress that boundary.
3. Let the unchanged downstream GCode size/scaling/rounding authority consume
   the preserved matrix. For this original four-entry/two-nozzle case it returns
   the existing exact error `Flush volumes matrix do not match to the correct size!`.
   Do not bypass that guard, pad the matrix or return Native's observed numbers.
   A supplied valid-size matrix with unresolved selector context is preserved;
   it is not globally rejected solely for having one type entry.
4. Classify the original Native-success/Ares-error result as **source-UB input,
   unsupported automatic reconstruction**, not Native-output parity PASS.
   Preserve the admitted RED, both Native outputs, first UB boundary and Ares
   outcome. Only an explicit disposition can close this case as
   a documented unsupported-input contract; it must not claim numerical parity.
5. A caller can explicitly supply per-nozzle types (e.g. two Standard entries)
   through the existing configuration interface. That is a **different effective
   option contract**, with its own input/override identities and complete Native
   byte comparison. It is not an implicit repair of, or substitute evidence for,
   the original witness. Genuine GUI provenance for a separate project stays a
   separate investigation rather than changing this input's provenance.

This handling is a bounded safe-Rust disposition of an upstream UB precondition,
not an independently designed Ares slicing algorithm. Existing defined-input
FM and generated-id behavior must remain byte-identical. No new API field,
loader bypass, compatibility fallback, emitter patch or configurable UB mode.

### Spec-first verification and exit decision

Capture a fresh current-main same-input public
API outcome with input/source/build hashes and Native producer/resource identity.
One source/Native trace establishes actual vector sizes and call location; do
not retry until a desired garbage enum appears. Already preserved UB executions
remain diagnostic evidence and are not a valid expected-G-code oracle.

Before any necessary production edit, separately bind tests to the
public bytes/options seam: original-input **defined rejection** (not a parity
claim), an explicit fully specified selector control using Native full bytes,
and supplied-valid-matrix preservation. This must not delete/relax/ignore the
historical parity RED or reuse the same label to turn its verdict into PASS.
If current code already implements the handling, prefer no production
change; record its verified behavior and close only by explicit disposition.
Otherwise new test-only RED precedes the minimal source-precondition correction.

Failure inventory: pretending CLI[new] vector repairs raw[current] storage;
confusing `get_at` with operator[]; importing Print::apply's -1→0 policy into
CLI; extending vectors or padding matrices; breaking dataset-free constants;
bypassing O3 or the exact matrix-size error; hiding original UB under a modified
fixture; claiming seed pinning defines UB. Verify unchanged whole-output FM and
generic generated-id controls, and all affected lint/Tier1/workspace gates if
production changes. Complete Native byte parity remains mandatory for defined
inputs, and explicitly inapplicable to the OOB invocation itself.

## Fresh-main verification and matrix record

Machine-readable record: [o8-x2d-scalar-flush-context-record.json](o8-x2d-scalar-flush-context-record.json).
Fresh `cargo build -p ares-cli` succeeded; binary/input/output hashes are
recorded. No production or test changes required.

On byte-identical original input, the public validated byte/options entry
through the fresh CLI returns exactly the existing matrix-size error, exit1,
with no G-code. `flush_matrix.rs:43–59` sees type count1<nozzle count2 and
declines before matrix assignment; the original four cells remain supplied,
while multiplier resizing remains independent. The unchanged export guard
requires eight cells and produces the exact error.

One fresh identified Native invocation (no cache/retry) succeeds, produces
164031 bytes, and logs invalid enum29611 between its before/after matrix
messages. It confirms the already source-bounded UB-adjacent observation,
not an expected enum/matrix or full-byte oracle. Historical Native outputs
and admitted RED remain untouched. Matrix classification is **UB_ADJACENT**,
`parity_pass=false`; do not count this row among PASS controls.

An explicitly separate Ares-only preservation control changes only the
supplied matrix to eight distinguishable cells[0,141,142,0,0,143,144,0].
It slices successfully with `nozzle_volume_type = Standard` still one entry
and exports [0,42,43,0,0,143,144,0] after the unchanged[0.3,1] scaling.
Thus scalar is not globally rejected or broadcast, and a valid-size supplied
matrix is retained rather than reconstructed. This derived control is not
the original fixture and has no Native parity verdict: Native reconstruction
on the still-undersized vector remains outside the defined domain.

## Related stock rows: thirteen multi-head singleton-selector cases

These historical stock rows belong to the **same upstream CLI UB boundary**
as O8-X2D-SCALAR, not thirteen independent Rust failures. All are recorded in
[the same machine-readable record](o8-x2d-scalar-flush-context-record.json)
under `related_stock_cases`, with **UB_ADJACENT / parity_pass=false**.
No original scalar RED, source option, guard, or comparator was changed.

Source: the single-attempt option matrix. The saved thirteen 3MF inputs were
replayed without editing their bytes. Fresh Ares observation uses the existing
cargo-built CLI and binds the input/binary/output hashes in the record.

The exact seed-replay producer `72d4hac8qmg5p9m22v2kifbp1hkxjk63` was
rebuilt to realize its existing separate-debug output. Raw Native SHA remains
`59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff`;
Build-ID `bdbbf4336095c2ce6283fc5441e1e56fbf595b99` matches the debug file.
One seed0 gdb invocation per input was retained, with output directories
remapped only to avoid overwriting original evidence. MyToolChanger0.8
SIGSEGV backtrace: `get_extruder_variant_string` at `PrintConfig.cpp:604`,
negative enum bits `0xe63fab60`; `get_index_for_extruder` at9034 with
extruder id3; `CLI::run` at `OrcaSlicer.cpp:3462`. The other twelve invocations
exited normally and produced G-code. This instability is not retry-based
oracle recovery or a new numerical contract.

**Shape and safe-Rust outcome differ from the original two-colour witness.**
These stock configurations have four/five nozzle entries, but singleton
`extruder_type` and `nozzle_volume_type`, and one colour/palette entry.
All thirteen Ares runs exit0 and emit G-code; none returns the original
X2D matrix-size error. This difference is explicitly accepted:
do not add rejection just to match the two-colour case.
The original X2D witness needs unavailable selectors for non-diagonal cells;
a singleton selector is not itself globally illegal. Preserve each input's
actual shape and observed result, rather than claiming identical error behavior.

For the twelve normal Native outputs, complete-byte comparisons with their
Ares outputs normalize **only the generated-by line**: **12 equal, 0 unequal**.
Raw/normalized hashes and byte lengths are recorded per row; there is
no first differing line. The SIGSEGV row has no comparison. These are observed
success-stream equalities, **not parity PASS**: all thirteen rows retain
`parity_pass=false` because the Native execution crosses the undefined
per-nozzle selector read. No retries, fixture repair, numeric expectations,
production change, or scalar rejection were introduced.

Cubicon's three export aborts are separate and not included in this group;
their reconstructed-input backtraces remain a separate follow-up.

## Closure: documented unsupported-input contract

Closed as a **documented unsupported-input contract**. Native's success on
the original input depends on the out-of-bounds per-nozzle selector read
(`OrcaSlicer.cpp:3457-3462`) and is not a parity PASS; Ares reports the
upstream error verbatim at the unchanged size guard
(`GCode.cpp:5596-5614`), exit 1, no G-code. `UB_ADJACENT`,
`parity_pass=false` and the admitted RED are retained unchanged. No UB emulation, matrix-guard relaxation,
`-1→0` import, vector extension or numeric expectation was introduced.

Re-verification with zero production change: the preserved input bytes (SHA `b980548a…`) through the fresh
public CLI return exactly `Flush volumes matrix do not match to the correct
size!`, exit 1, no output.

Pinned controls (`runner/x2d_flush_matrix.rs`):

- `x2d_flush_matrix_scalar_four_cell_input_fails_exactly_at_the_cli` —
  the original UB shape (scalar type + four-entry matrix) reconstructed
  at runtime through the same Native export path; asserts the exact CLI
  error text, exit 1 and no exported G-code.
- `x2d_flush_matrix_scalar_controls_require_native` — the control FAILs
  (not silently passes) when a Native is required but absent.

The legal-size eight-cell scalar shape is recorded as **runtime
UB_ADJACENT evidence, not a pinned Ares contract**: the reconstruction
predicate fires for the changed one-entry type vector
(`OrcaSlicer.cpp:3340-3351`), so Native crosses the undefined
per-nozzle selector read (`:3457-3462`) and rebuilds — the seeded
replay produced `0,102,186,0,0,339,621,0` (coincidentally the defined
[Standard,Standard] values) — while
the observed Ares run preserved the supplied matrix and scaled it once
(`0,42,43,0,0,143,144,0`). The input is outside the defined upstream
domain on both sides, so neither behavior is asserted as a contract
(pinning Ares's own behavior on a UB input would fix self-designed
behavior against the rewrite gate).

## Included, deferred and exit

Included: original-input fresh-main E2E RED, precise current/new context and
source prerequisite tracing, and an explicit resolution contract.
Deferred: changes to the now-integrated generic generated-id materialization,
timing, flush emission, geometry, UI, arbitrary input canonicalization and broad UB policy.

Before production, define the specific resolution contract
and retain a separately bound RED. Exit requires an accepted disposition of the
original same-input Native-success/Ares-error difference, complete byte evidence,
unchanged controls and ordinary Tier1/lint/workspace gates. An FM GREEN covering
only complete-vector inputs must cite this open follow-up rather than claim the
original reported input is fixed.
