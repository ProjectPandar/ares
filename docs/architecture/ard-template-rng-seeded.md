# ARD: seeded PlaceholderParser RNG rewrite

## Boundary and non-negotiable decisions

Port OrcaSlicer8500fcdccaa10b5099ac20d252af3a7c560046f1
`PlaceholderParser.hpp:16-26::ContextData::rng`,
`PlaceholderParser.cpp:647-657::expr::random`,1708-1716 and2319-2320, with
`GCode.cpp:2703-2709` initialization, into the existing Rust template evaluator.
This is an upstream rewrite slice, not a new Ares pipeline feature. The existing
fixed5489 LCG in `gcode_emit/value.rs` is a temporary incompatible scaffold and
must be removed, not kept as a fallback.

- Extend only `scripts/orca-replay/seed-only.patch`: template MT19937 takes the
  same explicit `ORCA_REPLAY_SEED` as the existing FuzzySkin replay engine. The
  wrapper already requires a u32 seed. Official Orca is unchanged. No geometry,
  traversal, timing, initialization or comparator patch is added.
- Rebuild via existing `scripts/orca-replay/build.nix`, same nixpkgs/compiler and
  baseline source store snapshot, copied to a private source directory. Record
  new store path, executable SHA256, source delta and derivation identity.
- Rust destination: reuse `slicer_context/rng.rs::Mt19937`; add the exact
  libstdc++ integer distribution alongside its existing double canonical draw.
  Template random chooses integer versus real from expression value types,
  not whether a floating value happens to have an integral magnitude.
- Keep template MT separate from the continuously consumed FuzzySkin stream.
  Use the public `SlicerContext::new(seed)` input, with no environment/time/I/O
  in core. One template stream belongs to each G-code emission and is shared
  across context-bearing template evaluations/config copies, matching ContextData ownership.
  `GCode.cpp:2551-2566` file-start processing supplies no ContextData: random()
  there must report the upstream unavailable-context parser error, not use a
  fallback stream. Initialize the shared seeded stream at the later GCode
  context boundary. Cover this distinction before implementing it.
  Wire it through `gcode_emit`, `value::Config`, template config construction and
  existing emission state; preserve four-platform support and Send of public
  slice inputs. Do not reseed per expression or per template.
- Integer distribution must match the selected libstdc++ implementation,
  including full signed32 range, inclusive endpoints and rejection draws.
  Real distribution must preserve draw order, f64 operations and upper-edge
  handling. Existing fuzzy stream output must remain unchanged.

Included: integer and real random(a,b), original expression type dispatch,
explicit seed plumbing, six vendor startup fixtures, job/stream isolation.
Deferred: unrelated parser functions, broader global-variable lifecycle,
physical clock entropy policy for callers (they already provide explicit seed),
other stock failures, Creality and the paused adapter lint slice.

## Evidence and acceptance

Pre-implementation diagnosis (retained externally) covered 31
native runs. All six templates match frozen inputs; clock-seeded native output
changes even with the old replay seed0 because its patch only affects FuzzySkin.

Write full-output public E2E tests before Rust implementation. Observe RED
against the rebuilt reference, then GREEN for six cases and seeds0/1, repeat0
and job isolation. Add an input-derived template probe for real/mixed argument
kinds and distribution edges before implementing those paths. Keep verifiable
input, native and Rust bytes plus generator-only comparator evidence.

Repeat-native identity uses only the established generator-line normalization;
raw timestamp differences are retained. Seed1 changes must be traced to random
expansions and any resulting writer/estimator consequences, never silently
filtered. If consequences extend beyond that source boundary, investigate and
report rather than weakening acceptance.

Workspace Nextest, default Clippy (no new warnings), fmt and actual core/WASM
wasm32 build; full official and seeded controls with no new FAIL.
No suppressions, guards, compatibility RNG, timing waiver or partial comparator.
Land with latest-main revalidation and ff-only integration.

## Implementation evidence

The new replay producer is
`/nix/store/1k0p5nlbs3zsfdcy08g3y5fb7bcm1qrv-orca-slicer-seed-replay-2.4.2/bin/orca-slicer`.
Native six-fixture qualification (retained externally):
seed0 repeats agree; seed1 changes
purge coordinates and derived M73/first-layer timing. Those timing lines remain
part of per-seed full-byte acceptance.

The retained pre-implementation RED record shows that
seven native repeat checks pass before full-output Rust divergences; file-start
also reproduces the unavailable-context discrepancy. An earlier run used the
Orca/Ares comparator incorrectly for two native streams; it is not the RED
basis. The correction uses the existing Orca normalization on both streams.

The two GREEN runs (seed0 and seed1) each pass all eight public E2Es,
including complete output, typed-distribution/shared-stream probes and job
isolation. These are pre-latest-main results.
The existing397 controls also require old/new producer output identity evidence;
the canonical control script is not changed by this candidate.
