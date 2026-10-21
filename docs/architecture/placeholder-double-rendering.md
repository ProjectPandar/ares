# PlaceholderParser unified double rendering

Status: docs-only SPEC, then separately committed Native-first public RED.
Base47821c760d5ea86102a34bbc78c092bbaf381214.
Independent branch worker6/placeholder-double-render-spec; worktree
/home/indexyz/ares.worker6-readonly-47821c760. CPU15, one job/test thread; no FULL.
Both approved W6 chains20e41b63f and8ddfa8ca5 stay frozen separately.
Production GREEN requires separate RED approval and authorization.

## Single upstream owner and exact Rust boundary

This is a source-cited libslic3r PlaceholderParser rewrite slice, not an Ares
number-display policy, new template engine or slicing pipeline feature.

`OrcaSlicer/src/libslic3r/PlaceholderParser.cpp:302-324`,
`Slic3r::client::expr::to_string`, handles TYPE_DOUBLE with ONE conversion:
`std::ostringstream ss; ss << this->d(); out = ss.str();` at319-321. The nearby
std::to_string block is disabled. A fresh stream has defaultfloat and precision6:
%g-style SIX SIGNIFICANT digits, scientific notation when the decimal exponent
is below-4 or at least6, otherwise fixed; omit insignificant trailing fractional
zeros and a now-empty decimal point. Preserve sign/exponent spelling as upstream.
Determine the style consistently with the rounded six-significant-digit result,
not a new fixed-decimal policy. Do not strip significant integral zeros.

This operation applies to ALL TYPE_DOUBLE values, including doubles with an
integral numeric value. They are NOT the distinct TYPE_INT case at310, which
uses std::to_string(integer). TYPE_STRING returns its string unchanged.

Rust destination: `project_slice/gcode_emit/value.rs::Value::as_string(:83-87)`
and its private `format_number(:106-117)`. Two current Number arms split the same
upstream operation: integral Number uses format {:.0}, others call the always
fixed-decimal/trimming helper. Both are partial compatibility shells around
ONE ostream-double operation. Included future rewrite: ONE source-faithful
precision6/%g rendering path replacing BOTH Number arms. Do NOT defer the
integral-DOUBLE arm or redesign numeric representation to evade this boundary.

## Non-negotiable architectural constraints / deferrals

- Only Number(f64) rendering in as_string and its private formatting boundary.
  Preserve actual Integer(i64) and String arms, plus all other value variants.
  Mathematically integral Number is still a double, not an Integer variant.
- Preserve lexer, expression evaluation, arithmetic, context lookup, Integer vs
  Number construction, template control flow, consumers, legacy expansion
  routing, custom-G-code execution and public API. No new format configuration,
  precision option, flag, fallback, input guard, epsilon or data-type redesign.
- No alternate number converter that special-cases only the two witnesses;
  port the cited ALL-double operation, not an Ares-specific exponent patch.
- Do not absorb GCodeProcessor/time/M73, writer coordinates, infill geometry or
  Aligned-rectilinear template identity. This is string rendering before the
  processor. No queued painted/side-cut, brim/EFC, combine-solid, concentric
  sparse, spiral, seam-occlusion or zero-area winding changes.
- Core remains portable for WASM browser, Windows, macOS and Linux. No native
  process/libc/file I/O in core. No new unit tests after production code.
- Any strict residual after the unified-double future correction means STOP
  and fresh review, not a changed fixture, mask, Integer/String edit or scope
  widening. Preserve all test/helper/input/SPEC bytes during GREEN.

## Qualified discovery evidence and corrected scope

External authority:
`/home/indexyz/ares-evidence/w6-47821-placeholder-mwdl1xn9/`.
`verification.json` binds the small pair, and `comparison.json` also retains the
large positive. The discovery draft had deferred the integral-double branch;
COORD explicitly SUPERSEDES that deferral with this unified owner boundary.

