# Slicing mesh affine rounding implementation plan

1. Run compressed polygon full output before production edits; change only
   X scale from the restored stretched fixture. Retain artifacts and exact RED.
2. Inspect native/Rust transformed vertices through an external debugger.
   Rank input composition, f32 product order, FMA and downstream polygon
   hypotheses; compare one boundary/variable at a time.
3. Port only proven upstream f32 affine arithmetic. Retain main dirty changes,
   do not assume earlier official-build observations prove this native build.
4. Run complete small outputs and original KSR, then consumer regressions,
   workspace Nextest, rustfmt/clippy and all four platform compile gates.
5. Independent read-only six-axis review, repair/re-review, then commit/push
   code and documents together when the finite slice's gates actually pass.

The prior KSR diagnostic is RED at layer49/Z9.8 internal-solid widths and small
path geometry. This is motivation to investigate, not proof of affine causality.

Initialized compressed RED65159 terminates100 /1.597s, whole pair
`/tmp/ares-ard76-compressed-red/case-jM38Tz`, first bridge travel line514.
Native debugger40525 and Rust19559 terminate0 (Rust inferior101 for RED).
Logs `/tmp/ares-ard76-native-mesh.log` and `/tmp/ares-ard76-rust-mesh.log`
show66 vertices each,16 differ; native X2512345 versus Rust2512344.75,
native X9618170 versus Rust9618169. This occurs before downstream polygons.
Native optimized ABI passes matrix inRCX (verified by disassembly), notRDX.
Transform-loop addresses0xda3be0..0xda3c12 in the initialized ELF show
separate mulps/addps in X/Y/Z/translation order. The unrelated later-address
disassembly probe is not evidence for this function.

External process-only counterfactual52901 terminates0 /1.63s: replace only
the16 differing mesh vertices at Rust slicer entry, and whole output passes
(`/tmp/ares-ard76-counterfactual/case-fB8N2J`). This is causal diagnosis, not
an acceptance oracle or source-coordinate injection in production/tests.
Ranked input-transform/downstream hypotheses are superseded by this precise
producer evidence. Production now uses `(x+y)+z`, then translation, all f32;
it changes both fusion and association, not only fusion. Cast-before semantics
and f64 composition remain unchanged. Ordinary full GREEN is still pending.

Consumer regression42280 completed100: 78 tests,77 passed and one old bbox
snapshot failed. Independent scalar C++20 evaluation of the same input,
compiled `-O2 -ffp-contract=off`, independently reproduced min
`c0fbc62e,c03e4085,403e0660` and max `40a65ffa,409b1a1c,40d40405`.
Source `PrintApply.cpp::trafo_for_bbox/transformed_its_bbox2d` composes f64,
clears XY translation, casts matrix/vertices to f32 and expands indexed bounds.
Only two old FMA-dependent expected bits are migrated; this synthetic test
does not invoke center_shift, so its old explanatory comments were removed.
The calculation is a scalar source-contract check, not execution of native
Eigen bbox code; native shared-consumer arithmetic confirmation remains open.
Retained calculator `/tmp/ares-bounds-independent.cpp`, executable
`/tmp/ares-ard76-bounds-independent`.

Ordinary E2E attempt9472 completed100 because native export could not start
(127); its recorded Nix output and derivation are now absent. This is neither
parity RED nor GREEN. Rebuild10468 uses the same retained initialized source
and documented Nixpkgs/build derivation, not a substituted reference.
Separate previous process-angle migration78107 completed0: three tests pass.

Expanded consumer regression9212 completed0:90 tests pass, including
transform,volume_bounds and seam_placement. The additional tilted 3MF witness
is133LOC in a separate child module, reuses only the polygon input generator,
and checks exported mixed-Z/translation through load_project before comparing
the full stream. Preflight79870 with the native gate disabled compiled all four
registered tests but is not native parity evidence. Formatter subsequently ran;
the earlier fmt-check39252 failure was the new long path expression, now fixed.

Shared-consumer source audit: bbox uses Eigen Transform3f; occlusion's
`SeamPlacer.cpp::compute_global_occlusion` instead separately invokes
`deps_src/admesh/stl.h::its_transform` for volume and centered object matrices,
with intermediate f32 storage and decimation. Existing Rust occlusion combines
the matrices before its f32 helper. Passing regressions does not prove this
adjacent native precision contract. Do not infer seam parity from the slicing
mesh producer capture; native precision/observable-output audit remains open.

