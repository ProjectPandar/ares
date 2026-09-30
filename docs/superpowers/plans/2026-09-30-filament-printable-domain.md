# Filament-printable domain implementation plan

Source/destination and non-negotiable limits: ARD-0060. Port boundary is the
existing project representation of upstream ConfigOptionInts and native domain
projection; no independent slicing design or two-extruder implementation here.

1. Add a failing domain regression for INT_MIN, INT_MAX-1, full-range seeded
   interior and masks 0/1/2/3. Use the established generated-plan and complete
   original-binary output seams, never source-structure pinning assertions.
2. Run direct original-binary probes to confirm all legal values, including
   masks that disable the first bit, preserve the single-extruder branch's
   output contract. Record INT_MAX as a separate rejected serialization case.
3. Replace this key's lone authored probe with storage-bounded generation;
   preserve strict application proof, complete output retention and explicit
   distinction from deferred dual-extruder mapping behavior.
4. Run selected sweep, workspace Nextest, rustfmt, Clippy and Tier-1 compiler
   checks. Independent read-only six-axis review, fix/re-review, then commit
   code and documents together. Do not merge results into the running d2949771
   full scan, which predates this slice.

Implementation and focused evidence:
- RED `/tmp/ares-printable-domain-red.log`: 1 failure, original lone probe
  violates the seven-sample range/mask contract (0.950s).
- GREEN `/tmp/ares-printable-direct.log`: 2 PASS (6.113s), seven first-reference
  complete ordered comparisons plus separate INT_MAX serialization rejection.
  Artifacts: `/tmp/ares-printable-direct.8xGOZ9`.
- Selected sweep `/tmp/ares-printable-sweep.log`: 1 PASS (7.116s), 7 generated,
  7 strictly compared, 0 rejected; `/tmp/ares-printable-sweep.8YnBEw`.
- All 14 successful paired manifests' file lengths and SHA256 hashes checked.
  Manifest reference-producer identity remains unknown; invocation evidence
  does not independently establish provenance.
- Independent pre-review found an orphaned probe comment; removed. Added
  exported single-nozzle cardinality and final reference config-literal
  assertions. Strengthened direct rerun `/tmp/ares-printable-final.log`:
  2 PASS (16.194s), `/tmp/ares-printable-final.ec7er8`, seven paired manifests'
  lengths/hashes verified and the separate nil-sentinel rejection retained.
- Initial Clippy exposed one new nesting warning in mask generation. Replaced
  the equivalent push loop with array-map/extend; no domain changes. Rerun
  `/tmp/ares-printable-lint-fix.log`: 2 PASS (15.539s), artifacts
  `/tmp/ares-printable-lint-fix.D8HSJJ`, seven paired manifests verified.
  `/tmp/ares-printable-test-clippy-lint-fix.log` returns to 28 existing test
  warnings, with fmt/diff checks passing. The workspace rerun was compiled
  before this equivalent lint-only change; focused tests cover the changed
  generator on its final version.

Initial workspace run: 7434 PASS / 9 skipped (272.545s),
`/tmp/ares-printable-workspace.log`. Post-review workspace rerun:
7434 PASS / 9 skipped (271.682s), `/tmp/ares-printable-workspace-final.log`.
The full command chain exited 0, including current-source Clippy (248 existing
core / 28 existing parity-test warnings), fmt and diff checks in
`/tmp/ares-printable-{clippy,test-clippy,fmt}-final.log`.
Independent six-axis final review approves this nine-file finite slice with
no remaining blockers; deferred multi-extruder/global behavior stays open.
WASM adapter, Windows workspace and macOS workspace compiler checks exited 0:
`/tmp/ares-printable-{wasm,windows,macos}.log`. These are compiler checks, not
platform runtime parity evidence. No production code changed.

The d2949771 full scan remains separately active in
`/tmp/ares-vitrification.WEOwPX`; its immutable results must not incorporate
this slice's focused counts.