Both positives are GUI-reachable custom machine_start_gcode expressions in an
appended COMMENT; the original startup program and physical printing stay intact.
GUI printer Custom G-code editor: Tab.cpp:4600. Geometry is an existing single
Classic unpainted object, interlocking/brim/skirt OFF. This avoids printer-command
execution/timing interpretations without filtering comments from strict output.

- Small expression0.000000001: Native emits scientific notation, Ares fixed.
  First strict NON-M73 canonical difference byte1762,line129,col18.
- Large expression1000000000000.0: an integral-valued DOUBLE; Native emits
  scientific notation, Ares integral decimal. First strict NON-M73 difference
  byte1763,line129,col19. It is included in the SAME rewrite slice, not deferred.
- Shared sole-key PASS control expression1.0 / 2.0.
- Every positive/control pair differs only in machine_start_gcode's appended
  expression comment, all other decoded JSON and ZIP payloads identical.
- Discovery Native2 stable FIRST in official/seeded, real byte API/CLI succeed,
  each raw API/CLI pair matches; both positives full FAIL/control full PASS.
  Comparisons retain all comments, M73, motion, time, metadata and footers.

Two accepted read-only GDB probes for the small value exit normally and preserve
complete non-GDB output. Native expr::to_string sees TYPE_DOUBLE1e-9; Rust
format_number receives f64 with the SAME bits0x3e112e0be826d695. Thus the first
producer is formatting, not parsing/evaluation/context/type admission. One
matched scalar per probe; no unobserved large-value runtime sample is claimed.
may-call-functionsOFF, field/argument reads only, no inferior calls or memory/
register writes. Discovery is not a production counterfactual or GREEN result.

## Frozen input-only RED matrix

Use these ALREADY Native-exported archives unchanged, no regeneration:

| Role | Fixture under placeholder-double-render/ | SHA256 |
|---|---|---|
| Small positive | small-double-target.3mf | `d5705287bf8ff2f47d1186e77f529e755e9b9db5be12aa4b9e533349a9e42739` |
| Large integral-double positive | integral-double-target.3mf | `d04abfff556d4a4bb5d0eb366803eb74f723f24f763d25754bfcdf00874a1ee8` |
| Sole-key PASS control | half-double-control.3mf | `afc80845229f5428aad1ab7cf04eaa69716e8fd0aa61e438165fa60ec2983e58` |

Both positive/control pairs have the single machine_start_gcode delta, no G-code
archive member, Native output or duplicate preexisting fixture SHA committed.
Retain meaningful labels and one shared control; no input search to hide residuals.

## Approval / verification sequence

1. Commit this docs-only SPEC, then input/test-only RED at an exact commit.
   Three direct calls to unchanged native_stream_case::check ->NativeCase,
   frozen input SHA, source citations, assert_two_tools=false for this single
   object. Dynamic live Native expected streams, NO hardcoded Native output,
   coordinate, formatted-number expected constants or error-as-success tests.
2. Official AND seeded must each show small FAIL + integral-double FAIL +
   shared control PASS, Native2 stable BEFORE public byte API/real CLI. Verify
   API success/CLI0 and BOTH COMPLETE Native/API +Native/CLI comparisons.
3. Canonical RED failure precedes its final raw assertion and adapter update:
   independently audit actual raw API/CLI files; do not claim that assertion ran.
   No copied engine, alternate normalizer, comment/M73/metadata mask or skip.
4. fmt and strict workspace/all-target clippy must pass; all changed text<400LOC,
   production/helpers unchanged. Send SPEC+RED RR and STOP BEFORE GREEN.
5. If separately authorized, future production-only GREEN must unify both Number
   arms and pass all3 complete streams in BOTH modes, raw equality, appropriate
   existing placeholder/template consumers and fmt/clippy/WASM gates. No universal
   formatter or Integral/String parity claim beyond verified source behavior.
6. No FULL, rebase, push or self-merge without a later explicit window. The two
   frozen W6 chains and user loop-test keep-deleted decision remain untouched.