Full workspace98871 and Clippy78258 are running with retained logs
`/tmp/ares-ard76-workspace.log` and `/tmp/ares-ard76-clippy.log`.
Independent reviewer accepted the two bit expectations as scalar calculation
evidence and the historical ARD correction, but withheld final approval pending
native arithmetic identity, ordinary complete GREEN and mixed-Z witness.

Independent actual-Eigen calculation39406 completed0 using retained
`/tmp/ares-ard76-bounds-eigen.cpp`, the current reference dependency
Eigen5.0.1 and Nix clang-wrapper21.1.8, `-std=c++20 -O3 -DNDEBUG` (no forced
floating-point contraction flag). Its actual Eigen affine multiplication,
cast<float>, point expression and indexed bbox expansion reproduce all six
bits above. This supplements the scalar calculation, but is not execution of
the native bbox function. The initial unqualified clang++ attempt failed127
because it was absent from PATH, not because the expression failed.

Clippy78258 completed0 (existing warnings retained). Full workspace98871
is still running; KSR semantic diagnostic is already RED130.561s at
layer49/Z9.8 with16 unmatched expected and16 actual internal-solid moves,
first width .37303 versus .37301. This does not support a KSR affine fix.
Current four-platform compile recheck50211 is running.

Compile recheck50211 completed0: ares-core checks pass for browser WASM,
Windows GNU,macOS and Linux; ares-wasm browser-target check passes as well.
These are compile gates, not four-platform runtime parity proof.

Full workspace98871 completed100/524.544s:7457 pass,1 fail,9 skipped.
The sole failure is the KSR semantic diagnostic described above. Existing
consumer snapshots and all other registered Rust tests pass; disabled native
gates do not count as reference parity evidence.

Native rebuild10468 completed0, exact same derivation/output. Final wrapper
SHA256 `3d05d4ccab740ca5e13aa94d0d00867a75f867ba2159b9f5b024dc0fcaa3bfaa`
and actual ELF SHA256 `8f0cf69dd9494ccfe53bf6749d6b685ccb5a0dd0df206694ce50d421007cc03d`
match the retained historical reference manifest exactly. A Nix GC root
`/tmp/ares-ard76-initialized-reference` now retains this output.

Ordinary complete-output run33385 completed0/5.237s: all four cases pass
(equal-cost bridge,stretched rehash,compressed polygon,tilted translation).
Full paired artifacts are retained under
`/tmp/ares-ard76-affine-green-restored-reference`; log has the same basename.
This is normal production execution, not the earlier debugger counterfactual.

Native bbox disassembly is retained at
`/tmp/ares-ard76-native-bbox-disassembly.log`: symbol update_volume_bboxes
starts0x11b4e00. Indexed-point path0x11b54a3..54cf loads X/Y/Z from the
vertex, multiplies X and Y separately, adds them, multiplies/adds Z, then adds
translation; repeated triangle loop0x11b54f0..55b2 uses the same order without
FMA. Together with source-owned PrintApply.cpp bbox calls and the actual-Eigen
calculator, this independently supports the bbox snapshot migration.

Independent six-axis re-review accepts this finite mesh/bbox repair: it
independently verified all four paired whole streams, exported tilted transform,
native bbox instructions and separate expected-value computation. Compressed
RED supplies sensitivity; tilted is an additional boundary regression. No new
required code repair was found. Existing seam precision contract stays deferred
and explicitly unclaimed. Combined submission remains withheld under the
workspace gate, since KSR full parity is not green.

Original KSR strict ordered-stream revalidation63348 is running against the
retained initialized native fixture; exact original project copied unchanged
to `/tmp/ares-ard76-ksr-ordered-input.XANS1V`. This is explicitly cached-reference
replay, not a fresh native slice. Full current outputs are retained under
`/tmp/ares-ard76-ksr-ordered-current`, log has the same basename.

Strict KSR replay63348 completed100/18.714s; full pair
`/tmp/ares-ard76-ksr-ordered-current/case-MGIbJ2`. Input and reference SHA256
match the manifest. First difference remains byte285432/line9946: M73P9R98
occurs one movement early; expected6321717 versus actual6322086 normalized
bytes. Raw current Ares stream is byte-identical (`cmp` exit0) to pre-ARD76
`/tmp/ares-ard75-ksr-full/case-HSdpgw/ares.gcode`. This repair improves the
compressed/tilted cases but makes no KSR output change; investigate the
remaining M73 and layer49 internal-solid producer separately.
