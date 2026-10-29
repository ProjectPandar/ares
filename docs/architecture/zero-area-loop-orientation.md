# Zero-area extrusion-loop orientation

Status: docs-only SPEC, followed by separately committed Native-first public RED.
Base4b3844e28711a6a1381ca847dbd390fe67f2347f.
Branch worker6/zero-area-orientation-spec; independent worktree
/home/indexyz/ares.worker6-readonly-4b384. CPU15, one job/test thread; no FULL.
The approved occlusion chain20e41b63f remains frozen in its separate worktree.
Production GREEN requires separate RED approval and authorization.

## Upstream owner and Rust destination

This is a source-cited libslic3r loop-orientation rewrite slice, not a new Ares
inward movement, fuzzy-skin, polygon-validation or travel policy.

- `OrcaSlicer/src/libslic3r/GCode.cpp:6003/6011` uses
  `loop.is_counter_clockwise()` to decide whether to swap inward angle inputs
  and negate the angle. The eventual move is emitted at6032.
- `ExtrusionEntity.hpp:464` delegates loop winding to its Polygon.
  `ExtrusionEntity.cpp:162-171::ExtrusionLoop::polygon` concatenates each path's
  points except its closure endpoint.
- `Polygon.cpp:71-74::Polygon::is_counter_clockwise` delegates to
  `ClipperLib::Orientation(points)`.
- `deps_src/clipper/clipper.hpp:200::Orientation` explicitly returns
  `Area(poly) >= 0`.
- `deps_src/clipper/clipper.cpp:167-179::Area` returns0 for size<3; otherwise it
  accumulates double area and returns its signed half. Zero has a defined CCW
  classification, including the short Polygon reached by this witness.

Exact Rust destination: the final strict `> 0` predicate in the PRIVATE
`project_slice/gcode_emit/motion/loop_paths/inward.rs:106-116`
`loop_is_counter_clockwise`. It currently sums signed area in i128 and classifies
zero as false. That predicate is an incomplete compatibility shell around the
upstream Orientation concept.

Included future rewrite: classify zero as counter-clockwise, reproducing
Orientation = Area >= 0 at this local boundary. Preserve the existing accumulator,
point traversal and nonzero behavior. Reuse the existing helper and consumers;
no new helper, public API, fallback, epsilon or degenerate-loop guard.

## Non-negotiable boundaries and deferred behavior

Only zero-area orientation classification is included. The upstream DOUBLE
accumulator and a general floating area-arithmetic port are explicitly DEFERRED;
do not replace the current i128 sum or redesign nonzero near-cancellation behavior.
Rounding, inward displacement/rotation arithmetic, all consumers, fuzzy generation,
path clipping and loop-role construction are also DEFERRED/unchanged.

Do not remove or deduplicate the generated two-distinct-point closed loop. Native
also generates it, completes successfully, and has an explicit short-path Area0
contract. This is not an admission error, Unsupported guard retirement, defensive
fallback, undefined-area claim or permission to reject an otherwise valid input.

No changes to seam occlusion/selection, mesh transforms, spiral/processor, timing,
M73, brim/EFC, painted/side-cut, combine-solid or concentric sparse. No changes to
tests, inputs, helpers or normalizer during a future GREEN. Core stays neutral
for WASM browsers, Windows, macOS and Linux; no filesystem/UI behavior enters core.
Any full-output residual after the classification-only change requires STOP and
fresh scope review, not rounding/consumer/fuzzy-generation compensation.

## Qualified discovery and proof limits

External authority:
`/home/indexyz/ares-evidence/w6-4b-fuzzy-inward-5myipx2g/verification.json`.
All raw arrays remain external; DISCOVERY-RR.md is a bounded summary.

-77 Native/Rust inward calls have identical center/first/second point inputs.
- Unique zero-area call5 is at printZ1.2. Native's closed path has3 points and
  only2 distinct vertices; closure removal produces a Polygon of size2, whose
  upstream Area is explicitly0. The original GUI input mesh is not degenerate.
- Persistent Native loop-role bits2 mean Internal, not Hole; Rust's Hole argument
  is also false. Rust's observed orientation tuple is false,false, whereas the
  source's zero>=0 CCW classification explains Native's actual rotation angle.
- Native captured angle is+.17027600016027156; strict>0 classification derives
  -1.9241191022329238 from the same inputs. Native normal-return destination is
  observed; the Rust micron destination is only host arithmetic, cross-checked
  against actual rounded G-code, not claimed as a captured Rust return.
- Three accepted read-only GDB probes exit normally and preserve complete
  non-GDB output. may-call-functionsOFF, no inferior calls or memory/register
  writes. The optimized-out angle probe is excluded. A Native local is_hole
  read contradicts persistent role bits and is explicitly UNTRUSTED/not used;
  the proof uses the actual role field, angle and normal-return destination.
- No production counterfactual or GREEN was performed. No claim is made about
  the deferred floating-area accumulator or arbitrary zero-area loop families.

## Frozen GUI-reachable single-key RED pair

Single existing Native-exported object, Classic/unpainted, displacement fuzzy
mode, interlocking OFF, brim OFF, skirt0. GUI `Tab.cpp:2737` exposes fuzzy_skin;
`PrintConfig.cpp:3540` defines its options. Change external ->none ONLY.

Use the already exported input-only archives unchanged and give them meaningful
fixture labels (not the temporary discovery beam1/beam0 directory names):

| Role | Fixture | SHA256 |
|---|---|---|
| Target | zero-area-orientation/fuzzy-external-target.3mf | `b71a3657d0fe805993e39c680c10310583446f67e94b7a671fc9ec4332900659` |
| Sole-key control | zero-area-orientation/fuzzy-none-control.3mf | `06693138931240d74774230ef570d19d65a02afe9ded7d0da97648693a67822a` |

Only Metadata/project_settings.config fuzzy_skin differs; all other ZIP payloads
match. No G-code member, regenerated input or duplicate existing fixture SHA.
Discovery Native2 was stable before byte API/real CLI in official and seeded;
target complete FAIL/control complete PASS, API success/CLI0, raw API=CLI.
First strict difference is NON-M73 at canonical byte11554,line542,col10: the
inward G1 XY. Preserve all M73, motion, time, metadata and footers in comparison.

## Authorized RED and later approval gates

Commit this docs-only SPEC first; then commit only two unchanged 3MF inputs,
a test module and its registration. Tests must call unchanged
`runner::native_stream_case::check` directly, with frozen SHA and dynamic Native
reference. No hardcoded Native output, coordinate/angle expected constants,
internal-only test, copied execution engine, normalizer or expected-error success.
This single-object pair does not require a second tool; use assert_two_tools=false.

At the exact RED tip, require both modes target FAIL/control PASS, Native2 stable
BEFORE real public byte API/CLI, both complete ordered comparisons, API success
and CLI0. Canonical failure precedes the final raw assertion/adapter update on
target: independently audit actual raw files, do not claim that assertion ran.
Check fmt, strict workspace/all-target clippy, clean scope and changed text<400LOC.
Send SPEC+RED RR and STOP BEFORE GREEN. No FULL, rebase, push or self-merge.
A separately authorized future GREEN must freeze RED inputs/tests/SPEC/helpers,
pass whole streams and source-adjacent Native gates, fmt/clippy/WASM; any residual
must be reported. Integration FULL requires its own later explicit window.
