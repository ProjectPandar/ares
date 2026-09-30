# Initialized bridge candidate direction implementation plan

1. Run the public small-overhang complete comparison before production edits.
   Retain all bytes and applied settings. Minimize one variable at a time if
   the first failure is not sensitive to candidate direction.
2. Compare native and Rust direction producers with an external debugger.
   Test ranked, falsifiable hypotheses before changing the source adapter.
3. Port only the proven `BridgeDetector.hpp` candidate collection behavior;
   preserve geometry/cost arithmetic. Main dirty changes remain untouched.
4. Re-run small complete output and original KSR diagnostics without comparator
   changes. Run Nextest, rustfmt, clippy and four Tier-1 compiler targets.
5. Independent six-axis review, repair and re-review; commit/push code and docs
   together only after the finite slice's gates pass.

Current observation: native KSR bridge runs X99.295 downward from Y143.843;
Ares runs X96.796 upward from Y132.141. Inner connections share Y132.302 and
Y143.682. The claimed 0.161mm whole-path translation was imprecise: it is the
outer-end assignment plus opposite traversal, not a shift of every point.
Source suggests candidate direction, but no causal repair is claimed yet.

First public baseline47354 terminated100 in1.922s: full pair retained at
`/tmp/ares-ard75-bridge-red/case-2nrIqL`, first difference line808 F919/F862
on an inner wall. This reproduces the known cooling/speed residual, not the
KSR bridge endpoint symptom. Remove only walls (`wall_loops=0`) for the next
minimization attempt; no production implementation change has been made.

Wall-free10566 terminated100 in1.930s, exact full pair
`/tmp/ares-ard75-bridge-no-walls/case-9oxBIR`, first bridge travel line2398.
Native debugger36499 terminated0 and Rust debugger10190 terminated0
(inferior101 expected for RED). Producer logs:
`/tmp/ares-ard75-native-no-walls-producer.log` and
`/tmp/ares-ard75-rust-input-producer.log`. Both three-edge and eight-edge
inputs match every coordinate and cost (9999999/10400001), but native direction
is(-1,-0), Rust(1,-0). Optimized-out-variable probe98296 is superseded by10190.

Ranked predictions: candidate collection traversal changes sign and removes
endpoint differences; different input geometry would change the captured
edges (excluded); normal/cost arithmetic would change the captured minimum
(not observed); downstream reversal alone would leave producer directions
equal (excluded). Port the demonstrated reference collection dependency only:
libstdc++ `bits/functional_hash.h`, `bits/hashtable.h`,
`libsupc++/hash_bytes.cc`, `src/c++11/hashtable_c++0x.cc`,
`src/shared/hashtable-aux.cc`, as already source-cited by the pending main
bridge-direction slice. Preserve main changes; isolation may overlap in this
source-owned repair but must not include unrelated dirty files in a commit.
Small complete GREEN and KSR revalidation are pending; no acceptance claimed.

Source adapter repair74271 terminated0: the same small wall-free public test
passes full output in1.886s. All paired bytes remain under
`/tmp/ares-ard75-bridge-green`. `cargo fmt --check`, patch whitespace checks and
changed Rust file sizes158/383/60 pass. No source-order snapshots were added.
Independent read-only review accepts the local source diagnosis but requires
precise libstdc++ version/owner and license attribution, independent container
differential evidence, and a complete-output witness sensitive to rehash order
before final approval. Existing dirty main evidence is not a substitute for
this initialized-reference revision. KSR semantic90958, full replay46941 and
core bridge tests99347 are running; no success or commit approval claimed.

Those gates are now terminal. KSR full46941 exits101 in28.93s, complete pair
`/tmp/ares-ard75-ksr-full/case-HSdpgw`: first difference moves from line2408
to line9946, M73 P9 R98 one command early. KSR semantic90958 exits100 in20.450s:
layer26 bridge mismatch is gone; next failure is layer49/Z9.8 Internal solid
infill, 16 unmatched entities (width0.37303 vs0.37301 and adjacent small-path
geometry). Neither comparator changed; these remain genuine failures.
Core bridge99347 exits100: 24 pass, 8 fail of32 in0.047s. Existing historical
container-dependent snapshots require careful migration; preserve still-valid
anchor/scale/PCA behavior and obtain independent evidence before replacing
expectations. No commit approval or complete parity is claimed.

Current collection differential extracts this revision's adapter into
`/tmp/ares-ard75-map-rust.rs`, compares with
`/tmp/ares-bridge-map-differential.cpp` compiled using linked-runtime GCC15.3.0.
Insertion sizes1/2/13/14/29/30/127/6284/13000, seeded keys over the full angle
domain, duplicates and +/-0 yield identical9670 output lines (diff exit0).
This is an external diagnostic, not a permanent source-pinning assertion.
Initial extraction attempts failed to compile; only the corrected probe that
excludes unrelated module imports/functions is valid evidence.

Initialized stretched16-sided public fixture91648 complete GREEN1.511s.
Remove only rehash bucket-group `.rev()`:93447 complete RED100 /1.509s,
`/tmp/ares-ard75-rehash-mutant/case-pW3eos`, line657 bridge travel native
X119.668 Y101.99 versus mutant X100.332 Y118.01; later bridge moves/wipe also
reverse. M73 is disabled and all requested settings verified. Restore only
`.rev()`:46945 complete GREEN0 /1.518s,
`/tmp/ares-ard75-rehash-restored`. Production mutation is no longer present.

Delete five obsolete selection tests targeting MSVC bucket/order mechanisms;
preserve single-line arithmetic, zero vectors, unique minimum and first-emplace.
External surface tests retain geometry/float-scale/cost/anchor routing checks;
orientation-independent axis assertions replace fixed sign snapshots where the
geometry fixes the axis. Relative per-anchor angle comparisons remain. Actual
signed orientation/endpoints are independently exercised by the public complete
small and rehash cases. Core migrated8239 terminated0: all27 tests pass in0.054s.
Read-only independent review confirms differential/rehash sensitivity/attribution
and snapshot migration without formal-comparator changes; no new mandatory code
fix found. GNU notice replaces obsolete MSVC bridge attribution, while unrelated
MSVC Clipper attribution remains. Both license texts are copied verbatim into
the isolated patch; main dirty versions remain untouched.
Clippy78852 terminated0 in3m58s. Current workspace93853 and Tier-1 check
chain59002 are live. No commit approval or full acceptance follows from local
passes; KSR layer49 and M73 remain open.

Workspace93853 terminated100: 7453 passed, 3 failed, 9 skipped in257.066s.
Besides KSR, two caller process-angle tests freeze historical automatic-sign
constants. Migrate to the same interface's behavior: relative30 degrees equals
automatic plus30 degrees regardless of model rotation, and automatic zero
is equal with/without model rotation. Preserve the independently fixed absolute
angle+model-rotation test. Do not insert current Rust output bits as the oracle.
Tier-1 chain59002 terminated0, including browser WASM adapter. Current fmt and
patch checks pass; process-angle migration still needs its targeted gate.
